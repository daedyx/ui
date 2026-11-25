//! Harte Verstöße in den geprüften TOML-Dateien (Review Prüflücken M-1): mehrzeilige Texte, doppelte Schlüssel und
//! doppelte Abschnitte. Sie liest ein echter TOML-Leser anders (oder gar nicht) als `toml_lite`, also werden sie nie
//! geprüft, sondern abgelehnt.
use crate::toml_lite::{Quotes, compact, items, strip_comment};

/// Kommt außerhalb einer Zeichenkette `"""` oder `'''` vor? Mehrzeilige Texte kennt dieser Leser nicht; in ihnen
/// ließe sich eine falsche Einstellung vor der echten verstecken (Review Prüflücken M-1).
fn has_multi_line_string(text: &str) -> bool {
    text.lines().any(|line| {
        let chars: Vec<char> = strip_comment(line).chars().collect();
        let mut q = Quotes::default();
        chars.iter().enumerate().any(|(i, c)| {
            let outside_before = q.open.is_none();
            q.outside(*c);
            let triple = chars
                .get(i..i.saturating_add(3))
                .is_some_and(|t| t.iter().all(|x| x == c));
            outside_before && (*c == '"' || *c == '\'') && triple
        })
    })
}

/// Harte Verstöße: mehrzeilige Texte und doppelte Schlüssel oder Abschnitte. Beides liest ein echter TOML-Leser anders
/// (oder gar nicht) als dieser, also wird es nie geprüft, sondern abgelehnt.
pub fn problems(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    if has_multi_line_string(text) {
        out.push(
            "mehrzeiliger Text (\"\"\" oder ''') ist in Prüf-Dateien nicht erlaubt".to_string(),
        );
    }
    let mut seen = std::collections::BTreeSet::new();
    for i in items(text) {
        if !seen.insert(i.key.clone()) {
            out.push(format!("Schlüssel `{}` steht doppelt", i.key));
        }
    }
    let mut headers = std::collections::BTreeSet::new();
    for line in text.lines().map(|l| strip_comment(l).trim()) {
        if line.starts_with('[') && !line.starts_with("[[") && !headers.insert(compact(line)) {
            out.push(format!("Abschnitt `{line}` steht doppelt"));
        }
    }
    out
}
