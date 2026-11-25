//! Grundbausteine aus dem Stilkatalog (plan-stil.md, Abschnitt 4). Jeder Baustein meldet
//! AccessKit-Rolle und Beschriftung; Farben und Maße kommen nur aus den Tokens.
mod badge;
mod button;
mod card;
mod dialog;
pub mod focus;
mod input;
mod list;
mod menu;
mod nav;
mod states;

pub use badge::{BadgeTone, badge};
pub use button::{ButtonKind, button, button_enabled};
pub use card::card;
pub use dialog::{ConfirmDialog, DialogAnswer, confirm_dialog};
pub use input::{MAX_INPUT_CHARS, text_input};
pub use list::{ListRow, list};
pub use menu::{MenuItem, menu_button};
pub use nav::{app_tile, nav_entry};
pub use states::{ErrorAction, empty_state, error_state, fatal_state};

use crate::tokens::TOKENS;
use eframe::egui::{CornerRadius, Ui, accesskit::Role};

/// Radius aus den Tokens als egui-Wert (begrenzt auf 255).
pub(crate) fn corner(value: u16) -> CornerRadius {
    CornerRadius::same(u8::try_from(value).unwrap_or(u8::MAX))
}

pub(crate) fn medium_corner() -> CornerRadius {
    corner(TOKENS.radius.medium)
}

/// Gibt dem Bereich `ui` eine Rolle und Beschriftung (Liste, Menü, Dialog, Gruppe).
pub(crate) fn name_container(ui: &Ui, role: Role, label: &str) {
    ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
        node.set_role(role);
        node.set_label(label.to_owned());
    });
}

/// Auswahl als echter Auswahlzustand (egui meldet sonst „umgeschaltet“ wie bei Schaltern).
pub(crate) fn mark_selected(ui: &Ui, id: eframe::egui::Id, selected: bool) {
    ui.ctx().accesskit_node_builder(id, |node| {
        node.clear_toggled();
        node.set_selected(selected);
    });
}

#[cfg(test)]
mod nav_tests;
#[cfg(test)]
pub(crate) mod support_tests;
#[cfg(test)]
mod surface_tests;
#[cfg(test)]
mod widget_tests;
