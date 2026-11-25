//! Startfehler-Fenster ohne echtes Fenster (kittest): Warnmeldung mit Titel, Text und „Beenden“; der Klick merkt sich
//! das Ende.
use super::StartupError;
use crate::theme;
use crate::tokens::AppAccent;
use eframe::egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

#[test]
fn startfehler_fenster_zeigt_meldung_und_endet_mit_beenden() {
    let error = StartupError::new("Daten fehlen", "Bitte neu installieren.", "Beenden");
    let mut h = Harness::builder()
        .with_size((480.0, 320.0))
        .with_context_setup(|ctx| theme::install(ctx, AppAccent::Tasks))
        .build_ui_state(|ui, e: &mut StartupError| e.show(ui), error);
    h.run();
    assert!(!h.state().quit_chosen());
    let _ = h.get_by_role_and_label(Role::Alert, "Daten fehlen");
    let _ = h.get_by_label("Bitte neu installieren.");
    h.get_by_role_and_label(Role::Button, "Beenden").click();
    h.run();
    assert!(h.state().quit_chosen());
}
