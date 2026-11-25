//! Auswahl der Sortierung über der Liste (Tasks T3a, `Features::sorting`): Von Hand, Fälligkeit, Priorität, Titel.
//! Ein Menü mit dem aktuellen Wert im Namen; nur „Von Hand“ erlaubt Ziehen und Alt+↑/↓.
use super::TasksUi;
use super::tree_model::SortMode;
use daedyx_ui::i18n::Texts;
use daedyx_ui::widgets::{MenuItem, menu_button};
use eframe::egui::Ui;

pub fn show(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts) {
    if !tasks.features.sorting || !tasks.search.trim().is_empty() {
        return;
    }
    ui.add_space(f32::from(daedyx_ui::tokens::TOKENS.spacing.s2));
    let labels: Vec<String> = SortMode::ALL.iter().map(|m| texts.t(m.key())).collect();
    let items: Vec<MenuItem<'_>> = labels
        .iter()
        .map(|l| MenuItem {
            label: l,
            shortcut: None,
            danger: false,
        })
        .collect();
    let title = texts.tf(
        "aufgaben.sortierung",
        &[("wert", &texts.t(tasks.sort.key()))],
    );
    if let Some(m) = menu_button(ui, &title, &items).and_then(|i| SortMode::ALL.get(i)) {
        tasks.sort = *m;
    }
}
