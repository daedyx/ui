use super::support_tests::{harness, harness_state};
use super::*;
use eframe::egui::{Key, accesskit::Role};
use egui_kittest::kittest::Queryable;

const DIALOG: ConfirmDialog<'static> = ConfirmDialog {
    id_salt: "probe-dialog",
    title: "Entwurf löschen?",
    text: "Der Entwurf wird endgültig entfernt.",
    confirm: "Löschen",
    cancel: "Abbrechen",
    danger: true,
};

fn dialog_harness() -> egui_kittest::Harness<'static, Option<DialogAnswer>> {
    harness_state(None::<DialogAnswer>, |ui, answer| {
        if let Some(a) = confirm_dialog(ui.ctx(), &DIALOG) {
            *answer = Some(a);
        }
    })
}

#[test]
fn dialog_hat_rolle_titel_und_knoepfe() {
    let mut h = dialog_harness();
    h.run();
    let _ = h.get_by_role_and_label(Role::Dialog, "Entwurf löschen?");
    let _ = h.get_by_role_and_label(Role::Button, "Abbrechen");
    h.get_by_role_and_label(Role::Button, "Löschen").click();
    h.run();
    assert_eq!(*h.state(), Some(DialogAnswer::Confirm));
}

#[test]
fn dialog_schliesst_mit_escape() {
    let mut h = dialog_harness();
    h.run();
    h.key_press(Key::Escape);
    h.run();
    assert_eq!(*h.state(), Some(DialogAnswer::Cancel));
}

fn items() -> Vec<MenuItem<'static>> {
    vec![
        MenuItem {
            label: "Umbenennen",
            shortcut: None,
            danger: false,
        },
        MenuItem {
            label: "Kopieren",
            shortcut: Some("Strg+C".into()),
            danger: false,
        },
        MenuItem {
            label: "Löschen",
            shortcut: Some("Entf".into()),
            danger: true,
        },
    ]
}

#[test]
fn menue_oeffnet_mit_rollen_und_meldet_auswahl() {
    let mut h = harness_state(None::<usize>, |ui, chosen| {
        if let Some(i) = menu_button(ui, "Weitere Aktionen", &items()) {
            *chosen = Some(i);
        }
    });
    h.run();
    assert!(
        h.query_by_role(Role::Menu).is_none(),
        "zu Beginn geschlossen"
    );
    h.get_by_role_and_label(Role::Button, "Weitere Aktionen")
        .click();
    h.run();
    let _ = h.get_by_role_and_label(Role::Menu, "Weitere Aktionen");
    assert_eq!(h.query_all_by_role(Role::MenuItem).count(), 3);
    h.get_by_role_and_label(Role::MenuItem, "Kopieren").click();
    h.run();
    h.run();
    assert_eq!(*h.state(), Some(1));
    assert!(
        h.query_by_role(Role::Menu).is_none(),
        "nach Auswahl geschlossen"
    );
}

#[test]
fn leerer_zustand_hat_rolle_titel_und_knopf() {
    let mut h = harness_state(false, |ui, pressed| {
        if empty_state(
            ui,
            "Noch keine Aufgaben",
            "Lege die erste an.",
            Some("Aufgabe anlegen"),
        ) {
            *pressed = true;
        }
    });
    h.run();
    let _ = h.get_by_role_and_label(Role::Group, "Noch keine Aufgaben");
    h.get_by_role_and_label(Role::Button, "Aufgabe anlegen")
        .click();
    h.run();
    assert!(*h.state());
}

#[test]
fn fehlerzustand_ist_warnmeldung_mit_zwei_knoepfen() {
    let mut h = harness_state(None::<ErrorAction>, |ui, action| {
        let answer = error_state(
            ui,
            "Abruf fehlgeschlagen",
            "Gleich noch einmal.",
            "Erneut versuchen",
            Some("Konto prüfen"),
        );
        if let Some(a) = answer {
            *action = Some(a);
        }
    });
    h.run();
    let _ = h.get_by_role_and_label(Role::Alert, "Abruf fehlgeschlagen");
    h.get_by_role_and_label(Role::Button, "Konto prüfen")
        .click();
    h.run();
    assert_eq!(*h.state(), Some(ErrorAction::Secondary));
}

#[test]
fn tab_gibt_einem_knopf_den_fokus() {
    let mut h = harness(|ui| {
        let _ = button(ui, ButtonKind::Primary, "Eins");
    });
    h.run();
    h.key_press(Key::Tab);
    h.run();
    assert!(h.ctx.memory(|m| m.focused()).is_some());
}
