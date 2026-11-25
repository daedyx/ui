//! Karte im Board: weiß mit Linie, Radius 8, Titel (höchstens zwei Zeilen), darunter Fälligkeit,
//! Priorität, Unteraufgaben. Klick wählt, Ziehen verschiebt (schwebende Kopie mit Schatten).
//! Tastatur: Alt+Pfeil links/rechts in die Nachbarspalte, Alt+Pfeil hoch/runter in der Spalte,
//! Leertaste hakt ab, Entf fragt vor dem Löschen.
use super::row_paint::spoken;
use super::{Ask, TasksUi};
use daedyx_tasks_core::types::Task;
use daedyx_ui::a11y::mark_selected;
use daedyx_ui::i18n::Texts;
use daedyx_ui::theme::{Look, look_ui};
use daedyx_ui::tokens::TOKENS;
use daedyx_ui::widgets::focus;
use eframe::egui::{
    Align2, FontId, Galley, Id, Key, LayerId, Modifiers, Order, Painter, Rect, Sense, Stroke,
    StrokeKind, Ui, WidgetInfo, accesskit::Role, pos2, vec2,
};
use std::sync::Arc;

/// Was beim Ziehen einer Karte mitgetragen wird.
#[derive(Debug, Clone)]
pub struct CardDrag(pub String);

/// Tastenwunsch auf einer Karte (die Spalte führt ihn aus, weil sie die Nachbarn kennt).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardKey {
    Column(isize),
    Step(isize),
}

pub fn card_id(task: &str) -> Id {
    Id::unique("karte").with(task)
}

fn title(ui: &Ui, look: &Look, t: &Task, width: f32) -> Arc<Galley> {
    let mut job = eframe::egui::text::LayoutJob::simple(
        t.title.clone(),
        FontId::proportional(TOKENS.font.steps.text.size),
        if t.done_at.is_some() {
            look.p.text_faint
        } else {
            look.p.text
        },
        width,
    );
    job.wrap.max_rows = 2;
    ui.painter().layout_job(job)
}

fn paint(
    painter: &Painter,
    look: &Look,
    rect: Rect,
    galley: Arc<Galley>,
    meta: &str,
    selected: bool,
) {
    let radius = u8::try_from(TOKENS.radius.medium).unwrap_or(u8::MAX);
    let pad = f32::from(TOKENS.spacing.s3);
    painter.rect_filled(rect, radius, look.p.surface);
    let edge = if selected {
        Stroke::new(1.5, look.accent.full)
    } else {
        Stroke::new(TOKENS.line.fine, look.p.line)
    };
    painter.rect_stroke(rect, radius, edge, StrokeKind::Inside);
    let h = galley.size().y;
    painter.galley(
        pos2(rect.left() + pad, rect.top() + pad),
        galley,
        look.p.text,
    );
    let small = FontId::proportional(TOKENS.font.steps.small.size);
    painter.text(
        pos2(rect.left() + pad, rect.top() + pad + h + 6.0),
        Align2::LEFT_TOP,
        meta,
        small,
        look.p.text_muted,
    );
}

/// Kurzzeile unter dem Titel: Fälligkeit, Priorität, Unteraufgaben erledigt/gesamt.
fn meta(tasks: &TasksUi, texts: &Texts, t: &Task) -> String {
    let mut parts = Vec::new();
    if let Some((due, _)) = tasks.due_label(texts, t) {
        parts.push(due);
    }
    if t.priority > 0 {
        parts.push(texts.t(super::row_paint::priority_key(t.priority)));
    }
    let kids: Vec<&Task> = tasks
        .items
        .iter()
        .filter(|k| k.parent_id.as_deref() == Some(t.id.as_str()))
        .collect();
    if !kids.is_empty() {
        let done = kids
            .iter()
            .filter(|k| k.done_at.is_some())
            .count()
            .to_string();
        parts.push(texts.tf(
            "aufgaben.karte.unter",
            &[("erledigt", &done), ("gesamt", &kids.len().to_string())],
        ));
    }
    parts.join(" · ")
}

/// Zeichnet eine Karte. Gibt Rechteck und einen Tastenwunsch zurück.
pub fn show(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, t: &Task) -> (Rect, Option<CardKey>) {
    let look = look_ui(ui);
    let pad = f32::from(TOKENS.spacing.s3);
    let width = ui.available_width();
    let galley = title(ui, &look, t, width - 2.0 * pad);
    let line = meta(tasks, texts, t);
    let extra = if line.is_empty() { 0.0 } else { 22.0 };
    let height = galley.size().y + 2.0 * pad + extra;
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let resp = ui.interact(rect, card_id(&t.id), Sense::click_and_drag());
    resp.widget_info(|| WidgetInfo::labeled(Role::ListItem, true, spoken(tasks, texts, t)));
    let selected = tasks.selected.as_deref() == Some(t.id.as_str());
    mark_selected(ui, resp.id, selected);
    resp.dnd_set_drag_payload(CardDrag(t.id.clone()));
    if resp.clicked() {
        tasks.selected = Some(t.id.clone());
    }
    let mut wish = None;
    if resp.has_focus() {
        let moves = [
            (Key::ArrowLeft, CardKey::Column(-1)),
            (Key::ArrowRight, CardKey::Column(1)),
            (Key::ArrowUp, CardKey::Step(-1)),
            (Key::ArrowDown, CardKey::Step(1)),
        ];
        wish = moves
            .into_iter()
            .find(|(k, _)| ui.input_mut(|i| i.consume_key(Modifiers::ALT, *k)))
            .map(|(_, w)| w);
        let [enter, space, del] = [Key::Enter, Key::Space, Key::Delete]
            .map(|k| ui.input_mut(|i| i.consume_key(Modifiers::NONE, k)));
        if enter {
            tasks.selected = Some(t.id.clone());
        }
        if space {
            tasks.toggle(&t.id);
        }
        if del {
            tasks.ask = Some(Ask::Task(t.id.clone()));
        }
    }
    if resp.dragged() {
        // Platz bleibt als Umriss stehen, die Karte schwebt mit Schatten unter dem Zeiger.
        let radius = u8::try_from(TOKENS.radius.medium).unwrap_or(u8::MAX);
        ui.painter().rect_stroke(
            rect,
            radius,
            Stroke::new(1.0, look.p.line_strong),
            StrokeKind::Inside,
        );
        if let Some(p) = ui.ctx().pointer_interact_pos() {
            let floating = Rect::from_center_size(p, rect.size());
            let painter = ui
                .ctx()
                .layer_painter(LayerId::new(Order::Tooltip, card_id(&t.id)));
            painter.add(ui.visuals().popup_shadow.as_shape(floating, radius));
            paint(&painter, &look, floating, galley, &line, true);
        }
    } else if ui.is_rect_visible(rect) {
        paint(ui.painter(), &look, rect, galley, &line, selected);
        if resp.has_focus() {
            focus::ring(ui.painter(), rect, &look);
        }
    }
    (rect, wish)
}
