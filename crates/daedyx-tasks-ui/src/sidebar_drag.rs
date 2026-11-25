//! Ziehen in der Seitenleiste (Tasks T3b): eigene Listen umsortieren (`Features::lists_order`, Tastaturweg Alt+↑/↓)
//! und eine Aufgabe aus der Liste oder vom Board auf eine Liste ziehen (`Features::move_to_list`; Tastaturweg ist das
//! Menü „Verschieben nach …“ im Detail). Wünsche werden gesammelt und nach dem Zeichnen ausgeführt.
use super::board_card::CardDrag;
use super::rows::RowDrag;
use super::sidebar_entry::row_id;
use super::{Sel, TasksUi};
use eframe::egui::{DragAndDrop, Key, Modifiers, Response, Ui};

/// Nutzlast beim Ziehen einer eigenen Liste.
#[derive(Debug, Clone)]
pub struct ListDrag(pub String);

pub enum SideWish {
    DropList { id: String, target: String },
    Step { id: String, step: isize },
    MoveTask { task: String, list: String },
}

fn carrying<T: std::any::Any + Send + Sync>(ui: &Ui) -> bool {
    DragAndDrop::has_payload_of_type::<T>(ui.ctx())
}

/// Ziehen, Ablegen und Alt+↑/↓ auf einem Eintrag der Leiste.
pub fn on_entry(ui: &mut Ui, tasks: &TasksUi, resp: &Response, sel: &Sel) -> Option<SideWish> {
    let Sel::List(list) = sel else {
        return None;
    };
    let mut wish = None;
    if tasks.features.lists_order {
        resp.dnd_set_drag_payload(ListDrag(list.clone()));
        // egui nimmt beim Ablegen jede Nutzlast weg, auch eine fremde: erst den Typ prüfen.
        if let Some(d) = carrying::<ListDrag>(ui)
            .then(|| resp.dnd_release_payload::<ListDrag>())
            .flatten()
            .filter(|d| d.0 != *list)
        {
            wish = Some(SideWish::DropList {
                id: d.0.clone(),
                target: list.clone(),
            });
        }
        if resp.has_focus() {
            for (key, step) in [(Key::ArrowUp, -1), (Key::ArrowDown, 1)] {
                if ui.input_mut(|i| i.consume_key(Modifiers::ALT, key)) {
                    wish = Some(SideWish::Step {
                        id: list.clone(),
                        step,
                    });
                }
            }
        }
    }
    if tasks.features.move_to_list {
        let task = if carrying::<RowDrag>(ui) {
            resp.dnd_release_payload::<RowDrag>().map(|d| d.0.clone())
        } else if carrying::<CardDrag>(ui) {
            resp.dnd_release_payload::<CardDrag>().map(|d| d.0.clone())
        } else {
            None
        };
        if let Some(task) = task {
            wish = Some(SideWish::MoveTask {
                task,
                list: list.clone(),
            });
        }
    }
    wish
}

pub fn apply(ui: &Ui, tasks: &mut TasksUi, wishes: Vec<SideWish>) {
    for w in wishes {
        match w {
            SideWish::DropList { id, target } => tasks.drop_list(&id, &target),
            SideWish::Step { id, step } => {
                tasks.list_step(&id, step);
                ui.memory_mut(|m| m.request_focus(row_id(&Sel::List(id))));
            }
            SideWish::MoveTask { task, list } => tasks.move_to_list(&task, &list),
        }
    }
}
