//! Start des Aufgabenbereichs mit einer Datenquelle der App und der Startauswahl.
use super::{Mode, Sel, TasksUi};
use crate::store::TaskStore;
use std::sync::Arc;

/// Bereich mit `store` (ohne: leer). `list` wählt Liste Nummer `n` (ab 0), mit `true` gleich im Board.
pub fn start_with(store: Option<TaskStore>, list: Option<(usize, bool)>) -> TasksUi {
    let mut ui = TasksUi::new(store);
    if let Some((n, board)) = list
        && let Some(id) = ui.lists.get(n).map(|l| l.id.clone())
    {
        ui.select(Sel::List(id));
        ui.mode = if board { Mode::Board } else { Mode::List };
    }
    ui
}

impl TasksUi {
    /// Dieselbe Datenbank für den Hintergrund-Takt (Erinnerungen).
    pub fn shared_store(&self) -> Option<Arc<TaskStore>> {
        self.store.clone()
    }
}
