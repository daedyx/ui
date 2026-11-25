//! Der Testrahmen selbst: feste Uhr, Speicher-Datenbank, Oberfläche ohne App (Tasks T1a).
use crate::test_support::{fixed_store, harness, harness_in, item_label};
use daedyx_tasks_core::smart::View;
use daedyx_ui::i18n::Language;
use eframe::egui::Key;
use eframe::egui::accesskit::Role;
use egui_kittest::kittest::Queryable;

#[test]
fn testrahmen_rechnet_heute_mit_der_festen_uhr() {
    let store = fixed_store();
    assert_eq!(store.today().to_string(), "2026-11-18");
    let today: Vec<String> = store
        .query(View::Today, None)
        .unwrap()
        .into_iter()
        .map(|t| t.title)
        .collect();
    assert!(
        today.contains(&"Wasserzähler ablesen".to_string()),
        "{today:?}"
    );
    assert!(
        !today.contains(&"Fahrrad zur Werkstatt bringen".to_string()),
        "überfällig ist nicht heute"
    );
}

#[test]
fn testrahmen_zeigt_die_leiste_ohne_app() {
    let h = harness(None);
    let _ = h.get_by_role_and_label(Role::List, "Aufgaben-Leiste");
    for start in ["Heute, ", "Geplant, ", "Haushalt", "Projekt Beispiel"] {
        assert!(item_label(&h, start).is_some(), "fehlt: {start}");
    }
}

#[test]
fn schnelleingabe_im_testrahmen_nutzt_die_feste_uhr() {
    let mut h = harness(Some((2, false)));
    h.get_by_role_and_label(Role::TextInput, "Neue Aufgabe")
        .click();
    h.run();
    h.get_by_role_and_label(Role::TextInput, "Neue Aufgabe")
        .type_text("Probe morgen");
    h.run();
    h.key_press(Key::Enter);
    h.run();
    h.run();
    assert_eq!(
        item_label(&h, "Probe, ").as_deref(),
        Some("Probe, Morgen, Keine, offen")
    );
}

#[test]
fn testrahmen_zeigt_englisch_und_franzoesisch() {
    for (language, name) in [(Language::En, "Today, "), (Language::Fr, "Aujourd")] {
        let h = harness_in(None, language);
        assert!(item_label(&h, name).is_some(), "{language:?}");
    }
}
