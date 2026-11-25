//! Sieht Clippy in der CI den Code hinter einer `cfg`-Bedingung (Review Prüflücken N-3, M-4)?
//!
//! Erlaubnisliste statt Verbotsliste: Die CI prüft mit Clippy unter Windows, ohne `--release`, mit allen Merkmalen,
//! einmal mit und einmal ohne `test` (`--all-targets`). Bekannt sind nur die Namen unten; jeder andere Name gilt als
//! unbekannt. Ist die Bedingung in keiner dieser Lagen sicher wahr, wird der Code nie geprüft (z. B.
//! `cfg(not(clippy))`, `cfg(not(debug_assertions))`, `cfg(not(windows))`, `cfg(any())`), und das ist eine
//! Ausnahme-Stelle mit Eintrag.
use crate::rust_attrs::{group, path, split_commas};
use crate::rust_tokens::Tok;

/// Wert in einer CI-Lage (`test` an oder aus); `None` = nicht sicher bekannt.
fn eval(t: &[Tok], test: bool) -> Option<bool> {
    let (name, len) = path(t);
    let inner = if t.get(len) == Some(&Tok::Punct('(')) {
        group(t, len)
    } else {
        None
    };
    let value = match (t.get(len), t.get(len.saturating_add(1))) {
        (Some(Tok::Punct('=')), Some(Tok::Lit(v))) => Some(v.trim_matches('"').to_string()),
        _ => None,
    };
    let parts = || {
        split_commas(inner.unwrap_or_default())
            .into_iter()
            .map(|p| eval(p, test))
            .collect::<Vec<_>>()
    };
    match (name.as_str(), inner, value.as_deref()) {
        ("not", Some(x), None) => eval(x, test).map(|v| !v),
        ("any", Some(_), None) => {
            let p = parts();
            if p.contains(&Some(true)) {
                Some(true)
            } else if p.iter().all(|v| *v == Some(false)) {
                Some(false)
            } else {
                None
            }
        }
        ("all", Some(_), None) => {
            let p = parts();
            if p.contains(&Some(false)) {
                Some(false)
            } else if p.iter().all(|v| *v == Some(true)) {
                Some(true)
            } else {
                None
            }
        }
        ("test", None, None) => Some(test),
        ("windows" | "clippy" | "debug_assertions", None, None) => Some(true),
        ("unix", None, None) => Some(false),
        ("target_os" | "target_family", None, Some(v)) => Some(v == "windows"),
        ("feature", None, Some(_)) => Some(true),
        _ => None,
    }
}

/// Ist die Bedingung in mindestens einer CI-Lage sicher wahr?
pub fn visible_to_clippy(t: &[Tok]) -> bool {
    [false, true]
        .into_iter()
        .any(|test| eval(t, test) == Some(true))
}

/// Bedingung als kurzer Text für die Ausnahme-Tabelle, z. B. `not(windows)` oder `target_os="macos"`.
pub fn render(t: &[Tok]) -> String {
    t.iter().fold(String::new(), |mut s, tok| {
        match tok {
            Tok::Ident(w) => s.push_str(w),
            Tok::Punct(p) => s.push(*p),
            Tok::Lit(l) => s.push_str(l),
        }
        s
    })
}
