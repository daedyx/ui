//! Uhr und Datenbank-Zugang der Aufgaben-Oberfläche.
use chrono::NaiveDateTime;
use rusqlite::Connection;
use std::sync::{Mutex, MutexGuard, PoisonError};

/// Uhr der Oberfläche: echte Zeit, in Tests fest.
#[derive(Debug, Clone, Copy)]
pub enum Clock {
    System,
    Fixed(NaiveDateTime),
}

/// Wo die Datenbank liegt; die App entscheidet. Eine vergiftete Sperre wird übernommen statt abzustürzen.
pub trait TaskDb: Send + Sync {
    fn lock(&self) -> MutexGuard<'_, Connection>;
}

impl TaskDb for Mutex<Connection> {
    // Ausnahme in docs/SICHERHEIT-AUSNAHMEN.md (Tasks T1a): eine Verbindung, kurz gesperrt wie bisher in Desk.
    #[allow(clippy::disallowed_methods)]
    fn lock(&self) -> MutexGuard<'_, Connection> {
        Mutex::lock(self).unwrap_or_else(PoisonError::into_inner)
    }
}
