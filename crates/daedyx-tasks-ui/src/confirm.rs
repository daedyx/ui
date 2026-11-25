//! Rückfrage vor dem Löschen einer Aufgabe oder eines Abschnitts (Esc oder „Abbrechen“ lässt alles stehen).
use super::{Ask, TasksUi};
use daedyx_ui::i18n::Texts;
use daedyx_ui::widgets::{ConfirmDialog, DialogAnswer, confirm_dialog};
use eframe::egui::Context;

pub fn show(ctx: &Context, tasks: &mut TasksUi, texts: &Texts) {
    let Some(ask) = tasks.ask.clone() else {
        return;
    };
    let (title, text) = match &ask {
        Ask::Task(id) => {
            let name = tasks
                .items
                .iter()
                .find(|t| &t.id == id)
                .map(|t| t.title.clone())
                .unwrap_or_default();
            let kids = tasks
                .items
                .iter()
                .filter(|t| t.parent_id.as_deref() == Some(id.as_str()))
                .count();
            let text = if kids > 0 {
                texts.tf(
                    "aufgaben.loeschen.text_unter",
                    &[("titel", &name), ("anzahl", &kids.to_string())],
                )
            } else {
                texts.tf("aufgaben.loeschen.text", &[("titel", &name)])
            };
            (texts.t("aufgaben.loeschen.titel"), text)
        }
        Ask::Section(id) => {
            let name = tasks
                .sections
                .iter()
                .find(|s| &s.id == id)
                .map(|s| s.name.clone())
                .unwrap_or_default();
            (
                texts.t("aufgaben.abschnitt.loeschen.titel"),
                texts.tf("aufgaben.abschnitt.loeschen.text", &[("name", &name)]),
            )
        }
        Ask::List(id) => {
            let name = tasks
                .lists
                .iter()
                .find(|l| &l.id == id)
                .map(|l| l.name.clone())
                .unwrap_or_default();
            (
                texts.t("aufgaben.liste.loeschen.titel"),
                texts.tf("aufgaben.liste.loeschen.text", &[("name", &name)]),
            )
        }
    };
    let (confirm, cancel) = (
        texts.t("aufgaben.loeschen.ja"),
        texts.t("aufgaben.loeschen.nein"),
    );
    let dialog = ConfirmDialog {
        id_salt: "aufgaben_loeschen",
        title: &title,
        text: &text,
        confirm: &confirm,
        cancel: &cancel,
        danger: true,
    };
    match confirm_dialog(ctx, &dialog) {
        Some(DialogAnswer::Confirm) => tasks.confirm_delete(),
        Some(DialogAnswer::Cancel) => tasks.ask = None,
        None => {}
    }
}
