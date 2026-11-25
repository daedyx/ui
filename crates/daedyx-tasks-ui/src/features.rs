//! Was der Baustein je App kann (Plan Tasks-Rust §3, T3a). Standard ist Desk, so wie M6a gebaut wurde: eine Ebene
//! Unteraufgaben, eine Erinnerung als Menü, einfache Wiederholung, keine Sortierungen, kein Ziehen in der Liste, keine
//! Zusatzfelder im Detail, kein Umsortieren der Listen, kein Ziehen auf Listen, kein „+ Karte“. Tasks schaltet alles ein ([`Features::TASKS`]).
use daedyx_tasks_core::edit_alarms::MAX_ALARMS;
use daedyx_tasks_core::tree::DESK_MAX_DEPTH;

/// Tiefste Ebene der Unteraufgaben in Tasks (0 = oberste Aufgabe), wie `TASKS_MAX_DEPTH` im Store von Tasks.
pub const TASKS_MAX_DEPTH: u32 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Features {
    /// Tiefste erlaubte Ebene (Desk 1, Tasks 6). Darüber zeigt das Detail einen Hinweis statt eines Eingabefelds.
    pub max_depth: u32,
    /// Höchstzahl Erinnerungen; bei 1 bleibt das einfache Menü (Desk), sonst die Liste mit „Entfernen“.
    pub alarms_max: usize,
    /// Wiederholung mit Abstand, Wochentagen, festem Monatstag und „ab Abhaken“.
    pub full_repeat: bool,
    /// Auswahl der vier Sortierungen über der Liste.
    pub sorting: bool,
    /// Aufgaben in der Liste per Ziehen (und Alt+↑/↓) umsortieren, nur in der Handreihenfolge.
    pub drag_rows: bool,
    /// Abschnitt und der Platzhalter „Zuständig“ im Detail.
    pub detail_extras: bool,
    /// Eigene Listen per Ziehen und Alt+↑/↓ umsortieren (T3b).
    pub lists_order: bool,
    /// Aufgabe auf eine Liste in der Leiste ziehen, dazu im Detail „Verschieben nach …“ (T3b).
    pub move_to_list: bool,
    /// „+ Karte“ unter jeder Spalte des Boards (T3b).
    pub board_cards: bool,
    /// Je Liste merken, ob sie als Board offen war (T3b; für die Sitzung).
    pub board_memory: bool,
}

impl Features {
    pub const DESK: Features = Features {
        max_depth: DESK_MAX_DEPTH,
        alarms_max: 1,
        full_repeat: false,
        sorting: false,
        drag_rows: false,
        detail_extras: false,
        lists_order: false,
        move_to_list: false,
        board_cards: false,
        board_memory: false,
    };

    pub const TASKS: Features = Features {
        max_depth: TASKS_MAX_DEPTH,
        alarms_max: MAX_ALARMS,
        full_repeat: true,
        sorting: true,
        drag_rows: true,
        detail_extras: true,
        lists_order: true,
        move_to_list: true,
        board_cards: true,
        board_memory: true,
    };

    /// Mehr als eine Ebene: Baum mit Zählern über alle Nachfahren und Ebene für Screenreader.
    pub fn deep(&self) -> bool {
        self.max_depth > DESK_MAX_DEPTH
    }
}

impl Default for Features {
    fn default() -> Features {
        Features::DESK
    }
}
