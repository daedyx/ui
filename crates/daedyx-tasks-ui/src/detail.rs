//! Detailbereich rechts (300 px): Titel, Erledigt, Fälligkeit, Priorität, Wiederholung, Erinnerung,
//! Notiz, Unteraufgaben, Löschen. Änderungen gelten sofort (Textfelder beim Verlassen oder Enter).
use super::detail_fields::{draft_due, due, priority, repeat_and_reminder};
use super::row_paint::check;
use super::{Ask, Draft, TasksUi};
use daedyx_tasks_core::types::Task;
use daedyx_ui::a11y::{mark_heading, named_scope};
use daedyx_ui::fonts;
use daedyx_ui::i18n::Texts;
use daedyx_ui::theme::look_ui;
use daedyx_ui::tokens::TOKENS;
use daedyx_ui::widgets::{ButtonKind, MAX_INPUT_CHARS, button, empty_state, text_input};
use eframe::egui::{
    FontId, Key, RichText, ScrollArea, Sense, TextEdit, Ui, accesskit::Role, pos2, vec2,
};
use serde_json::json;

/// Entwurf neu füllen, wenn eine andere Aufgabe gewählt wurde oder der Kern sie geändert hat
/// (z. B. neue Fälligkeit nach dem Abhaken einer Wiederholung). Tippen allein lädt nicht neu.
fn sync(tasks: &mut TasksUi, texts: &Texts, t: &Task) {
    let seen = (
        t.title.clone(),
        t.notes.clone(),
        t.due.clone(),
        t.due_has_time,
    );
    if tasks.draft.id != t.id || tasks.draft.seen != seen {
        let (date, time) = draft_due(tasks, texts, t);
        let subtask = if tasks.draft.id == t.id {
            std::mem::take(&mut tasks.draft.subtask)
        } else {
            String::new()
        };
        tasks.draft = Draft {
            id: t.id.clone(),
            title: t.title.clone(),
            notes: t.notes.clone(),
            date,
            time,
            subtask,
            seen,
        };
    }
}

fn subtasks(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, t: &Task) {
    let look = look_ui(ui);
    let kids: Vec<Task> = tasks
        .items
        .iter()
        .filter(|k| k.parent_id.as_deref() == Some(t.id.as_str()))
        .cloned()
        .collect();
    let name = texts.t("aufgaben.unteraufgaben");
    ui.label(RichText::new(&name).small().color(look.p.text_muted));
    named_scope(ui, "aufgaben_unter", Role::List, &name, |ui| {
        for k in &kids {
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 28.0), Sense::hover());
            let center = pos2(rect.left() + 12.0, rect.center().y);
            if check(ui, &look, k, center, texts) {
                tasks.toggle(&k.id);
            }
            let color = if k.done_at.is_some() {
                look.p.text_faint
            } else {
                look.p.text
            };
            let pos = pos2(rect.left() + 28.0, rect.center().y);
            ui.painter().text(
                pos,
                eframe::egui::Align2::LEFT_CENTER,
                &k.title,
                FontId::proportional(TOKENS.font.steps.text.size),
                color,
            );
        }
    });
    let r = text_input(
        ui,
        &mut tasks.draft.subtask,
        &texts.t("aufgaben.unter.neu"),
        &texts.t("aufgaben.unter.hinweis"),
        None,
    );
    if r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
        let title = std::mem::take(&mut tasks.draft.subtask);
        if !title.trim().is_empty() {
            tasks.add_subtask(&t.id, &title);
        }
        r.request_focus();
    }
}

fn body(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, t: &Task) {
    let look = look_ui(ui);
    let space = f32::from(TOKENS.spacing.s3);
    let heading = texts.t("aufgaben.detail.titel");
    let font = FontId::new(
        TOKENS.font.steps.text_bold.size,
        fonts::semibold_in(ui.ctx()),
    );
    let h = ui.label(RichText::new(&heading).font(font).color(look.p.text));
    mark_heading(ui, h.id, 2, &heading);
    ui.add_space(space);
    let title = text_input(
        ui,
        &mut tasks.draft.title,
        &texts.t("aufgaben.feld.titel"),
        "",
        None,
    );
    if title.lost_focus() && tasks.draft.title.trim() != t.title {
        let value = tasks.draft.title.trim().to_owned();
        tasks.set(&t.id, &[("title", json!(value))]);
        if tasks.error.is_some() {
            tasks.draft.title = t.title.clone();
        }
    }
    ui.add_space(space);
    let key = if t.done_at.is_some() {
        "aufgaben.detail.oeffnen"
    } else {
        "aufgaben.detail.erledigt"
    };
    if button(ui, ButtonKind::Secondary, &texts.t(key)).clicked() {
        tasks.toggle(&t.id);
    }
    ui.add_space(space);
    due(ui, tasks, texts, t);
    ui.add_space(space);
    priority(ui, tasks, texts, t);
    ui.horizontal_wrapped(|ui| repeat_and_reminder(ui, tasks, texts, t));
    ui.add_space(space);
    let notes_label = texts.t("aufgaben.feld.notiz");
    ui.label(RichText::new(&notes_label).small().color(look.p.text_muted));
    let id = ui.make_persistent_id("aufgaben_notiz");
    let notes = ui.add(
        TextEdit::multiline(&mut tasks.draft.notes)
            .id(id)
            .char_limit(MAX_INPUT_CHARS)
            .desired_rows(4)
            .desired_width(f32::INFINITY),
    );
    ui.ctx()
        .accesskit_node_builder(id, |n| n.set_label(notes_label.clone()));
    if notes.lost_focus() && tasks.draft.notes != t.notes {
        let value = tasks.draft.notes.clone();
        tasks.set(&t.id, &[("notes", json!(value))]);
    }
    ui.add_space(space);
    if t.parent_id.is_none() {
        subtasks(ui, tasks, texts, t);
        ui.add_space(space);
    }
    if button(ui, ButtonKind::Danger, &texts.t("aufgaben.detail.loeschen")).clicked() {
        tasks.ask = Some(Ask::Task(t.id.clone()));
    }
}

pub fn show(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts) {
    let Some(t) = tasks.selected_task().cloned() else {
        let _ = empty_state(
            ui,
            &texts.t("aufgaben.detail.leer.titel"),
            &texts.t("aufgaben.detail.leer.text"),
            None,
        );
        return;
    };
    sync(tasks, texts, &t);
    let name = texts.tf("aufgaben.detail.bereich", &[("titel", &t.title)]);
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            named_scope(ui, "aufgaben_detail", Role::Group, &name, |ui| {
                body(ui, tasks, texts, &t)
            });
        });
}
