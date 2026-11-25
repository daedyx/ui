//! Volle Wiederholung im Detail (Tasks T3a, `Features::full_repeat`; Vorlage `RepeatFields.tsx`): Art, Abstand 1–999,
//! bei „wöchentlich“ die Wochentage 1–7 als Kontrollkästchen, bei „monatlich“/„jährlich“ ein fester Tag 1–31 (leer =
//! Tag der Fälligkeit), „ab dem Abhaken zählen“. Zahlenfelder gelten beim Verlassen, Unsinn wird begrenzt.
use super::TasksUi;
use super::detail_fields::REPEATS;
use daedyx_tasks_core::repeat::MAX_INTERVAL;
use daedyx_tasks_core::types::{RepeatKind, RepeatRule, Task};
use daedyx_ui::a11y::named_scope;
use daedyx_ui::i18n::Texts;
use daedyx_ui::widgets::{ButtonKind, MenuItem, button, menu_button, text_input};
use eframe::egui::{Ui, WidgetInfo, accesskit::Role};

fn save(tasks: &mut TasksUi, id: &str, rule: Option<RepeatRule>) {
    let v = rule
        .and_then(|r| serde_json::to_value(r).ok())
        .unwrap_or(serde_json::Value::Null);
    tasks.set(id, &[("repeat_rule", v)]);
}

/// Neue Regel beim Wechsel der Art: Abstand und Zählweise bleiben, Wochentage nur wöchentlich, Monatstag nur
/// monatlich/jährlich.
pub fn with_kind(old: Option<&RepeatRule>, kind: Option<RepeatKind>) -> Option<RepeatRule> {
    let kind = kind?;
    let monthly = matches!(kind, RepeatKind::Monthly | RepeatKind::Yearly);
    Some(RepeatRule {
        weekdays: if kind == RepeatKind::Weekly {
            old.map(|r| r.weekdays.clone()).unwrap_or_default()
        } else {
            vec![]
        },
        month_day: if monthly {
            old.and_then(|r| r.month_day)
        } else {
            None
        },
        interval: old.map_or(1, |r| r.interval),
        from_done: old.is_some_and(|r| r.from_done),
        kind,
    })
}

/// Zahl aus einem Feld, begrenzt auf `lo..=hi`; leer oder unlesbar ergibt `None`.
pub fn clamp_field(text: &str, lo: u32, hi: u32) -> Option<u32> {
    text.trim()
        .parse::<i64>()
        .ok()
        .map(|n| u32::try_from(n.clamp(i64::from(lo), i64::from(hi))).unwrap_or(lo))
}

fn kind_menu(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, t: &Task) {
    let kind = t.repeat.as_ref().map(|r| r.kind.clone());
    let current = REPEATS
        .iter()
        .find(|(k, _)| *k == kind)
        .map_or("aufgaben.wiederholung.eigene", |(_, k)| k);
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
        save(tasks, &t.id, with_kind(t.repeat.as_ref(), k.clone()));
    }
}

/// Wochentage 1 (Montag) bis 7 (Sonntag): kurz auf dem Knopf, lang für Screenreader.
pub(crate) const DAYS_SHORT: [&str; 7] = [
    "aufgaben.wochentag.kurz.1",
    "aufgaben.wochentag.kurz.2",
    "aufgaben.wochentag.kurz.3",
    "aufgaben.wochentag.kurz.4",
    "aufgaben.wochentag.kurz.5",
    "aufgaben.wochentag.kurz.6",
    "aufgaben.wochentag.kurz.7",
];
const DAYS: [&str; 7] = [
    "aufgaben.wochentag.1",
    "aufgaben.wochentag.2",
    "aufgaben.wochentag.3",
    "aufgaben.wochentag.4",
    "aufgaben.wochentag.5",
    "aufgaben.wochentag.6",
    "aufgaben.wochentag.7",
];

fn weekdays(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, t: &Task, rule: &RepeatRule) {
    let name = texts.t("aufgaben.wiederholung.wochentage");
    named_scope(ui, "aufgaben_wochentage", Role::Group, &name, |ui| {
        ui.horizontal_wrapped(|ui| {
            for ((d, short), long) in (1..=7u8).zip(DAYS_SHORT).zip(DAYS) {
                let on = rule.weekdays.contains(&d);
                let kind = if on {
                    ButtonKind::Secondary
                } else {
                    ButtonKind::Quiet
                };
                let r = button(ui, kind, &texts.t(short));
                let spoken = texts.t(long);
                r.widget_info(|| WidgetInfo::selected(Role::CheckBox, true, on, &spoken));
                if r.clicked() {
                    let mut next = rule.clone();
                    if on {
                        next.weekdays.retain(|x| *x != d);
                    } else {
                        next.weekdays.push(d);
                        next.weekdays.sort_unstable();
                    }
                    save(tasks, &t.id, Some(next));
                }
            }
        });
    });
}

fn numbers(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, t: &Task, rule: &RepeatRule) {
    let unit = match rule.kind {
        RepeatKind::Daily => "aufgaben.wiederholung.tage",
        RepeatKind::Weekly => "aufgaben.wiederholung.wochen",
        RepeatKind::Monthly => "aufgaben.wiederholung.monate",
        RepeatKind::Yearly => "aufgaben.wiederholung.jahre",
    };
    let label = texts.tf(
        "aufgaben.wiederholung.abstand",
        &[("einheit", &texts.t(unit))],
    );
    let r = text_input(ui, &mut tasks.draft.interval, &label, "", None);
    if r.lost_focus() {
        let n = clamp_field(&tasks.draft.interval, 1, MAX_INTERVAL).unwrap_or(rule.interval);
        tasks.draft.interval = n.to_string();
        if n != rule.interval {
            save(
                tasks,
                &t.id,
                Some(RepeatRule {
                    interval: n,
                    ..rule.clone()
                }),
            );
        }
    }
    if matches!(rule.kind, RepeatKind::Monthly | RepeatKind::Yearly) {
        let label = texts.t("aufgaben.wiederholung.monatstag");
        let hint = texts.t("aufgaben.wiederholung.monatstag.hinweis");
        let r = text_input(ui, &mut tasks.draft.month_day, &label, &hint, None);
        if r.lost_focus() {
            let day = clamp_field(&tasks.draft.month_day, 1, 31).and_then(|d| u8::try_from(d).ok());
            tasks.draft.month_day = day.map(|d| d.to_string()).unwrap_or_default();
            if day != rule.month_day {
                save(
                    tasks,
                    &t.id,
                    Some(RepeatRule {
                        month_day: day,
                        ..rule.clone()
                    }),
                );
            }
        }
    }
}

pub fn show(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, t: &Task) {
    kind_menu(ui, tasks, texts, t);
    let Some(rule) = t.repeat.clone() else {
        return;
    };
    numbers(ui, tasks, texts, t, &rule);
    if rule.kind == RepeatKind::Weekly {
        weekdays(ui, tasks, texts, t, &rule);
    }
    let mut from_done = rule.from_done;
    if ui
        .checkbox(&mut from_done, texts.t("aufgaben.wiederholung.ab_abhaken"))
        .changed()
    {
        save(tasks, &t.id, Some(RepeatRule { from_done, ..rule }));
    }
}
