//! Aufgaben-Datenquelle: echter Kern, Datenbank nur im Arbeitsspeicher, erfundene Daten (Testrahmen).
use crate::test_support::fixed_store;
use daedyx_tasks_core::smart::View;
use serde_json::{Map, json};

fn titles(v: &[daedyx_tasks_core::types::Task]) -> Vec<String> {
    v.iter().map(|t| t.title.clone()).collect()
}

#[test]
fn demo_hat_drei_listen_mit_abschnitten() {
    let s = fixed_store();
    let lists = s.lists().unwrap();
    assert_eq!(
        lists.iter().map(|l| l.name.as_str()).collect::<Vec<_>>(),
        ["Haushalt", "Projekt Beispiel", "Lesen"]
    );
    let sections = s.sections(&lists[1].id).unwrap();
    assert_eq!(
        sections.iter().map(|x| x.name.as_str()).collect::<Vec<_>>(),
        ["Ideen", "In Arbeit", "Prüfen"]
    );
    assert!(s.sections(&lists[2].id).unwrap().is_empty());
}

#[test]
fn smart_listen_sehen_die_faelligkeiten_relativ_zu_heute() {
    let s = fixed_store();
    let overdue = titles(&s.query(View::Overdue, None).unwrap());
    assert!(
        overdue.contains(&"Fahrrad zur Werkstatt bringen".to_string()),
        "{overdue:?}"
    );
    assert!(
        overdue.contains(&"Rückmeldung von Clara Probe einholen".to_string()),
        "{overdue:?}"
    );
    let today = titles(&s.query(View::Today, None).unwrap());
    assert!(
        today.contains(&"Wasserzähler ablesen".to_string()),
        "{today:?}"
    );
    let done = titles(&s.query(View::Done, None).unwrap());
    assert_eq!(done.len(), 2, "{done:?}");
    let counts = s.counts().unwrap();
    assert_eq!(counts.len(), 4);
    assert!(counts.iter().all(|(_, n)| *n > 0));
}

#[test]
fn unteraufgaben_haengen_an_ihrer_aufgabe() {
    let s = fixed_store();
    let list = s.lists().unwrap().remove(0);
    let all = s.query(View::List, Some(&list.id)).unwrap();
    let market = all.iter().find(|t| t.title == "Wochenmarkt").unwrap();
    let subs: Vec<_> = all
        .iter()
        .filter(|t| t.parent_id.as_deref() == Some(market.id.as_str()))
        .collect();
    assert_eq!(subs.len(), 3);
    assert!(market.due_has_time && market.remind_minutes == Some(30));
}

#[test]
fn schnelleingabe_legt_eine_aufgabe_mit_faelligkeit_an() {
    let s = fixed_store();
    let list = s.lists().unwrap().remove(2);
    let t = s
        .quick(&list.id, None, "Steuer morgen !!")
        .unwrap()
        .unwrap();
    assert_eq!(
        (t.title.as_str(), t.due.as_deref(), t.priority),
        ("Steuer", Some("2026-11-19"), 2)
    );
    assert!(s.quick(&list.id, None, "   ").unwrap().is_none());
}

#[test]
fn abhaken_einer_wiederkehrenden_aufgabe_rueckt_sie_weiter() {
    let s = fixed_store();
    let list = s.lists().unwrap().remove(0);
    let flowers = s
        .query(View::List, Some(&list.id))
        .unwrap()
        .into_iter()
        .find(|t| t.title == "Blumen gießen")
        .unwrap();
    let after = s.toggle(&flowers.id).unwrap();
    assert!(
        after.done_at.is_none(),
        "wiederkehrende Aufgaben bleiben offen"
    );
    assert_ne!(after.due, flowers.due);
}

#[test]
fn abschnitte_umbenennen_ordnen_loeschen() {
    let s = fixed_store();
    let list = s.lists().unwrap().remove(1);
    let secs = s.sections(&list.id).unwrap();
    s.rename_section(&secs[0].id, "Sammlung").unwrap();
    let order: Vec<String> = secs.iter().rev().map(|x| x.id.clone()).collect();
    s.reorder_sections(&list.id, &order).unwrap();
    let now: Vec<_> = s
        .sections(&list.id)
        .unwrap()
        .into_iter()
        .map(|x| x.name)
        .collect();
    assert_eq!(now, ["Prüfen", "In Arbeit", "Sammlung"]);
    s.delete_section(&secs[2].id).unwrap();
    assert_eq!(s.sections(&list.id).unwrap().len(), 2);
    let homeless = s.query(View::List, Some(&list.id)).unwrap();
    assert!(
        homeless
            .iter()
            .any(|t| t.title.starts_with("Rückmeldung") && t.section_id.is_none()),
        "Karten wandern nach „Ohne Abschnitt“"
    );
}

#[test]
fn aenderungen_und_verschieben_gehen_durch_den_kern() {
    let s = fixed_store();
    let list = s.lists().unwrap().remove(1);
    let secs = s.sections(&list.id).unwrap();
    let t = s
        .query(View::List, Some(&list.id))
        .unwrap()
        .into_iter()
        .find(|t| t.title == "Druckerei anfragen")
        .unwrap();
    let moved = s
        .move_task(&t.id, &list.id, Some(&secs[2].id), 0.5)
        .unwrap();
    assert_eq!(moved.section_id.as_deref(), Some(secs[2].id.as_str()));
    let mut patch = Map::new();
    patch.insert("title".into(), json!(""));
    assert!(
        s.update(&t.id, &patch).is_err(),
        "leerer Titel wird abgelehnt"
    );
    patch.insert("title".into(), json!("Druckerei vergleichen"));
    patch.insert("priority".into(), json!(3));
    assert_eq!(s.update(&t.id, &patch).unwrap().priority, 3);
    s.delete(&t.id).unwrap();
    assert!(s.task(&t.id).is_err());
}

#[test]
fn suche_findet_titel_und_notiz() {
    let s = fixed_store();
    assert_eq!(
        titles(&s.search("zähler").unwrap()),
        ["Wasserzähler ablesen"]
    );
    assert_eq!(titles(&s.search("heft").unwrap()), ["Wasserzähler ablesen"]);
    assert!(s.search("   ").unwrap().is_empty());
}
