//! Review T1a N-1: eine Regel für Zeitlücken. In Apia fehlt der 30.12.2011 ganz (Wechsel der Datumsgrenze). Das
//! Uhrzeit-Feld im Detailbereich und die Schnelleingabe rechnen dort gleich: stundenweise vorrücken, Minuten bleiben.
//! Der Desk-Kalender nutzt dieselbe Kern-Funktion (Test in Desk).
use crate::TasksUi;
use crate::detail_fields::parse_due;
use crate::store::{Clock, TaskStore};
use chrono::{NaiveDate, TimeZone};
use chrono_tz::Pacific::Apia;
use daedyx_tasks_core::quick::parse_quick_task;
use daedyx_ui::i18n::Language;
use serde_json::json;

#[test]
fn detailfeld_und_schnelleingabe_rechnen_die_luecke_von_apia_gleich() {
    let now = NaiveDate::from_ymd_opt(2011, 12, 29)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let mut ui = TasksUi::new(Some(TaskStore::memory(Apia, Clock::Fixed(now)).unwrap()));
    let texts = crate::texts::texts(Language::De);
    for (time, words, hour, minute) in [("10:00", "10 Uhr", 0, 0), ("23:59", "23:59", 0, 59)] {
        ui.draft.date = "30.12.2011".into();
        ui.draft.time = time.into();
        let detail = parse_due(&ui, &texts).unwrap();
        let quick = parse_quick_task(&format!("Probe 30.12.2011 {words}"), now, Apia).unwrap();
        let want = Apia
            .with_ymd_and_hms(2011, 12, 31, hour, minute, 0)
            .single()
            .unwrap()
            .timestamp();
        assert_eq!(detail[0].1, json!(want.to_string()), "Detailfeld {time}");
        assert_eq!(
            quick.fields.get("due"),
            Some(&json!(want.to_string())),
            "Schnelleingabe {words}"
        );
    }
}
