//! Änderungen aus Liste und Detail für Tasks T3a: Unteraufgaben in erlaubter Tiefe, Erinnerungen, Umsortieren unter
//! Geschwistern (Ziehen und Alt+↑/↓). Alles über den Kern; Fehler wie in `actions` als Text-Schlüssel.
use super::TasksUi;
use super::model::drop_position;
use daedyx_tasks_core::types::Task;
use serde_json::{Map, Value};

impl TasksUi {
    /// Unteraufgabe unter `parent`, höchstens `Features::max_depth` Ebenen tief.
    pub fn add_subtask(&mut self, parent: &str, title: &str) {
        let Some(list) = self
            .items
            .iter()
            .find(|t| t.id == parent)
            .map(|t| t.list_id.clone())
        else {
            return;
        };
        let mut f = Map::new();
        f.insert("title".into(), Value::String(title.to_owned()));
        let depth = self.features.max_depth;
        let _ = self.run(|s| s.create_deep(&list, Some(parent), &f, depth));
    }

    /// Ebene der Aufgabe (0 = oberste); ohne Datenquelle 0.
    pub fn depth_of(&self, id: &str) -> u32 {
        self.store
            .as_deref()
            .and_then(|s| s.depth(id).ok())
            .unwrap_or(0)
    }

    pub fn alarms_of(&self, id: &str) -> Vec<i64> {
        self.store
            .as_deref()
            .and_then(|s| s.alarms(id).ok())
            .unwrap_or_default()
    }

    /// Erinnerungen ersetzen (aufsteigend, ohne Doppelte; mehr als erlaubt lehnt der Kern ab).
    pub fn set_alarms(&mut self, id: &str, minutes: &[i64]) {
        let mut v = minutes.to_vec();
        v.sort_unstable();
        v.dedup();
        if v.len() > self.features.alarms_max {
            self.error = Some(format!("{}invalid", super::actions::ERROR_PREFIX));
            return;
        }
        let _ = self.run(|s| s.set_alarms(id, &v));
        if self.draft.id == id {
            self.draft.alarms = self.alarms_of(id);
        }
    }

    /// Geschwister von `t` (gleiche Oberaufgabe, gleicher Abschnitt bei obersten) ohne `t`, nach Position.
    fn siblings(&self, t: &Task) -> Vec<Task> {
        let mut v: Vec<Task> = self
            .items
            .iter()
            .filter(|x| x.id != t.id && x.parent_id == t.parent_id && x.list_id == t.list_id)
            .cloned()
            .collect();
        v.sort_by(|a, b| a.position.total_cmp(&b.position));
        v
    }

    /// Aufgabe `id` an Stelle `index` unter ihren Geschwistern (Handreihenfolge); bleibt in Liste, Oberaufgabe und
    /// Abschnitt.
    pub fn reorder(&mut self, id: &str, index: usize) {
        let Some(t) = self.items.iter().find(|x| x.id == id).cloned() else {
            return;
        };
        let others: Vec<f64> = self.siblings(&t).iter().map(|x| x.position).collect();
        let pos = drop_position(&others, index);
        let depth = self.features.max_depth;
        let to = (
            t.list_id.as_str(),
            t.parent_id.as_deref(),
            t.section_id.as_deref(),
        );
        let _ = self.run(|s| s.move_deep(id, to, pos, depth));
    }

    /// Aufgabe `id` direkt vor `before` einsortieren (Ziehen auf eine Geschwister-Zeile).
    pub fn reorder_before(&mut self, id: &str, before: &str) {
        let Some(t) = self.items.iter().find(|x| x.id == id).cloned() else {
            return;
        };
        if let Some(i) = self.siblings(&t).iter().position(|x| x.id == before) {
            self.reorder(id, i);
        }
    }

    /// Eine Stelle nach oben (`-1`) oder unten (`+1`) unter den Geschwistern (Alt+↑/↓).
    pub fn reorder_step(&mut self, id: &str, step: isize) {
        let Some(t) = self.items.iter().find(|x| x.id == id).cloned() else {
            return;
        };
        let mut all = self.siblings(&t);
        all.push(t.clone());
        all.sort_by(|a, b| a.position.total_cmp(&b.position));
        let Some(at) = all.iter().position(|x| x.id == id) else {
            return;
        };
        if let Some(to) = at.checked_add_signed(step).filter(|to| *to < all.len()) {
            self.reorder(id, to);
        }
    }
}
