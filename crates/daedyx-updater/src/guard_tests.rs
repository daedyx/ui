//! Wächter über das Crate: jeder benutzte Schlüssel steht in `lang/de.json`, keine festen Texte, Farben nur aus den
//! Tokens; die Sprachen haben dieselben Schlüssel. `net.rs` und `install.rs` enthalten nur technische Kennungen
//! (User-Agent, Setup-Schalter) und werden nur auf Farbwerte geprüft.
use crate::texts::texts;
use daedyx_ui::i18n::Language;
use daedyx_ui::lint::{check_files, color_values, rust_files};
use std::path::Path;

#[test]
fn ui_regeln_gelten_fuer_das_update_bauteil() {
    let files = rust_files(&[Path::new(env!("CARGO_MANIFEST_DIR")).join("src")]);
    let technical = |f: &std::path::PathBuf| {
        ["net.rs", "install.rs", "test_support.rs"]
            .iter()
            .any(|n| f.ends_with(n))
            || f.to_string_lossy().ends_with("_tests.rs")
    };
    let (tech, rest): (Vec<_>, Vec<_>) = files.into_iter().partition(technical);
    assert!(rest.len() >= 6, "Quelltexte gefunden: {}", rest.len());
    for f in &tech {
        let src = std::fs::read_to_string(f).unwrap();
        assert!(
            color_values(&f.to_string_lossy(), &src).is_empty(),
            "{}",
            f.display()
        );
    }
    let found = check_files(&rest, &texts(Language::De), &[]);
    assert!(found.is_empty(), "Verstöße:\n{}", found.join("\n"));
}

#[test]
fn alle_sprachen_haben_dieselben_update_schluessel() {
    let de = texts(Language::De);
    assert_eq!(de.len(), 31);
    for lang in [Language::En, Language::Fr] {
        let other = texts(lang);
        let a: Vec<&String> = de.entries().map(|(k, _)| k).collect();
        let b: Vec<&String> = other.entries().map(|(k, _)| k).collect();
        assert_eq!(a, b, "{lang:?}");
        assert!(
            other
                .entries()
                .all(|(k, v)| k.starts_with("update.") && !v.trim().is_empty())
        );
    }
}
