//! Leerer Zustand und Fehlerzustand: Kreis mit Zeichen, Überschrift, ein Satz, Knöpfe.
//! Texte erklären, was los ist und was man tun kann (kommen aus i18n).
use super::button::{ButtonKind, button};
use super::name_container;
use crate::fonts;
use crate::theme::{Look, look_ui};
use crate::tokens::TOKENS;
use eframe::egui::{
    Align, Color32, FontId, Label, Layout, RichText, Sense, Stroke, Ui, UiBuilder, accesskit::Role,
    pos2, vec2,
};

const CIRCLE: f32 = 48.0;
const TEXT_WIDTH: f32 = 360.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorAction {
    Retry,
    Secondary,
}

/// Kreis mit einem einfachen Zeichen (gezeichnet, kein Schriftzeichen, keine Emoji).
fn circle(ui: &mut Ui, look: &Look, error: bool) {
    let (rect, _) = ui.allocate_exact_size(vec2(CIRCLE, CIRCLE), Sense::hover());
    let painter = ui.painter();
    let c = rect.center();
    let r = CIRCLE / 2.0;
    let stroke_w = TOKENS.line.focus;
    if error {
        let fg: Color32 = look.p.state.error.text;
        painter.circle_filled(c, r, look.p.state.error.soft);
        painter.line_segment(
            [pos2(c.x, c.y - r / 3.0), pos2(c.x, c.y + r / 12.0)],
            Stroke::new(stroke_w, fg),
        );
        painter.circle_filled(pos2(c.x, c.y + r / 3.0), stroke_w, fg);
    } else {
        painter.circle_filled(c, r, look.p.surface_sunken);
        painter.circle_stroke(c, r / 3.0, Stroke::new(stroke_w, look.p.text_muted));
    }
}

fn heading_and_text(ui: &mut Ui, look: &Look, title: &str, text: &str) {
    let font = FontId::new(
        TOKENS.font.steps.emphasis.size,
        fonts::semibold_in(ui.ctx()),
    );
    ui.label(RichText::new(title).font(font).color(look.p.text));
    let width = ui.available_width().min(TEXT_WIDTH);
    ui.set_max_width(width);
    ui.add(Label::new(RichText::new(text).color(look.p.text_muted)).wrap());
    ui.add_space(f32::from(TOKENS.spacing.s2));
}

fn centered<R>(
    ui: &mut Ui,
    salt: &str,
    role: Role,
    title: &str,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.scope_builder(UiBuilder::new().id_salt(salt), |ui| {
        name_container(ui, role, title);
        ui.with_layout(Layout::top_down(Align::Center), add).inner
    })
    .inner
}

/// Leerer Zustand. Gibt `true` zurück, wenn der (optionale) Knopf gedrückt wurde.
pub fn empty_state(ui: &mut Ui, title: &str, text: &str, action: Option<&str>) -> bool {
    let look = look_ui(ui);
    centered(ui, "daedyx-empty", Role::Group, title, |ui| {
        circle(ui, &look, false);
        heading_and_text(ui, &look, title, text);
        action.is_some_and(|label| button(ui, ButtonKind::Primary, label).clicked())
    })
}

/// Fehlerzustand (Screenreader: Warnmeldung). Knöpfe „Erneut versuchen“ und ein zweiter.
pub fn error_state(
    ui: &mut Ui,
    title: &str,
    text: &str,
    retry: &str,
    secondary: Option<&str>,
) -> Option<ErrorAction> {
    let look = look_ui(ui);
    centered(ui, "daedyx-error", Role::Alert, title, |ui| {
        circle(ui, &look, true);
        heading_and_text(ui, &look, title, text);
        ui.horizontal(|ui| {
            let again = button(ui, ButtonKind::Primary, retry).clicked();
            let other =
                secondary.is_some_and(|label| button(ui, ButtonKind::Secondary, label).clicked());
            if again {
                Some(ErrorAction::Retry)
            } else if other {
                Some(ErrorAction::Secondary)
            } else {
                None
            }
        })
        .inner
    })
}

/// Fehler, nach dem es nicht weitergeht (Startfehler-Fenster, Tasks T2): Warnmeldung mit nur einem Knopf. Gibt
/// `true` zurück, wenn „Beenden“ (Text `quit`) gedrückt wurde.
pub fn fatal_state(ui: &mut Ui, title: &str, text: &str, quit: &str) -> bool {
    let look = look_ui(ui);
    centered(ui, "daedyx-fatal", Role::Alert, title, |ui| {
        circle(ui, &look, true);
        heading_and_text(ui, &look, title, text);
        button(ui, ButtonKind::Primary, quit).clicked()
    })
}
