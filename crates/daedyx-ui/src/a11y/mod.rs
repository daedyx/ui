//! Hilfen für Screenreader: Bereiche mit Rolle und Name, echter Auswahlzustand, Baum-Ebene, Überschriften
//! (aus Desk übernommen, Tasks T1a; gemeinsam für alle egui-Apps).
use eframe::egui::{Id, Ui, UiBuilder, accesskit::Role};

/// Zeichnet `add` in einem eigenen Bereich mit Rolle und Beschriftung (Baum, Liste, Leiste …).
pub fn named_scope<R>(
    ui: &mut Ui,
    salt: &str,
    role: Role,
    label: &str,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.scope_builder(UiBuilder::new().id_salt(salt), |ui| {
        ui.ctx().accesskit_node_builder(ui.unique_id(), |n| {
            n.set_role(role);
            n.set_label(label.to_owned());
        });
        add(ui)
    })
    .inner
}

/// Auswahl als echter Zustand „ausgewählt“ (egui meldet sonst „umgeschaltet“ wie bei Schaltern).
pub fn mark_selected(ui: &Ui, id: Id, selected: bool) {
    ui.ctx().accesskit_node_builder(id, |n| {
        n.clear_toggled();
        n.set_selected(selected);
    });
}

/// Ebene im Baum (1 = oberste) und ob ein Eintrag aufgeklappt ist.
pub fn mark_tree(ui: &Ui, id: Id, level: usize, expanded: Option<bool>) {
    ui.ctx().accesskit_node_builder(id, |n| {
        n.set_level(level);
        if let Some(open) = expanded {
            n.set_expanded(open);
        }
    });
}

/// Eine Beschriftung wird für Screenreader zur Überschrift (mit ihrem Text als Name).
pub fn mark_heading(ui: &Ui, id: Id, level: usize, text: &str) {
    ui.ctx().accesskit_node_builder(id, |n| {
        n.set_role(Role::Heading);
        n.set_label(text.to_owned());
        n.set_level(level);
    });
}

#[cfg(test)]
mod tests;
