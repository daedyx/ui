//! Navigationsleiste nach Stil 1 (aus Desks Bereichsleiste verallgemeinert, Tasks T2): App-Kachel und Einträge.
//! Aktiver Eintrag: weiche Akzentfläche, Text in Akzentfarbe und halbfett. Schmal (56 px) zeigt der Eintrag nur den
//! Anfangsbuchstaben; der volle Name bleibt für Screenreader und erscheint beim Darüberfahren.
use super::{focus, mark_selected};
use crate::fonts;
use crate::theme::look_ui;
use crate::tokens::TOKENS;
use eframe::egui::{
    Align2, FontFamily, FontId, RichText, Sense, Ui, WidgetInfo, accesskit::Role, pos2, vec2,
};

const ROW: f32 = 40.0;
const TILE: f32 = 28.0;

fn radius() -> u8 {
    u8::try_from(TOKENS.radius.medium).unwrap_or(u8::MAX)
}

/// Ein Eintrag (Rolle „Reiter“, Auswahl als echter Auswahlzustand). `short`: nur der Anfangsbuchstabe.
pub fn nav_entry(ui: &mut Ui, label: &str, active: bool, short: bool) -> eframe::egui::Response {
    let look = look_ui(ui);
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::click());
    resp.widget_info(|| WidgetInfo::labeled(Role::Tab, true, label));
    mark_selected(ui, resp.id, active);
    let painter = ui.painter();
    if active {
        painter.rect_filled(rect, radius(), look.accent.soft);
    } else if resp.hovered() {
        painter.rect_filled(rect, radius(), look.p.hover);
    }
    let (family, color) = if active {
        (fonts::semibold_in(ui.ctx()), look.accent.text)
    } else {
        (FontFamily::Proportional, look.p.text)
    };
    let font = FontId::new(TOKENS.font.steps.text.size, family);
    if short {
        let first: String = label.chars().take(1).collect();
        painter.text(rect.center(), Align2::CENTER_CENTER, first, font, color);
    } else {
        let x = rect.left() + f32::from(TOKENS.spacing.s3);
        painter.text(
            pos2(x, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            font,
            color,
        );
    }
    if resp.has_focus() {
        focus::ring(painter, rect, &look);
    }
    if short {
        resp.on_hover_text(label)
    } else {
        resp
    }
}

/// App-Kachel in der Akzentfarbe, breit mit dem App-Namen daneben.
pub fn app_tile(ui: &mut Ui, title: &str, short: bool) {
    let look = look_ui(ui);
    ui.horizontal(|ui| {
        let (tile, _) = ui.allocate_exact_size(vec2(TILE, TILE), Sense::hover());
        ui.painter().rect_filled(tile, radius(), look.accent.full);
        if !short {
            let font = FontId::new(
                TOKENS.font.steps.text_bold.size,
                fonts::semibold_in(ui.ctx()),
            );
            ui.label(RichText::new(title).font(font).color(look.p.text));
        }
    });
}
