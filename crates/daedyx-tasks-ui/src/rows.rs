//! Zeilen der Aufgabenliste (44 px): runder Haken-Kreis, Titel (erledigt grau und durchgestrichen),
//! rechts Fälligkeit, Priorität, Erinnerung und Wiederholung. Unteraufgaben je Ebene eingerückt.
//! Tastatur auf einer Zeile: Pfeil hoch/runter, Leertaste hakt ab, Entf fragt vor dem Löschen; mit
//! `Features::drag_rows` in der Handreihenfolge zusätzlich Ziehen und Alt+↑/↓ (Tasks T3a).
use super::model::{Row, rows};
use super::row_paint::{CIRCLE, check, check_id, paint, spoken};
use super::tree_model::{SortMode, tree_rows};
use super::{Ask, TasksUi};
use daedyx_ui::a11y::mark_selected;
use daedyx_ui::i18n::Texts;
use daedyx_ui::theme::look_ui;
use daedyx_ui::tokens::TOKENS;
use eframe::egui::{
    Id, Key, Modifiers, Rect, Response, Sense, Ui, WidgetInfo, accesskit::Role, pos2, vec2,
};

const ROW: f32 = 44.0;
const INDENT: f32 = 28.0;

/// Nutzlast beim Ziehen einer Zeile (Aufgaben-Kennung).
#[derive(Debug, Clone)]
pub struct RowDrag(pub String);

pub fn row_id(task: &str) -> Id {
    Id::unique("aufgabe").with(task)
}

/// Zeilenfolge: Desk wie M6a, mit Baum oder Sortierungen der Baum (`tree_model`).
pub fn order(tasks: &TasksUi) -> Vec<Row> {
    let f = tasks.features;
    if f.deep() || f.sorting {
        tree_rows(&tasks.items, tasks.sort, tasks.zone())
    } else {
        rows(&tasks.items)
    }
}

/// Ziehen und Alt+↑/↓ nur in der Handreihenfolge und ohne Suche.
fn can_drag(tasks: &TasksUi) -> bool {
    tasks.features.drag_rows && tasks.sort == SortMode::Hand && tasks.search.trim().is_empty()
}

#[derive(Default)]
struct Wish {
    select: Option<String>,
    toggle: Option<String>,
    ask: Option<Ask>,
    step: Option<(String, isize)>,
    drop: Option<(String, String)>,
}

fn keys(ui: &mut Ui, tasks: &TasksUi, order: &[Row], pos: usize, id: &str, w: &mut Wish) {
    if can_drag(tasks) {
        for (key, step) in [(Key::ArrowUp, -1), (Key::ArrowDown, 1)] {
            if ui.input_mut(|i| i.consume_key(Modifiers::ALT, key)) {
                w.step = Some((id.to_owned(), step));
            }
        }
    }
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
        w.select = Some(n.id.clone());
        ui.memory_mut(|m| m.request_focus(row_id(&n.id)));
    }
    if space {
        w.toggle = Some(id.to_owned());
    }
    if del {
        w.ask = Some(Ask::Task(id.to_owned()));
    }
}

fn row(ui: &mut Ui, tasks: &TasksUi, texts: &Texts, row: &Row, rect: Rect) -> Response {
    let id = tasks
        .items
        .get(row.index)
        .map(|t| t.id.clone())
        .unwrap_or_default();
    let sense = if can_drag(tasks) {
        Sense::click_and_drag()
    } else {
        Sense::click()
    };
    let resp = ui.interact(rect, row_id(&id), sense);
    if let Some(t) = tasks.items.get(row.index) {
        resp.widget_info(|| WidgetInfo::labeled(Role::ListItem, true, spoken(tasks, texts, t)));
    }
    if tasks.features.deep() {
        let level = row.depth.saturating_add(1);
        ui.ctx()
            .accesskit_node_builder(resp.id, |n| n.set_level(level));
    }
    if can_drag(tasks) {
        resp.dnd_set_drag_payload(RowDrag(id));
    }
    resp
}

/// Zeichnet alle Zeilen und führt Auswahl, Abhaken, Löschfrage und Umsortieren aus.
pub fn show(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts) {
    let order = order(tasks);
    let mut w = Wish::default();
    for (pos, r) in order.iter().enumerate() {
        let Some(t) = tasks.items.get(r.index).cloned() else {
            continue;
        };
        let (full, _) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::hover());
        let indent = INDENT * u16::try_from(r.depth.min(8)).map_or(0.0, f32::from);
        let rect = Rect::from_min_max(pos2(full.left() + indent, full.top()), full.max);
        let resp = row(ui, tasks, texts, r, rect);
        let selected = tasks.selected.as_deref() == Some(t.id.as_str());
        mark_selected(ui, resp.id, selected);
        if resp.clicked() {
            w.select = Some(t.id.clone());
        }
        if let Some(d) = resp
            .dnd_release_payload::<RowDrag>()
            .filter(|d| d.0 != t.id)
        {
            w.drop = Some((d.0.clone(), t.id.clone()));
        }
        if resp.has_focus() {
            keys(ui, tasks, &order, pos, &t.id, &mut w);
        }
        if ui.is_rect_visible(rect) {
            paint(ui, tasks, texts, &t, rect, &resp, selected);
            let look = look_ui(ui);
            let center = pos2(
                rect.left() + f32::from(TOKENS.spacing.s3) + CIRCLE / 2.0,
                rect.center().y,
            );
            if check(ui, &look, &t, (center, check_id(&t.id)), texts) {
                w.toggle = Some(t.id.clone());
            }
        }
    }
    apply(ui, tasks, w);
}

fn apply(ui: &mut Ui, tasks: &mut TasksUi, w: Wish) {
    if let Some(id) = w.select {
        tasks.selected = Some(id);
    }
    if let Some(id) = w.toggle {
        tasks.toggle(&id);
    }
    if w.ask.is_some() {
        tasks.ask = w.ask;
    }
    if let Some((id, step)) = w.step {
        tasks.reorder_step(&id, step);
        ui.memory_mut(|m| m.request_focus(row_id(&id)));
    }
    if let Some((id, before)) = w.drop {
        tasks.reorder_before(&id, &before);
    }
}
