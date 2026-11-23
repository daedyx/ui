use super::support_tests::{harness, harness_state};
use super::*;
use eframe::egui::{Key, accesskit::Role};
use egui_kittest::kittest::{NodeT, Queryable};

#[test]
fn knopf_hat_rolle_beschriftung_und_meldet_klick() {
    let mut h = harness_state(0_u32, |ui, clicks| {
        for kind in [
            ButtonKind::Primary,
            ButtonKind::Secondary,
            ButtonKind::Quiet,
            ButtonKind::Danger,
        ] {
            if button(ui, kind, &format!("Knopf {kind:?}")).clicked() {
                *clicks += 1;
            }
        }
        let _ = button_enabled(ui, ButtonKind::Primary, "Gesperrt", false);
    });
    h.run();
    for kind in ["Primary", "Secondary", "Quiet", "Danger"] {
        let _ = h.get_by_role_and_label(Role::Button, &format!("Knopf {kind}"));
    }
    assert!(
        h.get_by_role_and_label(Role::Button, "Gesperrt")
            .accesskit_node()
            .is_disabled()
    );
    h.get_by_role_and_label(Role::Button, "Knopf Primary")
        .click();
    h.run();
    assert_eq!(*h.state(), 1);
}

#[test]
fn knopf_ist_so_hoch_wie_das_bedienelement_token() {
    let mut h = harness(|ui| {
        let r = button(ui, ButtonKind::Secondary, "OK");
        assert_eq!(r.rect.height(), crate::tokens::TOKENS.size.control);
        assert!(r.rect.width() >= 64.0);
    });
    h.run();
}

#[test]
fn eingabe_hat_rolle_beschriftung_und_fehler_als_beschreibung() {
    let mut h = harness_state(String::new(), |ui, text| {
        let _ = text_input(
            ui,
            text,
            "Betreff",
            "Worum geht es?",
            Some("Bitte einen Betreff eingeben."),
        );
    });
    h.run();
    let field = h.get_by_role_and_label(Role::TextInput, "Betreff");
    assert_eq!(
        field.accesskit_node().description().as_deref(),
        Some("Bitte einen Betreff eingeben.")
    );
    field.focus();
    h.run();
    h.get_by_role_and_label(Role::TextInput, "Betreff")
        .type_text("Probe");
    h.run();
    assert_eq!(h.state(), "Probe");
}

#[test]
fn eingabe_begrenzt_die_laenge() {
    let mut h = harness_state(String::new(), |ui, text| {
        let _ = text_input(ui, text, "Notiz", "", None);
    });
    h.run();
    h.get_by_role_and_label(Role::TextInput, "Notiz").focus();
    h.run();
    h.get_by_role_and_label(Role::TextInput, "Notiz")
        .type_text(&"x".repeat(MAX_INPUT_CHARS + 50));
    h.run();
    assert_eq!(h.state().chars().count(), MAX_INPUT_CHARS);
}

const ROWS: [ListRow<'static>; 3] = [
    ListRow {
        title: "Wochenplan entwerfen",
        detail: "Morgen",
        unread: true,
    },
    ListRow {
        title: "Material bestellen",
        detail: "Freitag",
        unread: false,
    },
    ListRow {
        title: "Rückmeldung zum Entwurf",
        detail: "Nächste Woche",
        unread: false,
    },
];

#[test]
fn liste_hat_rollen_auswahl_und_pfeiltasten() {
    let mut h = harness_state(None::<usize>, |ui, sel| {
        let _ = list(ui, "Aufgaben", &ROWS, sel);
    });
    h.run();
    let _ = h.get_by_role_and_label(Role::ListBox, "Aufgaben");
    assert_eq!(h.query_all_by_role(Role::ListBoxOption).count(), 3);
    h.get_by_role_and_label(Role::ListBoxOption, "Material bestellen")
        .click();
    h.run();
    assert_eq!(*h.state(), Some(1));
    let row = h.get_by_role_and_label(Role::ListBoxOption, "Material bestellen");
    assert_eq!(row.accesskit_node().is_selected(), Some(true));
    row.focus();
    h.run();
    h.key_press(Key::ArrowDown);
    h.run();
    assert_eq!(
        *h.state(),
        Some(2),
        "Pfeil nach unten wählt die nächste Zeile"
    );
    let row = h.get_by_role_and_label(Role::ListBoxOption, "Rückmeldung zum Entwurf");
    assert_eq!(
        row.parent().map(|p| p.accesskit_node().role()),
        Some(Role::ListBox),
        "Zeile hängt an der Liste"
    );
}

#[test]
fn karte_hat_rolle_titel_und_auswahl() {
    let mut h = harness_state(false, |ui, picked| {
        if card(ui, "Übungsplan Oktober", "Drei Termine.", *picked).clicked() {
            *picked = true;
        }
    });
    h.run();
    let node = h.get_by_role_and_label(Role::Group, "Übungsplan Oktober");
    assert_eq!(node.accesskit_node().is_selected(), Some(false));
    node.click();
    h.run();
    h.run();
    let node = h.get_by_role_and_label(Role::Group, "Übungsplan Oktober");
    assert_eq!(node.accesskit_node().is_selected(), Some(true));
}

#[test]
fn marken_haben_rolle_und_beschriftung() {
    let tones = [
        BadgeTone::Neutral,
        BadgeTone::Accent,
        BadgeTone::Ok,
        BadgeTone::Warning,
        BadgeTone::Error,
        BadgeTone::Info,
    ];
    let mut h = harness(move |ui| {
        for tone in tones {
            let _ = badge(ui, tone, &format!("Marke {tone:?}"));
        }
    });
    h.run();
    for tone in tones {
        let label = format!("Marke {tone:?}");
        let node = h.get_by_role_and_label(Role::Label, &label);
        assert_eq!(node.accesskit_node().role(), Role::Label);
    }
}
