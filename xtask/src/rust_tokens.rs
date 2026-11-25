//! Zerlegt Rust-Quelltext in Wörter und Satzzeichen für den Ausnahmenzähler. Kommentare (auch verschachtelt)
//! entfallen, Zeichenketten (auch roh, `r#"…"#`) und Zeichen werden zu `Lit`, damit Text in ihnen nie zählt.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tok {
    Ident(String),
    Punct(char),
    /// Zeichenkette mit ihrem Quelltext (samt Anführungszeichen), z. B. für `target_os = "macos"`.
    Lit(String),
}

fn at(c: &[char], i: usize) -> Option<char> {
    c.get(i).copied()
}

/// Ende eines Blockkommentars ab `i` (zeigt auf `/*`), verschachtelt.
fn block_comment_end(c: &[char], mut i: usize) -> usize {
    let mut depth = 0usize;
    while let Some(ch) = at(c, i) {
        let next = at(c, i.saturating_add(1));
        if ch == '/' && next == Some('*') {
            depth = depth.saturating_add(1);
            i = i.saturating_add(2);
        } else if ch == '*' && next == Some('/') {
            depth = depth.saturating_sub(1);
            i = i.saturating_add(2);
            if depth == 0 {
                return i;
            }
        } else {
            i = i.saturating_add(1);
        }
    }
    i
}

/// Ende einer Zeichenkette mit Escapes; `i` zeigt hinter das öffnende `"`.
fn string_end(c: &[char], mut i: usize) -> usize {
    while let Some(ch) = at(c, i) {
        i = i.saturating_add(if ch == '\\' { 2 } else { 1 });
        if ch == '"' {
            break;
        }
    }
    i
}

/// Rohe Zeichenkette `r#"…"#`; `i` zeigt hinter das Präfix (`r`, `br`, `cr`). `None`, wenn keine folgt.
fn raw_string_end(c: &[char], i: usize) -> Option<usize> {
    let hashes = c.get(i..)?.iter().take_while(|x| **x == '#').count();
    let mut j = i.saturating_add(hashes);
    if at(c, j) != Some('"') {
        return None;
    }
    j = j.saturating_add(1);
    while let Some(ch) = at(c, j) {
        j = j.saturating_add(1);
        let closing = c
            .get(j..j.saturating_add(hashes))
            .is_some_and(|h| h.iter().all(|x| *x == '#'));
        if ch == '"' && closing {
            return Some(j.saturating_add(hashes));
        }
    }
    Some(j)
}

/// Zeichen `'x'`, `'\n'` oder Lebensdauer `'a`; liefert die nächste Leseposition.
fn quote_end(c: &[char], i: usize) -> usize {
    let first = at(c, i.saturating_add(1));
    if first == Some('\\') {
        let mut j = i.saturating_add(3);
        while at(c, j).is_some_and(|x| x != '\'') {
            j = j.saturating_add(1);
        }
        return j.saturating_add(1);
    }
    if at(c, i.saturating_add(2)) == Some('\'') {
        i.saturating_add(3)
    } else {
        i.saturating_add(1)
    }
}

pub fn tokens(src: &str) -> Vec<Tok> {
    let c: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while let Some(ch) = at(&c, i) {
        let next = at(&c, i.saturating_add(1));
        if ch.is_whitespace() {
            i = i.saturating_add(1);
        } else if ch == '/' && next == Some('/') {
            while at(&c, i).is_some_and(|x| x != '\n') {
                i = i.saturating_add(1);
            }
        } else if ch == '/' && next == Some('*') {
            i = block_comment_end(&c, i);
        } else if ch == '"' {
            let start = i;
            i = string_end(&c, i.saturating_add(1));
            out.push(Tok::Lit(
                c.get(start..i).unwrap_or_default().iter().collect(),
            ));
        } else if ch == '\'' {
            i = quote_end(&c, i);
        } else if ch.is_alphanumeric() || ch == '_' {
            let len = c
                .get(i..)
                .unwrap_or_default()
                .iter()
                .take_while(|x| x.is_alphanumeric() || **x == '_')
                .count();
            let word: String = c
                .get(i..i.saturating_add(len))
                .unwrap_or_default()
                .iter()
                .collect();
            i = i.saturating_add(len);
            let raw = if matches!(word.as_str(), "r" | "br" | "cr") {
                raw_string_end(&c, i)
            } else {
                None
            };
            if let Some(end) = raw {
                let text: String = c.get(i..end).unwrap_or_default().iter().collect();
                i = end;
                out.push(Tok::Lit(text));
            } else {
                out.push(Tok::Ident(word));
            }
        } else {
            out.push(Tok::Punct(ch));
            i = i.saturating_add(1);
        }
    }
    out
}
