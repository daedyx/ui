//! Menü: Fläche, Radius 8, Zeilen 32 px, Kürzel rechts in schwachem Grau, Trennlinie vor
//! gefährlichen Einträgen wie „Löschen“. Öffnet unter seinem Knopf, Esc oder Klick daneben schließt.
use super::button::{ButtonKind, button};
use super::name_container;
use crate::theme::look_ui;
use crate::tokens::TOKENS;
use eframe::egui::{
    Align2, Area, FontFamily, FontId, Frame, Id, Key, Margin, Order, Response, Sense, Stroke, Ui,
    WidgetInfo, accesskit::Role, pos2, vec2,
};

const WIDTH: f32 = 220.0;

#[derive(Debug, Clone)]
pub struct MenuItem<'a> {
    pub label: &'a str,
    /// Fertige Anzeige des Kürzels (siehe `keys::label`).
    pub shortcut: Option<String>,
    pub danger: bool,
}

fn item(ui: &mut Ui, entry: &MenuItem<'_>) -> Response {
    let look = look_ui(ui);
    let (rect, resp) = ui.allocate_exact_size(vec2(WIDTH, TOKENS.size.control), Sense::click());
    resp.widget_info(|| WidgetInfo::labeled(Role::MenuItem, true, entry.label));
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        if resp.hovered() || resp.has_focus() {
            painter.rect_filled(rect, super::corner(TOKENS.radius.small), look.p.hover);
        }
        let font = FontId::new(TOKENS.font.steps.text.size, FontFamily::Proportional);
        let pad = f32::from(TOKENS.spacing.s3);
        let color = if entry.danger {
            look.p.state.error.text
        } else {
            look.p.text
        };
        painter.text(
            pos2(rect.left() + pad, rect.center().y),
            Align2::LEFT_CENTER,
            entry.label,
            font,
            color,
        );
        if let Some(keys) = &entry.shortcut {
            let small = FontId::new(TOKENS.font.steps.small.size, FontFamily::Proportional);
            let right = pos2(rect.right() - pad, rect.center().y);
            painter.text(right, Align2::RIGHT_CENTER, keys, small, look.p.text_faint);
        }
    }
    resp
}

/// Knopf `label`, der ein Menü öffnet. Gibt den Index des gewählten Eintrags zurück.
pub fn menu_button(ui: &mut Ui, label: &str, items: &[MenuItem<'_>]) -> Option<usize> {
    let opener = button(ui, ButtonKind::Secondary, label);
    let state_id = opener.id.with(Id::unique("daedyx-menu-open"));
    let mut open = ui.data(|d| d.get_temp::<bool>(state_id)).unwrap_or(false);
    if opener.clicked() {
        open = !open;
    }
    let mut chosen = None;
    if open {
        let look = look_ui(ui);
        let gap = f32::from(TOKENS.spacing.s1);
        let margin = i8::try_from(TOKENS.spacing.s1).unwrap_or(i8::MAX);
        let area = Area::new(state_id.with(1_u8))
            .order(Order::Foreground)
            .fixed_pos(pos2(opener.rect.left(), opener.rect.bottom() + gap))
            .show(ui.ctx(), |ui| {
                Frame::new()
                    .fill(look.p.surface)
                    .stroke(Stroke::new(TOKENS.line.fine, look.p.line))
                    .corner_radius(super::medium_corner())
                    .inner_margin(Margin::same(margin))
                    .shadow(ui.visuals().popup_shadow)
                    .show(ui, |ui| {
                        name_container(ui, Role::Menu, label);
                        ui.spacing_mut().item_spacing.y = 0.0;
                        for (index, entry) in items.iter().enumerate() {
                            if entry.danger && index > 0 {
                                ui.separator();
                            }
                            if item(ui, entry).clicked() {
                                chosen = Some(index);
                            }
                        }
                    });
            });
        let escape = ui.input(|i| i.key_pressed(Key::Escape));
        let outside = ui.input(|i| i.pointer.any_click())
            && !opener.clicked()
            && !ui
                .input(|i| i.pointer.interact_pos())
                .is_some_and(|p| area.response.rect.contains(p));
        if chosen.is_some() || escape || outside {
            open = false;
        }
    }
    ui.data_mut(|d| d.insert_temp(state_id, open));
    chosen
}
