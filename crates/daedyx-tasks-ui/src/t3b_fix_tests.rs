//! Review T3b (N3b-1 bis N3b-4): Ebene nach „Verschieben nach …“ aus einer Smart-Liste, „+ Karte“ behält den Text
//! beim Scheitern, Ziehen auf die eigene Liste bricht ab (Master-Entscheid), die Suche lädt nur Treffer. Feste Uhr,
//! nur erfundene Aufgaben.
use crate::Sel;
use crate::test_support::{click, drag, harness_tasks, id_of, item_label, select, type_into};
use daedyx_tasks_core::smart::View;
use eframe::egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use serde_json::{Map, Value};

#[test]
fn verschieben_aus_einer_smart_liste_zeigt_danach_wieder_das_unterfeld() {
    let mut h = harness_tasks(Some((2, false)));
    let list = h.state().list_id().unwrap().to_owned();
    let store = h.state().shared_store().unwrap();
    let mut parent: Option<String> = None;
    for level in 0..=6 {
        let mut f = Map::new();
        f.insert("title".into(), Value::String(format!("Stufe {level}")));
        let t = store.create_deep(&list, parent.as_deref(), &f, 6).unwrap();
        parent = Some(t.id);
    }
    h.state_mut().select(Sel::View(View::All));
    h.run();
    select(&mut h, "Stufe 6");
    let field = |h: &egui_kittest::Harness<'static, crate::TasksUi>| {
        h.query_all_by_role_and_label(Role::TextInput, "Neue Unteraufgabe")
            .count()
    };
    assert_eq!(field(&h), 0, "Ebene 6: Hinweis statt Feld");
    click(&mut h, Role::Button, "Verschieben nach …");
    click(&mut h, Role::MenuItem, "Haushalt");
    assert_eq!(
        h.state().selected,
        parent,
        "in „Alle“ bleibt die Aufgabe gewählt"
    );
    assert_eq!(h.state().draft.depth, 0);
    assert_eq!(field(&h), 1);
}

#[test]
fn plus_karte_behaelt_den_text_wenn_das_anlegen_scheitert() {
    let mut h = harness_tasks(Some((0, true)));
    h.state_mut().adding_card = Some((
        Some("geloeschter-abschnitt".into()),
        "Fenster putzen".into(),
    ));
    h.state_mut().add_card();
    assert!(h.state().error.is_some());
    assert_eq!(
        h.state().adding_card,
        Some((
            Some("geloeschter-abschnitt".into()),
            "Fenster putzen".into()
        ))
    );
    // Eingabe ohne Titel ergibt keine Karte: der Text bleibt ebenfalls stehen.
    h.state_mut().adding_card = Some((None, "!!".into()));
    h.state_mut().add_card();
    assert_eq!(h.state().adding_card, Some((None, "!!".into())));
    h.state_mut().adding_card = Some((None, "Fenster putzen".into()));
    h.state_mut().add_card();
    assert_eq!(
        h.state().adding_card,
        Some((None, String::new())),
        "angelegt: Feld leer und offen"
    );
}

#[test]
fn ziehen_auf_die_eigene_liste_bricht_ab_auch_bei_unteraufgaben() {
    let mut h = harness_tasks(Some((0, false)));
    let kid = id_of(&h, "Käse");
    let parent = h
        .state()
        .items
        .iter()
        .find(|t| t.id == kid)
        .and_then(|t| t.parent_id.clone());
    let from = h
        .get_by_role_and_label(Role::ListItem, &item_label(&h, "Käse, ").unwrap())
        .rect()
        .center();
    let side = h.get_by_role_and_label(Role::List, "Aufgaben-Leiste");
    let to = side
        .get_by_role_and_label(Role::ListItem, "Haushalt")
        .rect()
        .center();
    drag(&mut h, from, to);
    let store = h.state().shared_store().unwrap();
    assert_eq!(
        store.task(&kid).unwrap().parent_id,
        parent,
        "bleibt Unteraufgabe"
    );
    let list = h.state().list_id().unwrap().to_owned();
    h.state_mut().move_to_list(&kid, &list);
    assert_eq!(
        store.task(&kid).unwrap().parent_id,
        parent,
        "auch ohne Maus"
    );
}

#[test]
fn suche_laedt_nur_die_treffer_nicht_die_zaehler() {
    let mut h = harness_tasks(None);
    let before = h.state().count(View::All);
    let list = h.state().lists[0].id.clone();
    let mut f = Map::new();
    f.insert(
        "title".into(),
        Value::String("Gezeitentabelle kopieren".into()),
    );
    h.state()
        .shared_store()
        .unwrap()
        .create(&list, None, &f)
        .unwrap();
    type_into(&mut h, "Aufgaben suchen", "Gezeiten");
    assert_eq!(h.state().items.len(), 2, "Treffer sind neu");
    assert_eq!(
        h.state().count(View::All),
        before,
        "Zähler beim Tippen nicht neu gezählt"
    );
    type_into(&mut h, "Aufgaben suchen", "");
    assert_eq!(
        h.state().count(View::All),
        before.map(|n| n + 1),
        "nach der Suche wieder aktuell"
    );
}
