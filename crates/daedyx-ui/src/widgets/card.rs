//! Karte: Fläche, feine Linie, Radius 8, Innenabstand 12, ohne Schatten. Hover macht die Linie
//! dunkler, Auswahl zeigt einen Akzentrand. (Ziehen mit Schatten kommt mit dem Board.)
use super::{focus, medium_corner};
use crate::fonts;
use crate::theme::look_ui;
use crate::tokens::TOKENS;
use eframe::egui::{
    FontId, Frame, Margin, Response, RichText, Sense, Stroke, StrokeKind, Ui, WidgetInfo,
    accesskit::Role,
};

/// Karte mit Titel und einem Satz. Screenreader hören den Titel; die Auswahl als Zustand.
pub fn card(ui: &mut Ui, title: &str, body: &str, selected: bool) -> Response {
    let look = look_ui(ui);
    let pad = i8::try_from(TOKENS.spacing.s3).unwrap_or(i8::MAX);
    let inner = Frame::new()
        .fill(look.p.surface)
        .corner_radius(medium_corner())
        .inner_margin(Margin::same(pad))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let title_font = FontId::new(
                TOKENS.font.steps.text_bold.size,
                fonts::semibold_in(ui.ctx()),
            );
            ui.label(RichText::new(title).font(title_font).color(look.p.text));
            ui.label(RichText::new(body).color(look.p.text_muted));
        });
    let rect = inner.response.rect;
    let response = ui.interact(rect, inner.response.id.with(0_u8), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Group, true, title));
    super::mark_selected(ui, response.id, selected);
    let fine = TOKENS.line.fine;
    let stroke = if selected {
        Stroke::new(fine, look.accent.text)
    } else if response.hovered() {
        Stroke::new(fine, look.p.line_strong)
    } else {
        Stroke::new(fine, look.p.line)
    };
    ui.painter()
        .rect_stroke(rect, medium_corner(), stroke, StrokeKind::Inside);
    if response.has_focus() {
        focus::ring(ui.painter(), rect, &look);
    }
    response
}
