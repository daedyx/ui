//! Baum und Sortierung (Tasks T3a, Vorlage `sorting.test.ts` und `boardModel.test.ts` der alten Oberfläche).
use crate::tree_model::{SortMode, progress, title_key, tree_rows};
use chrono_tz::Europe::Berlin;
use daedyx_tasks_core::types::Task;

fn task(id: &str, parent: Option<&str>, pos: f64) -> Task {
    Task {
        id: id.into(),
        list_id: "l".into(),
        parent_id: parent.map(Into::into),
        section_id: None,
        title: id.into(),
        notes: String::new(),
        due: None,
        due_has_time: false,
        priority: 0,
        done_at: None,
        position: pos,
        repeat: None,
        remind_minutes: None,
        assignee_id: None,
        created_by: None,
        color: None,
        links: vec![],
    }
}

fn order(items: &[Task], mode: SortMode) -> Vec<(String, usize)> {
    tree_rows(items, mode, Berlin)
        .iter()
        .map(|r| (items[r.index].id.clone(), r.depth))
        .collect()
}

fn ids(v: &[(String, usize)]) -> Vec<&str> {
    v.iter().map(|(id, _)| id.as_str()).collect()
}

#[test]
fn handreihenfolge_ordnet_jede_ebene_direkt_unter_ihre_oberaufgabe() {
    let items = vec![
        task("b", None, 2.0),
        task("a2", Some("a"), 2.0),
        task("a", None, 1.0),
        task("a1x", Some("a1"), 1.0),
        task("a1", Some("a"), 1.0),
        task("a1xy", Some("a1x"), 1.0),
    ];
    let got = order(&items, SortMode::Hand);
    assert_eq!(ids(&got), ["a", "a1", "a1x", "a1xy", "a2", "b"]);
    let depths: Vec<usize> = got.iter().map(|(_, d)| *d).collect();
    assert_eq!(depths, [0, 1, 2, 3, 1, 0]);
}

#[test]
fn nach_faelligkeit_ohne_datum_zuletzt() {
    let mut items = vec![
        task("ohne", None, 1.0),
        task("spaet", None, 2.0),
        task("frueh", None, 3.0),
    ];
    items[1].due = Some("2026-12-01".into());
    items[2].due = Some("2026-11-20".into());
    assert_eq!(
        ids(&order(&items, SortMode::Due)),
        ["frueh", "spaet", "ohne"]
    );
}

#[test]
fn nach_prioritaet_hohe_zuerst() {
    let mut items = vec![
        task("keine", None, 1.0),
        task("hoch", None, 2.0),
        task("mittel", None, 3.0),
    ];
    items[1].priority = 3;
    items[2].priority = 2;
    assert_eq!(
        ids(&order(&items, SortMode::Priority)),
        ["hoch", "mittel", "keine"]
    );
}

#[test]
fn nach_titel_ohne_gross_klein_und_mit_umlauten() {
    let items = vec![
        task("Zebra", None, 1.0),
        task("äpfel", None, 2.0),
        task("Birne", None, 3.0),
    ];
    assert_eq!(
        ids(&order(&items, SortMode::Title)),
        ["äpfel", "Birne", "Zebra"]
    );
    assert_eq!(title_key("Straße"), "strasse");
}

#[test]
fn sortiert_nur_die_obersten_der_unterbaum_folgt_unveraendert() {
    let items = vec![
        task("B", None, 1.0),
        task("B-z", Some("B"), 1.0),
        task("B-a", Some("B"), 2.0),
        task("A", None, 2.0),
    ];
    assert_eq!(
        ids(&order(&items, SortMode::Title)),
        ["A", "B", "B-z", "B-a"]
    );
}

#[test]
fn unteraufgabe_ohne_oberaufgabe_in_der_ansicht_zaehlt_als_oberste() {
    let items = vec![task("kind", Some("fehlt"), 1.0), task("x", None, 2.0)];
    assert_eq!(
        order(&items, SortMode::Hand),
        [("kind".into(), 0), ("x".into(), 0)]
    );
}

#[test]
fn erledigte_nach_offenen_und_jede_aufgabe_genau_einmal() {
    let mut items = vec![task("fertig", None, 1.0), task("offen", None, 2.0)];
    items[0].done_at = Some(1);
    assert_eq!(ids(&order(&items, SortMode::Hand)), ["offen", "fertig"]);
    // Ein Kreis (kaputte Daten) darf weder hängen noch doppeln.
    let ring = vec![task("p", Some("q"), 1.0), task("q", Some("p"), 2.0)];
    assert_eq!(order(&ring, SortMode::Hand).len(), 2);
}

#[test]
fn zaehler_ueber_drei_ebenen_zaehlt_alle_nachfahren() {
    let mut items = vec![
        task("a", None, 1.0),
        task("b", Some("a"), 1.0),
        task("c", Some("b"), 1.0),
        task("d", Some("c"), 1.0),
    ];
    items[3].done_at = Some(1);
    let p = progress(&items);
    assert_eq!(p.get("a"), Some(&(1, 3)));
    assert_eq!(p.get("b"), Some(&(1, 2)));
    assert_eq!(p.get("c"), Some(&(1, 1)));
    assert_eq!(p.get("d"), None);
}
