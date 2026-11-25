//! Lint-Einstellungen außerhalb des Quelltexts (SICHERHEIT Anhang A, B; Review M-2):
//! - jedes Crate-Manifest nimmt genau die Workspace-Lints (`[lints] workspace = true`), sonst Stelle `manifest:lints`
//!   (`manifest_allow`);
//! - die Workspace-Lints behalten die Grundliste mindestens so streng; weitere `allow` sind Ausnahme-Stellen;
//! - `clippy.toml`: Schlüssel außerhalb der Grundliste und fehlende Sperren sind Ausnahme-Stellen.
//!
//! `.cargo/config.toml` und Toolchain-Dateien prüft `cargo_config`.
use crate::manifest_allow;
pub use crate::manifest_allow::crate_sites;
use crate::toml_lite::{compact, elements, field, items, unquote};

const FORBID: u8 = 3;
const DENY: u8 = 2;
const WARN: u8 = 1;
/// Grundliste der Workspace-Lints (Werkzeug, Lint, Mindeststufe).
const REQUIRED: [(&str, &str, u8); 22] = [
    ("rust", "unsafe_code", FORBID),
    ("rust", "unused_must_use", DENY),
    ("clippy", "all", DENY),
    ("clippy", "unwrap_used", DENY),
    ("clippy", "expect_used", DENY),
    ("clippy", "panic", DENY),
    ("clippy", "unreachable", DENY),
    ("clippy", "todo", DENY),
    ("clippy", "unimplemented", DENY),
    ("clippy", "indexing_slicing", DENY),
    ("clippy", "string_slice", DENY),
    ("clippy", "get_unwrap", DENY),
    ("clippy", "print_stdout", DENY),
    ("clippy", "print_stderr", DENY),
    ("clippy", "dbg_macro", DENY),
    ("clippy", "undocumented_unsafe_blocks", DENY),
    ("clippy", "multiple_unsafe_ops_per_block", DENY),
    ("clippy", "mem_forget", DENY),
    ("clippy", "exit", DENY),
    ("clippy", "cast_possible_truncation", WARN),
    ("clippy", "cast_sign_loss", WARN),
    ("clippy", "arithmetic_side_effects", WARN),
];
/// Erlaubte Schlüssel in `clippy.toml` (Anhang B); alle anderen sind Ausnahme-Stellen.
const CLIPPY_KEYS: [&str; 5] = [
    "allow-unwrap-in-tests",
    "allow-expect-in-tests",
    "allow-panic-in-tests",
    "allow-indexing-slicing-in-tests",
    "disallowed-methods",
];
const CLIPPY_DISALLOWED: [&str; 3] = [
    "std::process::Command::new",
    "std::env::temp_dir",
    "std::net::TcpStream::connect",
];

/// Stufe aus `"deny"` oder `{ level = "deny", priority = -1 }`; unbekannt = 0 (wie `allow`).
fn level(value: &str) -> u8 {
    let v = if value.trim_start().starts_with('{') {
        field(value, "level").map(|l| unquote(&l))
    } else {
        Some(unquote(value))
    };
    match v.as_deref() {
        Some("forbid") => FORBID,
        Some("deny") => DENY,
        Some("warn") => WARN,
        _ => 0,
    }
}

/// Wurzel-Manifest: (Probleme, Ausnahme-Stellen).
pub fn workspace(text: &str) -> (Vec<String>, Vec<String>) {
    let all = items(text);
    let mut problems = Vec::new();
    for (tool, lint, min) in REQUIRED {
        let key = format!("workspace.lints.{tool}.{lint}");
        if all
            .iter()
            .find(|i| i.key == key)
            .map_or(0, |i| level(&i.value))
            < min
        {
            problems.push(format!(
                "Cargo.toml: {key} fehlt oder ist schwächer als in der Grundliste"
            ));
        }
    }
    let sites = all
        .iter()
        .filter(|i| i.key.starts_with("workspace.lints.") && level(&i.value) == 0)
        .filter(|i| {
            !REQUIRED
                .iter()
                .any(|(t, l, _)| i.key == format!("workspace.lints.{t}.{l}"))
        })
        .map(|i| format!("{}=allow", i.key))
        .chain(manifest_allow::root_sites(&all))
        .collect();
    (problems, sites)
}

/// Ausnahme-Stellen in `clippy.toml` (leerer Text = Datei fehlt).
pub fn clippy_sites(text: &str) -> Vec<String> {
    let all = items(text);
    let mut out: Vec<String> = all
        .iter()
        .filter(|i| !CLIPPY_KEYS.contains(&i.key.as_str()))
        .map(|i| format!("{}={}", i.key, compact(&i.value)))
        .collect();
    let listed: Vec<String> = all
        .iter()
        .filter(|i| i.key == "disallowed-methods")
        .flat_map(|i| elements(&i.value))
        .map(|e| field(&e, "path").map_or_else(|| unquote(&e), |p| unquote(&p)))
        .collect();
    out.extend(
        CLIPPY_DISALLOWED
            .iter()
            .filter(|p| !listed.iter().any(|l| l == *p))
            .map(|p| format!("disallowed-methods-fehlt={p}")),
    );
    out
}

#[cfg(test)]
#[path = "manifest_tests.rs"]
mod tests;
