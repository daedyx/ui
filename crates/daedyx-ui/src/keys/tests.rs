use super::{Action, SHORTCUTS, find, label, pressed};
use crate::i18n::{Language, Texts};
use eframe::egui::{Key, Modifiers};
use egui_kittest::Harness;
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn jede_aktion_und_jede_tastenfolge_genau_einmal() {
    for (i, a) in SHORTCUTS.iter().enumerate() {
        for b in SHORTCUTS.iter().skip(i + 1) {
            assert_ne!(a.action, b.action, "Aktion doppelt");
            assert_ne!(
                a.shortcut, b.shortcut,
                "Tastenfolge doppelt: {:?}",
                a.action
            );
        }
    }
}

/// egui ignoriert beim Abfangen zusätzliches Umschalt/Alt. Darum muss eine spezifischere
/// Folge (Strg+Umschalt+Z) vor der allgemeineren (Strg+Z) stehen, sonst wird sie nie erkannt.
#[test]
fn keine_folge_verdeckt_eine_spaetere() {
    for (i, early) in SHORTCUTS.iter().enumerate() {
        for late in SHORTCUTS.iter().skip(i + 1) {
            let hidden = early.shortcut.logical_key == late.shortcut.logical_key
                && late
                    .shortcut
                    .modifiers
                    .matches_logically(early.shortcut.modifiers);
            assert!(!hidden, "{:?} verdeckt {:?}", early.action, late.action);
        }
    }
}

#[test]
fn jede_beschriftung_gibt_es_in_allen_sprachen() {
    for lang in Language::ALL {
        let t = Texts::for_language(lang);
        for s in SHORTCUTS {
            assert!(t.has(s.label_key), "{lang:?}: {}", s.label_key);
        }
    }
}

#[test]
fn anzeige_nennt_strg_unter_windows_und_cmd_auf_dem_mac() {
    let de = Texts::for_language(Language::De);
    let search = find(Action::Search).unwrap();
    assert_eq!(label(&de, search, false), "Strg+K");
    assert_eq!(label(&de, search, true), "Cmd+K");
    assert_eq!(
        label(&de, find(Action::Redo).unwrap(), false),
        "Strg+Umschalt+Z"
    );
    assert_eq!(label(&de, find(Action::Settings).unwrap(), false), "Strg+,");
    let fr = Texts::for_language(Language::Fr);
    assert_eq!(label(&fr, find(Action::Close).unwrap(), false), "Échap");
    assert_eq!(label(&de, find(Action::Delete).unwrap(), false), "Entf");
    assert_eq!(label(&de, find(Action::Help).unwrap(), false), "F1");
}

fn press(mods: Modifiers, key: Key) -> Option<Action> {
    let seen = Rc::new(Cell::new(None));
    let out = seen.clone();
    let mut h = Harness::new_ui(move |ui| {
        if let Some(a) = pressed(ui.ctx()) {
            out.set(Some(a));
        }
    });
    h.run();
    h.key_press_modifiers(mods, key);
    h.run();
    seen.get()
}

#[test]
fn tasten_im_fenster_werden_zur_aktion() {
    assert_eq!(press(Modifiers::COMMAND, Key::K), Some(Action::Search));
    assert_eq!(press(Modifiers::COMMAND, Key::Z), Some(Action::Undo));
    assert_eq!(
        press(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z),
        Some(Action::Redo)
    );
    assert_eq!(press(Modifiers::NONE, Key::Escape), Some(Action::Close));
    assert_eq!(press(Modifiers::NONE, Key::K), None);
}
