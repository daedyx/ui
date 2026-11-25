//! Eine Spalte des Boards: getönte Bahn mit Kopf (siehe `board_header`) und Karten.
//! Karten und Spaltenköpfe lassen sich ziehen; die Einfügemarke zeigt, wo eine Karte landet.
use super::TasksUi;
use super::board_card::{self, CardDrag, CardKey};
use super::board_header::{SectionDrag, header, menu};
use super::model::cards;
use daedyx_ui::a11y::named_scope;
use daedyx_ui::i18n::Texts;
use daedyx_ui::theme::look_ui;
use daedyx_ui::tokens::{Lane, TOKENS};
use eframe::egui::{
    Align, Color32, DragAndDrop, Frame, Layout, Margin, Stroke, Ui, accesskit::Role, vec2,
};

pub const WIDTH: f32 = 264.0;
/// Platz für den Knopf „Aktionen“ rechts im Spaltenkopf.
const MENU: f32 = 92.0;

/// Ergebnis einer Spalte für das Board (Karte verschieben, Abschnitt verschieben).
pub enum ColumnWish {
    Card {
        id: String,
        key: CardKey,
    },
    Drop {
        id: String,
        index: usize,
    },
    Section {
        id: String,
        step: isize,
    },
    /// Abschnitt `id` wurde auf den Kopf von `target` gezogen.
    SectionDrop {
        id: String,
        target: String,
    },
}

/// Zeichnet die Spalte `sid` (`None` = „Ohne Abschnitt“).
pub fn show(
    ui: &mut Ui,
    tasks: &mut TasksUi,
    texts: &Texts,
    sid: Option<&str>,
    name: &str,
    lane: Lane<Color32>,
) -> Option<ColumnWish> {
    let look = look_ui(ui);
    let list: Vec<daedyx_tasks_core::types::Task> = cards(&tasks.items, &tasks.sections, sid)
        .into_iter()
        .cloned()
        .collect();
    let pad = i8::try_from(TOKENS.spacing.s3).unwrap_or(i8::MAX);
    let frame = Frame::new()
        .fill(lane.soft)
        .corner_radius(u8::try_from(TOKENS.radius.large).unwrap_or(u8::MAX))
        .inner_margin(Margin::same(pad));
    let mut wish = None;
    let mut centers: Vec<(String, f32)> = Vec::new();
    // egui nimmt beim Ablegen jede Nutzlast weg, auch eine fremde: Ein Abschnitt, der über dieser Spalte losgelassen
    // wird, landet sonst in der Kartenablage und geht verloren. Deshalb vorher merken und unten selbst auswerten.
    let section_drag = DragAndDrop::payload::<SectionDrag>(ui.ctx());
    let (inner, dropped) = ui.dnd_drop_zone::<CardDrag, _>(frame, |ui| {
        ui.set_width(WIDTH);
        let label = texts.tf(
            "aufgaben.abschnitt.kopf",
            &[("name", name), ("anzahl", &list.len().to_string())],
        );
        named_scope(
            ui,
            &format!("spalte-{}", sid.unwrap_or("")),
            Role::List,
            &label,
            |ui| {
                // Die Spalte stapelt senkrecht (das Board selbst reiht waagerecht).
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    let (head, step, menu_step) = ui
                        .horizontal(|ui| {
                            let menu_width = if sid.is_some() { MENU } else { 0.0 };
                            let (head, step) = ui
                                .allocate_ui(vec2(ui.available_width() - menu_width, 32.0), |ui| {
                                    header(ui, tasks, texts, sid, name, list.len(), lane)
                                })
                                .inner;
                            (
                                head,
                                step,
                                sid.and_then(|s| menu(ui, tasks, texts, s, name)),
                            )
                        })
                        .inner;
                    if let (Some(s), Some(st)) = (sid, step.or(menu_step)) {
                        wish = Some(ColumnWish::Section {
                            id: s.to_owned(),
                            step: st,
                        });
                    }
                    if let (Some(d), Some(s)) = (head.dnd_release_payload::<SectionDrag>(), sid)
                        && d.0 != s
                    {
                        wish = Some(ColumnWish::SectionDrop {
                            id: d.0.clone(),
                            target: s.to_owned(),
                        });
                    }
                    ui.add_space(f32::from(TOKENS.spacing.s2));
                    for t in &list {
                        let (rect, key) = board_card::show(ui, tasks, texts, t);
                        centers.push((t.id.clone(), rect.center().y));
                        if let Some(k) = key {
                            wish = Some(ColumnWish::Card {
                                id: t.id.clone(),
                                key: k,
                            });
                        }
                        ui.add_space(f32::from(TOKENS.spacing.s2));
                    }
                    super::board_add::show(ui, tasks, texts, sid, name);
                    ui.add_space(24.0);
                });
            },
        );
    });
    let released = ui.input(|i| i.pointer.any_released());
    if let (Some(d), Some(s), None) = (section_drag, sid, wish.as_ref())
        && released
        && inner.response.contains_pointer()
        && d.0 != s
    {
        wish = Some(ColumnWish::SectionDrop {
            id: d.0.clone(),
            target: s.to_owned(),
        });
    }
    let pointer = ui.ctx().pointer_interact_pos();
    if inner.response.dnd_hover_payload::<CardDrag>().is_some()
        && let Some(p) = pointer
    {
        let y = centers
            .iter()
            .map(|(_, c)| *c)
            .filter(|c| *c < p.y)
            .fold(inner.response.rect.top() + 40.0, f32::max)
            + 14.0;
        let x = inner.response.rect.x_range();
        ui.painter()
            .hline(x.shrink(8.0), y, Stroke::new(2.0, look.accent.full));
    }
    if let (Some(d), Some(p)) = (dropped, pointer) {
        let index = centers
            .iter()
            .filter(|(id, c)| *id != d.0 && *c < p.y)
            .count();
        wish = Some(ColumnWish::Drop {
            id: d.0.clone(),
            index,
        });
    }
    wish
}
