use super::*;
use crate::widgets::support_tests::harness;
use egui_kittest::kittest::{NodeT, Queryable};

#[test]
fn bereich_meldet_rolle_und_namen() {
    let h = harness(|ui| {
        named_scope(ui, "probe", Role::Tree, "Probe-Baum", |ui| {
            ui.label("Inhalt")
        });
    });
    let _ = h.get_by_role_and_label(Role::Tree, "Probe-Baum");
}

#[test]
fn auswahl_ebene_und_ueberschrift_kommen_beim_screenreader_an() {
    let h = harness(|ui| {
        let r = ui.selectable_label(true, "Eintrag");
        mark_selected(ui, r.id, true);
        mark_tree(ui, r.id, 2, Some(false));
        let head = ui.label("Kopf");
        mark_heading(ui, head.id, 1, "Kopf");
    });
    let item = h.get_by_label("Eintrag");
    let node = item.accesskit_node();
    assert_eq!(node.is_selected(), Some(true));
    assert!(node.toggled().is_none(), "nicht als Schalter gemeldet");
    assert_eq!(node.level(), Some(2));
    assert_eq!(node.data().is_expanded(), Some(false));
    let _ = h.get_by_role_and_label(Role::Heading, "Kopf");
}
