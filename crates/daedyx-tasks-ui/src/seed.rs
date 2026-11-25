//! Füllt eine Aufgaben-Datenbank mit erfundenen Aufgaben aus einer JSON-Datei der App (Desk:
//! `testdata/erfunden/aufgaben.json`). Alles läuft über die normalen Kern-Funktionen, also gelten dieselben
//! Prüfungen wie beim Bedienen.
use crate::colors::stored;
use crate::store::TaskStore;
use chrono::{Days, NaiveDate, NaiveTime};
use chrono_tz::Tz;
use daedyx_tasks_core::TaskError;
use daedyx_tasks_core::repeat::local_to_utc;
use serde::Deserialize;
use serde_json::{Map, Value, json};

#[derive(Deserialize)]
struct Demo {
    listen: Vec<DemoList>,
}

#[derive(Deserialize)]
struct DemoList {
    name: String,
    abschnitte: Vec<String>,
    aufgaben: Vec<DemoTask>,
}

#[derive(Deserialize)]
struct DemoTask {
    titel: String,
    tag: Option<i64>,
    uhrzeit: Option<String>,
    prioritaet: Option<u8>,
    notiz: Option<String>,
    abschnitt: Option<usize>,
    wiederholung: Option<Value>,
    erinnerung: Option<i64>,
    #[serde(default)]
    erledigt: bool,
    #[serde(default)]
    unter: Vec<String>,
}

fn shift(today: NaiveDate, days: i64) -> Option<NaiveDate> {
    let n = Days::new(days.unsigned_abs());
    if days < 0 {
        today.checked_sub_days(n)
    } else {
        today.checked_add_days(n)
    }
}

fn fields(t: &DemoTask, today: NaiveDate, tz: Tz, section: Option<&str>) -> Map<String, Value> {
    let mut f = Map::new();
    f.insert("title".into(), json!(t.titel));
    let day = t.tag.and_then(|d| shift(today, d));
    let time = t
        .uhrzeit
        .as_deref()
        .and_then(|u| NaiveTime::parse_from_str(u, "%H:%M").ok());
    match (day, time) {
        (Some(d), Some(tm)) => {
            if let Some(secs) = local_to_utc(tz, d.and_time(tm)) {
                f.insert("due".into(), json!(secs.to_string()));
                f.insert("due_has_time".into(), json!(true));
            }
        }
        (Some(d), None) => {
            f.insert("due".into(), json!(d.format("%Y-%m-%d").to_string()));
        }
        _ => {}
    }
    let optional = [
        ("priority", t.prioritaet.map(|p| json!(p))),
        ("notes", t.notiz.as_ref().map(|n| json!(n))),
        ("repeat_rule", t.wiederholung.clone()),
        ("remind_minutes", t.erinnerung.map(|m| json!(m))),
        ("section_id", section.map(|s| json!(s))),
    ];
    for (key, value) in optional {
        if let Some(v) = value {
            f.insert(key.into(), v);
        }
    }
    f
}

/// Legt Listen, Abschnitte und Aufgaben aus `json` an (Tage relativ zu heute nach der Uhr von `store`).
pub fn fill(store: &TaskStore, json: &str) -> Result<(), TaskError> {
    let demo: Demo = serde_json::from_str(json).map_err(|_| TaskError::Invalid("demo"))?;
    let (today, tz) = (store.today(), store.zone());
    for (n, l) in demo.listen.iter().enumerate() {
        let list = store.create_list(&l.name, &stored(n))?;
        let mut ids = Vec::new();
        for name in &l.abschnitte {
            ids.push(store.create_section(&list.id, name)?.id);
        }
        for t in &l.aufgaben {
            let section = t.abschnitt.and_then(|i| ids.get(i)).map(String::as_str);
            let task = store.create(&list.id, None, &fields(t, today, tz, section))?;
            for sub in &t.unter {
                let mut f = Map::new();
                f.insert("title".into(), json!(sub));
                store.create(&list.id, Some(&task.id), &f)?;
            }
            if t.erledigt {
                store.toggle(&task.id)?;
            }
        }
    }
    Ok(())
}
