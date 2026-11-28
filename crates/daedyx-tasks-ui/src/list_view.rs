//! Mitte im Listenmodus: Kopfzeile (Titel, Umschalter Liste | Board), Schnelleingabe, Zeilen.
use super::sidebar::view_key;
use super::{Mode, Sel, TasksUi, rows};
use daedyx_ui::a11y::{mark_heading, mark_selected, named_scope};
use daedyx_ui::fonts;
use daedyx_ui::i18n::Texts;
use daedyx_ui::theme::look_ui;
use daedyx_ui::tokens::TOKENS;
use daedyx_ui::widgets::{ButtonKind, button, empty_state, text_input};
use eframe::egui::{FontId, Key, RichText, ScrollArea, Ui, accesskit::Role};

/// Titel der Mitte: Suchergebnisse, Smart-Liste oder Name der Liste.
pub fn title(tasks: &TasksUi, texts: &Texts) -> String {
    if !tasks.search.trim().is_empty() {
        return texts.tf("aufgaben.suche.titel", &[("suche", tasks.search.trim())]);
    }
    match &tasks.sel {
        Sel::View(v) => texts.t(view_key(*v)),
        Sel::List(id) => tasks
            .lists
            .iter()
            .find(|l| &l.id == id)
            .map(|l| l.name.clone())
            .unwrap_or_default(),
    }
}

/// Kopfzeile: Überschrift, bei Listen der Umschalter (falls an) und das Menü, darunter eine Fehlermeldung, falls es eine gibt.
pub fn header(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts) {
    let look = look_ui(ui);
    let name = title(tasks, texts);
    ui.horizontal_wrapped(|ui| {
        let font = FontId::new(
            TOKENS.font.steps.emphasis.size,
            fonts::semibold_in(ui.ctx()),
        );
        let h = ui.label(RichText::new(&name).font(font).color(look.p.text));
        mark_heading(ui, h.id, 1, &name);
        if tasks.list_id().is_some() && tasks.search.trim().is_empty() {
            ui.add_space(f32::from(TOKENS.spacing.s4));
            if tasks.mode_switch {
                mode_switch(ui, tasks, texts);
                ui.add_space(f32::from(TOKENS.spacing.s2));
            }
            super::list_menu::show(ui, tasks, texts);
        }
        super::list_sort::show(ui, tasks, texts);
    });
    if let Some(key) = tasks.error.clone() {
        let text = texts.t(&key);
        ui.label(RichText::new(text).color(look.p.state.error.text));
    }
    ui.add_space(f32::from(TOKENS.spacing.s3));
}

/// Umschalter Liste/Board (Rolle Tab), nur wenn `TasksUi::mode_switch` an ist.
fn mode_switch(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts) {
    named_scope(
        ui,
        "aufgaben_modus",
        Role::TabList,
        &texts.t("aufgaben.modus"),
        |ui| {
            for (mode, key) in [
                (Mode::List, "aufgaben.modus.liste"),
                (Mode::Board, "aufgaben.modus.board"),
            ] {
                let active = tasks.mode == mode;
                let kind = if active {
                    ButtonKind::Secondary
                } else {
                    ButtonKind::Quiet
                };
                let r = button(ui, kind, &texts.t(key));
                r.widget_info(|| eframe::egui::WidgetInfo::labeled(Role::Tab, true, texts.t(key)));
                mark_selected(ui, r.id, active);
                if r.clicked() {
                    tasks.set_mode(mode);
                }
            }
        },
    );
}

/// Eingabezeile „Neue Aufgabe eingeben und Enter drücken“ (Schnelleingabe mit Tag, Uhrzeit, `!`).
pub fn quick_entry(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts) {
    if tasks.lists.is_empty() || !tasks.search.trim().is_empty() {
        return;
    }
    let resp = text_input(
        ui,
        &mut tasks.quick,
        &texts.t("aufgaben.neu.feld"),
        &texts.t("aufgaben.neu.hinweis"),
        None,
    );
    if resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
        tasks.quick_add();
        resp.request_focus();
    }
    ui.add_space(f32::from(TOKENS.spacing.s3));
}

pub fn show(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts) {
    header(ui, tasks, texts);
    quick_entry(ui, tasks, texts);
    if tasks.items.is_empty() {
        let (t, x) = if tasks.search.trim().is_empty() {
            (
                texts.t("aufgaben.leer.titel"),
                texts.t("aufgaben.leer.text"),
            )
        } else {
            (
                texts.t("aufgaben.suche.leer.titel"),
                texts.t("aufgaben.suche.leer.text"),
            )
        };
        let _ = empty_state(ui, &t, &x, None);
        return;
    }
    let name = texts.t("aufgaben.liste.name");
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            named_scope(ui, "aufgaben_zeilen", Role::List, &name, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                rows::show(ui, tasks, texts);
            });
        });
}
