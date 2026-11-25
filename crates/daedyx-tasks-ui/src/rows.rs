//! Zeilen der Aufgabenliste (44 px): runder Haken-Kreis, Titel (erledigt grau und durchgestrichen),
//! rechts Fälligkeit, Priorität, Erinnerung und Wiederholung. Unteraufgaben eingerückt.
//! Tastatur auf einer Zeile: Pfeil hoch/runter, Leertaste hakt ab, Entf fragt vor dem Löschen.
use super::model::{Row, rows};
use super::row_paint::{CIRCLE, check, paint, spoken};
use super::{Ask, TasksUi};
use daedyx_ui::a11y::mark_selected;
use daedyx_ui::i18n::Texts;
use daedyx_ui::theme::look_ui;
use daedyx_ui::tokens::TOKENS;
use eframe::egui::{Id, Key, Modifiers, Rect, Sense, Ui, WidgetInfo, accesskit::Role, pos2, vec2};

const ROW: f32 = 44.0;
const INDENT: f32 = 28.0;

pub fn row_id(task: &str) -> Id {
    Id::unique("aufgabe").with(task)
}

/// Zeichnet alle Zeilen und führt Auswahl, Abhaken und Löschfrage aus.
pub fn show(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts) {
    let order: Vec<Row> = rows(&tasks.items);
    let (mut select, mut toggle, mut ask) = (None, None, None);
    for (pos, row) in order.iter().enumerate() {
        let Some(t) = tasks.items.get(row.index) else {
            continue;
        };
        let (full, _) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::hover());
        let indent = if row.depth > 0 { INDENT } else { 0.0 };
        let rect = Rect::from_min_max(pos2(full.left() + indent, full.top()), full.max);
        let resp = ui.interact(rect, row_id(&t.id), Sense::click());
        resp.widget_info(|| WidgetInfo::labeled(Role::ListItem, true, spoken(tasks, texts, t)));
        let selected = tasks.selected.as_deref() == Some(t.id.as_str());
        mark_selected(ui, resp.id, selected);
        if resp.clicked() {
            select = Some(t.id.clone());
        }
        if resp.has_focus() {
            let [down, up, space, del] = [Key::ArrowDown, Key::ArrowUp, Key::Space, Key::Delete]
                .map(|k| ui.input_mut(|i| i.consume_key(Modifiers::NONE, k)));
            let next = if down {
                order.get(pos.saturating_add(1))
            } else if up {
                pos.checked_sub(1).and_then(|p| order.get(p))
            } else {
                None
            };
            if let Some(n) = next.and_then(|r| tasks.items.get(r.index)) {
                select = Some(n.id.clone());
                ui.memory_mut(|m| m.request_focus(row_id(&n.id)));
            }
            if space {
                toggle = Some(t.id.clone());
            }
            if del {
                ask = Some(Ask::Task(t.id.clone()));
            }
        }
        if ui.is_rect_visible(rect) {
            paint(ui, tasks, texts, t, rect, &resp, selected);
            let look = look_ui(ui);
            let center = pos2(
                rect.left() + f32::from(TOKENS.spacing.s3) + CIRCLE / 2.0,
                rect.center().y,
            );
            if check(ui, &look, t, center, texts) {
                toggle = Some(t.id.clone());
            }
        }
    }
    if let Some(id) = select {
        tasks.selected = Some(id);
    }
    if let Some(id) = toggle {
        tasks.toggle(&id);
    }
    if ask.is_some() {
        tasks.ask = ask;
    }
}
