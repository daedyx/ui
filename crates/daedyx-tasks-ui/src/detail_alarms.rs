//! Mehrere Erinnerungen im Detail (Tasks T3a, `Features::alarms_max` > 1; Vorlage `AlarmFields.tsx`): Liste mit
//! „Entfernen“, Menü „Erinnerung hinzufügen“ mit den noch freien Vorlagen, bei der Höchstzahl ein Hinweis. Ohne
//! Fälligkeit ist alles gesperrt und ein Hinweis steht da.
use super::TasksUi;
use daedyx_tasks_core::types::Task;
use daedyx_ui::a11y::named_scope;
use daedyx_ui::i18n::Texts;
use daedyx_ui::theme::look_ui;
use daedyx_ui::widgets::{ButtonKind, MenuItem, button, menu_button};
use eframe::egui::{RichText, Ui, WidgetInfo, accesskit::Role};

/// Vorlagen beim Hinzufügen: Minuten vor der Fälligkeit und Textschlüssel.
pub const PRESETS: [(i64, &str); 7] = [
    (0, "aufgaben.erinnerung.puenktlich"),
    (5, "aufgaben.erinnerung.min5"),
    (30, "aufgaben.erinnerung.min30"),
    (60, "aufgaben.erinnerung.stunde"),
    (1440, "aufgaben.erinnerung.tag"),
    (2880, "aufgaben.erinnerung.tage2"),
    (10080, "aufgaben.erinnerung.woche"),
];

/// Lesbarer Text; Werte außerhalb der Vorlagen (z. B. aus einer anderen App) als Minuten.
pub fn label(texts: &Texts, minutes: i64) -> String {
    PRESETS.iter().find(|(m, _)| *m == minutes).map_or_else(
        || {
            texts.tf(
                "aufgaben.erinnerung.minuten",
                &[("n", &minutes.to_string())],
            )
        },
        |(_, k)| texts.t(k),
    )
}

fn list(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, t: &Task, name: &str) {
    let current = tasks.draft.alarms.clone();
    named_scope(ui, "aufgaben_erinnerungen", Role::List, name, |ui| {
        for m in &current {
            ui.horizontal_wrapped(|ui| {
                let text = label(texts, *m);
                ui.label(&text);
                let r = button(
                    ui,
                    ButtonKind::Quiet,
                    &texts.t("aufgaben.erinnerung.entfernen.kurz"),
                );
                let spoken = texts.tf("aufgaben.erinnerung.entfernen", &[("text", &text)]);
                r.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), &spoken));
                if r.clicked() {
                    let rest: Vec<i64> = current.iter().copied().filter(|x| x != m).collect();
                    tasks.set_alarms(&t.id, &rest);
                }
            });
        }
    });
}

pub fn show(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, t: &Task) {
    let look = look_ui(ui);
    let name = texts.t("aufgaben.erinnerungen");
    ui.label(RichText::new(&name).small().color(look.p.text_muted));
    let has_due = t.due.is_some();
    ui.add_enabled_ui(has_due, |ui| {
        list(ui, tasks, texts, t, &name);
        if tasks.draft.alarms.len() >= tasks.features.alarms_max {
            let max = tasks.features.alarms_max.to_string();
            let hint = texts.tf("aufgaben.erinnerung.hoechstens", &[("n", &max)]);
            ui.label(RichText::new(hint).small().color(look.p.text_muted));
            return;
        }
        let free: Vec<(i64, String)> = PRESETS
            .iter()
            .filter(|(m, _)| !tasks.draft.alarms.contains(m))
            .map(|(m, k)| (*m, texts.t(k)))
            .collect();
        let items: Vec<MenuItem<'_>> = free
            .iter()
            .map(|(_, l)| MenuItem {
                label: l,
                shortcut: None,
                danger: false,
            })
            .collect();
        if let Some((m, _)) = menu_button(ui, &texts.t("aufgaben.erinnerung.hinzufuegen"), &items)
            .and_then(|i| free.get(i))
        {
            let mut all = tasks.draft.alarms.clone();
            all.push(*m);
            tasks.set_alarms(&t.id, &all);
        }
    });
    if !has_due {
        let hint = texts.t("aufgaben.erinnerung.braucht_faelligkeit");
        ui.label(RichText::new(hint).small().color(look.p.text_muted));
    }
}
