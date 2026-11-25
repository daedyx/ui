//! Leisteneintrag und Startfehler-Zustand (Tasks T2): Rollen, Namen, Auswahl, Klick.
use super::support_tests::harness_state;
use super::*;
use eframe::egui::accesskit::Role;
use egui_kittest::kittest::{NodeT, Queryable};

#[test]
fn leisteneintrag_ist_reiter_mit_name_und_auswahl() {
    let mut h = harness_state(0_u8, |ui, clicks| {
        if nav_entry(ui, "Liste", true, false).clicked() {
            *clicks += 1;
        }
        let _ = nav_entry(ui, "Board", false, false);
    });
    h.run();
    let liste = h.get_by_role_and_label(Role::Tab, "Liste");
    assert!(liste.accesskit_node().is_selected() == Some(true));
    let board = h.get_by_role_and_label(Role::Tab, "Board");
    assert!(board.accesskit_node().is_selected() == Some(false));
    h.get_by_role_and_label(Role::Tab, "Liste").click();
    h.run();
    assert_eq!(*h.state(), 1);
}

#[test]
fn schmaler_leisteneintrag_behaelt_den_vollen_namen() {
    let mut h = harness_state((), |ui, ()| {
        let _ = nav_entry(ui, "Einstellungen", false, true);
    });
    h.run();
    let _ = h.get_by_role_and_label(Role::Tab, "Einstellungen");
}

#[test]
fn startfehler_ist_warnmeldung_mit_beenden() {
    let mut h = harness_state(false, |ui, quit| {
        if fatal_state(
            ui,
            "Start nicht möglich",
            "Die Datei ist kaputt.",
            "Beenden",
        ) {
            *quit = true;
        }
    });
    h.run();
    let _ = h.get_by_role_and_label(Role::Alert, "Start nicht möglich");
    assert!(
        h.query_by_role_and_label(Role::Button, "Erneut versuchen")
            .is_none()
    );
    h.get_by_role_and_label(Role::Button, "Beenden").click();
    h.run();
    assert!(*h.state());
}
