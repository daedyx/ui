//! Menü „Listen-Aktionen“ in der Kopfzeile einer eigenen Liste: Umbenennen (Feld in der Leiste), eine der sechs
//! Tokenfarben, Löschen mit Rückfrage. Dieselben Wege gibt es auf der Listenzeile (F2, Entf).
use super::colors::NAMES;
use super::{Ask, TasksUi};
use daedyx_ui::i18n::Texts;
use daedyx_ui::widgets::{MenuItem, menu_button};
use eframe::egui::Ui;

pub fn show(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts) {
    let Some(id) = tasks.list_id().map(str::to_owned) else {
        return;
    };
    let name = tasks
        .lists
        .iter()
        .find(|l| l.id == id)
        .map(|l| l.name.clone())
        .unwrap_or_default();
    let mut labels = vec![texts.t("aufgaben.liste.umbenennen")];
    labels.extend(
        NAMES
            .iter()
            .map(|k| texts.tf("aufgaben.liste.farbe", &[("farbe", &texts.t(k))])),
    );
    labels.push(texts.t("aufgaben.liste.loeschen"));
    let last = labels.len().saturating_sub(1);
    let items: Vec<MenuItem<'_>> = labels
        .iter()
        .enumerate()
        .map(|(i, l)| MenuItem {
            label: l,
            shortcut: None,
            danger: i == last,
        })
        .collect();
    match menu_button(ui, &texts.t("aufgaben.liste.menue"), &items) {
        Some(0) => tasks.renaming_list = Some((id, name)),
        Some(i) if i == last => tasks.ask = Some(Ask::List(id)),
        Some(i) => tasks.set_list_color(&id, i.saturating_sub(1)),
        None => {}
    }
}
