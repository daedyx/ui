//! Tastatur im Menü (Tasks T3b, Vorlage `ContextMenu.test.tsx`): erster Eintrag fokussiert, Pfeile im Kreis, Esc
//! schließt mit Fokusrückgabe an den Knopf, Enter wählt, Klick daneben schließt.
use super::support_tests::harness_state;
use super::{MenuItem, menu_button};
use eframe::egui::accesskit::Role;
use eframe::egui::{Event, Key, Modifiers, PointerButton, Pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

fn menu() -> Harness<'static, Option<usize>> {
    let mut h = harness_state(None::<usize>, |ui, chosen| {
        let items = ["Eins", "Zwei", "Drei"].map(|label| MenuItem {
            label,
            shortcut: None,
            danger: false,
        });
        if let Some(i) = menu_button(ui, "Aktionen", &items) {
            *chosen = Some(i);
        }
    });
    h.run();
    h.get_by_role_and_label(Role::Button, "Aktionen").click();
    h.run();
    h.run();
    h
}

fn focused(h: &Harness<'static, Option<usize>>, label: &str) -> bool {
    h.get_by_role_and_label(Role::MenuItem, label)
        .accesskit_node()
        .is_focused()
}

#[test]
fn oeffnen_fokussiert_den_ersten_eintrag_und_pfeile_wandern_im_kreis() {
    let mut h = menu();
    assert!(focused(&h, "Eins"));
    for (key, want) in [
        (Key::ArrowDown, "Zwei"),
        (Key::ArrowDown, "Drei"),
        (Key::ArrowDown, "Eins"),
        (Key::ArrowUp, "Drei"),
    ] {
        h.key_press(key);
        h.run();
        h.run();
        assert!(focused(&h, want), "{want}");
    }
}

#[test]
fn enter_waehlt_und_schliesst() {
    let mut h = menu();
    h.key_press(Key::ArrowDown);
    h.run();
    h.key_press(Key::Enter);
    h.run();
    h.run();
    assert_eq!(*h.state(), Some(1));
    assert!(h.query_by_role(Role::Menu).is_none());
}

#[test]
fn escape_schliesst_und_gibt_den_fokus_an_den_knopf() {
    let mut h = menu();
    h.key_press(Key::Escape);
    h.run();
    h.run();
    assert!(h.query_by_role(Role::Menu).is_none());
    assert!(
        h.get_by_role_and_label(Role::Button, "Aktionen")
            .accesskit_node()
            .is_focused()
    );
    assert_eq!(*h.state(), None);
}

#[test]
fn klick_daneben_schliesst_ohne_auswahl() {
    let mut h = menu();
    let at = Pos2::new(390.0, 290.0);
    h.hover_at(at);
    h.run();
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
        h.run();
    }
    h.run();
    assert!(h.query_by_role(Role::Menu).is_none());
    assert_eq!(*h.state(), None);
}
