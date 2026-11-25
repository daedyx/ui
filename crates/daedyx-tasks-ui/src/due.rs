//! Fälligkeit als kurzer Text („Heute“, „Morgen 15:30“, „24.12.“) und ihre Tönung (überfällig rot,
//! heute bernstein, sonst neutral). Muster aus der Sprachdatei; ein kaputtes Muster ergibt „?“.
use super::TasksUi;
use chrono::{DateTime, NaiveDate, TimeZone};
use chrono_tz::Tz;
use daedyx_tasks_core::smart::due_day;
use daedyx_tasks_core::types::Task;
use daedyx_ui::i18n::Texts;
use daedyx_ui::widgets::BadgeTone;
use std::fmt::Write;

/// Schreibt einen chrono-Wert; ein unbrauchbares Muster aus der Sprachdatei ergibt „?“ statt eines Absturzes.
fn formatted(value: impl std::fmt::Display) -> String {
    let mut text = String::new();
    if write!(text, "{value}").is_err() {
        return "?".into();
    }
    text
}

impl TasksUi {
    pub fn today(&self) -> NaiveDate {
        self.store.as_deref().map_or(NaiveDate::MIN, |s| s.today())
    }

    pub fn zone(&self) -> Tz {
        self.store.as_deref().map_or(chrono_tz::UTC, |s| s.zone())
    }

    /// Uhrzeit der Fälligkeit in der Anzeigezone (nur bei `due_has_time`).
    pub fn due_time(&self, t: &Task) -> Option<chrono::NaiveTime> {
        let secs: i64 = t.due.as_deref().filter(|_| t.due_has_time)?.parse().ok()?;
        Some(daedyx_tasks_core::due_time::local_at(self.zone(), secs)?.time())
    }

    /// Kurztext und Tönung der Fälligkeit; `None` ohne Fälligkeit.
    pub fn due_label(&self, texts: &Texts, t: &Task) -> Option<(String, BadgeTone)> {
        let day = due_day(t, self.zone())?;
        let today = self.today();
        let diff = day.signed_duration_since(today).num_days();
        let mut text = match diff {
            0 => texts.t("aufgaben.faellig.heute"),
            1 => texts.t("aufgaben.faellig.morgen"),
            -1 => texts.t("aufgaben.faellig.gestern"),
            _ => formatted(day.format(&texts.t("aufgaben.datum.kurz"))),
        };
        if let Some(time) = self.due_time(t) {
            text.push(' ');
            text.push_str(&formatted(time.format(&texts.t("aufgaben.zeit.format"))));
        }
        let done = t.done_at.is_some();
        let tone = if done {
            BadgeTone::Neutral
        } else if diff < 0 || self.past_time(t) {
            BadgeTone::Error
        } else if diff == 0 {
            BadgeTone::Warning
        } else {
            BadgeTone::Neutral
        };
        Some((text, tone))
    }

    fn past_time(&self, t: &Task) -> bool {
        let Some(secs) = t
            .due
            .as_deref()
            .filter(|_| t.due_has_time)
            .and_then(|d| d.parse::<i64>().ok())
        else {
            return false;
        };
        let now = self.store.as_deref().map(|s| s.now());
        let now_secs = now
            .and_then(|n| self.zone().from_local_datetime(&n).earliest())
            .map(|d: DateTime<Tz>| d.timestamp());
        now_secs.is_some_and(|n| secs < n)
    }
}
