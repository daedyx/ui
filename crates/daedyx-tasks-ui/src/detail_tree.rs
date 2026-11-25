//! Unteraufgaben im Detail: abhaken, neue anlegen (Enter). Mit Baum (Tasks T3a) dazu „↑ Oberaufgabe“, „Öffnen“ je
//! Unteraufgabe und auf der tiefsten Ebene ein Hinweis statt des Eingabefelds; Desk bleibt bei einer Ebene.
use super::TasksUi;
use super::row_paint::{check, detail_check_id};
use daedyx_tasks_core::types::Task;
use daedyx_ui::a11y::named_scope;
use daedyx_ui::i18n::Texts;
use daedyx_ui::theme::look_ui;
use daedyx_ui::tokens::TOKENS;
use daedyx_ui::widgets::{ButtonKind, button, text_input};
use eframe::egui::{FontId, Key, RichText, Sense, Ui, WidgetInfo, accesskit::Role, pos2, vec2};

/// „↑ Oberaufgabe“ über dem Detail einer Unteraufgabe (nur mit Baum).
pub fn parent_link(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, t: &Task) {
    let Some(p) = t.parent_id.as_deref().filter(|_| tasks.features.deep()) else {
        return;
    };
    let Some(parent) = tasks.items.iter().find(|x| x.id == p).cloned() else {
        return;
    };
    let label = texts.t("aufgaben.detail.oberaufgabe.kurz");
    let r = button(ui, ButtonKind::Quiet, &label);
    let name = texts.tf("aufgaben.detail.oberaufgabe", &[("titel", &parent.title)]);
    r.widget_info(|| WidgetInfo::labeled(Role::Button, true, &name));
    if r.clicked() {
        tasks.selected = Some(parent.id);
    }
    ui.add_space(f32::from(TOKENS.spacing.s2));
}

fn kid_row(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, k: &Task) {
    let look = look_ui(ui);
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 28.0), Sense::hover());
    let center = pos2(rect.left() + 12.0, rect.center().y);
    if check(ui, &look, k, (center, detail_check_id(&k.id)), texts) {
        tasks.toggle(&k.id);
    }
    let color = if k.done_at.is_some() {
        look.p.text_faint
    } else {
        look.p.text
    };
    ui.painter().text(
        pos2(rect.left() + 28.0, rect.center().y),
        eframe::egui::Align2::LEFT_CENTER,
        &k.title,
        FontId::proportional(TOKENS.font.steps.text.size),
        color,
    );
    if tasks.features.deep() {
        let at = eframe::egui::Rect::from_min_size(
            pos2(rect.right() - 72.0, rect.top()),
            vec2(72.0, 28.0),
        );
        let mut child = ui.new_child(eframe::egui::UiBuilder::new().max_rect(at));
        let r = button(
            &mut child,
            ButtonKind::Quiet,
            &texts.t("aufgaben.unter.oeffnen.kurz"),
        );
        let name = texts.tf("aufgaben.unter.oeffnen", &[("titel", &k.title)]);
        r.widget_info(|| WidgetInfo::labeled(Role::Button, true, &name));
        if r.clicked() {
            tasks.selected = Some(k.id.clone());
        }
    }
}

/// Liste der direkten Unteraufgaben und das Feld für eine neue; auf der tiefsten Ebene ein Hinweis.
pub fn subtasks(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, t: &Task) {
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
            kid_row(ui, tasks, texts, k);
        }
    });
    if tasks.draft.depth >= tasks.features.max_depth {
        let hint = texts.tf(
            "aufgaben.unter.ende",
            &[("ebene", &tasks.features.max_depth.to_string())],
        );
        ui.label(RichText::new(hint).small().color(look.p.text_muted));
        return;
    }
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
