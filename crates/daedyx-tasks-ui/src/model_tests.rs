//! Rechenregeln des Aufgabenbereichs (Zeilenfolge, Spalten, Einfügeposition, Abschnitte verschieben).
use crate::model::{Row, cards, columns, drop_position, dropped_sections, moved_sections, rows};
use daedyx_tasks_core::types::{Section, Task};

fn task(id: &str, parent: Option<&str>, section: Option<&str>, pos: f64, done: bool) -> Task {
    Task {
        id: id.into(),
        list_id: "l".into(),
        parent_id: parent.map(Into::into),
        section_id: section.map(Into::into),
        title: id.into(),
        notes: String::new(),
        due: None,
        due_has_time: false,
        priority: 0,
        done_at: done.then_some(1),
        position: pos,
        repeat: None,
        remind_minutes: None,
        assignee_id: None,
        created_by: None,
        color: None,
        links: vec![],
    }
}

fn section(id: &str) -> Section {
    Section {
        id: id.into(),
        list_id: "l".into(),
        name: id.into(),
        position: 0.0,
        color: None,
    }
}

#[test]
fn unteraufgaben_stehen_unter_ihrer_aufgabe_erledigte_zuletzt() {
    let items = vec![
        task("a", None, None, 1.0, true),
        task("b", None, None, 2.0, false),
        task("b1", Some("b"), None, 1.0, false),
        task("waise", Some("fehlt"), None, 1.0, false),
    ];
    let r = rows(&items);
    let ids: Vec<(&str, usize)> = r
        .iter()
        .map(|Row { index, depth }| (items[*index].id.as_str(), *depth))
        .collect();
    assert_eq!(ids, [("b", 0), ("b1", 1), ("waise", 0), ("a", 0)]);
}

#[test]
fn spalten_beginnen_mit_ohne_abschnitt_und_karten_sind_sortiert() {
    let cols = columns(&[section("s1"), section("s2")]);
    assert_eq!(cols, [None, Some("s1".into()), Some("s2".into())]);
    let items = vec![
        task("x", None, Some("s1"), 3.0, false),
        task("y", None, Some("s1"), 1.0, false),
        task("k", Some("y"), None, 1.0, false),
    ];
    let c: Vec<&str> = cards(&items, Some("s1"))
        .iter()
        .map(|t| t.id.as_str())
        .collect();
    assert_eq!(c, ["y", "x"]);
    assert!(
        cards(&items, None).is_empty(),
        "Unteraufgaben sind keine Karten"
    );
}

#[test]
fn einfuegeposition_liegt_zwischen_den_nachbarn() {
    assert_eq!(drop_position(&[], 0), 1.0);
    assert_eq!(drop_position(&[2.0, 4.0], 0), 1.0);
    assert_eq!(drop_position(&[2.0, 4.0], 1), 3.0);
    assert_eq!(drop_position(&[2.0, 4.0], 2), 5.0);
    assert_eq!(drop_position(&[2.0, 4.0], 99), 5.0);
}

#[test]
fn abschnitte_verschieben_und_ablegen() {
    let s = [section("a"), section("b"), section("c")];
    assert_eq!(
        moved_sections(&s, "b", -1),
        Some(vec!["b".into(), "a".into(), "c".into()])
    );
    assert_eq!(moved_sections(&s, "a", -1), None);
    assert_eq!(moved_sections(&s, "c", 1), None);
    assert_eq!(
        dropped_sections(&s, "a", "c"),
        Some(vec!["b".into(), "c".into(), "a".into()])
    );
    assert_eq!(dropped_sections(&s, "a", "a"), None);
    assert_eq!(dropped_sections(&s, "x", "a"), None);
}
