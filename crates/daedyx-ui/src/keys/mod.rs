//! Tastenkürzel: eine einzige Tabelle für Programm und Hilfe-Seite. `Modifiers::COMMAND` ist
//! Strg unter Windows und Cmd auf dem Mac. Die Anzeige-Namen kommen aus den Sprachdateien.
use crate::i18n::Texts;
use eframe::egui::{Context, Key, KeyboardShortcut, Modifiers};

/// Was ein Tastenkürzel auslöst (Gerüst: allgemeine Aktionen, Apps ergänzen ihre eigenen).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Search,
    New,
    Redo,
    Undo,
    Settings,
    Help,
    Delete,
    Close,
}

/// Ein Eintrag der Tabelle.
#[derive(Debug, Clone, Copy)]
pub struct Shortcut {
    pub action: Action,
    pub shortcut: KeyboardShortcut,
    /// Schlüssel der Beschreibung in `lang/*.json`.
    pub label_key: &'static str,
}

const fn entry(
    action: Action,
    modifiers: Modifiers,
    key: Key,
    label_key: &'static str,
) -> Shortcut {
    Shortcut {
        action,
        shortcut: KeyboardShortcut::new(modifiers, key),
        label_key,
    }
}

const CMD: Modifiers = Modifiers::COMMAND;
const CMD_SHIFT: Modifiers = Modifiers::COMMAND.plus(Modifiers::SHIFT);

/// Alle Kürzel. Spezifischere Folgen stehen vor allgemeineren (Test).
pub const SHORTCUTS: &[Shortcut] = &[
    entry(Action::Search, CMD, Key::K, "kuerzel.suche"),
    entry(Action::New, CMD, Key::N, "kuerzel.neu"),
    entry(Action::Redo, CMD_SHIFT, Key::Z, "kuerzel.wiederholen"),
    entry(Action::Undo, CMD, Key::Z, "kuerzel.rueckgaengig"),
    entry(Action::Settings, CMD, Key::Comma, "kuerzel.einstellungen"),
    entry(Action::Help, Modifiers::NONE, Key::F1, "kuerzel.hilfe"),
    entry(
        Action::Delete,
        Modifiers::NONE,
        Key::Delete,
        "kuerzel.loeschen",
    ),
    entry(
        Action::Close,
        Modifiers::NONE,
        Key::Escape,
        "kuerzel.schliessen",
    ),
];

/// Tabelleneintrag einer Aktion.
pub fn find(action: Action) -> Option<&'static Shortcut> {
    SHORTCUTS.iter().find(|s| s.action == action)
}

/// Fängt das erste gedrückte Kürzel dieses Bildes ab (die Taste geht an kein Eingabefeld mehr).
pub fn pressed(ctx: &Context) -> Option<Action> {
    ctx.input_mut(|i| {
        SHORTCUTS
            .iter()
            .find(|s| i.consume_shortcut(&s.shortcut))
            .map(|s| s.action)
    })
}

fn key_name(texts: &Texts, key: Key) -> String {
    match key {
        Key::Escape => texts.t("taste.esc"),
        Key::Delete => texts.t("taste.entf"),
        other => other.symbol_or_name().to_string(),
    }
}

/// Anzeige wie „Strg+Umschalt+Z“ (Windows) oder „Cmd+Umschalt+Z“ (Mac).
pub fn label(texts: &Texts, shortcut: &Shortcut, mac: bool) -> String {
    let m = shortcut.shortcut.modifiers;
    let mut parts = Vec::new();
    if m.command || m.ctrl || m.mac_cmd {
        parts.push(texts.t(if mac { "taste.cmd" } else { "taste.strg" }));
    }
    if m.shift {
        parts.push(texts.t("taste.umschalt"));
    }
    if m.alt {
        parts.push(texts.t("taste.alt"));
    }
    parts.push(key_name(texts, shortcut.shortcut.logical_key));
    parts.join("+")
}

#[cfg(test)]
mod tests;
