//! Wächter über das Crate (eigener Wächter, `OWN_GUARD` in daedyx-ui): jeder benutzte Schlüssel steht in
//! `lang/de.json`, keine festen Texte, Farben nur aus den Tokens; die Sprachen haben dieselben Schlüssel. `ids.rs`
//! enthält nur technische Kennungen für Windows (Registry-Namen, Schalter), keine Oberflächentexte; sie wird wie in
//! Desk nur auf Farbwerte geprüft.
use crate::texts::texts;
use daedyx_ui::i18n::Language;
use daedyx_ui::lint::{check_files, color_values, rust_files};
use std::path::Path;

#[test]
fn ui_regeln_gelten_fuer_den_hintergrund() {
    let files = rust_files(&[Path::new(env!("CARGO_MANIFEST_DIR")).join("src")]);
    let (ids, rest): (Vec<_>, Vec<_>) = files.into_iter().partition(|f| f.ends_with("ids.rs"));
    assert_eq!(ids.len(), 1);
    assert!(rest.len() > 10, "Quelltexte gefunden: {}", rest.len());
    let src = std::fs::read_to_string(&ids[0]).unwrap();
    assert!(color_values("ids.rs", &src).is_empty());
    let found = check_files(&rest, &texts(Language::De), &[]);
    assert!(found.is_empty(), "Verstöße:\n{}", found.join("\n"));
}

#[test]
fn alle_sprachen_haben_dieselben_hintergrund_schluessel() {
    let de = texts(Language::De);
    assert_eq!(de.len(), 14, "8 aus Desk, 6 für den Vorlauf (Tasks T4)");
    for lang in [Language::En, Language::Fr] {
        let other = texts(lang);
        let a: Vec<&String> = de.entries().map(|(k, _)| k).collect();
        let b: Vec<&String> = other.entries().map(|(k, _)| k).collect();
        assert_eq!(a, b, "{lang:?}");
        assert!(
            other
                .entries()
                .all(|(k, v)| k.starts_with("hintergrund.") && !v.trim().is_empty())
        );
    }
}
