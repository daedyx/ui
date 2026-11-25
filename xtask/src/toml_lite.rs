//! Kleiner TOML-Leser nur für die eigenen Prüfungen (`deny.toml`, `clippy.toml`, `Cargo.toml`, `.cargo/config.toml`).
//! Liefert je Wert den vollen Schlüssel (`abschnitt.schlüssel`) und den Rohwert ohne Kommentare; mehrzeilige Felder
//! werden zusammengefügt. Kein vollständiger TOML-Parser (keine `"""`-Texte): Unbekanntes bleibt als Rohtext stehen
//! und wird dadurch von den Prüfungen als Abweichung gemeldet, nicht still übergangen.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub key: String,
    pub value: String,
}

/// Merkt sich, ob ein Zeichen in einer Zeichenkette steht (`"…"` mit Escapes oder `'…'` ohne).
#[derive(Default)]
pub(crate) struct Quotes {
    pub(crate) open: Option<char>,
    escaped: bool,
}

impl Quotes {
    /// `true`, wenn das Zeichen außerhalb einer Zeichenkette steht (Anführungszeichen selbst zählen als innen).
    pub(crate) fn outside(&mut self, c: char) -> bool {
        match self.open {
            Some(q) => {
                if self.escaped {
                    self.escaped = false;
                } else if c == '\\' && q == '"' {
                    self.escaped = true;
                } else if c == q {
                    self.open = None;
                }
                false
            }
            None if c == '"' || c == '\'' => {
                self.open = Some(c);
                false
            }
            None => true,
        }
    }
}

pub(crate) fn strip_comment(line: &str) -> &str {
    let mut q = Quotes::default();
    line.char_indices()
        .find(|(_, c)| q.outside(*c) && *c == '#')
        .map_or(line, |(i, _)| line.get(..i).unwrap_or(line))
}

/// Klammertiefe (`[`/`{` minus `]`/`}`) außerhalb von Zeichenketten.
fn depth(s: &str) -> i64 {
    let mut q = Quotes::default();
    s.chars()
        .filter(|c| q.outside(*c))
        .fold(0i64, |d, c| match c {
            '[' | '{' => d.saturating_add(1),
            ']' | '}' => d.saturating_sub(1),
            _ => d,
        })
}

fn clean_key(k: &str) -> String {
    split_top(k, '.')
        .iter()
        .map(|p| unquote(p))
        .collect::<Vec<_>>()
        .join(".")
}

/// Alle Werte mit vollem Schlüssel, in Reihenfolge.
pub fn items(text: &str) -> Vec<Item> {
    let mut out = Vec::new();
    let mut section = String::new();
    let mut pending: Option<(String, String, i64)> = None;
    let mut tables: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for raw in text.lines() {
        let line = strip_comment(raw).trim();
        if let Some((key, mut value, d)) = pending.take() {
            value.push(' ');
            value.push_str(line);
            let d = d.saturating_add(depth(line));
            if d > 0 {
                pending = Some((key, value, d))
            } else {
                out.push(Item { key, value })
            }
            continue;
        }
        if line.starts_with('[') {
            section = clean_key(line.trim_start_matches('[').trim_end_matches(']'));
            if line.starts_with("[[") {
                // Liste von Tabellen (`[[bin]]`): jeder Eintrag bekommt eine eigene Nummer, damit gleiche Schlüssel
                // in verschiedenen Einträgen nicht als doppelt gelten.
                let n = tables.entry(section.clone()).or_insert(0usize);
                *n = n.saturating_add(1);
                section = format!("{section}#{n}");
            }
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = if section.is_empty() {
            clean_key(k)
        } else {
            format!("{section}.{}", clean_key(k))
        };
        let value = v.trim().to_string();
        let d = depth(&value);
        if d > 0 {
            pending = Some((key, value, d))
        } else {
            out.push(Item { key, value })
        }
    }
    if let Some((key, value, _)) = pending {
        out.push(Item { key, value });
    }
    out
}

/// Inhalte aller Zeichenketten eines Rohwerts, in Reihenfolge.
pub fn strings(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    let (mut open, mut escaped, mut cur) = (None::<char>, false, String::new());
    for c in value.chars() {
        match open {
            None if c == '"' || c == '\'' => open = Some(c),
            None => {}
            Some(_) if escaped => {
                cur.push(c);
                escaped = false;
            }
            Some('"') if c == '\\' => escaped = true,
            Some(q) if c == q => {
                out.push(std::mem::take(&mut cur));
                open = None;
            }
            Some(_) => cur.push(c),
        }
    }
    out
}

/// Teile von `s`, getrennt durch `sep` außerhalb von Zeichenketten und Klammern; leere Teile entfallen.
pub fn split_top(s: &str, sep: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut q = Quotes::default();
    let (mut d, mut cur) = (0i64, String::new());
    for c in s.chars() {
        if q.outside(c) {
            match c {
                '[' | '{' | '(' => d = d.saturating_add(1),
                ']' | '}' | ')' => d = d.saturating_sub(1),
                _ if c == sep && d == 0 => {
                    parts.push(std::mem::take(&mut cur));
                    continue;
                }
                _ => {}
            }
        }
        cur.push(c);
    }
    parts.push(cur);
    parts
        .into_iter()
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

/// Oberste Elemente einer Liste `[a, { … }, "b"]` als Rohtext.
pub fn elements(value: &str) -> Vec<String> {
    let v = value.trim();
    split_top(
        v.strip_prefix('[')
            .and_then(|x| x.strip_suffix(']'))
            .unwrap_or(v),
        ',',
    )
}

/// Rohwert des Felds `key` in einer Inline-Tabelle `{ key = …, … }`.
pub fn field(table: &str, key: &str) -> Option<String> {
    let inner = table.trim().strip_prefix('{')?.strip_suffix('}')?;
    split_top(inner, ',').into_iter().find_map(|part| {
        let (k, v) = part.split_once('=')?;
        (unquote(k) == key).then(|| v.trim().to_string())
    })
}

/// Erste Zeichenkette eines Rohwerts, sonst der Rohwert selbst (z. B. `true`, `0.9`).
pub fn unquote(value: &str) -> String {
    strings(value)
        .into_iter()
        .next()
        .unwrap_or_else(|| value.trim().to_string())
}

/// Rohwert ohne Leerraum, für Vergleiche und Meldungen.
pub fn compact(value: &str) -> String {
    value.chars().filter(|c| !c.is_whitespace()).collect()
}

#[cfg(test)]
#[path = "toml_lite_tests.rs"]
mod tests;
