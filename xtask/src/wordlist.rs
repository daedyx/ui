//! Lokale Wortliste mit echten Namen, Orten und Domains. Sie ist selbst personenbezogen und liegt deshalb
//! nie im Repository und nie in CI, sondern nur auf dem eigenen Rechner (SICHERHEIT 3.2).
//! Ort: Umgebungsvariable `DAEDYX_SCAN_WORDS`, sonst `%APPDATA%\Daedyx\verboten.txt` (Windows)
//! bzw. `~/.config/daedyx/verboten.txt`. Fehlt die Datei, wird ohne Wortliste geprüft.
use std::path::PathBuf;

fn default_path() -> Option<PathBuf> {
    if cfg!(windows) {
        std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("Daedyx").join("verboten.txt"))
    } else {
        std::env::var_os("HOME").map(|h| {
            PathBuf::from(h)
                .join(".config")
                .join("daedyx")
                .join("verboten.txt")
        })
    }
}

/// Ein Wort je Zeile; leere Zeilen und Zeilen mit `#` am Anfang zählen nicht. Kleingeschrieben.
pub fn parse(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_lowercase)
        .collect()
}

/// Wörter und ob eine Liste gefunden wurde.
pub fn load() -> (Vec<String>, bool) {
    let path = std::env::var_os("DAEDYX_SCAN_WORDS")
        .map(PathBuf::from)
        .or_else(default_path);
    match path.and_then(|p| std::fs::read_to_string(p).ok()) {
        Some(text) => (parse(&text), true),
        None => (Vec::new(), false),
    }
}
