//! Eingabefeld: Beschriftung darüber, vertiefte Fläche, Fehler mit rotem Rand und Hilfetext.
use crate::theme::look_ui;
use crate::tokens::TOKENS;
use eframe::egui::{
    Margin, Response, RichText, Stroke, StrokeKind, TextEdit, Ui, accesskit::Invalid, vec2,
};

/// Grenze für jede Eingabe von außen (SICHERHEIT 2.3); Apps können enger begrenzen.
pub const MAX_INPUT_CHARS: usize = 10_000;

/// Einzeiliges Eingabefeld. Screenreader hören `label`, bei Fehler zusätzlich den Fehlertext.
pub fn text_input(
    ui: &mut Ui,
    text: &mut String,
    label: &str,
    hint: &str,
    error: Option<&str>,
) -> Response {
    let look = look_ui(ui);
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = f32::from(TOKENS.spacing.s1);
        ui.label(RichText::new(label).small().color(look.p.text_muted));
        let pad_x = i8::try_from(TOKENS.spacing.s3).unwrap_or(i8::MAX);
        let pad_y = i8::try_from(TOKENS.spacing.s1.saturating_add(2)).unwrap_or(i8::MAX);
        let id = ui.make_persistent_id(label);
        let edit = TextEdit::singleline(text)
            .id(id)
            .hint_text(hint)
            .char_limit(MAX_INPUT_CHARS)
            .margin(Margin::symmetric(pad_x, pad_y))
            .min_size(vec2(0.0, TOKENS.size.control))
            .desired_width(f32::INFINITY);
        let response = ui.add(edit);
        // Der Platzhalter ist kein Name: Screenreader sollen die Beschriftung hören.
        ui.ctx().accesskit_node_builder(id, |node| {
            node.set_label(label.to_owned());
            if let Some(message) = error {
                node.set_invalid(Invalid::True);
                node.set_description(message.to_owned());
            }
        });
        if let Some(message) = error {
            let width = TOKENS.line.fine;
            let stroke = Stroke::new(width, look.p.state.error.text);
            ui.painter().rect_stroke(
                response.rect,
                super::medium_corner(),
                stroke,
                StrokeKind::Inside,
            );
            ui.label(
                RichText::new(message)
                    .small()
                    .color(look.p.state.error.text),
            );
        }
        response
    })
    .inner
}
