//! Baum und Erinnerungen über den Kern (Tasks T3a): Ebene einer Aufgabe, Anlegen und Verschieben mit erlaubter Tiefe,
//! mehrere Erinnerungen je Aufgabe. Die Tiefe gibt die App vor (`Features::max_depth`).
use crate::store::TaskStore;
use daedyx_tasks_core::TaskError;
use daedyx_tasks_core::edit_alarms;
use daedyx_tasks_core::edit_tasks as edit;
use daedyx_tasks_core::tree;
use daedyx_tasks_core::types::Task;
use serde_json::{Map, Value};

impl TaskStore {
    /// Ebene der Aufgabe: 0 = oberste, 1 = Unteraufgabe usw.
    pub fn depth(&self, id: &str) -> Result<u32, TaskError> {
        self.with(|c| tree::depth_of(c, id))
    }

    /// Neue Aufgabe, unter `parent` höchstens `max_depth` Ebenen tief (sonst `TooDeep`).
    pub fn create_deep(
        &self,
        list: &str,
        parent: Option<&str>,
        fields: &Map<String, Value>,
        max_depth: u32,
    ) -> Result<Task, TaskError> {
        let now = self.now_ms();
        self.with(|c| edit::create_task_depth(c, list, parent, fields, now, max_depth))
    }

    /// Verschiebt samt Unterbaum (Liste, Oberaufgabe, Abschnitt, Position), höchstens `max_depth` Ebenen tief.
    pub fn move_deep(
        &self,
        id: &str,
        to: (&str, Option<&str>, Option<&str>),
        position: f64,
        max_depth: u32,
    ) -> Result<Task, TaskError> {
        let (list, parent, section) = to;
        let now = self.now_ms();
        self.with(|c| edit::move_task_depth(c, id, list, parent, section, position, now, max_depth))
    }

    /// Erinnerungen in Minuten vor der Fälligkeit, aufsteigend.
    pub fn alarms(&self, id: &str) -> Result<Vec<i64>, TaskError> {
        self.with(|c| edit_alarms::alarms(c, id))
    }

    /// Ersetzt die Erinnerungen (höchstens `MAX_ALARMS`, der Kern prüft).
    pub fn set_alarms(&self, id: &str, minutes: &[i64]) -> Result<(), TaskError> {
        let now = self.now_ms();
        self.with(|c| edit_alarms::set_alarms(c, id, minutes, now))
    }
}
