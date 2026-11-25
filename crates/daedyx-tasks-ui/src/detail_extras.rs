//! Zusatzfelder im Detail (Tasks T3a, `Features::detail_extras`): Abschnitt der Aufgabe (nur oberste Aufgaben in
//! einer Liste mit Abschnitten) und das Feld „Zuständig“ als gesperrter Platzhalter für die spätere Zusammenarbeit.
use super::TasksUi;
use daedyx_tasks_core::types::Task;
use daedyx_ui::i18n::Texts;
use daedyx_ui::theme::look_ui;
use daedyx_ui::widgets::{ButtonKind, MenuItem, button, menu_button};
use eframe::egui::{RichText, Ui};

impl TasksUi {
    /// Aufgabe in einen anderen Abschnitt derselben Liste (`None` = „Ohne Abschnitt“), Position bleibt.
    pub fn move_to_section(&mut self, id: &str, section: Option<&str>) {
        let Some(t) = self.items.iter().find(|x| x.id == id).cloned() else {
            return;
        };
        let depth = self.features.max_depth;
        let _ = self.run(|s| s.move_deep(id, (&t.list_id, None, section), t.position, depth));
    }
}

fn section(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, t: &Task) {
    if t.parent_id.is_some() || tasks.sections.is_empty() {
        return;
    }
    let none = texts.t("aufgaben.abschnitt.ohne");
    let mut choices: Vec<(Option<String>, String)> = vec![(None, none.clone())];
    choices.extend(
        tasks
            .sections
            .iter()
            .map(|s| (Some(s.id.clone()), s.name.clone())),
    );
    let current = choices
        .iter()
        .find(|(id, _)| *id == t.section_id)
        .map_or(none, |(_, n)| n.clone());
    let items: Vec<MenuItem<'_>> = choices
        .iter()
        .map(|(_, l)| MenuItem {
            label: l,
            shortcut: None,
            danger: false,
        })
        .collect();
    let title = texts.tf("aufgaben.feld.abschnitt", &[("wert", &current)]);
    if let Some((id, _)) = menu_button(ui, &title, &items).and_then(|i| choices.get(i))
        && *id != t.section_id
    {
        tasks.move_to_section(&t.id, id.as_deref());
    }
}

fn assignee(ui: &mut Ui, texts: &Texts) {
    let look = look_ui(ui);
    ui.add_enabled_ui(false, |ui| {
        let _ = button(ui, ButtonKind::Quiet, &texts.t("aufgaben.zustaendig"));
    });
    ui.label(
        RichText::new(texts.t("aufgaben.zustaendig.bald"))
            .small()
            .color(look.p.text_muted),
    );
}

pub fn show(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, t: &Task) {
    if !tasks.features.detail_extras {
        return;
    }
    section(ui, tasks, texts, t);
    assignee(ui, texts);
}
