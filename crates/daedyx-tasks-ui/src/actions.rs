//! Änderungen aus der Oberfläche. Jede geht durch den Aufgaben-Kern und lädt danach neu; ein Fehler
//! wird als Text-Schlüssel gemerkt (`aufgaben.fehler.<code>`) und in der Kopfzeile gezeigt.
use super::model::{cards, drop_position, dropped_sections, moved_sections};
use super::{Ask, Mode, TasksUi};
use crate::store::TaskStore;
use daedyx_tasks_core::TaskError;
use serde_json::{Map, Value};

/// Präfix der Fehlertexte; der Rest ist `TaskError::code()` (Test: jeder Code hat einen Text).
pub const ERROR_PREFIX: &str = "aufgaben.fehler.";

impl TasksUi {
    pub(crate) fn run<T>(
        &mut self,
        f: impl FnOnce(&TaskStore) -> Result<T, TaskError>,
    ) -> Option<T> {
        let result = f(self.store.as_deref()?);
        let out = match result {
            Ok(v) => {
                self.error = None;
                Some(v)
            }
            Err(e) => {
                self.error = Some(format!("{ERROR_PREFIX}{}", e.code()));
                None
            }
        };
        self.refresh();
        out
    }

    /// Schnelleingabe in die gewählte Liste (in Smart-Listen in die erste Liste; im Board ohne Abschnitt).
    pub fn quick_add(&mut self) {
        let text = std::mem::take(&mut self.quick);
        let Some(list) = self
            .list_id()
            .map(str::to_owned)
            .or_else(|| self.lists.first().map(|l| l.id.clone()))
        else {
            return;
        };
        let today = self.sel == super::Sel::View(daedyx_tasks_core::smart::View::Today);
        if let Some(Some(t)) = self.run(|s| s.quick(&list, None, &text)) {
            // In „Heute“ ohne genannten Tag: fällig heute, sonst verschwände sie gleich wieder aus der Ansicht.
            if today && t.due.is_none() {
                let day = self
                    .store
                    .as_deref()
                    .map(|s| s.today().format("%Y-%m-%d").to_string());
                if let Some(day) = day {
                    self.set(&t.id, &[("due", serde_json::json!(day))]);
                }
            }
            self.selected = Some(t.id);
        }
    }

    pub fn toggle(&mut self, id: &str) {
        let _ = self.run(|s| s.toggle(id));
    }

    /// Ein Feld der Aufgabe ändern (Namen wie im Kern: `title`, `notes`, `due`, `priority` …).
    pub fn set(&mut self, id: &str, fields: &[(&str, Value)]) {
        let patch: Map<String, Value> = fields
            .iter()
            .map(|(k, v)| ((*k).to_owned(), v.clone()))
            .collect();
        let _ = self.run(|s| s.update(id, &patch));
    }

    /// Nach der Rückfrage: löschen, was gefragt wurde.
    pub fn confirm_delete(&mut self) {
        match self.ask.take() {
            Some(Ask::Task(id)) => {
                if self.run(|s| s.delete(&id)).is_some()
                    && self.selected.as_deref() == Some(id.as_str())
                {
                    self.selected = None;
                }
            }
            Some(Ask::Section(id)) => {
                let _ = self.run(|s| s.delete_section(&id));
            }
            Some(Ask::List(id)) => {
                let was_shown = self.list_id() == Some(id.as_str());
                let gone = self.run(|s| s.delete_list(&id)).is_some();
                if gone && was_shown {
                    self.select(super::Sel::View(daedyx_tasks_core::smart::View::Today));
                }
            }
            None => {}
        }
    }

    /// Karte `id` in Spalte `column` an Stelle `index` (unter den übrigen Karten dieser Spalte).
    pub fn move_card(&mut self, id: &str, column: Option<&str>, index: usize) {
        let Some(list) = self.list_id().map(str::to_owned) else {
            return;
        };
        let others: Vec<f64> = cards(&self.items, &self.sections, column)
            .iter()
            .filter(|t| t.id != id)
            .map(|t| t.position)
            .collect();
        let pos = drop_position(&others, index);
        let _ = self.run(|s| s.move_task(id, &list, column, pos));
    }

    pub fn move_section(&mut self, id: &str, step: isize) {
        if let (Some(list), Some(ids)) = (
            self.list_id().map(str::to_owned),
            moved_sections(&self.sections, id, step),
        ) {
            let _ = self.run(|s| s.reorder_sections(&list, &ids));
        }
    }

    pub fn drop_section(&mut self, id: &str, target: &str) {
        if let (Some(list), Some(ids)) = (
            self.list_id().map(str::to_owned),
            dropped_sections(&self.sections, id, target),
        ) {
            let _ = self.run(|s| s.reorder_sections(&list, &ids));
        }
    }

    pub fn rename_section(&mut self, id: &str, name: &str) {
        let _ = self.run(|s| s.rename_section(id, name));
    }

    pub fn add_section(&mut self) {
        let name = std::mem::take(&mut self.new_section);
        if let Some(list) = self
            .list_id()
            .map(str::to_owned)
            .filter(|_| !name.trim().is_empty())
        {
            let _ = self.run(|s| s.create_section(&list, &name));
        }
    }

    /// Neue Liste mit der nächsten Tokenfarbe; sie wird gleich ausgewählt.
    pub fn add_list(&mut self) {
        let name = std::mem::take(&mut self.new_list);
        if name.trim().is_empty() {
            return;
        }
        let color = super::colors::stored(self.lists.len());
        if let Some(l) = self.run(|s| s.create_list(&name, &color)) {
            self.select(super::Sel::List(l.id));
        }
    }

    pub fn rename_list(&mut self, id: &str, name: &str) {
        let _ = self.run(|s| s.rename_list(id, name));
    }

    /// Farbe Nummer `i` der Tokens (siehe `colors`).
    pub fn set_list_color(&mut self, id: &str, i: usize) {
        let color = super::colors::stored(i);
        let _ = self.run(|s| s.set_list_color(id, &color));
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = if self.list_id().is_some() {
            mode
        } else {
            Mode::List
        };
    }
}
