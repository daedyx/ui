//! Board einer Liste: Spalten „Ohne Abschnitt“ und die Abschnitte nebeneinander (waagerecht rollbar),
//! rechts ein Feld für einen neuen Abschnitt. Führt aus, was Karten und Spalten wünschen.
use super::TasksUi;
use super::board_card::{CardKey, card_id};
use super::board_column::{self, ColumnWish, WIDTH};
use super::list_view::{header, quick_entry};
use super::model::{cards, columns};
use daedyx_ui::i18n::Texts;
use daedyx_ui::theme::look_ui;
use daedyx_ui::tokens::TOKENS;
use daedyx_ui::widgets::text_input;
use eframe::egui::{Align, Key, Layout, ScrollArea, Ui};

/// Karte per Tastatur: in die Nachbarspalte (ans Ende) oder in der Spalte eins hoch/runter.
fn card_key(tasks: &mut TasksUi, id: &str, key: CardKey) {
    let cols = columns(&tasks.sections);
    let Some(task) = tasks.items.iter().find(|t| t.id == id) else {
        return;
    };
    let here = task.section_id.clone();
    let Some(col) = cols.iter().position(|c| *c == here) else {
        return;
    };
    match key {
        CardKey::Column(step) => {
            let Some(target) = col
                .checked_add_signed(step)
                .and_then(|i| cols.get(i))
                .cloned()
            else {
                return;
            };
            let end = cards(&tasks.items, target.as_deref()).len();
            tasks.move_card(id, target.as_deref(), end);
        }
        CardKey::Step(step) => {
            let list = cards(&tasks.items, here.as_deref());
            let Some(pos) = list.iter().position(|t| t.id == id) else {
                return;
            };
            if let Some(index) = pos.checked_add_signed(step).filter(|i| *i < list.len()) {
                tasks.move_card(id, here.as_deref(), index);
            }
        }
    }
}

fn apply(ui: &Ui, tasks: &mut TasksUi, column: Option<&str>, wish: ColumnWish) {
    match wish {
        ColumnWish::Card { id, key } => {
            card_key(tasks, &id, key);
            ui.memory_mut(|m| m.request_focus(card_id(&id)));
        }
        ColumnWish::Drop { id, index } => tasks.move_card(&id, column, index),
        ColumnWish::Section { id, step } => tasks.move_section(&id, step),
        ColumnWish::SectionDrop { id, target } => tasks.drop_section(&id, &target),
    }
}

pub fn show(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts) {
    header(ui, tasks, texts);
    quick_entry(ui, tasks, texts);
    let look = look_ui(ui);
    let k = &look.p.kanban;
    let lanes = [k.open, k.planned, k.in_progress, k.review, k.done, k.stuck];
    let mut cols: Vec<(Option<String>, String)> = vec![(None, texts.t("aufgaben.abschnitt.ohne"))];
    cols.extend(
        tasks
            .sections
            .iter()
            .map(|s| (Some(s.id.clone()), s.name.clone())),
    );
    let mut todo: Vec<(Option<String>, ColumnWish)> = Vec::new();
    ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                ui.spacing_mut().item_spacing.x = f32::from(TOKENS.spacing.s3);
                for (i, (sid, name)) in cols.iter().enumerate() {
                    let lane = lanes
                        .get(i.checked_rem(lanes.len()).unwrap_or(0))
                        .copied()
                        .unwrap_or(k.open);
                    if let Some(w) =
                        board_column::show(ui, tasks, texts, sid.as_deref(), name, lane)
                    {
                        todo.push((sid.clone(), w));
                    }
                }
                ui.vertical(|ui| {
                    ui.set_width(WIDTH);
                    let r = text_input(
                        ui,
                        &mut tasks.new_section,
                        &texts.t("aufgaben.abschnitt.neu"),
                        &texts.t("aufgaben.abschnitt.neu.hinweis"),
                        None,
                    );
                    if r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                        tasks.add_section();
                    }
                });
            });
        });
    for (column, wish) in todo {
        apply(ui, tasks, column.as_deref(), wish);
    }
}
