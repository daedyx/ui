//! Mehrere Erinnerungen (Tasks T3a, Vorlagen `AlarmFields.test.tsx` und `TasksDeep.test.tsx`): sortiert, ohne
//! Doppelte, höchstens zehn, ohne Fälligkeit gesperrt, Fälligkeit entfernen löscht sie. Nur erfundene Aufgaben.
use crate::test_support::{click, harness_tasks, id_of, select, type_into};
use eframe::egui::accesskit::Role;
use egui_kittest::kittest::{NodeT, Queryable};

const BOOK: &str = "Buch über Wolken zurückgeben";

fn add(h: &mut egui_kittest::Harness<'static, crate::TasksUi>, preset: &str) {
    click(h, Role::Button, "Erinnerung hinzufügen");
    click(h, Role::MenuItem, preset);
}

#[test]
fn legt_zwei_an_behaelt_sie_beim_erneuten_oeffnen_und_entfernt_eine() {
    let mut h = harness_tasks(Some((2, false)));
    select(&mut h, BOOK);
    add(&mut h, "1 Tag vorher");
    add(&mut h, "zur Fälligkeit");
    let id = id_of(&h, BOOK);
    assert_eq!(h.state().alarms_of(&id), [0, 1440], "aufsteigend");
    select(&mut h, "Artikel über Gezeiten");
    select(&mut h, BOOK);
    assert_eq!(h.state().draft.alarms, [0, 1440]);
    // Schon vorhandene Vorlagen werden nicht mehr angeboten.
    click(&mut h, Role::Button, "Erinnerung hinzufügen");
    assert_eq!(
        h.query_all_by_role_and_label(Role::MenuItem, "1 Tag vorher")
            .count(),
        0
    );
    h.key_press(eframe::egui::Key::Escape);
    h.run();
    click(
        &mut h,
        Role::Button,
        "Erinnerung „zur Fälligkeit“ entfernen",
    );
    assert_eq!(h.state().alarms_of(&id), [1440]);
}

#[test]
fn zeigt_ungewoehnliche_werte_lesbar_und_ohne_doppelte() {
    let mut h = harness_tasks(Some((2, false)));
    let id = id_of(&h, BOOK);
    h.state_mut().set_alarms(&id, &[7, 60, 7]);
    assert_eq!(h.state().alarms_of(&id), [7, 60]);
    select(&mut h, BOOK);
    let _ = h.get_by_label("7 Minuten vorher");
    let _ = h.get_by_label("1 Stunde vorher");
}

#[test]
fn zehn_gehen_die_elfte_wird_abgelehnt_und_nichts_mehr_angeboten() {
    let mut h = harness_tasks(Some((2, false)));
    let id = id_of(&h, BOOK);
    let ten: Vec<i64> = (1..=10).collect();
    h.state_mut().set_alarms(&id, &ten);
    assert_eq!(h.state().alarms_of(&id).len(), 10);
    let eleven: Vec<i64> = (1..=11).collect();
    h.state_mut().set_alarms(&id, &eleven);
    assert_eq!(h.state().error.as_deref(), Some("aufgaben.fehler.invalid"));
    assert_eq!(h.state().alarms_of(&id).len(), 10);
    select(&mut h, BOOK);
    assert_eq!(h.query_all_by_label("Erinnerung hinzufügen").count(), 0);
    let _ = h.get_by_label("Höchstens 10 Erinnerungen je Aufgabe.");
}

#[test]
fn ohne_faelligkeit_ist_alles_gesperrt_mit_hinweis() {
    let mut h = harness_tasks(Some((2, false)));
    select(&mut h, "Artikel über Gezeiten");
    let _ = h.get_by_label("Erinnerungen brauchen eine Fälligkeit.");
    let add = h.get_by_role_and_label(Role::Button, "Erinnerung hinzufügen");
    assert!(add.accesskit_node().is_disabled());
}

#[test]
fn faelligkeit_entfernen_loescht_auch_die_erinnerungen() {
    let mut h = harness_tasks(Some((2, false)));
    let id = id_of(&h, BOOK);
    h.state_mut().set_alarms(&id, &[0, 60]);
    select(&mut h, BOOK);
    type_into(&mut h, "Fällig am", "");
    let t = h
        .state()
        .items
        .iter()
        .find(|t| t.id == id)
        .cloned()
        .unwrap();
    assert_eq!(t.due, None);
    assert!(h.state().alarms_of(&id).is_empty());
}
