//! Ausnahme-Stellen in Rust-Quelltext (SICHERHEIT B.4): `allow`/`expect` (auch in `cfg_attr`, mit Leerzeichen und
//! über mehrere Zeilen), `ignore` an Tests und das Schlüsselwort `unsafe`. Kommentare, Zeichenketten und Zeichen
//! zählen nicht. Je Stelle ein Text: die Regeln eines Attributs sortiert und mit Komma getrennt
//! (`clippy::exit,clippy::print_stdout`), oder `ignore` bzw. `unsafe`.

use crate::cfg_eval;
use crate::rust_tokens::{Tok, tokens};

/// Inhalt der Klammer, die bei `open` beginnt (alle Klammerarten zählen für die Tiefe).
pub(crate) fn group(t: &[Tok], open: usize) -> Option<&[Tok]> {
    let mut depth = 0usize;
    for (k, tok) in t.iter().enumerate().skip(open) {
        match tok {
            Tok::Punct('(' | '[' | '{') => depth = depth.saturating_add(1),
            Tok::Punct(')' | ']' | '}') => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return t.get(open.saturating_add(1)..k);
                }
            }
            _ => {}
        }
    }
    None
}

/// Teile auf oberster Ebene, getrennt durch Kommas.
pub(crate) fn split_commas(t: &[Tok]) -> Vec<&[Tok]> {
    let mut parts = Vec::new();
    let (mut depth, mut start) = (0usize, 0usize);
    for (k, tok) in t.iter().enumerate() {
        match tok {
            Tok::Punct('(' | '[' | '{') => depth = depth.saturating_add(1),
            Tok::Punct(')' | ']' | '}') => depth = depth.saturating_sub(1),
            Tok::Punct(',') if depth == 0 => {
                parts.push(t.get(start..k).unwrap_or_default());
                start = k.saturating_add(1);
            }
            _ => {}
        }
    }
    parts.push(t.get(start..).unwrap_or_default());
    parts.into_iter().filter(|p| !p.is_empty()).collect()
}

/// Pfad am Anfang (`clippy::x`) und die Anzahl der Teile, die er belegt.
pub(crate) fn path(t: &[Tok]) -> (String, usize) {
    let len = t
        .iter()
        .take_while(|x| matches!(x, Tok::Ident(_) | Tok::Punct(':')))
        .count();
    let text = t
        .get(..len)
        .unwrap_or_default()
        .iter()
        .fold(String::new(), |mut s, x| {
            match x {
                Tok::Ident(w) => s.push_str(w),
                Tok::Punct(p) => s.push(*p),
                Tok::Lit(_) => {}
            }
            s
        });
    (text, len)
}

/// Ein Attribut-Inhalt (`allow(…)`, `cfg_attr(…, …)`, `ignore`, …) → Stellen.
fn meta(t: &[Tok], out: &mut Vec<String>) {
    let (name, len) = path(t);
    let inner = if t.get(len) == Some(&Tok::Punct('(')) {
        group(t, len)
    } else {
        None
    };
    match (name.as_str(), inner) {
        ("allow" | "expect", Some(inner)) => {
            let mut rules: Vec<String> = split_commas(inner)
                .into_iter()
                .filter(|p| !p.contains(&Tok::Punct('=')))
                .map(|p| path(p).0)
                .collect();
            rules.sort();
            out.push(rules.join(","));
        }
        ("cfg_attr", Some(inner)) => split_commas(inner)
            .into_iter()
            .skip(1)
            .for_each(|p| meta(p, out)),
        // Code, den Clippy in der CI nie sieht, braucht einen Eintrag (Review Prüflücken N-3, M-4).
        ("cfg", Some(inner)) if !cfg_eval::visible_to_clippy(inner) => {
            out.push(format!("cfg:{}", cfg_eval::render(inner)))
        }
        ("ignore", _) => out.push("ignore".to_string()),
        _ => {}
    }
}

/// Inhalte aller Attribute `#[…]` und `#![…]`.
fn attributes(t: &[Tok]) -> Vec<&[Tok]> {
    t.iter()
        .enumerate()
        .filter(|(_, tok)| **tok == Tok::Punct('#'))
        .filter_map(|(i, _)| {
            let bang = usize::from(t.get(i.saturating_add(1)) == Some(&Tok::Punct('!')));
            let open = i.saturating_add(1).saturating_add(bang);
            if t.get(open) == Some(&Tok::Punct('[')) {
                group(t, open)
            } else {
                None
            }
        })
        .collect()
}

/// Alle Ausnahme-Stellen einer Rust-Datei.
pub fn sites(src: &str) -> Vec<String> {
    let t = tokens(src);
    let mut out: Vec<String> = t
        .iter()
        .filter(|tok| matches!(tok, Tok::Ident(w) if w == "unsafe"))
        .map(|_| "unsafe".to_string())
        .collect();
    for inner in attributes(&t) {
        meta(inner, &mut out);
    }
    out
}

/// Attribute, deren Inhalt aus einem Makro-Platzhalter kommt (`#[$m]`, `#[cfg_attr(x, $m)]`). Darin kann ein `allow`
/// stehen, das hier niemand sieht (Review Prüflücken M-2); deshalb ist jedes davon hart rot.
pub fn macro_attributes(src: &str) -> usize {
    attributes(&tokens(src))
        .into_iter()
        .filter(|inner| inner.contains(&Tok::Punct('$')))
        .count()
}

#[cfg(test)]
#[path = "rust_attrs_tests.rs"]
mod tests;
