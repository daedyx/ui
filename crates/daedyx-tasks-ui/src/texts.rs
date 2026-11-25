//! Texte der Aufgaben-Oberfläche (Schlüssel `aufgaben.*` in `lang/de.json`, `en.json`, `fr.json`). Die App führt
//! dieselben Schlüssel mit demselben Wortlaut in ihren eigenen Sprachdateien (Desk prüft das per Test) und reicht
//! ihre Texte an [`crate::show`] durch.
use daedyx_ui::i18n::{Language, Texts};

/// Sprachdatei des Crates.
pub fn source(language: Language) -> &'static str {
    match language {
        Language::De => include_str!("../lang/de.json"),
        Language::En => include_str!("../lang/en.json"),
        Language::Fr => include_str!("../lang/fr.json"),
    }
}

/// Texte einer Sprache. Eine kaputte Datei (verhindert ein Test) zeigt `[[schlüssel]]` statt abzustürzen.
pub fn texts(language: Language) -> Texts {
    Texts::from_json(source(language)).unwrap_or_default()
}

/// Quellordner dieses Crates, damit der Wächter der App (feste Texte, Farben, Schlüssel) auch hier prüft. Nur mit
/// dem Merkmal `lint` (Entwicklungs-Abhängigkeit), nie im fertigen Programm.
#[cfg(feature = "lint")]
pub fn source_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}
