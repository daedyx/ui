//! Datenquelle der Aufgaben-Oberfläche: der Aufgaben-Kern auf einer Datenbank der App. Die App reicht die Datenbank
//! über [`TaskDb`] herein (Desk: Speicher-Datenbank der Vorschau oder die des Kerns). [`TaskStore::memory`] legt eine
//! leere Speicher-Datenbank mit dem Kern-Schema an (Testrahmen, Vorschau). Nie ein Datenordner, nie der Tresor.
use chrono::{NaiveDate, NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use daedyx_tasks_core::edit_done::toggle_done;
use daedyx_tasks_core::edit_lists as lists;
use daedyx_tasks_core::edit_tasks as edit;
use daedyx_tasks_core::quick::parse_quick_task;
use daedyx_tasks_core::smart::{self, View};
use daedyx_tasks_core::types::{Task, TaskList};
use daedyx_tasks_core::{TaskError, store};
use rusqlite::Connection;
use serde_json::{Map, Value};
use std::sync::Mutex;

pub use crate::store_db::{Clock, TaskDb};

/// Abhaken: Standard ist der Kern; Desk setzt seine Fassung mit zweiter Regelprüfung ein.
pub type ToggleFn = fn(&Connection, &str, u64, Tz) -> Result<Task, TaskError>;

pub struct TaskStore {
    conn: Box<dyn TaskDb>,
    tz: Tz,
    clock: Clock,
    toggle: ToggleFn,
}

impl TaskStore {
    /// Leere Datenbank im Arbeitsspeicher mit dem Schema des Kerns.
    pub fn memory(tz: Tz, clock: Clock) -> Result<TaskStore, TaskError> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        daedyx_tasks_core::migrate(&conn).map_err(|_| TaskError::Invalid("schema"))?;
        Ok(TaskStore::on(Box::new(Mutex::new(conn)), tz, clock))
    }

    /// Aufgaben auf einer vorhandenen Datenbank, ohne erfundene Einträge.
    pub fn on(db: Box<dyn TaskDb>, tz: Tz, clock: Clock) -> TaskStore {
        TaskStore {
            conn: db,
            tz,
            clock,
            toggle: toggle_done,
        }
    }

    /// Eigene Fassung des Abhakens (Desk: `daedyx_desk_core::tasks::edit_done::toggle_done`).
    pub fn with_toggle(mut self, toggle: ToggleFn) -> TaskStore {
        self.toggle = toggle;
        self
    }

    pub fn zone(&self) -> Tz {
        self.tz
    }

    /// Ortszeit der Anzeige.
    pub fn now(&self) -> NaiveDateTime {
        match self.clock {
            Clock::Fixed(t) => t,
            Clock::System => chrono::Utc::now().with_timezone(&self.tz).naive_local(),
        }
    }

    pub fn today(&self) -> NaiveDate {
        self.now().date()
    }

    pub(crate) fn now_ms(&self) -> u64 {
        let secs = self
            .tz
            .from_local_datetime(&self.now())
            .earliest()
            .map_or(0, |d| d.timestamp());
        u64::try_from(secs).unwrap_or(0).saturating_mul(1000)
    }

    pub(crate) fn with<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, TaskError>,
    ) -> Result<T, TaskError> {
        let conn = self.conn.lock();
        f(&conn)
    }

    pub fn lists(&self) -> Result<Vec<TaskList>, TaskError> {
        self.with(store::lists)
    }

    pub fn create_list(&self, name: &str, color: &str) -> Result<TaskList, TaskError> {
        let now = self.now_ms();
        self.with(|c| lists::create_list(c, name, color, now))
    }

    pub fn rename_list(&self, id: &str, name: &str) -> Result<(), TaskError> {
        let now = self.now_ms();
        self.with(|c| lists::rename_list(c, id, name, now))
    }

    pub fn set_list_color(&self, id: &str, color: &str) -> Result<(), TaskError> {
        let now = self.now_ms();
        self.with(|c| lists::set_list_color(c, id, color, now))
    }

    /// Löscht die Liste samt Aufgaben und Abschnitten (im Kern als Grabstein).
    pub fn delete_list(&self, id: &str) -> Result<(), TaskError> {
        let now = self.now_ms();
        self.with(|c| lists::delete_list(c, id, now))
    }

    pub fn query(&self, view: View, list: Option<&str>) -> Result<Vec<Task>, TaskError> {
        let (today, tz) = (self.today(), self.tz);
        self.with(|c| smart::query(c, view, list, today, tz))
    }

    pub fn counts(&self) -> Result<Vec<(View, usize)>, TaskError> {
        let (today, tz) = (self.today(), self.tz);
        self.with(|c| smart::counts(c, today, tz))
            .map(|a| a.to_vec())
    }

    pub fn search(&self, text: &str) -> Result<Vec<Task>, TaskError> {
        self.with(|c| smart::search(c, text, 50))
    }

    pub fn task(&self, id: &str) -> Result<Task, TaskError> {
        self.with(|c| store::task(c, id))
    }

    pub fn create(
        &self,
        list: &str,
        parent: Option<&str>,
        fields: &Map<String, Value>,
    ) -> Result<Task, TaskError> {
        let now = self.now_ms();
        self.with(|c| edit::create_task(c, list, parent, fields, now))
    }

    /// Schnelleingabe („Steuer morgen 15 Uhr !!“). `Ok(None)`, wenn kein Titel übrig bleibt.
    pub fn quick(
        &self,
        list: &str,
        section: Option<&str>,
        text: &str,
    ) -> Result<Option<Task>, TaskError> {
        let Some(q) = parse_quick_task(text, self.now(), self.tz) else {
            return Ok(None);
        };
        let mut fields = q.fields;
        if let Some(s) = section {
            fields.insert("section_id".into(), Value::String(s.to_owned()));
        }
        self.create(list, None, &fields).map(Some)
    }

    pub fn update(&self, id: &str, patch: &Map<String, Value>) -> Result<Task, TaskError> {
        let now = self.now_ms();
        self.with(|c| edit::update_task(c, id, patch, now))
    }

    pub fn toggle(&self, id: &str) -> Result<Task, TaskError> {
        let (now, tz) = (self.now_ms(), self.tz);
        let toggle = self.toggle;
        self.with(|c| toggle(c, id, now, tz))
    }

    pub fn move_task(
        &self,
        id: &str,
        list: &str,
        section: Option<&str>,
        position: f64,
    ) -> Result<Task, TaskError> {
        let now = self.now_ms();
        self.with(|c| edit::move_task(c, id, list, None, section, position, now))
    }

    pub fn delete(&self, id: &str) -> Result<(), TaskError> {
        let now = self.now_ms();
        self.with(|c| edit::delete_task(c, id, now))
    }
}
