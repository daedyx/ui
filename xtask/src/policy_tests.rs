use super::*;

fn files(list: &[(&str, &str)]) -> (Vec<String>, std::collections::BTreeMap<String, String>) {
    let paths = list.iter().map(|(p, _)| p.to_string()).collect();
    let map = list
        .iter()
        .map(|(p, t)| (p.to_string(), t.to_string()))
        .collect();
    (paths, map)
}

#[test]
fn every_kind_of_file_is_checked() {
    let (paths, map) = files(&[
        (
            "src/a.rs",
            "#[cfg_attr(all(), allow(clippy::unwrap_used))]\nfn f() {}",
        ),
        ("crates/x/Cargo.toml", "[package]\nname = \"x\"\n"),
        (
            "crates/x/clippy.toml",
            "avoid-breaking-exported-api = false\n",
        ),
        (
            ".cargo/config.toml",
            "[build]\nrustflags = [\"-A\", \"warnings\"]\n",
        ),
        ("README.md", "#[allow(x)] in Text zählt nicht"),
    ]);
    let (sites, problems) = collect(&paths, "", |p| map.get(p).cloned().unwrap_or_default());
    assert!(
        sites.contains(&("src/a.rs".to_string(), "clippy::unwrap_used".to_string())),
        "{sites:?}"
    );
    assert!(
        problems.iter().any(|m| m.starts_with(".cargo/config.toml")),
        "{problems:?}"
    );
    // Fehlen deny.toml und clippy.toml, fehlen alle Sperren: das sind Stellen ohne Eintrag.
    assert!(
        sites
            .iter()
            .any(|(p, r)| p == "deny.toml" && r == "advisories.yanked=fehlt"),
        "{sites:?}"
    );
    assert!(
        sites
            .iter()
            .any(|(p, r)| p == "clippy.toml" && r.starts_with("disallowed-methods-fehlt=")),
        "{sites:?}"
    );
    assert!(
        sites.contains(&(
            "crates/x/Cargo.toml".to_string(),
            "manifest:lints".to_string()
        )),
        "{sites:?}"
    );
    assert!(
        problems
            .iter()
            .any(|m| m.starts_with("crates/x/clippy.toml")),
        "{problems:?}"
    );
    assert!(
        problems.iter().any(|m| m.starts_with("Cargo.toml")),
        "Grundliste der Workspace-Lints fehlt: {problems:?}"
    );
    assert!(!sites.iter().any(|(p, _)| p == "README.md"));
}

#[test]
fn cargo_settings_and_macro_attributes_stop_the_check() {
    let (paths, map) = files(&[
        (
            ".cargo/config.toml",
            "[alias]\nxtask = \"run --quiet --package xtask --\"\n[target.x86_64-pc-windows-msvc]\nrunner = \"ok.cmd\"\n",
        ),
        (
            "crates/a/.cargo/config.toml",
            "[alias]\nxtask = \"run --quiet --package xtask --\"\n",
        ),
        ("rust-toolchain.toml", "[toolchain]\nchannel = \"1.99.1\"\n"),
        (
            "src/m.rs",
            "macro_rules! w { ($m:meta) => { #[$m] fn f() {} }; }",
        ),
        ("deny.toml", "a = \"\"\"\nx\"\"\"\n"),
        (".github/workflows/ci.yml", "env:\n  RUSTC_WRAPPER: w.cmd\n"),
    ]);
    let (sites, problems) = collect(&paths, "", |p| map.get(p).cloned().unwrap_or_default());
    for p in [
        ".cargo/config.toml",
        "crates/a/.cargo/config.toml",
        "rust-toolchain.toml",
        "src/m.rs",
        "deny.toml",
    ] {
        assert!(
            problems.iter().any(|m| m.starts_with(p)),
            "{p}: {problems:?}"
        );
    }
    assert!(
        sites.contains(&(
            ".github/workflows/ci.yml".to_string(),
            "workflow:rustc_wrapper".to_string()
        )),
        "{sites:?}"
    );
}

#[test]
fn build_scripts_and_crate_settings_are_sites() {
    let (paths, map) = files(&[
        ("crates/a/build.rs", "fn main() {}"),
        (
            "crates/a/Cargo.toml",
            "[package]\nname = \"a\"\n[lib]\ntest = false\n[lints]\nworkspace = true\n",
        ),
    ]);
    let (sites, _) = collect(&paths, "", |p| map.get(p).cloned().unwrap_or_default());
    assert!(
        sites.contains(&("crates/a/build.rs".to_string(), "build.rs".to_string())),
        "{sites:?}"
    );
    assert!(
        sites.contains(&(
            "crates/a/Cargo.toml".to_string(),
            "manifest:lib.test".to_string()
        )),
        "{sites:?}"
    );
}

#[test]
fn the_cargo_workspace_may_live_below_the_repository_root() {
    // Wie in Desk: Cargo-Workspace unter app/src-tauri/, Doku und Skripte an der Repo-Wurzel.
    let alias = "[alias]\nxtask = \"run --quiet --package xtask --\"\n";
    let (paths, map) = files(&[
        ("app/src-tauri/.cargo/config.toml", alias),
        (".cargo/config.toml", alias),
        ("app/src-tauri/deny.toml", "probe = 1\n"),
    ]);
    let (sites, problems) = collect(&paths, "app/src-tauri/", |p| {
        map.get(p).cloned().unwrap_or_default()
    });
    assert!(
        !problems
            .iter()
            .any(|m| m.starts_with("app/src-tauri/.cargo/config.toml")),
        "{problems:?}"
    );
    assert!(
        problems.iter().any(|m| m.starts_with(".cargo/config.toml")),
        "{problems:?}"
    );
    assert!(
        sites.contains(&("app/src-tauri/deny.toml".to_string(), "probe=1".to_string())),
        "{sites:?}"
    );
}

#[test]
fn only_build_scripts_at_a_crate_root_count_and_the_root_package_is_judged_once() {
    let ws = "[workspace]\nmembers = [\".\", \"crates/a\"]\ndefault-members = [\"crates/a\", \".\"]\nresolver = \"3\"\n";
    let root = format!(
        "{ws}[package]\nname = \"app\"\n[lib]\nname = \"app_lib\"\ncrate-type = [\"rlib\"]\n"
    );
    let (paths, map) = files(&[
        ("Cargo.toml", root.as_str()),
        ("build.rs", "fn main() {}"),
        (
            "crates/a/Cargo.toml",
            "[package]\nname = \"a\"\n[lints]\nworkspace = true\n",
        ),
        ("crates/a/src/mail/build.rs", "pub fn build() {}"),
    ]);
    let (sites, problems) = collect(&paths, "", |p| map.get(p).cloned().unwrap_or_default());
    assert!(
        sites.contains(&("build.rs".to_string(), "build.rs".to_string())),
        "{sites:?}"
    );
    assert!(
        !sites.iter().any(|(p, _)| p == "crates/a/src/mail/build.rs"),
        "ein Modul namens build.rs ist kein Build-Skript"
    );
    let manifest: Vec<_> = sites
        .iter()
        .filter(|(p, r)| p == "Cargo.toml" && r.starts_with("manifest:"))
        .collect();
    assert!(
        manifest.iter().any(|(_, r)| r == "manifest:lints"),
        "Paket ohne Workspace-Lints: {manifest:?}"
    );
    assert_eq!(
        manifest.len(),
        1,
        "workspace.*, default-members (gleiche Menge) und crate-type sind erlaubt: {manifest:?}"
    );
    assert!(
        !problems.iter().any(|m| m.contains("[lints]")),
        "eigene Lints sind eine Stelle, kein harter Fehler: {problems:?}"
    );
}

#[test]
fn default_members_must_cover_all_members() {
    let ws = "[workspace]\nmembers = [\".\", \"crates/a\"]\ndefault-members = [\".\"]\n";
    let (paths, map) = files(&[("Cargo.toml", ws)]);
    let (sites, _) = collect(&paths, "", |p| map.get(p).cloned().unwrap_or_default());
    assert!(
        sites.contains(&(
            "Cargo.toml".to_string(),
            "manifest:workspace.default-members".to_string()
        )),
        "{sites:?}"
    );
}
