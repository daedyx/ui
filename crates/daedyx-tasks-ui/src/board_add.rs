//! „+ Karte“ unter einer Spalte des Boards (Tasks T3b, `Features::board_cards`; Vorlage `BoardAddField.tsx`): Knopf
//! öffnet ein Feld, Enter legt die Karte in dieser Spalte an (Schnelleingabe mit Tag, Uhrzeit, `!`) und lässt das
//! Feld für die nächste offen, Esc oder Verlassen ohne Text schließt es.
use super::TasksUi;
use daedyx_ui::i18n::Texts;
use daedyx_ui::widgets::{ButtonKind, button, text_input};
use eframe::egui::{Id, Key, Ui, WidgetInfo, accesskit::Role};

fn focus_flag() -> Id {
    Id::unique("aufgaben_karte_neu_fokus")
}

fn field(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, name: &str) {
    let label = texts.tf("aufgaben.karte.feld", &[("name", name)]);
    let hint = texts.t("aufgaben.neu.hinweis");
    let Some((_, draft)) = tasks.adding_card.as_mut() else {
        return;
    };
    let r = text_input(ui, draft, &label, &hint, None);
    if ui
        .data_mut(|d| d.remove_temp::<bool>(focus_flag()))
        .unwrap_or(false)
    {
        r.request_focus();
    }
    if r.lost_focus() {
        let (enter, escape) = ui.input(|i| (i.key_pressed(Key::Enter), i.key_pressed(Key::Escape)));
        let empty = tasks
            .adding_card
            .as_ref()
            .is_none_or(|(_, d)| d.trim().is_empty());
        if enter && !empty {
            tasks.add_card();
            r.request_focus();
        } else if escape || empty {
            tasks.adding_card = None;
        }
    }
}

pub fn show(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, sid: Option<&str>, name: &str) {
    if !tasks.features.board_cards {
        return;
    }
    let open = tasks
        .adding_card
        .as_ref()
        .is_some_and(|(s, _)| s.as_deref() == sid);
    if open {
        field(ui, tasks, texts, name);
        return;
    }
    let r = button(ui, ButtonKind::Quiet, &texts.t("aufgaben.karte.neu.kurz"));
    let spoken = texts.tf("aufgaben.karte.neu", &[("name", name)]);
    r.widget_info(|| WidgetInfo::labeled(Role::Button, true, &spoken));
    if r.clicked() {
        tasks.adding_card = Some((sid.map(str::to_owned), String::new()));
        ui.data_mut(|d| d.insert_temp(focus_flag(), true));
    }
}
