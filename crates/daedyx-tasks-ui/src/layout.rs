//! Aufbau des Bereichs Aufgaben: Leiste (240 px, Fensterton), Mitte (Liste oder Board, Fläche),
//! rechts der Detailbereich (300 px), sobald eine Aufgabe gewählt ist. Darüber die Löschrückfrage.
use super::{Mode, TasksUi, board, confirm, detail, list_view, sidebar};
use daedyx_ui::i18n::Texts;
use daedyx_ui::theme::look_ui;
use daedyx_ui::tokens::TOKENS;
use eframe::egui::{CentralPanel, Frame, Margin, Panel, Stroke, Ui};

const SIDEBAR: f32 = 240.0;
const DETAIL: f32 = 300.0;

fn margin(v: u8) -> Margin {
    Margin::same(i8::try_from(v).unwrap_or(i8::MAX))
}

pub fn show(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts) {
    let look = look_ui(ui);
    let p = look.p;
    let line = Stroke::new(TOKENS.line.fine, p.line);
    Panel::left("aufgaben_leiste")
        .resizable(false)
        .exact_size(SIDEBAR)
        .frame(
            Frame::new()
                .fill(p.window)
                .inner_margin(margin(TOKENS.spacing.s2)),
        )
        .show(ui, |ui| sidebar::show(ui, tasks, texts));
    if tasks.selected.is_some() {
        Panel::right("aufgaben_detail")
            .resizable(false)
            .exact_size(DETAIL)
            .frame(
                Frame::new()
                    .fill(p.surface)
                    .stroke(line)
                    .inner_margin(margin(TOKENS.spacing.s4)),
            )
            .show(ui, |ui| detail::show(ui, tasks, texts));
    }
    CentralPanel::default_margins()
        .frame(
            Frame::new()
                .fill(p.surface)
                .stroke(line)
                .inner_margin(margin(TOKENS.spacing.s4)),
        )
        .show(ui, |ui| match tasks.mode {
            Mode::Board if tasks.list_id().is_some() && tasks.search.trim().is_empty() => {
                board::show(ui, tasks, texts)
            }
            _ => list_view::show(ui, tasks, texts),
        });
    confirm::show(ui.ctx(), tasks, texts);
}
