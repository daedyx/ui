//! Leiste in Tasks T3b (Vorlagen `TasksArea.test.tsx`, `TaskBoard.test.tsx`): Listen per Ziehen und Alt+↑/↓
//! umsortieren, Aufgabe aus der Liste oder vom Board auf eine Liste ziehen, Tastaturweg „Verschieben nach …“.
//! Desk sieht nichts davon. Nur erfundene Aufgaben.
use crate::test_support::{click, drag, harness, harness_tasks, id_of, item_label, select};
use eframe::egui::accesskit::Role;
use eframe::egui::{Key, Modifiers, Pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

type H = Harness<'static, crate::TasksUi>;

fn names(h: &H) -> Vec<String> {
    h.state().lists.iter().map(|l| l.name.clone()).collect()
}

fn entry(h: &H, name: &str) -> Pos2 {
    let side = h.get_by_role_and_label(Role::List, "Aufgaben-Leiste");
    side.get_by_role_and_label(Role::ListItem, name)
        .rect()
        .center()
}

fn row(h: &H, title: &str) -> Pos2 {
    let label = item_label(h, &format!("{title}, ")).unwrap();
    h.query_all_by_role_and_label(Role::ListItem, &label)
        .next()
        .unwrap()
        .rect()
        .center()
}

fn list_id(h: &H, name: &str) -> String {
    h.state()
        .lists
        .iter()
        .find(|l| l.name == name)
        .map(|l| l.id.clone())
        .unwrap()
}

#[test]
fn listen_mit_alt_pfeil_umsortieren() {
    let mut h = harness_tasks(None);
    let side = h.get_by_role_and_label(Role::List, "Aufgaben-Leiste");
    side.get_by_role_and_label(Role::ListItem, "Haushalt")
        .focus();
    h.run();
    h.key_press_modifiers(Modifiers::ALT, Key::ArrowDown);
    h.run();
    h.run();
    assert_eq!(names(&h), ["Projekt Beispiel", "Haushalt", "Lesen"]);
    h.key_press_modifiers(Modifiers::ALT, Key::ArrowUp);
    h.run();
    h.run();
    assert_eq!(names(&h), ["Haushalt", "Projekt Beispiel", "Lesen"]);
}

#[test]
fn liste_mit_der_maus_an_eine_andere_stelle_ziehen() {
    let mut h = harness_tasks(None);
    let (from, to) = (entry(&h, "Lesen"), entry(&h, "Haushalt"));
    drag(&mut h, from, to);
    assert_eq!(names(&h), ["Lesen", "Haushalt", "Projekt Beispiel"]);
}

#[test]
fn aufgabe_aus_der_liste_auf_eine_andere_liste_ziehen() {
    let mut h = harness_tasks(Some((2, false)));
    let (from, to) = (row(&h, "Artikel über Gezeiten"), entry(&h, "Haushalt"));
    let id = id_of(&h, "Artikel über Gezeiten");
    drag(&mut h, from, to);
    let store = h.state().shared_store().unwrap();
    assert_eq!(store.task(&id).unwrap().list_id, list_id(&h, "Haushalt"));
    assert!(
        h.state().items.iter().all(|t| t.id != id),
        "nicht mehr in „Lesen“"
    );
}

#[test]
fn karte_vom_board_auf_eine_liste_ziehen() {
    let mut h = harness_tasks(Some((0, true)));
    let id = id_of(&h, "Keller aufräumen");
    let (from, to) = (row(&h, "Keller aufräumen"), entry(&h, "Lesen"));
    drag(&mut h, from, to);
    let store = h.state().shared_store().unwrap();
    assert_eq!(store.task(&id).unwrap().list_id, list_id(&h, "Lesen"));
}

#[test]
fn verschieben_nach_als_tastaturweg_samt_unteraufgaben() {
    let mut h = harness_tasks(Some((0, false)));
    select(&mut h, "Wochenmarkt");
    let kid = id_of(&h, "Käse");
    click(&mut h, Role::Button, "Verschieben nach …");
    click(&mut h, Role::MenuItem, "Lesen");
    let store = h.state().shared_store().unwrap();
    assert_eq!(
        store.task(&kid).unwrap().list_id,
        list_id(&h, "Lesen"),
        "Unteraufgaben ziehen mit"
    );
    assert!(h.state().items.iter().all(|t| t.title != "Wochenmarkt"));
}

#[test]
fn desk_kann_weder_listen_umsortieren_noch_verschieben() {
    let mut h = harness(Some((0, false)));
    let side = h.get_by_role_and_label(Role::List, "Aufgaben-Leiste");
    side.get_by_role_and_label(Role::ListItem, "Haushalt")
        .focus();
    h.run();
    h.key_press_modifiers(Modifiers::ALT, Key::ArrowDown);
    h.run();
    assert_eq!(names(&h), ["Haushalt", "Projekt Beispiel", "Lesen"]);
    // Desk nimmt Alt+Pfeil wie Pfeil (Wechsel der Auswahl, wie M6a); zurück zu „Haushalt“.
    let first = h.state().lists[0].id.clone();
    h.state_mut().select(crate::Sel::List(first));
    h.run();
    select(&mut h, "Keller aufräumen");
    assert_eq!(h.query_all_by_label("Verschieben nach …").count(), 0);
}
