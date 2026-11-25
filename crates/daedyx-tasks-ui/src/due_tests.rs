//! Fälligkeit im Baustein (Pflichtenheft `dueDay.test.ts`, `dueLabel.test.ts`, zwei Zeilen aus `dateEntry.test.ts`
//! gegen die Kern-API, Entscheid T3-1) und zwei Review-Funde T3a (N-2 Zähler bei Kreis, N-4 Ebene nach Verschieben).
//! Feste Uhr: Mittwoch, 18.11.2026, 10:00 Berlin. Nur erfundene Aufgaben.
use crate::TasksUi;
use crate::detail_fields::{draft_due, parse_due};
use crate::test_support::{fixed_store, now};
use crate::tree_model::{due_day, progress};
use chrono::NaiveDate;
use chrono_tz::Europe::Berlin;
use daedyx_tasks_core::quick::parse_quick_task;
use daedyx_tasks_core::types::Task;
use daedyx_ui::i18n::{Language, Texts};
use daedyx_ui::widgets::BadgeTone;
use serde_json::{Value, json};

fn de() -> Texts {
    crate::texts::texts(Language::De)
}

fn ui() -> TasksUi {
    TasksUi::new(Some(fixed_store()))
}

fn task(due: Option<&str>, timed: bool, done: bool) -> Task {
    let mut t = ui().items.first().cloned().unwrap();
    t.due = due.map(Into::into);
    t.due_has_time = timed;
    t.done_at = done.then_some(1);
    t.parent_id = None;
    t
}

fn label(due: Option<&str>, timed: bool, done: bool) -> Option<(String, BadgeTone)> {
    ui().due_label(&de(), &task(due, timed, done))
}

/// UTC-Sekunden für eine Ortszeit in Berlin.
fn secs(day: &str, time: &str) -> String {
    let d = NaiveDate::parse_from_str(day, "%Y-%m-%d").unwrap();
    let t = chrono::NaiveTime::parse_from_str(time, "%H:%M").unwrap();
    daedyx_tasks_core::repeat::local_to_utc(Berlin, d.and_time(t))
        .unwrap()
        .to_string()
}

#[test]
fn ohne_faelligkeit_gibt_es_keinen_tag_und_keine_marke() {
    assert_eq!(due_day(&task(None, false, false), Berlin), None);
    assert_eq!(label(None, false, false), None);
}

#[test]
fn ein_datum_bleibt_ohne_uhrzeit() {
    let t = task(Some("2026-11-30"), false, false);
    assert_eq!(due_day(&t, Berlin), NaiveDate::from_ymd_opt(2026, 11, 30));
    assert_eq!(
        draft_due(&ui(), &de(), &t),
        ("30.11.2026".into(), String::new())
    );
}

#[test]
fn tag_und_uhrzeit_hin_und_zurueck_in_ortszeit() {
    let mut u = ui();
    u.draft.date = "30.11.2026".into();
    u.draft.time = "18:30".into();
    let [(_, due), (_, timed)] = parse_due(&u, &de()).unwrap();
    assert_eq!(due, json!(secs("2026-11-30", "18:30")));
    let t = task(due.as_str(), timed == json!(true), false);
    assert_eq!(
        draft_due(&u, &de(), &t),
        ("30.11.2026".into(), "18:30".into())
    );
}

#[test]
fn leerer_tag_loescht_tag_ohne_uhrzeit_ist_ganztaegig() {
    let mut u = ui();
    let cleared = parse_due(&u, &de()).unwrap();
    assert_eq!(cleared[0].1, Value::Null);
    u.draft.date = "30.11.2026".into();
    let day = parse_due(&u, &de()).unwrap();
    assert_eq!(
        (day[0].1.clone(), day[1].1.clone()),
        (json!("2026-11-30"), json!(false))
    );
    u.draft.date = "31.02.2026".into();
    assert!(
        parse_due(&u, &de()).is_none(),
        "unmöglicher Tag wird nicht gespeichert"
    );
}

#[test]
fn unlesbare_sekunden_ergeben_keinen_tag() {
    assert_eq!(due_day(&task(Some("abc"), true, false), Berlin), None);
    assert_eq!(label(Some("abc"), true, false), None);
}

#[test]
fn heute_morgen_gestern_mit_toenung() {
    assert_eq!(
        label(Some("2026-11-18"), false, false),
        Some(("Heute".into(), BadgeTone::Warning))
    );
    assert_eq!(
        label(Some("2026-11-19"), false, false).map(|l| l.0),
        Some("Morgen".into())
    );
    assert_eq!(
        label(Some("2026-11-17"), false, false),
        Some(("Gestern".into(), BadgeTone::Error))
    );
}

#[test]
fn andere_tage_mit_wochentag_in_fremden_jahren_mit_jahr() {
    assert_eq!(
        label(Some("2026-11-23"), false, false).map(|l| l.0),
        Some("Mo, 23.11.".into())
    );
    assert_eq!(
        label(Some("2027-01-05"), false, false).map(|l| l.0),
        Some("05.01.2027".into())
    );
}

#[test]
fn mit_uhrzeit_haengt_sie_dran() {
    let due = secs("2026-11-18", "18:30");
    assert_eq!(
        label(Some(&due), true, false).map(|l| l.0),
        Some("Heute 18:30".into())
    );
}

#[test]
fn erledigte_aufgaben_sind_nie_ueberfaellig() {
    assert_eq!(
        label(Some("2026-11-01"), false, true).map(|l| l.1),
        Some(BadgeTone::Neutral)
    );
}

#[test]
fn datum_ohne_jahr_das_vorbei_ist_gilt_fuers_naechste_jahr_und_zeitpunkt_wird_gemerkt() {
    let q = parse_quick_task("Arzt 3.1.", now(), Berlin).unwrap();
    assert_eq!(q.fields.get("due"), Some(&json!("2027-01-03")));
    let timed = parse_quick_task("Arzt morgen 15 Uhr", now(), Berlin).unwrap();
    assert_eq!(timed.fields.get("due_has_time"), Some(&json!(true)));
    let day = parse_quick_task("Arzt morgen", now(), Berlin).unwrap();
    assert_ne!(day.fields.get("due_has_time"), Some(&json!(true)));
}

#[test]
fn zaehler_bei_einem_kreis_zaehlt_jede_oberaufgabe_hoechstens_einmal() {
    let mut a = task(None, false, false);
    let mut b = a.clone();
    a.id = "a".into();
    b.id = "b".into();
    a.parent_id = Some("b".into());
    b.parent_id = Some("a".into());
    let p = progress(&[a, b]);
    assert_eq!(p.get("a"), Some(&(0, 1)));
    assert_eq!(p.get("b"), Some(&(0, 1)));
}
