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
