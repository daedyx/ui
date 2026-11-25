//! Erlaubnisliste für Manifeste (Review Prüflücken M-5, N-8). Jeder Schlüssel, der hier nicht steht, ist eine
//! Ausnahme-Stelle `manifest:<schlüssel>`. Gemeint sind vor allem Einstellungen, die Tests oder Prüfungen still
//! abschalten (`[lib] test = false`, `harness`, `doctest`, `autotests`), ein eigenes Build-Skript (`build`) und
//! Ersatz-Abhängigkeiten (`[patch]`, `[replace]`). Unbekanntes ist immer ein Fund.
use crate::toml_lite::{Item, compact, items, strings, unquote};

/// Erlaubte Felder in `[package]` (auch als `<feld>.workspace = true`).
const PACKAGE_KEYS: [&str; 15] = [
    "name",
    "version",
    "edition",
    "description",
    "license",
    "license-file",
    "publish",
    "authors",
    "repository",
    "readme",
    "homepage",
    "documentation",
    "keywords",
    "categories",
    "rust-version",
];
/// Abhängigkeits- und Merkmal-Tabellen: ihr Inhalt ist frei (die Lieferkette prüft `cargo deny`).
const FREE_TABLES: [&str; 4] = [
    "dependencies",
    "dev-dependencies",
    "build-dependencies",
    "features",
];
/// Erlaubte Felder je Ziel (`[lib]`, `[[bin]]`, `[[test]]`, `[[example]]`, `[[bench]]`).
/// (`crate-type` nur bei `[lib]`; es ändert die Art der Bibliothek, nicht welche Tests laufen.)
const TARGET_KEYS: [&str; 3] = ["name", "path", "required-features"];
const TARGETS: [&str; 5] = ["lib", "bin", "test", "example", "bench"];
/// Erlaubte Werte im Wurzel-Manifest (`[profile.release]`); leere Liste = jeder Wert (reine Optimierung).
/// Sicherheitsrelevant sind `overflow-checks` (an), `debug` (aus), `strip` (an) und `panic` (nur `abort`).
const ROOT_FIXED: [(&str, &[&str]); 7] = [
    ("profile.release.overflow-checks", &["true"]),
    ("profile.release.debug", &["false"]),
    ("profile.release.strip", &["symbols", "true"]),
    ("profile.release.panic", &["abort"]),
    ("profile.release.lto", &[]),
    ("profile.release.codegen-units", &[]),
    ("profile.release.opt-level", &[]),
];

fn crate_key_allowed(key: &str) -> bool {
    let (head, rest) = key.split_once('.').unwrap_or((key, ""));
    match head {
        "package" => {
            let (field, tail) = rest.split_once('.').unwrap_or((rest, ""));
            PACKAGE_KEYS.contains(&field) && (tail.is_empty() || tail == "workspace")
        }
        "lints" => true, // genau `workspace = true`, geprüft in `crate_sites`
        "target" => FREE_TABLES.iter().any(|t| key.contains(&format!(".{t}."))),
        _ if FREE_TABLES.contains(&head) => !rest.is_empty(),
        _ => {
            let kind = head.split('#').next().unwrap_or(head);
            TARGETS.contains(&kind)
                && (TARGET_KEYS.contains(&rest) || (kind == "lib" && rest == "crate-type"))
        }
    }
}

/// Teile, die nur im Wurzel-Manifest gelten und dort `root_sites` prüft.
const ROOT_HEADS: [&str; 4] = ["workspace", "profile", "patch", "replace"];

/// Nimmt das Crate genau die Workspace-Lints (`[lints] workspace = true`), ohne eigene Stufen?
fn uses_workspace_lints(all: &[Item]) -> bool {
    let lints: Vec<_> = all
        .iter()
        .filter(|i| i.key == "lints" || i.key.starts_with("lints."))
        .collect();
    match lints.as_slice() {
        [one] => {
            (one.key == "lints.workspace" && one.value.trim() == "true")
                || (one.key == "lints" && compact(&one.value) == "{workspace=true}")
        }
        _ => false,
    }
}

/// Ausnahme-Stellen eines Crate-Manifests (nur mit `[package]`). `is_root`: das Wurzel-Manifest, dessen
/// Workspace-Teile `root_sites` prüft. Eigene Lint-Stufen sind die Stelle `manifest:lints` (z. B. ein
/// `*-sys`-Crate mit `unsafe` oder eine alte Hülle, die bewusst nicht unter den strengen Regeln steht).
pub fn crate_sites(text: &str, is_root: bool) -> Vec<String> {
    let all = items(text);
    if !all.iter().any(|i| i.key.starts_with("package.")) {
        return Vec::new();
    }
    let mut out: Vec<String> = all
        .iter()
        .filter(|i| !(is_root && ROOT_HEADS.contains(&i.key.split('.').next().unwrap_or_default())))
        .filter(|i| !crate_key_allowed(&i.key))
        .map(|i| format!("manifest:{}", i.key))
        .collect();
    if !uses_workspace_lints(&all) {
        out.push("manifest:lints".to_string());
    }
    out
}

fn sorted(value: &str) -> Vec<String> {
    let mut v = strings(value);
    v.sort();
    v
}

/// Ausnahme-Stellen des Wurzel-Manifests außerhalb der Lints (die prüft `manifest::workspace`).
pub fn root_sites(all: &[Item]) -> Vec<String> {
    let mut out = Vec::new();
    for i in all {
        let k = i.key.as_str();
        let head = k.split('.').next().unwrap_or(k);
        let crate_part =
            matches!(head, "package" | "lints" | "target") || FREE_TABLES.contains(&head) || {
                let kind = head.split('#').next().unwrap_or(head);
                TARGETS.contains(&kind)
            };
        if crate_part || k.starts_with("workspace.lints.") {
            continue; // Crate-Teil prüft `crate_sites`, Lints prüft `manifest::workspace`
        }
        if let Some((_, allowed)) = ROOT_FIXED.iter().find(|(f, _)| *f == k) {
            let v = unquote(&i.value);
            if !allowed.is_empty() && !allowed.contains(&v.as_str()) {
                out.push(format!("manifest:{k}={v}"));
            }
            continue;
        }
        // `default-members` bestimmt, was ein nacktes `cargo test` prüft: erlaubt nur, wenn es alle Mitglieder nennt.
        if k == "workspace.default-members" {
            let members = all
                .iter()
                .find(|m| m.key == "workspace.members")
                .map(|m| sorted(&m.value));
            if members != Some(sorted(&i.value)) {
                out.push(format!("manifest:{k}"));
            }
            continue;
        }
        let allowed = matches!(k, "workspace.resolver" | "workspace.members")
            || k.starts_with("workspace.package.")
            || k.starts_with("workspace.dependencies.");
        if !allowed {
            out.push(format!("manifest:{k}"));
        }
    }
    out
}
