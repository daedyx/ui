//! Auswahl wechseln und neu laden: Listen, Zähler, Aufgaben und Abschnitte kommen nach jeder Änderung frisch aus dem
//! Kern. Mit `Features::board_memory` merkt sich der Bereich je Liste, ob sie als Board offen war (Tasks T3b).
use super::{Mode, Sel, TasksUi};
use daedyx_tasks_core::smart::View;

impl TasksUi {
    pub fn select(&mut self, sel: Sel) {
        if self.sel != sel || !self.search.is_empty() {
            self.sel = sel;
            self.selected = None;
            self.search.clear();
            match self.list_id().map(str::to_owned) {
                None => self.mode = Mode::List,
                Some(id) if self.features.board_memory => {
                    self.mode = if self.board_lists.contains(&id) {
                        Mode::Board
                    } else {
                        Mode::List
                    };
                }
                Some(_) => {}
            }
            self.adding_card = None;
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
        self.progress = super::tree_model::progress(&self.items);
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

impl TasksUi {
    /// Merkt sich je Liste, ob sie als Board offen ist (jedes Bild, damit auch Umschalter der App zählen).
    pub(crate) fn remember_mode(&mut self) {
        if !self.features.board_memory {
            return;
        }
        if let Some(id) = self.list_id().map(str::to_owned) {
            if self.mode == Mode::Board {
                self.board_lists.insert(id);
            } else {
                self.board_lists.remove(&id);
            }
        }
    }
}

impl TasksUi {
    /// Nur die Treffer neu laden, wenn sich der Suchtext ändert (Review T3b, N3b-4): Listen und Zähler bleiben.
    pub(crate) fn refresh_search(&mut self) {
        if self.search.trim().is_empty() {
            self.refresh();
            return;
        }
        let Some(store) = self.store.as_deref() else {
            return;
        };
        self.items = store.search(&self.search).unwrap_or_default();
        self.progress = super::tree_model::progress(&self.items);
    }
}
