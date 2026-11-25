//! Wächter über das Crate (eigener Wächter, `OWN_GUARD` in daedyx-ui): keine Oberflächentexte, keine Farbwerte.
//! `link.rs` enthält nur Linker-Optionen und Cargo-Zeilen (früher Desk `build.rs`, den kein Wächter las); sie
//! wird nur auf Farbwerte geprüft.
use daedyx_ui::i18n::Texts;
use daedyx_ui::lint::{check_files, color_values, rust_files};
use std::path::Path;

#[test]
fn ui_regeln_gelten_fuer_den_bau_helfer() {
    let files = rust_files(&[Path::new(env!("CARGO_MANIFEST_DIR")).join("src")]);
    let (link, rest): (Vec<_>, Vec<_>) = files.into_iter().partition(|f| f.ends_with("link.rs"));
    assert_eq!(link.len(), 1);
    assert!(rest.len() >= 2, "Quelltexte gefunden: {}", rest.len());
    let src = std::fs::read_to_string(&link[0]).unwrap();
    assert!(color_values("link.rs", &src).is_empty());
    let found = check_files(&rest, &Texts::default(), &[]);
    assert!(found.is_empty(), "Verstöße:\n{}", found.join("\n"));
}
