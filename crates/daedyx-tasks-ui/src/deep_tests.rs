//! Unteraufgaben bis Ebene 6 (Tasks T3a, Vorlage `TasksDeep.test.tsx`): anlegen, einrücken mit Ebene für
//! Screenreader, Zähler über alle Nachfahren, Hinweis auf der letzten Ebene, „↑ Oberaufgabe“, „Öffnen“. Desk bleibt
//! bei einer Ebene. Nur erfundene Aufgaben.
use crate::test_support::{click, harness, harness_tasks, id_of, item_label, select, type_into};
use eframe::egui::accesskit::Role;
use egui_kittest::kittest::{NodeT, Queryable};

/// Kette „Ebene 0“ … „Ebene n“ in der Liste „Lesen“ über die Aktionen des Bausteins.
fn chain(h: &mut egui_kittest::Harness<'static, crate::TasksUi>, n: u32) {
    let list = h.state().list_id().unwrap().to_owned();
    h.state_mut().quick = "Ebene 0".into();
    h.state_mut().quick_add();
    for level in 1..=n {
        let parent = id_of(h, &format!("Ebene {}", level.saturating_sub(1)));
        h.state_mut()
            .add_subtask(&parent, &format!("Ebene {level}"));
        assert_eq!(h.state().error, None, "Ebene {level} in {list}");
    }
    h.run();
    h.run();
}

#[test]
fn ebene_6_geht_ebene_7_ergibt_hinweis_statt_absturz() {
    let mut h = harness_tasks(Some((2, false)));
    chain(&mut h, 6);
    let deepest = id_of(&h, "Ebene 6");
    h.state_mut().add_subtask(&deepest, "Ebene 7");
    assert_eq!(h.state().error.as_deref(), Some("aufgaben.fehler.too_deep"));
    assert!(h.state().items.iter().all(|t| t.title != "Ebene 7"));
    select(&mut h, "Ebene 6");
    assert_eq!(
        h.query_all_by_role_and_label(Role::TextInput, "Neue Unteraufgabe")
            .count(),
        0
    );
    let _ = h.get_by_label("Hier geht es nicht tiefer: Unteraufgaben gibt es bis Ebene 6.");
    select(&mut h, "Ebene 5");
    let _ = h.get_by_role_and_label(Role::TextInput, "Neue Unteraufgabe");
}

#[test]
fn eingerueckt_nach_ebene_mit_ebene_fuer_screenreader_und_zaehler_ueber_alle() {
    let mut h = harness_tasks(Some((2, false)));
    chain(&mut h, 3);
    for level in 0..=3usize {
        let label = item_label(&h, &format!("Ebene {level}, ")).unwrap();
        let node = h.get_by_role_and_label(Role::ListItem, &label);
        assert_eq!(
            node.accesskit_node().level(),
            Some(level + 1),
            "Ebene {level}"
        );
    }
    let x0 = h
        .get_by_role_and_label(Role::ListItem, &item_label(&h, "Ebene 0, ").unwrap())
        .rect()
        .left();
    let x2 = h
        .get_by_role_and_label(Role::ListItem, &item_label(&h, "Ebene 2, ").unwrap())
        .rect()
        .left();
    assert!(x2 > x0 + 40.0, "Ebene 2 weiter eingerückt");
    let _ = h.get_by_label("0/3 Unteraufgaben");
    let _ = h.get_by_label("0/2 Unteraufgaben");
}

#[test]
fn unteraufgabe_im_detail_anlegen_und_oeffnen() {
    let mut h = harness_tasks(Some((2, false)));
    select(&mut h, "Artikel über Gezeiten");
    type_into(&mut h, "Neue Unteraufgabe", "Quellen sammeln");
    let kid = id_of(&h, "Quellen sammeln");
    click(
        &mut h,
        Role::Button,
        "Unteraufgabe „Quellen sammeln“ öffnen",
    );
    assert_eq!(h.state().selected.as_deref(), Some(kid.as_str()));
    type_into(&mut h, "Neue Unteraufgabe", "Bibliothek");
    assert_eq!(h.state().depth_of(&id_of(&h, "Bibliothek")), 2);
}

#[test]
fn zeigt_in_der_unteraufgabe_die_oberaufgabe_und_springt_dorthin_zurueck() {
    let mut h = harness_tasks(Some((0, false)));
    select(&mut h, "Käse");
    let parent = id_of(&h, "Wochenmarkt");
    click(&mut h, Role::Button, "Zur Oberaufgabe „Wochenmarkt“");
    assert_eq!(h.state().selected.as_deref(), Some(parent.as_str()));
}

#[test]
fn desk_bleibt_bei_einer_ebene() {
    let mut h = harness(Some((0, false)));
    select(&mut h, "Käse");
    assert_eq!(
        h.query_all_by_role_and_label(Role::TextInput, "Neue Unteraufgabe")
            .count(),
        0
    );
    assert_eq!(h.query_all_by_label("↑ Oberaufgabe").count(), 0);
    let kaese = id_of(&h, "Käse");
    h.state_mut().add_subtask(&kaese, "Zu tief für Desk");
    assert_eq!(h.state().error.as_deref(), Some("aufgaben.fehler.too_deep"));
    assert!(item_label(&h, "Wochenmarkt, ").is_some());
    assert_eq!(
        h.query_all_by_label("0/3 Unteraufgaben").count(),
        0,
        "Desk-Zeilen ohne Zähler"
    );
}
