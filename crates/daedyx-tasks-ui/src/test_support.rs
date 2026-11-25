//! Testrahmen: Speicher-Datenbank mit dem Kern-Schema, erfundene Aufgaben aus `testdata/aufgaben.json`, feste Uhr
//! (Mittwoch, 18.11.2026, 10:00, Berlin) und die Oberfläche ohne Fenster (kittest). Nie ein Datenordner.
use crate::store::{Clock, TaskStore};
use crate::{TasksUi, seed, show, start_with};
use chrono::{NaiveDate, NaiveDateTime};
use chrono_tz::Europe::Berlin;
use daedyx_ui::i18n::Language;
use daedyx_ui::theme;
use daedyx_ui::tokens::AppAccent;
use eframe::egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

const DEMO: &str = include_str!("../testdata/aufgaben.json");

pub fn now() -> NaiveDateTime {
    NaiveDate::from_ymd_opt(2026, 11, 18)
        .unwrap()
        .and_hms_opt(10, 0, 0)
        .unwrap()
}

/// Speicher-Datenbank mit den erfundenen Aufgaben, Uhr fest.
pub fn fixed_store() -> TaskStore {
    let store = TaskStore::memory(Berlin, Clock::Fixed(now())).unwrap();
    seed::fill(&store, DEMO).unwrap();
    store
}

/// Die Aufgaben-Oberfläche allein (ohne App) in der Sprache `language`.
pub fn harness_in(list: Option<(usize, bool)>, language: Language) -> Harness<'static, TasksUi> {
    let texts = crate::texts::texts(language);
    let mut h = Harness::builder()
        .with_size((1100.0, 760.0))
        .with_context_setup(|ctx| theme::install(ctx, AppAccent::Tasks))
        .build_ui_state(
            move |ui, tasks: &mut TasksUi| show(ui, tasks, &texts),
            start_with(Some(fixed_store()), list),
        );
    h.run();
    h.run();
    h
}

pub fn harness(list: Option<(usize, bool)>) -> Harness<'static, TasksUi> {
    harness_in(list, Language::De)
}

/// Name eines Listeneintrags, der mit `start` beginnt.
pub fn item_label(h: &Harness<'static, TasksUi>, start: &str) -> Option<String> {
    h.query_all_by_role(Role::ListItem)
        .filter_map(|n| n.accesskit_node().label())
        .find(|l| l.starts_with(start))
}

/// Wie [`harness`], aber mit allem, was Tasks einschaltet (`Features::TASKS`).
pub fn harness_tasks(list: Option<(usize, bool)>) -> Harness<'static, TasksUi> {
    let mut h = harness(list);
    // Höher als der Desk-Rahmen: das Detail hat mit allen Feldern mehr Inhalt, kittest klickt am Bildschirmort.
    h.set_size(eframe::egui::vec2(1100.0, 1400.0));
    h.state_mut().features = crate::Features::TASKS;
    h.run();
    h
}

/// Klickt den Knopf `name` und lässt zwei Bilder laufen.
pub fn click(h: &mut Harness<'static, TasksUi>, role: Role, name: &str) {
    h.get_by_role_and_label(role, name).click();
    h.run();
    h.run();
}

/// Ersetzt den Inhalt des Felds `name` durch `text` und verlässt es mit Enter.
pub fn type_into(h: &mut Harness<'static, TasksUi>, name: &str, text: &str) {
    h.get_by_role_and_label(Role::TextInput, name).click();
    h.run();
    h.key_press_modifiers(eframe::egui::Modifiers::COMMAND, eframe::egui::Key::A);
    h.key_press(eframe::egui::Key::Backspace);
    h.run();
    h.get_by_role_and_label(Role::TextInput, name)
        .type_text(text);
    h.run();
    h.key_press(eframe::egui::Key::Enter);
    h.run();
    h.run();
}

/// Wählt die Aufgabe mit diesem Titel (Detail rechts).
pub fn select(h: &mut Harness<'static, TasksUi>, title: &str) {
    let id = h
        .state()
        .items
        .iter()
        .find(|t| t.title == title)
        .map(|t| t.id.clone())
        .unwrap();
    h.state_mut().selected = Some(id);
    h.run();
    h.run();
}

/// Kennung der Aufgabe mit diesem Titel.
pub fn id_of(h: &Harness<'static, TasksUi>, title: &str) -> String {
    h.state()
        .items
        .iter()
        .find(|t| t.title == title)
        .map(|t| t.id.clone())
        .unwrap()
}
