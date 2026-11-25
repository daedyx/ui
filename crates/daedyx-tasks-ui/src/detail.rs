//! Detailbereich rechts (300 px): Titel, Erledigt, Fälligkeit, Priorität, Wiederholung, Erinnerung,
//! Notiz, Unteraufgaben, Löschen. Änderungen gelten sofort (Textfelder beim Verlassen oder Enter).
use super::detail_fields::{draft_due, due, priority, reminder_menu, repeat_menu};
use super::{Ask, Draft, TasksUi};
use super::{detail_alarms, detail_extras, detail_repeat, detail_tree};
use daedyx_tasks_core::types::Task;
use daedyx_ui::a11y::{mark_heading, named_scope};
use daedyx_ui::fonts;
use daedyx_ui::i18n::Texts;
use daedyx_ui::theme::look_ui;
use daedyx_ui::tokens::TOKENS;
use daedyx_ui::widgets::{ButtonKind, MAX_INPUT_CHARS, button, empty_state, text_input};
use eframe::egui::{FontId, RichText, ScrollArea, TextEdit, Ui, accesskit::Role};
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
            depth: tasks.depth_of(&t.id),
            parent: t.parent_id.clone(),
            alarms: tasks.alarms_of(&t.id),
            ..Draft::default()
        };
    }
    // Verschoben unter eine andere Oberaufgabe (Titel und Notiz gleich): Ebene neu lesen (Review T3a, N-4).
    if tasks.draft.parent != t.parent_id {
        tasks.draft.depth = tasks.depth_of(&t.id);
        tasks.draft.parent = t.parent_id.clone();
    }
    if tasks.draft.repeat_seen != t.repeat {
        let r = t.repeat.as_ref();
        tasks.draft.interval = r.map(|r| r.interval.to_string()).unwrap_or_default();
        tasks.draft.month_day = r
            .and_then(|r| r.month_day)
            .map(|d| d.to_string())
            .unwrap_or_default();
        tasks.draft.repeat_seen = t.repeat.clone();
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
    detail_tree::parent_link(ui, tasks, texts, t);
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
    if tasks.features.full_repeat {
        detail_repeat::show(ui, tasks, texts, t);
    } else {
        ui.horizontal_wrapped(|ui| repeat_menu(ui, tasks, texts, t));
    }
    if tasks.features.alarms_max > 1 {
        detail_alarms::show(ui, tasks, texts, t);
    } else {
        ui.horizontal_wrapped(|ui| reminder_menu(ui, tasks, texts, t));
    }
    detail_extras::show(ui, tasks, texts, t);
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
    if tasks.draft.depth < tasks.features.max_depth || tasks.features.deep() {
        detail_tree::subtasks(ui, tasks, texts, t);
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
