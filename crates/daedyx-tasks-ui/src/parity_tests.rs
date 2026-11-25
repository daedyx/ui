//! Gleichstand mit der alten Oberfläche (Pflichtenheft-Matrix, `TasksArea.test.tsx` und `TaskBoard.test.tsx`), mit
//! der Tasks-Fassung des Bausteins (`Features::TASKS`): Leiste, Schnelleingabe, Abhaken, Detail, Listen, Board mit
//! Abschnitten. Feste Uhr (18.11.2026), nur erfundene Aufgaben.
use crate::test_support::{click, drag, harness_tasks, id_of, item_label, select, type_into};
use crate::{Sel, TasksUi};
use daedyx_tasks_core::smart::View;
use eframe::egui::accesskit::Role;
use eframe::egui::{Key, Modifiers, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

type H = Harness<'static, TasksUi>;

fn headers(h: &H) -> Vec<String> {
    h.query_all_by_role(Role::ColumnHeader)
        .filter_map(|n| n.accesskit_node().label())
        .collect()
}

fn task(h: &H, title: &str) -> daedyx_tasks_core::types::Task {
    h.state()
        .items
        .iter()
        .find(|t| t.title == title)
        .cloned()
        .unwrap()
}

#[test]
fn smart_listen_mit_zahlen_und_start_bei_heute() {
    let h = harness_tasks(None);
    assert_eq!(h.state().sel, Sel::View(View::Today));
    for label in ["Heute, 1", "Geplant, 8", "Überfällig, 2", "Alle, 15"] {
        let _ = h.get_by_role_and_label(Role::ListItem, label);
    }
}

#[test]
fn schnelleingabe_mit_prioritaet_und_in_heute_mit_faelligkeit_heute() {
    let mut h = harness_tasks(None);
    type_into(&mut h, "Neue Aufgabe", "Paket abholen !!");
    let t = task(&h, "Paket abholen");
    assert_eq!(
        (t.priority, t.due.as_deref()),
        (2, Some("2026-11-18")),
        "in „Heute“ fällig heute"
    );
}

#[test]
fn hakt_eine_aufgabe_ab_und_wieder_auf() {
    let mut h = harness_tasks(Some((2, false)));
    click(&mut h, Role::CheckBox, "Artikel über Gezeiten erledigt");
    assert!(task(&h, "Artikel über Gezeiten").done_at.is_some());
    click(&mut h, Role::CheckBox, "Artikel über Gezeiten erledigt");
    assert!(task(&h, "Artikel über Gezeiten").done_at.is_none());
}

#[test]
fn oeffnet_einzelheiten_und_speichert_titel_und_prioritaet() {
    let mut h = harness_tasks(Some((2, false)));
    select(&mut h, "Artikel über Gezeiten");
    type_into(&mut h, "Titel", "Artikel über Ebbe");
    click(&mut h, Role::RadioButton, "Hoch");
    assert_eq!(task(&h, "Artikel über Ebbe").priority, 3);
}

#[test]
fn legt_eine_liste_an_benennt_sie_um_und_loescht_sie_nach_rueckfrage() {
    let mut h = harness_tasks(None);
    type_into(&mut h, "Neue Liste", "Garten");
    assert!(h.state().lists.iter().any(|l| l.name == "Garten"));
    h.get_by_role_and_label(Role::ListItem, "Garten").focus();
    h.run();
    h.key_press(Key::F2);
    h.run();
    h.run();
    h.get_by_role_and_label(Role::TextInput, "Name der Liste")
        .type_text("arbeit");
    h.run();
    h.key_press(Key::Enter);
    h.run();
    h.run();
    h.get_by_role_and_label(Role::ListItem, "Gartenarbeit")
        .focus();
    h.run();
    h.key_press(Key::Delete);
    h.run();
    h.run();
    let _ = h.get_by_role_and_label(Role::Dialog, "Liste löschen?");
    click(&mut h, Role::Button, "Löschen");
    assert!(
        h.state()
            .lists
            .iter()
            .all(|l| !l.name.starts_with("Garten"))
    );
}

#[test]
fn abschnitte_anlegen_umbenennen_ordnen_und_loeschen_die_karten_bleiben() {
    let mut h = harness_tasks(Some((1, true)));
    h.get_by_role_and_label(Role::TextInput, "Neuer Abschnitt")
        .focus();
    h.run();
    h.get_by_role_and_label(Role::TextInput, "Neuer Abschnitt")
        .type_text("Fertig");
    h.run();
    h.key_press(Key::Enter);
    h.run();
    h.run();
    assert_eq!(
        headers(&h).last().map(String::as_str),
        Some("Fertig, 0 Karten")
    );
    h.get_by_role_and_label(Role::ColumnHeader, "Prüfen, 1 Karten")
        .focus();
    h.run();
    h.key_press_modifiers(Modifiers::ALT, Key::ArrowLeft);
    h.run();
    h.run();
    assert_eq!(
        headers(&h)[2..4],
        ["Prüfen, 1 Karten", "In Arbeit, 2 Karten"]
    );
    // Ziel ist die dritte Spalte: die vierte liegt rechts außerhalb des Testfensters.
    let from = h
        .get_by_role_and_label(Role::ColumnHeader, "Ideen, 2 Karten")
        .rect()
        .center();
    let to = h
        .get_by_role_and_label(Role::ColumnHeader, "Prüfen, 1 Karten")
        .rect()
        .center();
    drag(&mut h, from, to);
    assert_eq!(
        headers(&h)[1..4],
        ["Prüfen, 1 Karten", "Ideen, 2 Karten", "In Arbeit, 2 Karten"]
    );
    h.get_by_role_and_label(Role::ColumnHeader, "Ideen, 2 Karten")
        .focus();
    h.run();
    h.key_press(Key::Delete);
    h.run();
    h.run();
    click(&mut h, Role::Button, "Löschen");
    assert_eq!(
        headers(&h)[0],
        "Ohne Abschnitt, 2 Karten",
        "Karten wandern nach „Ohne Abschnitt“"
    );
}

#[test]
fn karte_per_ziehen_in_einen_abschnitt_und_abgehakt_unten_im_abschnitt() {
    let mut h = harness_tasks(Some((1, true)));
    let card = item_label(&h, "Druckerei anfragen, ").unwrap();
    let from = h
        .get_by_role_and_label(Role::ListItem, &card)
        .rect()
        .center();
    let to = h
        .get_by_role_and_label(Role::ColumnHeader, "In Arbeit, 2 Karten")
        .rect()
        .center()
        + vec2(0.0, 40.0);
    drag(&mut h, from, to);
    let work = h
        .state()
        .sections
        .iter()
        .find(|s| s.name == "In Arbeit")
        .map(|s| s.id.clone());
    assert_eq!(task(&h, "Druckerei anfragen").section_id, work);
    let id = id_of(&h, "Druckerei anfragen");
    h.state_mut().toggle(&id);
    h.run();
    let col = crate::model::cards(&h.state().items, &h.state().sections, work.as_deref());
    assert_eq!(
        col.last().map(|t| t.id.as_str()),
        Some(id.as_str()),
        "erledigt bleibt im Abschnitt, unten"
    );
}

#[test]
fn traegt_viele_abschnitte_in_ihrer_reihenfolge_und_nimmt_karten_im_letzten_auf() {
    let mut h = harness_tasks(Some((2, true)));
    for name in ["A", "B", "C", "D", "E", "F", "G"] {
        h.state_mut().new_section = name.into();
        h.state_mut().add_section();
    }
    h.run();
    let names: Vec<String> = h.state().sections.iter().map(|s| s.name.clone()).collect();
    assert_eq!(names, ["A", "B", "C", "D", "E", "F", "G"]);
    let last = h.state().sections.last().map(|s| s.id.clone());
    h.state_mut().adding_card = Some((last.clone(), "Rezension schreiben".into()));
    h.state_mut().add_card();
    assert_eq!(task(&h, "Rezension schreiben").section_id, last);
}
