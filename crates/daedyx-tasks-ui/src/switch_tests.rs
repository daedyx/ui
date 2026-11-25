//! Umschalter Liste/Board im Listenkopf als Einstellung des Bausteins (Tasks T2-1): Desk zeigt ihn, Tasks blendet
//! ihn aus und wechselt über die eigene Navigationsleiste. Das Listenmenü hat einen eigenen, eindeutigen Namen.
use crate::test_support::harness;
use eframe::egui::accesskit::Role;
use egui_kittest::kittest::Queryable;

#[test]
fn umschalter_ist_standard_und_laesst_sich_ausblenden() {
    for board in [false, true] {
        let mut h = harness(Some((0, board)));
        assert!(h.state().mode_switch);
        let _ = h.get_by_role_and_label(Role::TabList, "Ansicht der Liste");
        h.state_mut().mode_switch = false;
        h.run();
        assert_eq!(
            h.query_all_by_role_and_label(Role::TabList, "Ansicht der Liste")
                .count(),
            0
        );
        assert_eq!(h.query_all_by_role(Role::Tab).count(), 0);
        let _ = h.get_by_label("Listen-Aktionen");
    }
}

#[test]
fn listenmenue_heisst_nicht_wie_der_reiter_liste() {
    let h = harness(Some((0, false)));
    let _ = h.get_by_label("Listen-Aktionen");
    assert_eq!(h.query_all_by_label("Liste").count(), 1, "nur der Reiter");
}
