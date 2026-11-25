//! Seitenleiste der Aufgaben: Suchfeld, Smart-Listen (Heute, Geplant, Überfällig, Alle, Erledigt) mit
//! Zählern, darunter die eigenen Listen. Pfeil hoch/runter wechselt, wenn ein Eintrag den Fokus hat.
use super::colors::list_color;
use super::sidebar_entry::{Entry, entry, row_id};
use super::sidebar_lists::{keys, new_list, rename_field};
use super::{Sel, TasksUi};
use daedyx_tasks_core::smart::View;
use daedyx_ui::a11y::named_scope;
use daedyx_ui::i18n::Texts;
use daedyx_ui::theme::look_ui;
use daedyx_ui::tokens::TOKENS;
use daedyx_ui::widgets::text_input;
use eframe::egui::{Key, Modifiers, RichText, Ui, accesskit::Role};

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
    let mut wishes = Vec::new();
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
                let drag = tasks.features.lists_order && matches!(e.sel, Sel::List(_));
                let resp = entry(ui, e, active, drag, texts);
                if let Some(w) = super::sidebar_drag::on_entry(ui, tasks, &resp, &e.sel) {
                    wishes.push(w);
                }
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
    super::sidebar_drag::apply(ui, tasks, wishes);
    if let Some(sel) = go {
        tasks.select(sel);
    }
}
