//! Seitenleiste der Aufgaben: Suchfeld, Smart-Listen (Heute, Geplant, Überfällig, Alle, Erledigt) mit
//! Zählern, darunter die eigenen Listen. Pfeil hoch/runter wechselt, wenn ein Eintrag den Fokus hat.
use super::colors::list_color;
use super::sidebar_lists::{keys, new_list, rename_field};
use super::{Sel, TasksUi};
use daedyx_tasks_core::smart::View;
use daedyx_ui::a11y::{mark_selected, named_scope};
use daedyx_ui::fonts;
use daedyx_ui::i18n::Texts;
use daedyx_ui::theme::look_ui;
use daedyx_ui::tokens::TOKENS;
use daedyx_ui::widgets::{focus, text_input};
use eframe::egui::{
    Align2, Color32, FontFamily, FontId, Id, Key, Modifiers, RichText, Sense, Ui, WidgetInfo,
    accesskit::Role, pos2, vec2,
};

const ROW: f32 = 32.0;
pub const VIEWS: [View; 5] = [
    View::Today,
    View::Planned,
    View::Overdue,
    View::All,
    View::Done,
];

pub fn view_key(v: View) -> &'static str {
    match v {
        View::Today => "aufgaben.ansicht.heute",
        View::Planned => "aufgaben.ansicht.geplant",
        View::Overdue => "aufgaben.ansicht.ueberfaellig",
        View::All => "aufgaben.ansicht.alle",
        View::Done => "aufgaben.ansicht.erledigt",
        View::List => "aufgaben.ansicht.liste",
    }
}

fn row_id(sel: &Sel) -> Id {
    match sel {
        Sel::View(v) => Id::unique("aufgaben_ansicht").with(view_key(*v)),
        Sel::List(id) => Id::unique("aufgaben_liste").with(id.as_str()),
    }
}

/// Ein Eintrag: Name links, Zähler rechts; ausgewählt = weiche Akzentfläche.
/// Ein Eintrag der Leiste: Auswahl, Name, Zähler (Smart-Listen) oder Farbpunkt (eigene Listen).
struct Entry {
    sel: Sel,
    name: String,
    count: Option<usize>,
    dot: Option<Color32>,
}

fn entry(ui: &mut Ui, e: &Entry, active: bool, texts: &Texts) -> eframe::egui::Response {
    let (sel, name, count) = (&e.sel, e.name.as_str(), e.count);
    let look = look_ui(ui);
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::hover());
    let resp = ui.interact(rect, row_id(sel), Sense::click());
    let label = match count {
        Some(n) => texts.tf(
            "aufgaben.leiste.eintrag",
            &[("name", name), ("anzahl", &n.to_string())],
        ),
        None => name.to_owned(),
    };
    resp.widget_info(|| WidgetInfo::labeled(Role::ListItem, true, &label));
    mark_selected(ui, resp.id, active);
    if ui.is_rect_visible(rect) {
        let radius = u8::try_from(TOKENS.radius.medium).unwrap_or(u8::MAX);
        let painter = ui.painter();
        if active {
            painter.rect_filled(rect, radius, look.accent.soft);
        } else if resp.hovered() {
            painter.rect_filled(rect, radius, look.p.hover);
        }
        let (family, color) = if active {
            (fonts::semibold_in(ui.ctx()), look.accent.text)
        } else {
            (FontFamily::Proportional, look.p.text)
        };
        let pad = f32::from(TOKENS.spacing.s3);
        let font = FontId::new(TOKENS.font.steps.text.size, family);
        let mut x = rect.left() + pad;
        if let Some(dot) = e.dot {
            painter.circle_filled(pos2(x + 4.0, rect.center().y), 4.0, dot);
            x += 16.0;
        }
        painter.text(
            pos2(x, rect.center().y),
            Align2::LEFT_CENTER,
            name,
            font.clone(),
            color,
        );
        if let Some(n) = count.filter(|n| *n > 0) {
            painter.text(
                pos2(rect.right() - pad, rect.center().y),
                Align2::RIGHT_CENTER,
                n.to_string(),
                font,
                look.p.text_muted,
            );
        }
        if resp.has_focus() {
            focus::ring(painter, rect, &look);
        }
    }
    resp
}

fn heading(ui: &mut Ui, text: &str) {
    let look = look_ui(ui);
    ui.add_space(f32::from(TOKENS.spacing.s3));
    ui.label(RichText::new(text).small().color(look.p.text_muted));
}

pub fn show(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts) {
    let before = tasks.search.clone();
    let _ = text_input(
        ui,
        &mut tasks.search,
        &texts.t("aufgaben.suche.feld"),
        &texts.t("aufgaben.suche.hinweis"),
        None,
    );
    if tasks.search != before {
        tasks.selected = None;
        tasks.refresh();
    }
    let mut all: Vec<Entry> = VIEWS
        .iter()
        .map(|v| Entry {
            sel: Sel::View(*v),
            name: texts.t(view_key(*v)),
            count: (*v != View::Done).then(|| tasks.count(*v)).flatten(),
            dot: None,
        })
        .collect();
    let views = all.len();
    let dark = ui.visuals().dark_mode;
    all.extend(tasks.lists.iter().enumerate().map(|(i, l)| Entry {
        sel: Sel::List(l.id.clone()),
        name: l.name.clone(),
        count: None,
        dot: Some(list_color(dark, &l.color, i)),
    }));
    let mut go: Option<Sel> = None;
    named_scope(
        ui,
        "aufgaben_leiste",
        Role::List,
        &texts.t("aufgaben.leiste"),
        |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            heading(ui, &texts.t("aufgaben.leiste.ansichten"));
            for (pos, e) in all.iter().enumerate() {
                if pos == views {
                    heading(ui, &texts.t("aufgaben.leiste.listen"));
                }
                if let Sel::List(id) = &e.sel
                    && rename_field(ui, tasks, texts, id)
                {
                    continue;
                }
                let active = tasks.search.trim().is_empty() && tasks.sel == e.sel;
                let resp = entry(ui, e, active, texts);
                if resp.clicked() {
                    go = Some(e.sel.clone());
                }
                if resp.has_focus() {
                    if let Sel::List(id) = &e.sel {
                        keys(ui, tasks, id, &e.name);
                    }
                    let down = ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown));
                    let up = ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp));
                    let next = if down {
                        all.get(pos.saturating_add(1))
                    } else if up {
                        pos.checked_sub(1).and_then(|p| all.get(p))
                    } else {
                        None
                    };
                    if let Some(n) = next {
                        go = Some(n.sel.clone());
                        ui.memory_mut(|m| m.request_focus(row_id(&n.sel)));
                    }
                }
            }
        },
    );
    new_list(ui, tasks, texts);
    if let Some(sel) = go {
        tasks.select(sel);
    }
}
