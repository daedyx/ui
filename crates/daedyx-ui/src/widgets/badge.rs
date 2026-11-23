//! Marke (Badge): 20 px hoch, rund, halbfette 12 px. Für Zähler und Zustände.
use crate::fonts;
use crate::theme::look_ui;
use crate::tokens::TOKENS;
use eframe::egui::{
    Align2, Color32, FontId, Response, Sense, Ui, WidgetInfo, accesskit::Role, vec2,
};

const HEIGHT: f32 = 20.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeTone {
    Neutral,
    Accent,
    Ok,
    Warning,
    Error,
    Info,
}

pub fn badge(ui: &mut Ui, tone: BadgeTone, text: &str) -> Response {
    let look = look_ui(ui);
    let s = &look.p.state;
    let (fg, bg): (Color32, Color32) = match tone {
        BadgeTone::Neutral => (look.p.text_muted, look.p.surface_sunken),
        BadgeTone::Accent => (look.accent.text, look.accent.soft),
        BadgeTone::Ok => (s.ok.text, s.ok.soft),
        BadgeTone::Warning => (s.warning.text, s.warning.soft),
        BadgeTone::Error => (s.error.text, s.error.soft),
        BadgeTone::Info => (s.info.text, s.info.soft),
    };
    let font = FontId::new(TOKENS.font.steps.small.size, fonts::semibold());
    let galley = ui.painter().layout_no_wrap(text.to_owned(), font, fg);
    let pad = f32::from(TOKENS.spacing.s2);
    let size = vec2((galley.size().x + 2.0 * pad).max(HEIGHT), HEIGHT);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(Role::Label, true, text));
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, super::corner(TOKENS.radius.round), bg);
        let pos = Align2::CENTER_CENTER
            .anchor_size(rect.center(), galley.size())
            .min;
        painter.galley(pos, galley, fg);
    }
    response
}
