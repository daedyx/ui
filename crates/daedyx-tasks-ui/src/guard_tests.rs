//! Wächter über das Crate: jeder benutzte Schlüssel steht in `lang/de.json`, keine festen Texte, Farben nur aus
//! den Tokens; die Sprachen haben dieselben Schlüssel; jeder Fehlercode des Kerns hat einen Text.
use crate::texts::texts;
use daedyx_tasks_core::TaskError;
use daedyx_ui::i18n::Language;
use daedyx_ui::lint::{check_files, rust_files};
use std::path::Path;

#[test]
fn ui_regeln_gelten_fuer_das_aufgaben_crate() {
    let files = rust_files(&[Path::new(env!("CARGO_MANIFEST_DIR")).join("src")]);
    assert!(files.len() > 20, "Quelltexte gefunden: {}", files.len());
    let found = check_files(&files, &texts(Language::De), &[]);
    assert!(found.is_empty(), "Verstöße:\n{}", found.join("\n"));
}

#[test]
fn alle_sprachen_haben_dieselben_aufgaben_schluessel() {
    let de = texts(Language::De);
    assert!(de.len() > 100, "{}", de.len());
    for lang in [Language::En, Language::Fr] {
        let other = texts(lang);
        let a: Vec<&String> = de.entries().map(|(k, _)| k).collect();
        let b: Vec<&String> = other.entries().map(|(k, _)| k).collect();
        assert_eq!(a, b, "{lang:?}");
        assert!(
            other
                .entries()
                .all(|(k, v)| k.starts_with("aufgaben.") && !v.trim().is_empty())
        );
    }
}

#[test]
fn jeder_fehlercode_des_kerns_hat_einen_text_im_crate() {
    let errors = [
        TaskError::ListNotFound,
        TaskError::SectionNotFound,
        TaskError::TaskNotFound,
        TaskError::Invalid("x"),
        TaskError::TooDeep,
        TaskError::Db(rusqlite::Error::InvalidQuery),
    ];
    for lang in Language::ALL {
        let t = texts(lang);
        for e in &errors {
            let key = format!("{}{}", crate::actions::ERROR_PREFIX, e.code());
            assert!(t.has(&key), "{lang:?}: {key}");
        }
    }
}

/// Jeder Text lässt sich mit der Ersatzschrift von `daedyx-ui` zeichnen (ohne Systemschriften). Sonst fehlen Zeichen
/// oder egui stürzt beim Zeichnen ab; Desk prüft dasselbe für seine Kopie der Texte (T3a: „↑“ fehlte).
#[test]
fn ersatzschrift_kann_alle_aufgaben_texte() {
    use eframe::egui::{Context, FontFamily, FontId};
    let ctx = Context::default();
    ctx.set_fonts(daedyx_ui::fonts::definitions(|_| None));
    let mut out = ctx.run_ui(Default::default(), |_| {});
    out.textures_delta.clear();
    let id = FontId::new(14.0, FontFamily::Proportional);
    for lang in Language::ALL {
        for (k, v) in texts(lang).entries() {
            assert!(ctx.fonts_mut(|f| f.has_glyphs(&id, v)), "{lang:?} {k}: {v}");
        }
    }
}
