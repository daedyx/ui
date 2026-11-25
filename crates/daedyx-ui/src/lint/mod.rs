//! Prüftests über Quelltexte (nur in Tests übersetzt; andere Crates nutzen sie über das Merkmal
//! `lint` als Entwicklungs-Abhängigkeit):
//! 1. jeder im Code benutzte Textschlüssel steht in `de.json`,
//! 2. keine festen Oberflächentexte im Code (alles über i18n),
//! 3. keine Farbwerte außerhalb der Token-Datei.
mod lexer;
#[cfg(test)]
mod tests;

use crate::i18n::{Language, Texts};
use lexer::lex;
use std::path::{Path, PathBuf};

const FILE_ENDINGS: &[&str] = &[".json", ".rs", ".png", ".ttf", ".ttc", ".toml", ".md"];

/// Quelltexte aller Crates im Workspace, ohne Testdateien und ohne diesen Prüfordner. Ein Crate mit eigenen
/// Sprachdateien (`lang/de.json`, z. B. `daedyx-tasks-ui`) prüft sich selbst mit seinen Texten (eigener Wächter-Test).
pub fn source_files() -> Vec<PathBuf> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let crates = here.join("..");
    let own_texts =
        |dir: &Path| dir.join("lang").join("de.json").is_file() && !dir.ends_with("daedyx-ui");
    let dirs: Vec<PathBuf> = std::fs::read_dir(&crates)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|d| !own_texts(d))
                .map(|d| d.join("src"))
                .collect()
        })
        .unwrap_or_default();
    rust_files(&dirs)
}

/// Alle `.rs`-Dateien unter `dirs`, ohne Testdateien (`tests.rs`, `*_tests.rs`, Ordner `tests`)
/// und ohne Ordner `lint`.
pub fn rust_files(dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack: Vec<PathBuf> = dirs.to_vec();
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for path in entries.filter_map(|e| e.ok()).map(|e| e.path()) {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if path.is_dir() {
                if name != "lint" && name != "tests" {
                    stack.push(path);
                }
            } else if name.ends_with(".rs") && name != "tests.rs" && !name.ends_with("_tests.rs") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn is_key(s: &str) -> bool {
    s.contains('.')
        && s.starts_with(|c: char| c.is_ascii_lowercase())
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.')
        && !s.contains("..")
        && !s.ends_with('.')
        && !FILE_ENDINGS.iter().any(|e| s.ends_with(e))
}

/// Regel 1: dotted Schlüssel im Code, die in `de.json` fehlen.
pub fn missing_keys(name: &str, src: &str, texts: &Texts) -> Vec<String> {
    lex(src)
        .strings
        .into_iter()
        .filter(|(_, s)| is_key(s) && !texts.has(s))
        .map(|(line, s)| format!("{name}:{line}: Schlüssel {s:?} fehlt in de.json"))
        .collect()
}

fn without_placeholders(s: &str) -> String {
    let mut out = String::new();
    let mut depth = 0u32;
    for c in s.chars() {
        match c {
            '{' => depth = depth.saturating_add(1),
            '}' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out.replace("\\n", " ").replace("\\t", " ")
}

fn looks_like_prose(s: &str) -> bool {
    let t = without_placeholders(s);
    let words: Vec<&str> = t
        .split_whitespace()
        .filter(|w| w.chars().filter(|c| c.is_alphabetic()).count() >= 2)
        .collect();
    let non_ascii = t.chars().any(|c| c.is_alphabetic() && !c.is_ascii());
    let mut chars = t.chars();
    let capital_word = matches!((chars.next(), chars.next()), (Some(a), Some(b)) if a.is_uppercase() && b.is_lowercase());
    words.len() >= 2 || non_ascii || capital_word
}

/// Regel 2: Zeichenketten im Code, die wie sichtbarer Text aussehen.
pub fn fixed_texts(name: &str, src: &str) -> Vec<String> {
    lex(src)
        .strings
        .into_iter()
        .filter(|(_, s)| looks_like_prose(s))
        .map(|(line, s)| format!("{name}:{line}: fester Text {s:?} (gehört nach lang/*.json)"))
        .collect()
}

fn has_hex_color(s: &str) -> bool {
    s.match_indices('#').any(|(i, _)| {
        let digits = s
            .get(i.saturating_add(1)..)
            .unwrap_or("")
            .chars()
            .take_while(|c| c.is_ascii_hexdigit())
            .count();
        digits == 6 || digits == 8
    })
}

/// Regel 3: Farbwerte im Code statt aus den Tokens.
pub fn color_values(name: &str, src: &str) -> Vec<String> {
    let lexed = lex(src);
    let mut out = Vec::new();
    for (i, line) in lexed.code.lines().enumerate() {
        let banned = [
            "Color32::from_",
            "Rgba::from",
            "Hsva::new",
            "Hsva {",
            "Color32::gray",
        ];
        let constant = line.match_indices("Color32::").any(|(p, m)| {
            let rest = line.get(p.saturating_add(m.len())..).unwrap_or("");
            rest.starts_with(|c: char| c.is_ascii_uppercase()) && !rest.starts_with("TRANSPARENT")
        });
        if constant || banned.iter().any(|b| line.contains(b)) {
            out.push(format!(
                "{name}:{}: Farbwert im Code (nur aus tokens)",
                i.saturating_add(1)
            ));
        }
    }
    for (line, s) in lexed.strings {
        if has_hex_color(&s) {
            out.push(format!(
                "{name}:{line}: Farbcode {s:?} im Code (nur aus tokens)"
            ));
        }
    }
    out
}

/// Alle drei Regeln über den ganzen Workspace.
pub fn check_workspace() -> Vec<String> {
    check_files(
        &source_files(),
        &Texts::for_language(Language::De),
        &["tokens/generated.rs"],
    )
}

/// Alle drei Regeln über `files`. Schlüssel müssen in `texts` stehen; Dateien, deren Pfad auf einen
/// Eintrag in `color_exempt` endet, dürfen Farbwerte enthalten (erzeugte Tokens, Farben aus Mails).
pub fn check_files(files: &[PathBuf], texts: &Texts, color_exempt: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for path in files {
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        let name = path.to_string_lossy().replace('\\', "/");
        let short = name.split("/crates/").last().unwrap_or(&name).to_string();
        out.extend(missing_keys(&short, &src, texts));
        out.extend(fixed_texts(&short, &src));
        if !color_exempt.iter().any(|e| short.ends_with(e)) {
            out.extend(color_values(&short, &src));
        }
    }
    out
}
