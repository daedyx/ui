//! Auswahl links, Ansicht der Liste, Rückfrage vor dem Löschen.
use daedyx_tasks_core::smart::View;

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
