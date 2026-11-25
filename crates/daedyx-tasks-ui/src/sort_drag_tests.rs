//! Sortierungen, Ziehen in der Liste und der Tastaturweg Alt+↑/↓ (Tasks T3a, Vorlagen `sorting.test.ts` und
//! `TasksArea.test.tsx`), dazu Abschnitt und „Zuständig“ im Detail. Desk sieht nichts davon. Nur erfundene Aufgaben.
use crate::test_support::{click, harness, harness_tasks, id_of, item_label, select};
use eframe::egui::accesskit::Role;
use eframe::egui::{Event, Key, Modifiers, PointerButton};
use egui_kittest::kittest::{NodeT, Queryable};

const BOOK: &str = "Buch über Wolken zurückgeben";
const ART: &str = "Artikel über Gezeiten";

/// Titel der Zeilen von oben nach unten.
fn titles(h: &egui_kittest::Harness<'static, crate::TasksUi>) -> Vec<String> {
    crate::rows::order(h.state())
        .iter()
        .filter_map(|r| h.state().items.get(r.index).map(|t| t.title.clone()))
        .collect()
}

fn center(h: &egui_kittest::Harness<'static, crate::TasksUi>, title: &str) -> eframe::egui::Pos2 {
    let label = item_label(h, &format!("{title}, ")).unwrap();
    h.get_by_role_and_label(Role::ListItem, &label)
        .rect()
        .center()
}

#[test]
fn sortierung_wechseln_titel_und_zurueck_von_hand() {
    let mut h = harness_tasks(Some((2, false)));
    assert_eq!(titles(&h)[..2], [BOOK, ART]);
    click(&mut h, Role::Button, "Sortierung: von Hand");
    click(&mut h, Role::MenuItem, "Titel");
    assert_eq!(titles(&h)[..2], [ART, BOOK]);
    assert_eq!(
        titles(&h)[2],
        "Gedichte von Ben Muster",
        "erledigte zuletzt"
    );
    click(&mut h, Role::Button, "Sortierung: Titel");
    click(&mut h, Role::MenuItem, "von Hand");
    assert_eq!(titles(&h)[..2], [BOOK, ART]);
}

#[test]
fn zeile_mit_der_maus_vor_eine_andere_ziehen() {
    let mut h = harness_tasks(Some((2, false)));
    let (from, to) = (center(&h, ART), center(&h, BOOK));
    h.hover_at(from);
    h.run();
    h.drag_at(from);
    h.run();
    for step in 1..=10 {
        h.hover_at(from + (to - from) * (step as f32 / 10.0));
        h.run();
    }
    assert!(
        eframe::egui::DragAndDrop::has_any_payload(&h.ctx),
        "die Zeile wird gezogen"
    );
    h.event(Event::PointerButton {
        pos: to,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run();
    h.run();
    h.run();
    assert_eq!(titles(&h)[..2], [ART, BOOK]);
}

#[test]
fn alt_pfeil_verschiebt_ohne_maus_und_nur_von_hand() {
    let mut h = harness_tasks(Some((2, false)));
    let label = item_label(&h, &format!("{BOOK}, ")).unwrap();
    h.get_by_role_and_label(Role::ListItem, &label).focus();
    h.run();
    h.key_press_modifiers(Modifiers::ALT, Key::ArrowDown);
    h.run();
    h.run();
    assert_eq!(titles(&h)[..2], [ART, BOOK]);
    h.key_press_modifiers(Modifiers::ALT, Key::ArrowUp);
    h.run();
    h.run();
    assert_eq!(titles(&h)[..2], [BOOK, ART]);
    h.state_mut().sort = crate::tree_model::SortMode::Priority;
    h.run();
    let positions: Vec<f64> = h.state().items.iter().map(|t| t.position).collect();
    let label = item_label(&h, &format!("{BOOK}, ")).unwrap();
    h.get_by_role_and_label(Role::ListItem, &label).focus();
    h.run();
    h.key_press_modifiers(Modifiers::ALT, Key::ArrowDown);
    h.run();
    let after: Vec<f64> = h.state().items.iter().map(|t| t.position).collect();
    assert_eq!(positions, after, "in anderer Sortierung kein Umsortieren");
}

#[test]
fn abschnitt_im_detail_und_zustaendig_als_platzhalter() {
    let mut h = harness_tasks(Some((0, false)));
    select(&mut h, "Keller aufräumen");
    click(&mut h, Role::Button, "Abschnitt: Später");
    click(&mut h, Role::MenuItem, "Diese Woche");
    let id = id_of(&h, "Keller aufräumen");
    let section = h
        .state()
        .items
        .iter()
        .find(|t| t.id == id)
        .and_then(|t| t.section_id.clone());
    let wanted = h.state().sections.first().map(|s| s.id.clone());
    assert_eq!(section, wanted);
    let assignee = h.get_by_role_and_label(Role::Button, "Zuständig: niemand");
    assert!(assignee.accesskit_node().is_disabled());
    let _ = h.get_by_label("Kommt mit der Zusammenarbeit.");
}

#[test]
fn desk_hat_weder_sortierung_noch_zusatzfelder() {
    let mut h = harness(Some((0, false)));
    assert_eq!(h.query_all_by_label("Sortierung: von Hand").count(), 0);
    select(&mut h, "Keller aufräumen");
    assert_eq!(h.query_all_by_label("Abschnitt: Später").count(), 0);
    assert_eq!(h.query_all_by_label("Zuständig: niemand").count(), 0);
}
