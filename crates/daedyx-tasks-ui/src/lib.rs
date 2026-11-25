//! Aufgaben-Oberfläche der Daedyx-Apps (egui), herausgelöst aus Desk M6a (Tasks T1a, nur Umzug): Seitenleiste
//! (Suche, Smart-Listen, Listen), in der Mitte Liste oder Board, rechts der Detailbereich. Alle Änderungen gehen
//! sofort durch den Aufgaben-Kern `daedyx-tasks-core` (kein „Speichern“). Texte über `lang/*.json` (Schlüssel
//! `aufgaben.*`, die App führt sie in ihren eigenen Sprachdateien), Farben nur aus den Tokens von `daedyx-ui`.
mod actions;
mod board;
mod board_card;
mod board_column;
mod board_header;
pub mod colors;
mod confirm;
mod detail;
mod detail_fields;
mod draft;
mod draft_commit;
mod due;
mod layout;
mod list_menu;
mod list_view;
pub mod model;
mod row_paint;
mod rows;
pub mod seed;
mod sidebar;
mod sidebar_lists;
mod start;
pub mod store;
mod store_db;
mod store_due;
mod store_sections;
pub mod texts;

pub use draft::Draft;
pub use layout::show;
pub use start::start_with;

use daedyx_tasks_core::smart::View;
use daedyx_tasks_core::types::{Section, Task, TaskList};
use std::sync::Arc;
pub use store::{Clock, TaskDb, TaskStore};

/// Was links ausgewählt ist.
#[derive(Debug, Clone, PartialEq)]
pub enum Sel {
    View(View),
    List(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    List,
    Board,
}

/// Rückfrage vor dem Löschen.
#[derive(Debug, Clone, PartialEq)]
pub enum Ask {
    Task(String),
    Section(String),
    List(String),
}

pub struct TasksUi {
    store: Option<Arc<TaskStore>>,
    pub sel: Sel,
    pub mode: Mode,
    pub lists: Vec<TaskList>,
    pub counts: Vec<(View, usize)>,
    pub items: Vec<Task>,
    pub sections: Vec<Section>,
    pub selected: Option<String>,
    pub quick: String,
    pub search: String,
    pub ask: Option<Ask>,
    /// Abschnitt, der gerade umbenannt wird, mit Entwurf.
    pub renaming: Option<(String, String)>,
    pub new_section: String,
    pub new_list: String,
    /// Liste, die gerade umbenannt wird, mit Entwurf.
    pub renaming_list: Option<(String, String)>,
    pub draft: Draft,
    /// Umschalter Liste/Board im Listenkopf zeigen. Desk ja; Tasks nein, dort wechselt die Navigationsleiste.
    pub mode_switch: bool,
    /// Schlüssel der letzten Fehlermeldung (`aufgaben.fehler.*`).
    pub error: Option<String>,
}

impl TasksUi {
    pub fn new(store: Option<TaskStore>) -> TasksUi {
        let mut ui = TasksUi {
            store: store.map(Arc::new),
            sel: Sel::View(View::Today),
            mode: Mode::List,
            lists: Vec::new(),
            counts: Vec::new(),
            items: Vec::new(),
            sections: Vec::new(),
            selected: None,
            quick: String::new(),
            search: String::new(),
            ask: None,
            renaming: None,
            new_section: String::new(),
            new_list: String::new(),
            renaming_list: None,
            draft: Draft::default(),
            mode_switch: true,
            error: None,
        };
        ui.refresh();
        ui
    }

    pub fn list_id(&self) -> Option<&str> {
        match &self.sel {
            Sel::List(id) => Some(id),
            Sel::View(_) => None,
        }
    }

    pub fn selected_task(&self) -> Option<&Task> {
        let id = self.selected.as_deref()?;
        self.items.iter().find(|t| t.id == id)
    }

    pub fn count(&self, view: View) -> Option<usize> {
        self.counts
            .iter()
            .find(|(v, _)| *v == view)
            .map(|(_, n)| *n)
    }

    pub fn select(&mut self, sel: Sel) {
        if self.sel != sel || !self.search.is_empty() {
            self.sel = sel;
            self.selected = None;
            self.search.clear();
            if self.list_id().is_none() {
                self.mode = Mode::List;
            }
            self.refresh();
        }
    }

    /// Lädt Listen, Zähler, Aufgaben und Abschnitte neu (nach jeder Änderung).
    pub fn refresh(&mut self) {
        let Some(store) = self.store.as_deref() else {
            return;
        };
        self.lists = store.lists().unwrap_or_default();
        self.counts = store.counts().unwrap_or_default();
        if let Sel::List(id) = &self.sel
            && !self.lists.iter().any(|l| &l.id == id)
        {
            self.sel = Sel::View(View::Today);
        }
        let (view, list) = match &self.sel {
            Sel::View(v) => (*v, None),
            Sel::List(id) => (View::List, Some(id.as_str())),
        };
        self.items = if self.search.trim().is_empty() {
            store.query(view, list).unwrap_or_default()
        } else {
            store.search(&self.search).unwrap_or_default()
        };
        self.sections = list
            .map(|l| store.sections(l).unwrap_or_default())
            .unwrap_or_default();
        if self
            .selected
            .as_deref()
            .is_some_and(|id| !self.items.iter().any(|t| t.id == id))
        {
            self.selected = None;
        }
    }
}

#[cfg(test)]
mod colors_tests;
#[cfg(test)]
mod draft_commit_tests;
#[cfg(test)]
mod gap_tests;
#[cfg(test)]
mod guard_tests;
#[cfg(test)]
mod harness_tests;
#[cfg(test)]
mod model_tests;
#[cfg(test)]
mod store_tests;
#[cfg(test)]
mod switch_tests;
#[cfg(test)]
mod test_support;
