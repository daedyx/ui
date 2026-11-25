//! Cargo-Einstellungen, mit denen sich das ganze Prüf-Tor abschalten lässt (Review Prüflücken H-1).
//! Ein `runner` meldet jeden Test als bestanden, ein `rustc-wrapper` hängt `--cap-lints=allow` an, ein eigener
//! `rustc`, `linker` oder eine Toolchain-Datei tauscht das Werkzeug. Deshalb gilt eine Positivliste:
//! - genau eine `.cargo/config.toml` im Wurzelordner, darin nur der `xtask`-Alias (sonst hart rot);
//! - keine `rust-toolchain`/`rust-toolchain.toml` (die CI nagelt die Version selbst fest; sonst hart rot);
//! - in Workflows ist jedes Setzen solcher Umgebungsvariablen und jedes `--config`/`cargo +…` eine Ausnahme-Stelle.
use crate::toml_lite::{items, unquote};

const ROOT_CONFIG: &str = ".cargo/config.toml";
const XTASK_ALIAS: &str = "run --quiet --package xtask --";
/// Umgebungsvariablen, die Compiler, Lints, Toolchain oder Konfigurationsort ändern (Großschreibung).
const ENV_NAMES: [&str; 10] = [
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "RUSTDOCFLAGS",
    "RUSTC",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "RUSTC_BOOTSTRAP",
    "RUSTUP_TOOLCHAIN",
    "CLIPPY_CONF_DIR",
    "CARGO_HOME",
];
/// Vorsilben ganzer Gruppen (`CARGO_BUILD_RUSTC_WRAPPER`, `CARGO_TARGET_…_RUNNER`, …).
const ENV_PREFIXES: [&str; 4] = [
    "CARGO_BUILD_",
    "CARGO_TARGET_",
    "CARGO_PROFILE_",
    "CARGO_ALIAS_",
];
/// Befehlszeilen, die Einstellungen oder Toolchain am Repo vorbei ändern.
const FLAGS: [&str; 3] = ["--config", "--cap-lints", "cargo +"];

fn file_name(p: &str) -> &str {
    p.rsplit('/').next().unwrap_or(p)
}

/// Eine Cargo-Konfigurationsdatei, egal in welchem Ordner (Cargo liest alle vom Arbeitsordner aufwärts).
/// Groß- und Kleinschreibung zählt nicht (Windows liest `.CARGO\Config.toml` genauso).
pub fn is_config_file(p: &str) -> bool {
    let p = p.to_ascii_lowercase();
    p.ends_with(".cargo/config.toml") || p.ends_with(".cargo/config")
}

pub fn is_toolchain_file(p: &str) -> bool {
    matches!(
        file_name(&p.to_ascii_lowercase()),
        "rust-toolchain" | "rust-toolchain.toml"
    )
}

/// Harte Verstöße einer Cargo-Konfigurationsdatei.
/// `ws`: Ordner des Cargo-Workspace ab der Repo-Wurzel (siehe `policy::collect`).
pub fn config_problems(path: &str, ws: &str, text: &str) -> Vec<String> {
    if path != format!("{ws}{ROOT_CONFIG}") {
        return vec![format!(
            "{path}: nur {ws}{ROOT_CONFIG} im Workspace-Ordner ist erlaubt"
        )];
    }
    items(text)
        .into_iter()
        .filter(|i| !(i.key == "alias.xtask" && unquote(&i.value) == XTASK_ALIAS))
        .map(|i| format!("{path}: `{}` ist nicht erlaubt (nur der xtask-Alias; runner, rustc-wrapper, rustflags u. a. schalten Prüfungen ab)", i.key))
        .collect()
}

/// Steht an `rest` ein Setzen (`NAME:`, `NAME=`, `NAME =`)?
fn is_assignment(rest: &str) -> bool {
    matches!(rest.trim_start().chars().next(), Some(':' | '='))
}

fn word_start(upper: &str, i: usize) -> bool {
    upper
        .get(..i)
        .and_then(|b| b.chars().next_back())
        .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_'))
}

/// Ausnahme-Stellen in einer Workflow-Datei: gesetzte Umgebungsvariablen aus `ENV_NAMES`/`ENV_PREFIXES` und `FLAGS`.
pub fn workflow_sites(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let code = line.split(" #").next().unwrap_or(line);
        let upper = code.to_ascii_uppercase();
        for name in ENV_NAMES {
            let hit = upper.match_indices(name).any(|(i, _)| {
                word_start(&upper, i)
                    && upper
                        .get(i.saturating_add(name.len())..)
                        .is_some_and(is_assignment)
            });
            if hit {
                out.push(format!("workflow:{}", name.to_ascii_lowercase()));
            }
        }
        for prefix in ENV_PREFIXES {
            let hit = upper.match_indices(prefix).any(|(i, _)| {
                let rest = upper
                    .get(i.saturating_add(prefix.len())..)
                    .unwrap_or_default();
                let tail = rest.trim_start_matches(|c: char| c.is_ascii_alphanumeric() || c == '_');
                word_start(&upper, i) && tail.len() < rest.len() && is_assignment(tail)
            });
            if hit {
                out.push(format!("workflow:{}", prefix.to_ascii_lowercase()));
            }
        }
        out.extend(
            FLAGS
                .iter()
                .filter(|f| code.contains(**f))
                .map(|f| format!("workflow:{f}")),
        );
    }
    out
}

#[cfg(test)]
#[path = "cargo_config_tests.rs"]
mod tests;
