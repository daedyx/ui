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
    /// Ebene der Aufgabe (0 = oberste), beim Laden gemerkt.
    pub depth: u32,
    /// Erinnerungen in Minuten vor der Fälligkeit (nur mit mehreren Erinnerungen genutzt).
    pub alarms: Vec<i64>,
    /// Felder der vollen Wiederholung: Abstand und fester Monatstag als Eingabetext.
    pub interval: String,
    pub month_day: String,
    /// Regel beim letzten Laden, damit die Zahlenfelder bei einer Änderung von außen nachziehen.
    pub repeat_seen: Option<daedyx_tasks_core::types::RepeatRule>,
}
