//! Aufgaben-Oberfläche der Daedyx-Apps (egui), herausgelöst aus Desk M6a (Tasks T1a, nur Umzug): Seitenleiste
//! (Suche, Smart-Listen, Listen), in der Mitte Liste oder Board, rechts der Detailbereich. Alle Änderungen gehen
//! sofort durch den Aufgaben-Kern `daedyx-tasks-core` (kein „Speichern“). Texte über `lang/*.json` (Schlüssel
//! `aufgaben.*`, die App führt sie in ihren eigenen Sprachdateien), Farben nur aus den Tokens von `daedyx-ui`.
mod actions;
mod actions_lists;
mod actions_tree;
mod board;
mod board_add;
mod board_card;
mod board_column;
mod board_header;
pub mod colors;
mod confirm;
mod detail;
mod detail_alarms;
mod detail_extras;
mod detail_fields;
mod detail_repeat;
mod detail_tree;
mod draft;
mod draft_commit;
mod due;
mod features;
mod kinds;
mod layout;
mod list_menu;
mod list_sort;
mod list_view;
pub mod model;
mod refresh;
mod row_paint;
mod rows;
pub mod seed;
mod sidebar;
mod sidebar_drag;
mod sidebar_entry;
mod sidebar_lists;
mod start;
pub mod store;
mod store_db;
mod store_due;
mod store_sections;
mod store_tree;
pub mod texts;
pub mod tree_model;

pub use draft::Draft;
pub use features::Features;
pub use kinds::{Ask, Mode, Sel};
pub use layout::show;
pub use start::start_with;

use daedyx_tasks_core::smart::View;
use daedyx_tasks_core::types::{Section, Task, TaskList};
use std::sync::Arc;
pub use store::{Clock, TaskDb, TaskStore};

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
    /// Was der Baustein in dieser App kann (Standard Desk).
    pub features: Features,
    /// Reihenfolge der obersten Aufgaben (nur mit `Features::sorting`).
    pub sort: tree_model::SortMode,
    /// Listen, die zuletzt als Board offen waren (`Features::board_memory`).
    pub board_lists: std::collections::HashSet<String>,
    /// Spalte (`None` = „Ohne Abschnitt“) mit offenem Feld „+ Karte“ und dessen Entwurf.
    pub adding_card: Option<(Option<String>, String)>,
    /// Erledigt/gesamt über alle Nachfahren je Aufgabe, einmal je `refresh()` gerechnet (Zeilen und Karten lesen nur).
    pub progress: std::collections::HashMap<String, (usize, usize)>,
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
            features: Features::DESK,
            sort: tree_model::SortMode::Hand,
            board_lists: Default::default(),
            adding_card: None,
            progress: Default::default(),
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
}

#[cfg(test)]
mod alarm_tests;
#[cfg(test)]
mod board_more_tests;
#[cfg(test)]
mod colors_tests;
#[cfg(test)]
mod deep_tests;
#[cfg(test)]
mod draft_commit_tests;
#[cfg(test)]
mod due_tests;
#[cfg(test)]
mod gap_tests;
#[cfg(test)]
mod guard_tests;
#[cfg(test)]
mod harness_tests;
#[cfg(test)]
mod lists_tests;
#[cfg(test)]
mod model_tests;
#[cfg(test)]
mod parity_tests;
#[cfg(test)]
mod repeat_tests;
#[cfg(test)]
mod sort_drag_tests;
#[cfg(test)]
mod store_tests;
#[cfg(test)]
mod switch_tests;
#[cfg(test)]
mod t3b_fix_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tree_model_tests;
