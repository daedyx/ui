use super::{Language, Texts};
use crate::fonts;
use eframe::egui::{FontFamily, FontId};

#[test]
fn texte_und_platzhalter() {
    let t = Texts::from_json(r#"{"a":"Hallo {name}!","b":"ohne"}"#).unwrap();
    assert_eq!(t.t("b"), "ohne");
    assert_eq!(t.tf("a", &[("name", "Anna")]), "Hallo Anna!");
}

#[test]
fn fehlender_schluessel_ist_sichtbar_statt_absturz() {
    let t = Texts::from_json("{}").unwrap();
    assert_eq!(t.t("x.y"), "[[x.y]]");
}

#[test]
fn kaputte_oder_verschachtelte_datei_wird_abgelehnt() {
    assert!(Texts::from_json("nein").is_err());
    assert!(Texts::from_json(r#"{"a":{"b":"c"}}"#).is_err());
}

fn placeholders(text: &str) -> Vec<String> {
    let mut v: Vec<String> = text
        .split('{')
        .skip(1)
        .filter_map(|p| p.split('}').next())
        .map(String::from)
        .collect();
    v.sort();
    v
}

/// Englisch und Französisch haben genau die deutschen Schlüssel, mit denselben Platzhaltern.
#[test]
fn alle_sprachen_haben_dieselben_schluessel_und_platzhalter() {
    let de = Texts::for_language(Language::De);
    assert!(de.len() > 20, "deutsche Datei geladen");
    for lang in Language::ALL {
        let other = Texts::for_language(lang);
        assert_eq!(other.len(), de.len(), "{lang:?}: Anzahl Schlüssel");
        for (key, text) in de.entries() {
            assert!(other.has(key), "{lang:?}: {key} fehlt");
            assert_eq!(
                placeholders(text),
                placeholders(&other.t(key)),
                "{lang:?}: Platzhalter {key}"
            );
            assert!(!other.t(key).trim().is_empty(), "{lang:?}: {key} leer");
        }
    }
}

#[test]
fn sprachen_sind_verschieden_und_nennen_sich_selbst() {
    assert_eq!(Language::De.native_name(), "Deutsch");
    assert_eq!(Language::En.native_name(), "English");
    assert_eq!(Language::Fr.native_name(), "Français");
    let key = "knopf.abbrechen";
    assert_ne!(
        Texts::for_language(Language::De).t(key),
        Texts::for_language(Language::En).t(key)
    );
    assert_ne!(
        Texts::for_language(Language::En).t(key),
        Texts::for_language(Language::Fr).t(key)
    );
}

#[test]
fn sprache_aus_der_systemkennung() {
    assert_eq!(Language::from_locale(Some("de-DE")), Language::De);
    assert_eq!(Language::from_locale(Some("DE_at")), Language::De);
    assert_eq!(Language::from_locale(Some("fr-CA")), Language::Fr);
    assert_eq!(Language::from_locale(Some("en-US")), Language::En);
    assert_eq!(Language::from_locale(Some("nl-NL")), Language::En);
    assert_eq!(Language::from_locale(None), Language::De);
}

/// Jedes Zeichen der drei Sprachdateien kann schon die eingebaute Ersatzschrift zeigen
/// (also auch ohne Systemschrift keine Kästchen). Emoji sind bewusst nicht dabei.
#[test]
fn ersatzschrift_kann_alle_zeichen_der_sprachdateien() {
    let ctx = eframe::egui::Context::default();
    ctx.set_fonts(fonts::definitions(|_| None));
    let mut out = ctx.run_ui(Default::default(), |_| {});
    out.textures_delta.clear();
    let probe = FontId::new(14.0, FontFamily::Proportional);
    assert!(
        !ctx.fonts_mut(|f| f.has_glyphs(&probe, "\u{6F22}")),
        "Probe: CJK fehlt wirklich"
    );
    for lang in Language::ALL {
        for (key, text) in Texts::for_language(lang).entries() {
            for family in [FontFamily::Proportional, fonts::semibold()] {
                let id = FontId::new(14.0, family);
                let ok = ctx.fonts_mut(|f| f.has_glyphs(&id, text));
                assert!(
                    ok,
                    "{lang:?} {key}: Zeichen fehlt in der Ersatzschrift: {text}"
                );
            }
        }
    }
}
