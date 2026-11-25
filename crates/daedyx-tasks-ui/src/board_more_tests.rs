//! Board und Suche in Tasks T3b (Vorlagen `TaskBoard.test.tsx`, `TasksArea.test.tsx`, `useDebouncedField.test.tsx`):
//! „+ Karte“ je Spalte, Board-Modus je Liste gemerkt, Suchfeld mit Treffern und Ende der Suche, Felder speichern
//! beim Verlassen und mit Enter ohne Verlust beim Wechsel der Aufgabe. Nur erfundene Aufgaben.
use crate::test_support::{click, harness, harness_tasks, id_of, item_label, select, type_into};
use crate::{Mode, Sel};
use eframe::egui::accesskit::Role;
use egui_kittest::kittest::Queryable;

#[test]
fn plus_karte_legt_die_karte_in_ihrer_spalte_an_und_bleibt_offen() {
    let mut h = harness_tasks(Some((0, true)));
    click(&mut h, Role::Button, "Karte in „Später“ hinzufügen");
    type_into(&mut h, "Neue Karte in „Später“", "Fenster putzen !");
    let later = h
        .state()
        .sections
        .iter()
        .find(|s| s.name == "Später")
        .map(|s| s.id.clone());
    let t = h
        .state()
        .items
        .iter()
        .find(|t| t.title == "Fenster putzen")
        .cloned()
        .unwrap();
    assert_eq!(t.section_id, later);
    assert_eq!(t.priority, 1, "Schnelleingabe gilt auch hier");
    let _ = h.get_by_role_and_label(Role::TextInput, "Neue Karte in „Später“");
    assert_eq!(
        h.query_all_by_label("Sortierung: von Hand").count(),
        0,
        "keine Sortierung im Board"
    );
    h.key_press(eframe::egui::Key::Escape);
    h.run();
    h.run();
    assert_eq!(h.state().adding_card, None);
}

#[test]
fn board_modus_wird_je_liste_gemerkt() {
    let mut h = harness_tasks(Some((0, true)));
    let ids: Vec<String> = h.state().lists.iter().map(|l| l.id.clone()).collect();
    h.state_mut().select(Sel::List(ids[1].clone()));
    h.run();
    assert_eq!(h.state().mode, Mode::List);
    h.state_mut().select(Sel::List(ids[0].clone()));
    h.run();
    assert_eq!(h.state().mode, Mode::Board, "Haushalt war als Board offen");
}

#[test]
fn desk_ohne_plus_karte_und_ohne_gedaechtnis() {
    let mut h = harness(Some((0, true)));
    assert_eq!(h.query_all_by_label("+ Karte").count(), 0);
    let ids: Vec<String> = h.state().lists.iter().map(|l| l.id.clone()).collect();
    h.state_mut().select(Sel::List(ids[1].clone()));
    h.run();
    h.state_mut().select(Sel::List(ids[0].clone()));
    h.run();
    assert_eq!(
        h.state().mode,
        Mode::Board,
        "Desk lässt den Modus beim Listenwechsel stehen (wie M6a)"
    );
}

#[test]
fn suchfeld_zeigt_treffer_und_leeren_beendet_die_suche() {
    let mut h = harness_tasks(None);
    type_into(&mut h, "Aufgaben suchen", "Gezeiten");
    let _ = h.get_by_label("Suche: „Gezeiten“");
    assert!(item_label(&h, "Artikel über Gezeiten, ").is_some());
    assert_eq!(h.state().items.len(), 1);
    type_into(&mut h, "Aufgaben suchen", "");
    assert!(h.query_all_by_label("Heute").count() >= 1);
    assert_eq!(h.query_all_by_label("Suche: „Gezeiten“").count(), 0);
    assert!(item_label(&h, "Wasserzähler ablesen, ").is_some());
}

#[test]
fn feld_speichert_mit_enter_und_beim_verlassen() {
    let mut h = harness_tasks(Some((2, false)));
    select(&mut h, "Artikel über Gezeiten");
    type_into(&mut h, "Titel", "Artikel über Wolken");
    assert!(
        h.state()
            .items
            .iter()
            .any(|t| t.title == "Artikel über Wolken"),
        "Enter"
    );
    let field = h.get_by_role_and_label(Role::TextInput, "Titel");
    field.click();
    h.run();
    h.get_by_role_and_label(Role::TextInput, "Titel")
        .type_text(" und Meer");
    h.run();
    h.key_press(eframe::egui::Key::Tab);
    h.run();
    h.run();
    assert!(
        h.state()
            .items
            .iter()
            .any(|t| t.title == "Artikel über Wolken und Meer"),
        "Verlassen"
    );
}

#[test]
fn kein_verlust_beim_wechsel_der_aufgabe() {
    let mut h = harness_tasks(Some((2, false)));
    let id = id_of(&h, "Artikel über Gezeiten");
    select(&mut h, "Artikel über Gezeiten");
    h.get_by_role_and_label(Role::TextInput, "Titel").click();
    h.run();
    h.get_by_role_and_label(Role::TextInput, "Titel")
        .type_text("!");
    h.run();
    let other = item_label(&h, "Buch über Wolken zurückgeben, ").unwrap();
    h.get_by_role_and_label(Role::ListItem, &other).click();
    h.run();
    h.run();
    let store = h.state().shared_store().unwrap();
    // Der Cursor steht dort, wo kittest geklickt hat; entscheidend ist, dass die Eingabe gespeichert ist.
    let title = store.task(&id).unwrap().title;
    assert!(
        title.contains('!') && title.len() == "Artikel über Gezeiten!".len(),
        "{title}"
    );
    assert_ne!(h.state().selected.as_deref(), Some(id.as_str()));
}
