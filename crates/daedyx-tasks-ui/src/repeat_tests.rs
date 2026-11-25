//! Volle Wiederholung (Tasks T3a, Vorlagen `RepeatFields.test.tsx` und `repeatNext.test.ts`): Art, Abstand 1–999,
//! Wochentage 1–7 als Kontrollkästchen, fester Monatstag 1–31 (leer = Tag der Fälligkeit), „ab dem Abhaken“.
//! Feste Uhr (18.11.2026), nur erfundene Aufgaben.
use crate::detail_repeat::{clamp_field, with_kind};
use crate::test_support::{click, harness, harness_tasks, id_of, select, type_into};
use daedyx_tasks_core::types::{RepeatKind, RepeatRule};
use eframe::egui::accesskit::Role;
use egui_kittest::kittest::{NodeT, Queryable};
use serde_json::json;

const ART: &str = "Artikel über Gezeiten";

fn rule(h: &egui_kittest::Harness<'static, crate::TasksUi>, id: &str) -> Option<RepeatRule> {
    h.state()
        .items
        .iter()
        .find(|t| t.id == id)
        .and_then(|t| t.repeat.clone())
}

fn due(h: &egui_kittest::Harness<'static, crate::TasksUi>, id: &str) -> Option<String> {
    h.state()
        .items
        .iter()
        .find(|t| t.id == id)
        .and_then(|t| t.due.clone())
}

#[test]
fn monatstag_31_rutscht_in_kurzen_monaten_und_kehrt_zurueck() {
    let mut h = harness_tasks(Some((2, false)));
    let id = id_of(&h, ART);
    let r = json!({ "kind": "monthly", "interval": 1, "monthDay": 31 });
    h.state_mut()
        .set(&id, &[("due", json!("2027-01-31")), ("repeat_rule", r)]);
    for want in ["2027-02-28", "2027-03-31", "2027-04-30"] {
        h.state_mut().toggle(&id);
        assert_eq!(due(&h, &id).as_deref(), Some(want));
    }
}

#[test]
fn wochentage_nur_eins_bis_sieben_als_kontrollkaestchen() {
    let mut h = harness_tasks(Some((2, false)));
    let id = id_of(&h, ART);
    select(&mut h, ART);
    click(&mut h, Role::Button, "Wiederholung: keine");
    click(&mut h, Role::MenuItem, "wöchentlich");
    let group = h.get_by_role_and_label(Role::Group, "Wochentage");
    assert_eq!(group.query_all_by_role(Role::CheckBox).count(), 7);
    click(&mut h, Role::CheckBox, "Donnerstag");
    click(&mut h, Role::CheckBox, "Montag");
    assert_eq!(rule(&h, &id).map(|r| r.weekdays), Some(vec![1, 4]));
    assert_eq!(
        h.get_by_role_and_label(Role::CheckBox, "Montag")
            .accesskit_node()
            .toggled(),
        Some(eframe::egui::accesskit::Toggled::True)
    );
    click(&mut h, Role::CheckBox, "Montag");
    assert_eq!(rule(&h, &id).map(|r| r.weekdays), Some(vec![4]));
}

#[test]
fn fester_monatstag_begrenzt_auf_1_bis_31_und_leer_loescht_den_anker() {
    let mut h = harness_tasks(Some((2, false)));
    let id = id_of(&h, ART);
    select(&mut h, ART);
    click(&mut h, Role::Button, "Wiederholung: keine");
    click(&mut h, Role::MenuItem, "monatlich");
    type_into(&mut h, "Fester Tag im Monat", "15");
    assert_eq!(rule(&h, &id).and_then(|r| r.month_day), Some(15));
    type_into(&mut h, "Fester Tag im Monat", "45");
    assert_eq!(rule(&h, &id).and_then(|r| r.month_day), Some(31));
    type_into(&mut h, "Fester Tag im Monat", "");
    assert_eq!(rule(&h, &id).and_then(|r| r.month_day), None);
    assert_eq!(rule(&h, &id).map(|r| r.kind), Some(RepeatKind::Monthly));
}

#[test]
fn abstand_und_ab_abhaken() {
    let mut h = harness_tasks(Some((2, false)));
    let id = id_of(&h, ART);
    select(&mut h, ART);
    click(&mut h, Role::Button, "Wiederholung: keine");
    click(&mut h, Role::MenuItem, "täglich");
    assert_eq!(
        h.query_all_by_label("Fester Tag im Monat").count(),
        0,
        "nicht bei täglich"
    );
    click(&mut h, Role::CheckBox, "Ab dem Abhaken zählen");
    assert_eq!(rule(&h, &id).map(|r| r.from_done), Some(true));
    let _ = h.get_by_role_and_label(Role::TextInput, "Abstand in Tagen");
}

#[test]
fn wechsel_der_art_und_begrenzen_der_zahlen() {
    let monthly = RepeatRule {
        kind: RepeatKind::Monthly,
        interval: 3,
        weekdays: vec![],
        from_done: true,
        month_day: Some(31),
    };
    let daily = with_kind(Some(&monthly), Some(RepeatKind::Daily)).unwrap();
    assert_eq!(
        (daily.interval, daily.month_day, daily.from_done),
        (3, None, true)
    );
    let yearly = with_kind(Some(&monthly), Some(RepeatKind::Yearly)).unwrap();
    assert_eq!(
        yearly.month_day,
        Some(31),
        "jährlich zeigt den Tag ebenfalls"
    );
    assert_eq!(with_kind(Some(&monthly), None), None);
    assert_eq!(clamp_field("45", 1, 31), Some(31));
    assert_eq!(clamp_field("0", 1, 999), Some(1));
    assert_eq!(clamp_field("5000", 1, 999), Some(999));
    assert_eq!(clamp_field("", 1, 31), None);
    assert_eq!(clamp_field("abc", 1, 31), None);
}

#[test]
fn desk_behaelt_das_einfache_menue() {
    let mut h = harness(Some((2, false)));
    select(&mut h, ART);
    let _ = h.get_by_role_and_label(Role::Button, "Wiederholung: keine");
    assert_eq!(h.query_all_by_label("Erinnerung hinzufügen").count(), 0);
    let _ = h.get_by_role_and_label(Role::Button, "Erinnerung: keine");
}
