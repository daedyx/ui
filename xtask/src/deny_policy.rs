//! Lockerungen in `deny.toml` (SICHERHEIT 4.2, Anhang C) als Ausnahme-Stellen. Was von der festgehaltenen Grundliste
//! abweicht oder dort nicht vorkommt, braucht einen Eintrag in der Tabelle: Meldungen in `ignore`, gelockerte feste
//! Werte (`yanked`, `unmaintained`, `unknown-git`, …), neue Lizenzen, Git-Quellen, `skip`/`skip-tree`, Lizenz-Ausnahmen
//! und jeder unbekannte Schlüssel. Die Grundliste zu ändern heißt, dieses Werkzeug zu ändern (Sicherheitsdatei).
use crate::toml_lite::{Item, compact, elements, field, items, split_top, strings, unquote};

/// Lizenzen, die mit MIT verträglich sind (SICHERHEIT Anhang C, Stand 2026-11-23).
const BASE_LICENSES: [&str; 13] = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Zlib",
    "Unicode-3.0",
    "CC0-1.0",
    "0BSD",
    "BSL-1.0",
    "MPL-2.0",
    "CDLA-Permissive-2.0",
];
const BASE_ALLOW_GIT: [&str; 1] = ["https://github.com/daedyx/core"];
const BASE_ALLOW_REGISTRY: [&str; 1] = ["https://github.com/rust-lang/crates.io-index"];
const BASE_BANNED: [&str; 2] = ["openssl", "native-tls"];
/// Feste Werte: Schlüssel und erlaubte Werte. Fehlt der Schlüssel, gilt die (lockere) Voreinstellung von cargo-deny.
const FIXED: [(&str, &[&str]); 7] = [
    ("graph.all-features", &["true"]),
    ("advisories.yanked", &["deny"]),
    ("advisories.unmaintained", &["all"]),
    ("bans.multiple-versions", &["warn", "deny"]),
    ("bans.wildcards", &["deny"]),
    ("sources.unknown-registry", &["deny"]),
    ("sources.unknown-git", &["deny"]),
];
const MIN_CONFIDENCE: f64 = 0.9;

fn extra(key: &str, value: &str, base: &[&str]) -> Vec<String> {
    strings(value)
        .into_iter()
        .filter(|s| !base.contains(&s.as_str()))
        .map(|s| format!("{key}={s}"))
        .collect()
}

/// Name aus `"name"`, `"name@1.0"` oder `{ crate = "…" }` / `{ name = "…" }`.
fn name_of(element: &str) -> String {
    if element.starts_with('{') {
        field(element, "crate")
            .or_else(|| field(element, "name"))
            .map_or_else(|| compact(element), |n| unquote(&n))
    } else {
        unquote(element)
    }
}

fn ignore_site(element: &str) -> String {
    match field(element, "id") {
        Some(id) => unquote(&id),
        None if element.starts_with('{') => format!("advisories.ignore={}", name_of(element)),
        None => unquote(element),
    }
}

fn licence_exception(element: &str) -> String {
    let mut allowed = field(element, "allow")
        .map(|a| strings(&a))
        .unwrap_or_default();
    allowed.sort();
    format!(
        "licenses.exceptions={}:{}",
        name_of(element),
        allowed.join("+")
    )
}

/// Einengungen eines Eintrags in `bans.deny`.
fn ban_narrowing(element: &str) -> Vec<String> {
    let name = name_of(element);
    let mut out = Vec::new();
    if name.contains('@') {
        out.push(format!("bans.deny-version={name}"));
    }
    if let Some(inner) = element
        .trim()
        .strip_prefix('{')
        .and_then(|x| x.strip_suffix('}'))
    {
        for part in split_top(inner, ',') {
            let key = unquote(part.split_once('=').map_or(part.as_str(), |(k, _)| k));
            if !matches!(key.as_str(), "name" | "crate" | "reason") {
                out.push(format!(
                    "bans.deny-feld={}:{key}",
                    name.split('@').next().unwrap_or_default()
                ));
            }
        }
    }
    out
}

fn fixed_sites(all: &[Item]) -> Vec<String> {
    let mut out = Vec::new();
    for (key, allowed) in FIXED {
        match all.iter().find(|i| i.key == key).map(|i| unquote(&i.value)) {
            Some(v) if allowed.contains(&v.as_str()) => {}
            Some(v) => out.push(format!("{key}={v}")),
            None => out.push(format!("{key}=fehlt")),
        }
    }
    let conf = all
        .iter()
        .find(|i| i.key == "licenses.confidence-threshold")
        .map(|i| i.value.trim().to_string());
    if !conf
        .as_deref()
        .and_then(|c| c.parse::<f64>().ok())
        .is_some_and(|c| c >= MIN_CONFIDENCE)
    {
        out.push(format!(
            "licenses.confidence-threshold={}",
            conf.unwrap_or_else(|| "fehlt".to_string())
        ));
    }
    let banned: Vec<String> = all
        .iter()
        .filter(|i| i.key == "bans.deny")
        .flat_map(|i| elements(&i.value))
        .collect();
    for b in BASE_BANNED {
        if !banned.iter().any(|e| name_of(e) == b) {
            out.push(format!("bans.deny-fehlt={b}"));
        }
    }
    // Ein Verbot gilt nur ganz: jedes Zusatzfeld (`version`, `wrappers`, `use-instead`, …) und jede Version im Namen
    // engt es ein und ist eine Lockerung (Review Prüflücken M-3).
    out.extend(banned.iter().flat_map(|e| ban_narrowing(e)));
    out
}

/// Alle Ausnahme-Stellen in `deny.toml` (leerer Text = Datei fehlt = alle festen Werte fehlen).
pub fn sites(text: &str) -> Vec<String> {
    let all = items(text);
    let mut out = Vec::new();
    for Item { key, value } in &all {
        let k = key.as_str();
        match k {
            _ if FIXED.iter().any(|(f, _)| *f == k) => {}
            "licenses.confidence-threshold" | "bans.deny" => {}
            "advisories.ignore" => out.extend(elements(value).iter().map(|e| ignore_site(e))),
            "licenses.allow" => out.extend(extra(k, value, &BASE_LICENSES)),
            "licenses.exceptions" => {
                out.extend(elements(value).iter().map(|e| licence_exception(e)))
            }
            "sources.allow-git" => out.extend(extra(k, value, &BASE_ALLOW_GIT)),
            "sources.allow-registry" => out.extend(extra(k, value, &BASE_ALLOW_REGISTRY)),
            "bans.skip" | "bans.skip-tree" => out.extend(
                elements(value)
                    .iter()
                    .map(|e| format!("{k}={}", name_of(e))),
            ),
            _ => out.push(format!("{k}={}", compact(value))),
        }
    }
    out.extend(fixed_sites(&all));
    out
}

/// `.cargo/audit.toml`: nur die Meldungs-Ausnahmen (gespiegelt aus `deny.toml`), jeder andere Schlüssel ist eine
/// Ausnahme-Stelle (z. B. `[yanked] enabled = false`).
pub fn audit_sites(text: &str) -> Vec<String> {
    items(text)
        .into_iter()
        .filter(|i| i.key != "advisories.ignore")
        .map(|i| format!("{}={}", i.key, compact(&i.value)))
        .collect()
}

#[cfg(test)]
#[path = "deny_policy_tests.rs"]
mod tests;
