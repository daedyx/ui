use super::*;

const CRATE: &str =
    "[package]\nname = \"a\"\n\n[dependencies]\nx = \"1\"\n\n[lints]\nworkspace = true\n";

/// Eigene Lint-Stufen eines Crates sind eine Ausnahme-Stelle `manifest:lints` (Eintrag mit Ja des Betreuers), damit
/// begründete Fälle wie ein `*-sys`-Crate mit `unsafe` oder eine alte Hülle (Desk E6) eingetragen werden können.
#[test]
fn crate_manifest_must_use_the_workspace_lints_only() {
    let lints = |t: &str| {
        crate_sites(t, false)
            .into_iter()
            .filter(|s| s == "manifest:lints")
            .count()
    };
    assert_eq!(lints(CRATE), 0);
    assert_eq!(
        lints("lints = { workspace = true }\n[package]\nname = \"a\"\n"),
        0
    );
    assert!(
        crate_sites("[workspace]\nmembers = []\n", false).is_empty(),
        "ohne [package] nichts zu prüfen"
    );
    assert_eq!(lints("[package]\nname = \"a\"\n"), 1, "fehlt");
    assert_eq!(
        lints(&format!(
            "{CRATE}\n[lints.clippy]\nunwrap_used = \"allow\"\n"
        )),
        1
    );
    assert_eq!(
        lints(&CRATE.replace("workspace = true", "workspace = false")),
        1
    );
    assert_eq!(
        lints(&CRATE.replace(
            "[lints]\nworkspace = true",
            "[lints.rust]\nunsafe_code = \"allow\""
        )),
        1
    );
}

const WORKSPACE: &str = r#"[workspace]
members = ["crates/*"]

[workspace.lints.rust]
unsafe_code = "forbid"
unused_must_use = "deny"

[workspace.lints.clippy]
all = { level = "deny", priority = -1 }
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
unreachable = "deny"
todo = "deny"
unimplemented = "deny"
indexing_slicing = "deny"
string_slice = "deny"
get_unwrap = "deny"
print_stdout = "deny"
print_stderr = "deny"
dbg_macro = "deny"
undocumented_unsafe_blocks = "deny"
multiple_unsafe_ops_per_block = "deny"
cast_possible_truncation = "warn"
cast_sign_loss = "warn"
arithmetic_side_effects = "warn"
mem_forget = "deny"
exit = "deny"
"#;

#[test]
fn workspace_lints_keep_the_base_list() {
    let (p, s) = workspace(WORKSPACE);
    assert!(p.is_empty() && s.is_empty(), "{p:?} {s:?}");
    let (p, _) = workspace(&WORKSPACE.replace("unwrap_used = \"deny\"", "unwrap_used = \"allow\""));
    assert_eq!(p.len(), 1, "{p:?}");
    let (p, _) = workspace(&WORKSPACE.replace("expect_used = \"deny\"\n", ""));
    assert_eq!(p.len(), 1, "{p:?}");
    let (p, _) = workspace(&WORKSPACE.replace(
        "arithmetic_side_effects = \"warn\"",
        "arithmetic_side_effects = \"allow\"",
    ));
    assert_eq!(p.len(), 1, "{p:?}");
    let (p, _) =
        workspace(&WORKSPACE.replace("unsafe_code = \"forbid\"", "unsafe_code = \"deny\""));
    assert_eq!(p.len(), 1, "{p:?}");
    let (p, _) = workspace(&WORKSPACE.replace("level = \"deny\"", "level = \"allow\""));
    assert_eq!(p.len(), 1, "{p:?}");
    let (p, s) = workspace(&format!("{WORKSPACE}too_many_lines = \"allow\"\n"));
    assert!(p.is_empty());
    assert_eq!(s, vec!["workspace.lints.clippy.too_many_lines=allow"]);
}

const CLIPPY: &str = r#"allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-indexing-slicing-in-tests = true
disallowed-methods = [
  { path = "std::process::Command::new", reason = "x" },
  { path = "std::env::temp_dir", reason = "x" },
  { path = "std::net::TcpStream::connect", reason = "x" },
]
"#;

#[test]
fn clippy_toml_loosening_counts() {
    assert!(clippy_sites(CLIPPY).is_empty());
    let more = clippy_sites(&format!(
        "{CLIPPY}too-many-arguments-threshold = 99\nallow-print-in-tests = true\n"
    ));
    assert_eq!(
        more,
        vec![
            "too-many-arguments-threshold=99",
            "allow-print-in-tests=true"
        ]
    );
    let fewer =
        clippy_sites(&CLIPPY.replace("  { path = \"std::env::temp_dir\", reason = \"x\" },\n", ""));
    assert_eq!(fewer, vec!["disallowed-methods-fehlt=std::env::temp_dir"]);
    assert_eq!(
        clippy_sites("").len(),
        3,
        "fehlende clippy.toml = alle Sperren fehlen"
    );
}

const FULL_CRATE: &str = r#"[package]
name = "a"
version.workspace = true
edition.workspace = true
description = "x"
license = "MIT"
publish = false

[[bin]]
name = "a"
path = "src/main.rs"
required-features = ["h"]

[features]
h = []

[dependencies]
x = "1"
y = { workspace = true }

[target.'cfg(windows)'.dependencies]
z = "1"

[dev-dependencies]
w = "1"

[lints]
workspace = true
"#;

#[test]
fn crate_settings_outside_the_allowlist_need_an_entry() {
    assert!(
        crate_sites(FULL_CRATE, false).is_empty(),
        "{:?}",
        crate_sites(FULL_CRATE, false)
    );
    for (extra, want) in [
        ("[lib]\ntest = false\n", "manifest:lib.test"),
        ("[lib]\ndoctest = false\n", "manifest:lib.doctest"),
        ("[lib]\nharness = false\n", "manifest:lib.harness"),
        (
            "[[test]]\nname = \"t\"\nharness = false\n",
            "manifest:test#1.harness",
        ),
        (
            "[[bin]]\nname = \"b\"\ntest = false\n",
            "manifest:bin#2.test",
        ),
        (
            "[patch.crates-io]\nserde = { path = \"vendor/serde\" }\n",
            "manifest:patch.crates-io.serde",
        ),
        (
            "[replace]\n\"serde:1.0.0\" = { path = \"vendor/serde\" }\n",
            "manifest:replace.serde:1.0.0",
        ),
        (
            "[profile.release]\nopt-level = 0\n",
            "manifest:profile.release.opt-level",
        ),
    ] {
        assert_eq!(
            crate_sites(&format!("{FULL_CRATE}{extra}"), false),
            vec![want.to_string()],
            "{extra}"
        );
    }
    for to in [
        "publish = false\nautotests = false\n",
        "publish = false\nbuild = \"build.rs\"\n",
    ] {
        let got = crate_sites(&FULL_CRATE.replace("publish = false\n", to), false);
        assert_eq!(got.len(), 1, "{to} {got:?}");
        assert!(
            got.iter()
                .all(|s| s == "manifest:package.autotests" || s == "manifest:package.build"),
            "{got:?}"
        );
    }
}

#[test]
fn workspace_settings_outside_the_allowlist_need_an_entry() {
    let base = format!(
        "{WORKSPACE}\n[workspace.package]\nedition = \"2024\"\n\n[workspace.dependencies]\nx = \"1\"\n\n[profile.release]\noverflow-checks = true\ndebug = false\nstrip = \"symbols\"\n"
    );
    let (p, s) = workspace(&base);
    assert!(p.is_empty() && s.is_empty(), "{p:?} {s:?}");
    let (_, s) = workspace(&format!(
        "{base}[patch.crates-io]\nx = {{ path = \"v/x\" }}\n"
    ));
    assert_eq!(s, vec!["manifest:patch.crates-io.x"]);
    let (_, s) = workspace(&base.replace(
        "members = [\"crates/*\"]",
        "members = [\"crates/*\"]\nexclude = [\"crates/b\"]",
    ));
    assert_eq!(s, vec!["manifest:workspace.exclude"]);
    let (_, s) = workspace(&base.replace("overflow-checks = true", "overflow-checks = false"));
    assert_eq!(s, vec!["manifest:profile.release.overflow-checks=false"]);
}

#[test]
fn release_optimisation_switches_are_allowed_but_not_weakening_ones() {
    let base = format!(
        "{WORKSPACE}\n[profile.release]\ncodegen-units = 1\nlto = true\nopt-level = 3\npanic = \"abort\"\nstrip = true\noverflow-checks = true\ndebug = false\n"
    );
    let (_, s) = workspace(&base);
    assert!(s.is_empty(), "{s:?}");
    for (from, to, want) in [
        (
            "panic = \"abort\"",
            "panic = \"unwind\"",
            "manifest:profile.release.panic=unwind",
        ),
        (
            "strip = true",
            "strip = false",
            "manifest:profile.release.strip=false",
        ),
        (
            "debug = false",
            "debug = true",
            "manifest:profile.release.debug=true",
        ),
    ] {
        let (_, s) = workspace(&base.replace(from, to));
        assert_eq!(s, vec![want.to_string()], "{to}");
    }
}
