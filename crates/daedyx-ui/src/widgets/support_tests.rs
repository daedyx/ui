//! Hilfen für die Baustein-Tests: Harness mit installiertem Thema (Desk, hell).
use crate::theme;
use crate::tokens::AppAccent;
use egui_kittest::Harness;

pub fn harness<'a>(app: impl FnMut(&mut eframe::egui::Ui) + 'a) -> Harness<'a> {
    Harness::builder()
        .with_size((640.0, 480.0))
        .with_context_setup(|ctx| theme::install(ctx, AppAccent::Desk))
        .build_ui(app)
}

pub fn harness_state<'a, S>(
    state: S,
    app: impl FnMut(&mut eframe::egui::Ui, &mut S) + 'a,
) -> Harness<'a, S> {
    Harness::builder()
        .with_size((640.0, 480.0))
        .with_context_setup(|ctx| theme::install(ctx, AppAccent::Desk))
        .build_ui_state(app, state)
}
