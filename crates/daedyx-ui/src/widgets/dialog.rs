//! Bestätigungsdialog: Fläche, Radius 12, Innenabstand 24, Dialog-Schatten, Abdunklung dahinter,
//! höchstens 400 px breit. Knöpfe rechts unten, der gefährliche zuletzt. Esc bricht ab.
use super::button::{ButtonKind, button};
use super::name_container;
use crate::fonts;
use crate::theme::look_ui;
use crate::tokens::TOKENS;
use eframe::egui::{
    Align, Context, FontId, Frame, Id, Layout, Margin, Modal, RichText, Stroke, accesskit::Role,
};

const MAX_WIDTH: f32 = 400.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogAnswer {
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, Copy)]
pub struct ConfirmDialog<'a> {
    pub id_salt: &'a str,
    pub title: &'a str,
    pub text: &'a str,
    pub confirm: &'a str,
    pub cancel: &'a str,
    /// Bestätigen ist eine zerstörende Aktion (roter Knopf).
    pub danger: bool,
}

/// Zeigt den Dialog in diesem Bild. Antwort, sobald ein Knopf gedrückt oder Esc gedrückt wurde.
pub fn confirm_dialog(ctx: &Context, d: &ConfirmDialog<'_>) -> Option<DialogAnswer> {
    let dark = ctx.global_style().visuals.dark_mode;
    let look = crate::theme::look(ctx, dark);
    let pad = i8::try_from(TOKENS.spacing.s6).unwrap_or(i8::MAX);
    let frame = Frame::new()
        .fill(look.p.surface)
        .stroke(Stroke::new(TOKENS.line.fine, look.p.line))
        .corner_radius(super::corner(TOKENS.radius.large))
        .inner_margin(Margin::same(pad))
        .shadow(ctx.global_style().visuals.window_shadow);
    let modal = Modal::new(Id::unique(d.id_salt))
        .backdrop_color(look.p.overlay)
        .frame(frame)
        .show(ctx, |ui| {
            name_container(ui, Role::Dialog, d.title);
            ui.set_max_width(MAX_WIDTH);
            let look = look_ui(ui);
            let title_font = FontId::new(
                TOKENS.font.steps.emphasis.size,
                fonts::semibold_in(ui.ctx()),
            );
            ui.label(RichText::new(d.title).font(title_font).color(look.p.text));
            ui.label(RichText::new(d.text).color(look.p.text_muted));
            ui.add_space(f32::from(TOKENS.spacing.s4));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let kind = if d.danger {
                    ButtonKind::Danger
                } else {
                    ButtonKind::Primary
                };
                let confirm = button(ui, kind, d.confirm);
                let cancel = button(ui, ButtonKind::Secondary, d.cancel);
                if ui.memory(|m| m.focused().is_none()) {
                    cancel.request_focus();
                }
                if confirm.clicked() {
                    Some(DialogAnswer::Confirm)
                } else if cancel.clicked() {
                    Some(DialogAnswer::Cancel)
                } else {
                    None
                }
            })
            .inner
        });
    if modal.should_close() && modal.inner.is_none() {
        return Some(DialogAnswer::Cancel);
    }
    modal.inner
}
