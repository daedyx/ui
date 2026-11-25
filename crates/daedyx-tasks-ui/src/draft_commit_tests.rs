//! Entwurf vor dem Beenden (Tasks T4): geänderte Felder gelten, ein unbrauchbares Datum hält die App offen.
use crate::test_support::fixed_store;
use crate::{Draft, TasksUi};
use daedyx_ui::i18n::Language;

fn ui_with_first_task() -> (TasksUi, String) {
    let mut ui = crate::start_with(Some(fixed_store()), Some((0, false)));
    let t = ui.items.first().cloned().unwrap();
    let texts = crate::texts::texts(Language::De);
    let (date, time) = crate::detail_fields::draft_due(&ui, &texts, &t);
    ui.draft = Draft {
        id: t.id.clone(),
        title: t.title.clone(),
        notes: t.notes.clone(),
        date,
        time,
        subtask: String::new(),
        seen: (
            t.title.clone(),
            t.notes.clone(),
            t.due.clone(),
            t.due_has_time,
        ),
    };
    (ui, t.id)
}

#[test]
fn ohne_entwurf_oder_ohne_aenderung_ist_nichts_zu_tun() {
    let texts = crate::texts::texts(Language::De);
    let mut empty = crate::start_with(Some(fixed_store()), None);
    assert!(empty.commit_draft(&texts));
    let (mut ui, id) = ui_with_first_task();
    let before = ui.store.as_deref().unwrap().task(&id).unwrap();
    assert!(ui.commit_draft(&texts));
    assert_eq!(ui.store.as_deref().unwrap().task(&id).unwrap(), before);
}

#[test]
fn titel_und_notiz_gelten_vor_dem_beenden() {
    let texts = crate::texts::texts(Language::De);
    let (mut ui, id) = ui_with_first_task();
    ui.draft.title = "  Fahrrad zur Inspektion  ".into();
    ui.draft.notes = "Bremsen prüfen lassen".into();
    assert!(ui.commit_draft(&texts));
    let t = ui.store.as_deref().unwrap().task(&id).unwrap();
    assert_eq!(t.title, "Fahrrad zur Inspektion");
    assert_eq!(t.notes, "Bremsen prüfen lassen");
}

#[test]
fn unbrauchbares_datum_haelt_die_app_offen() {
    let texts = crate::texts::texts(Language::De);
    let (mut ui, id) = ui_with_first_task();
    let before = ui.store.as_deref().unwrap().task(&id).unwrap();
    ui.draft.date = "31.02.2026".into();
    assert!(!ui.commit_draft(&texts));
    assert_eq!(ui.error.as_deref(), Some("aufgaben.fehler.datum"));
    assert_eq!(
        ui.store.as_deref().unwrap().task(&id).unwrap().due,
        before.due
    );
    ui.draft.date = "24.12.2026".into();
    ui.draft.time = String::new();
    assert!(ui.commit_draft(&texts));
    let t = ui.store.as_deref().unwrap().task(&id).unwrap();
    assert_eq!(t.due.as_deref(), Some("2026-12-24"));
}
