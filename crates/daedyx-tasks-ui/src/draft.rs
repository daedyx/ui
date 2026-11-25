//! Entwurf der Felder im Detailbereich.

/// Eingaben im Detailbereich, die erst beim Verlassen des Feldes (oder Enter) gelten.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Draft {
    pub id: String,
    pub title: String,
    pub notes: String,
    pub date: String,
    pub time: String,
    pub subtask: String,
    /// Stand der Aufgabe beim letzten Laden (Titel, Notiz, Fälligkeit, mit Uhrzeit).
    pub seen: (String, String, Option<String>, bool),
}
