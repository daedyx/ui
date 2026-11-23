//! Sichtbarer Tastaturfokus: 2 px Ring in der Fokusfarbe, 2 px Abstand nach außen
//! (der egui-Standard ist zu schwach).
use crate::theme::Look;
use crate::tokens::TOKENS;
use eframe::egui::{Painter, Rect, Stroke, StrokeKind};

pub fn ring(painter: &Painter, rect: Rect, look: &Look) {
    let width = TOKENS.line.focus;
    let radius = super::corner(TOKENS.radius.medium.saturating_add(2));
    painter.rect_stroke(
        rect.expand(width),
        radius,
        Stroke::new(width, look.p.focus_ring),
        StrokeKind::Outside,
    );
}
