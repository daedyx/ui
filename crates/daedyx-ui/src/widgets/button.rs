//! Knopf in vier Arten: wichtig (Akzent voll), zweiter (Rand), leise (ohne Rand), gefährlich.
use super::{focus, medium_corner};
use crate::fonts;
use crate::theme::{Look, look_ui};
use crate::tokens::TOKENS;
use eframe::egui::{
    Align2, Color32, FontId, Response, Sense, Stroke, StrokeKind, Ui, WidgetInfo, accesskit::Role,
    vec2,
};

/// Mindestbreite, damit kurze und lange (französische) Beschriftungen gleich ruhig wirken.
const MIN_WIDTH: f32 = 64.0;
/// „Aus“ ist 45 % blasser.
const DISABLED_ALPHA: f32 = 0.55;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Quiet,
    Danger,
}

/// (Schrift, Fläche, Rand) je Art.
fn colors(kind: ButtonKind, look: &Look) -> (Color32, Color32, Stroke) {
    let p = look.p;
    let fine = TOKENS.line.fine;
    match kind {
        ButtonKind::Primary => (look.accent.on_full, look.accent.full, Stroke::NONE),
        ButtonKind::Secondary => (p.text, p.surface, Stroke::new(fine, p.line_strong)),
        ButtonKind::Quiet => (p.text, Color32::TRANSPARENT, Stroke::NONE),
        ButtonKind::Danger => (p.state.error.on_full, p.state.error.full, Stroke::NONE),
    }
}

pub fn button(ui: &mut Ui, kind: ButtonKind, text: &str) -> Response {
    button_enabled(ui, kind, text, true)
}

pub fn button_enabled(ui: &mut Ui, kind: ButtonKind, text: &str, enabled: bool) -> Response {
    let look = look_ui(ui);
    let (fg, fill, stroke) = colors(kind, &look);
    let alpha = if enabled { 1.0 } else { DISABLED_ALPHA };
    let font = FontId::new(TOKENS.font.steps.text_bold.size, fonts::semibold());
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font, fg.gamma_multiply(alpha));
    let pad = f32::from(TOKENS.spacing.s3);
    let width = (galley.size().x + 2.0 * pad).max(MIN_WIDTH);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(width, TOKENS.size.control), sense);
    response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, text));
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let radius = medium_corner();
        let stroke = Stroke::new(stroke.width, stroke.color.gamma_multiply(alpha));
        painter.rect(
            rect,
            radius,
            fill.gamma_multiply(alpha),
            stroke,
            StrokeKind::Inside,
        );
        if enabled && response.is_pointer_button_down_on() {
            painter.rect_filled(rect, radius, look.p.pressed);
        } else if enabled && response.hovered() {
            painter.rect_filled(rect, radius, look.p.hover);
        }
        let pos = Align2::CENTER_CENTER
            .anchor_size(rect.center(), galley.size())
            .min;
        painter.galley(pos, galley, fg);
        if response.has_focus() {
            focus::ring(painter, rect, &look);
        }
    }
    response
}
