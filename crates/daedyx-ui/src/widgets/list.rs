//! Liste: Zeilen 56 px mit Linie unten. Ausgewählt = weiche Akzentfläche plus 3 px Strich links
//! (auch ohne Farbe erkennbar). Ungelesen = halbfetter Titel und Akzentpunkt. Pfeiltasten
//! wandern, wenn eine Zeile den Fokus hat.
use super::{focus, name_container};
use crate::fonts;
use crate::theme::{Look, look_ui};
use crate::tokens::TOKENS;
use eframe::egui::{
    Align2, FontId, Id, Key, Modifiers, Rect, Response, Sense, Stroke, Ui, UiBuilder, WidgetInfo,
    accesskit::Role, pos2, vec2,
};

const STRIP: f32 = 3.0;
const DOT: f32 = 4.0;

#[derive(Debug, Clone, Copy)]
pub struct ListRow<'a> {
    pub title: &'a str,
    pub detail: &'a str,
    pub unread: bool,
}

fn row_id(list: Id, index: usize) -> Id {
    list.with(index)
}

fn paint_row(ui: &Ui, rect: Rect, row: &ListRow<'_>, selected: bool, resp: &Response, look: &Look) {
    let painter = ui.painter();
    let p = look.p;
    if selected {
        painter.rect_filled(rect, 0, look.accent.soft);
        let strip = Rect::from_min_size(rect.min, vec2(STRIP, rect.height()));
        painter.rect_filled(strip, 0, look.accent.full);
    } else if resp.hovered() {
        painter.rect_filled(rect, 0, p.hover);
    }
    painter.hline(
        rect.x_range(),
        rect.bottom(),
        Stroke::new(TOKENS.line.fine, p.line),
    );
    let pad = f32::from(TOKENS.spacing.s4);
    let steps = &TOKENS.font.steps;
    let family = if row.unread {
        fonts::semibold_in(ui.ctx())
    } else {
        eframe::egui::FontFamily::Proportional
    };
    let top = rect.top() + f32::from(TOKENS.spacing.s2);
    let left = rect.left() + pad;
    painter.text(
        pos2(left, top),
        Align2::LEFT_TOP,
        row.title,
        FontId::new(steps.text.size, family),
        p.text,
    );
    let detail_top = top + steps.text.line_height + f32::from(TOKENS.spacing.s1);
    let small = FontId::new(steps.small.size, eframe::egui::FontFamily::Proportional);
    painter.text(
        pos2(left, detail_top),
        Align2::LEFT_TOP,
        row.detail,
        small,
        p.text_muted,
    );
    if row.unread {
        let center = pos2(rect.left() + pad / 2.0, top + steps.text.line_height / 2.0);
        painter.circle_filled(center, DOT, look.accent.full);
    }
    if resp.has_focus() {
        focus::ring(painter, rect.shrink(TOKENS.line.focus), look);
    }
}

/// Zeichnet die Liste und gibt den Index einer angeklickten oder per Tastatur gewählten Zeile
/// zurück. `name` ist die Beschriftung der ganzen Liste für Screenreader.
pub fn list(
    ui: &mut Ui,
    name: &str,
    rows: &[ListRow<'_>],
    selected: &mut Option<usize>,
) -> Option<usize> {
    let look = look_ui(ui);
    let list_id = ui.make_persistent_id(name);
    let builder = UiBuilder::new().id_salt(name);
    ui.scope_builder(builder, |ui| {
        name_container(ui, Role::ListBox, name);
        ui.spacing_mut().item_spacing.y = 0.0;
        let mut chosen = None;
        let mut move_to = None;
        for (index, row) in rows.iter().enumerate() {
            let size = vec2(ui.available_width(), TOKENS.size.list_row);
            let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
            let resp = ui.interact(rect, row_id(list_id, index), Sense::click());
            let is_selected = *selected == Some(index);
            resp.widget_info(|| {
                WidgetInfo::selected(Role::ListBoxOption, true, is_selected, row.title)
            });
            super::mark_selected(ui, resp.id, is_selected);
            if resp.clicked() {
                chosen = Some(index);
            }
            if resp.has_focus() {
                let (down, up) = ui.input_mut(|i| {
                    (
                        i.consume_key(Modifiers::NONE, Key::ArrowDown),
                        i.consume_key(Modifiers::NONE, Key::ArrowUp),
                    )
                });
                if down && index.saturating_add(1) < rows.len() {
                    move_to = Some(index.saturating_add(1));
                } else if up && index > 0 {
                    move_to = Some(index.saturating_sub(1));
                }
            }
            if ui.is_rect_visible(rect) {
                paint_row(ui, rect, row, is_selected, &resp, &look);
            }
        }
        if let Some(next) = move_to {
            ui.memory_mut(|m| m.request_focus(row_id(list_id, next)));
            chosen = Some(next);
        }
        if chosen.is_some() {
            *selected = chosen;
        }
        chosen
    })
    .inner
}
