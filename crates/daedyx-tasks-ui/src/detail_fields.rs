//! Felder im Detailbereich: Fälligkeit (Tag und Uhrzeit), Priorität, Wiederholung, Erinnerung.
//! Tag und Uhrzeit gelten beim Verlassen des Feldes; Unsinn zeigt einen Fehler statt etwas zu speichern.
use super::TasksUi;
use crate::row_paint::priority_key;
use chrono::{NaiveDate, NaiveTime};
use daedyx_tasks_core::repeat::local_to_utc;
use daedyx_tasks_core::types::{RepeatKind, Task};
use daedyx_ui::a11y::named_scope;
use daedyx_ui::i18n::Texts;
use daedyx_ui::tokens::TOKENS;
use daedyx_ui::widgets::{ButtonKind, MenuItem, button, menu_button, text_input};
use eframe::egui::{Ui, WidgetInfo, accesskit::Role};
use serde_json::{Value, json};

/// Tag und Uhrzeit der Aufgabe als Eingabetext (Muster aus der Sprachdatei).
pub fn draft_due(tasks: &TasksUi, texts: &Texts, t: &Task) -> (String, String) {
    let pattern = texts.t("aufgaben.datum.eingabe");
    let Some(due) = t.due.as_deref() else {
        return (String::new(), String::new());
    };
    if t.due_has_time {
        let local = due
            .parse::<i64>()
            .ok()
            .and_then(|s| daedyx_tasks_core::due_time::local_at(tasks.zone(), s));
        return local.map_or_else(Default::default, |d| {
            (
                d.format(&pattern).to_string(),
                d.format("%H:%M").to_string(),
            )
        });
    }
    let day = NaiveDate::parse_from_str(due, "%Y-%m-%d").ok();
    (
        day.map(|d| d.format(&pattern).to_string())
            .unwrap_or_default(),
        String::new(),
    )
}

/// Neue Werte für `due`/`due_has_time` aus den Eingaben; `None` = Eingabe unbrauchbar.
fn parse_due(tasks: &TasksUi, texts: &Texts) -> Option<[(&'static str, Value); 2]> {
    let (date, time) = (tasks.draft.date.trim(), tasks.draft.time.trim());
    if date.is_empty() {
        return time
            .is_empty()
            .then_some([("due", Value::Null), ("due_has_time", json!(false))]);
    }
    let day = NaiveDate::parse_from_str(date, &texts.t("aufgaben.datum.eingabe")).ok()?;
    if time.is_empty() {
        return Some([
            ("due", json!(day.format("%Y-%m-%d").to_string())),
            ("due_has_time", json!(false)),
        ]);
    }
    let clock = NaiveTime::parse_from_str(time, "%H:%M").ok()?;
    let secs = local_to_utc(tasks.zone(), day.and_time(clock))?;
    Some([
        ("due", json!(secs.to_string())),
        ("due_has_time", json!(true)),
    ])
}

pub fn due(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, t: &Task) {
    let bad = tasks.error.as_deref() == Some("aufgaben.fehler.datum");
    let err = bad.then(|| texts.t("aufgaben.fehler.datum"));
    let a = text_input(
        ui,
        &mut tasks.draft.date,
        &texts.t("aufgaben.feld.datum"),
        &texts.t("aufgaben.feld.datum.hinweis"),
        err.as_deref(),
    );
    let b = text_input(
        ui,
        &mut tasks.draft.time,
        &texts.t("aufgaben.feld.zeit"),
        &texts.t("aufgaben.feld.zeit.hinweis"),
        None,
    );
    if a.lost_focus() || b.lost_focus() {
        let now = draft_due(tasks, texts, t);
        if now != (tasks.draft.date.clone(), tasks.draft.time.clone()) {
            match parse_due(tasks, texts) {
                Some(fields) => tasks.set(&t.id, &fields),
                None => tasks.error = Some("aufgaben.fehler.datum".into()),
            }
        }
    }
}

pub fn priority(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, t: &Task) {
    let name = texts.t("aufgaben.feld.prioritaet");
    ui.label(eframe::egui::RichText::new(&name).small());
    named_scope(ui, "aufgaben_prioritaet", Role::RadioGroup, &name, |ui| {
        ui.horizontal(|ui| {
            for p in 0..=3u8 {
                let label = texts.t(priority_key(p));
                let on = t.priority == p;
                let r = button(
                    ui,
                    if on {
                        ButtonKind::Secondary
                    } else {
                        ButtonKind::Quiet
                    },
                    &label,
                );
                r.widget_info(|| WidgetInfo::selected(Role::RadioButton, true, on, &label));
                if r.clicked() && !on {
                    tasks.set(&t.id, &[("priority", json!(p))]);
                }
            }
        });
    });
    ui.add_space(f32::from(TOKENS.spacing.s2));
}

const REPEATS: [(Option<RepeatKind>, &str); 5] = [
    (None, "aufgaben.wiederholung.keine"),
    (Some(RepeatKind::Daily), "aufgaben.wiederholung.taeglich"),
    (
        Some(RepeatKind::Weekly),
        "aufgaben.wiederholung.woechentlich",
    ),
    (Some(RepeatKind::Monthly), "aufgaben.wiederholung.monatlich"),
    (Some(RepeatKind::Yearly), "aufgaben.wiederholung.jaehrlich"),
];

const REMINDERS: [(Option<i64>, &str); 5] = [
    (None, "aufgaben.erinnerung.keine"),
    (Some(0), "aufgaben.erinnerung.puenktlich"),
    (Some(15), "aufgaben.erinnerung.viertel"),
    (Some(60), "aufgaben.erinnerung.stunde"),
    (Some(1440), "aufgaben.erinnerung.tag"),
];

/// Wiederholung und Erinnerung als Menüs (Erinnerung nur als Angabe; die Vorschau meldet nichts).
pub fn repeat_and_reminder(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, t: &Task) {
    let kind = t.repeat.as_ref().map(|r| r.kind.clone());
    let current = REPEATS
        .iter()
        .find(|(k, _)| *k == kind)
        .map_or("aufgaben.wiederholung.eigene", |(_, key)| key);
    let labels: Vec<String> = REPEATS.iter().map(|(_, k)| texts.t(k)).collect();
    let items: Vec<MenuItem<'_>> = labels
        .iter()
        .map(|l| MenuItem {
            label: l,
            shortcut: None,
            danger: false,
        })
        .collect();
    let title = texts.tf("aufgaben.feld.wiederholung", &[("wert", &texts.t(current))]);
    if let Some((k, _)) = menu_button(ui, &title, &items).and_then(|i| REPEATS.get(i)) {
        let rule = k
            .as_ref()
            .map(|kind| json!({ "kind": kind, "interval": 1 }))
            .unwrap_or(Value::Null);
        tasks.set(&t.id, &[("repeat_rule", rule)]);
    }
    let current = REMINDERS
        .iter()
        .find(|(m, _)| *m == t.remind_minutes)
        .map_or("aufgaben.erinnerung.eigene", |(_, key)| key);
    let labels: Vec<String> = REMINDERS.iter().map(|(_, k)| texts.t(k)).collect();
    let items: Vec<MenuItem<'_>> = labels
        .iter()
        .map(|l| MenuItem {
            label: l,
            shortcut: None,
            danger: false,
        })
        .collect();
    let title = texts.tf("aufgaben.feld.erinnerung", &[("wert", &texts.t(current))]);
    if let Some((m, _)) = menu_button(ui, &title, &items).and_then(|i| REMINDERS.get(i)) {
        tasks.set(
            &t.id,
            &[("remind_minutes", m.map_or(Value::Null, |v| json!(v)))],
        );
    }
}
