//! Abschnitte „Backend-Brücke“ und „Tastenkürzel“ der Galerie.
use crate::app::Galerie;
use daedyx_ui::keys::{self, SHORTCUTS};
use daedyx_ui::theme::look_ui;
use daedyx_ui::widgets::{ButtonKind, button};
use eframe::egui::{RichText, Ui};

pub fn bridge(ui: &mut Ui, g: &mut Galerie) {
    ui.horizontal(|ui| {
        if button(ui, ButtonKind::Primary, &g.texts.t("bruecke.fragen")).clicked() {
            g.waiting = g.bridge.as_ref().is_some_and(|b| b.send(7).is_ok());
        }
        let status = match (g.bridge.is_some(), g.waiting, g.answer) {
            (false, _, _) => g.texts.t("bruecke.getrennt"),
            (true, true, _) => g.texts.t("bruecke.wartet"),
            (true, false, Some(n)) => g.texts.tf("bruecke.antwort", &[("wert", &n.to_string())]),
            (true, false, None) => String::new(),
        };
        ui.label(status);
    });
}

pub fn shortcuts(ui: &mut Ui, g: &Galerie) {
    let mac = cfg!(target_os = "macos");
    let faint = look_ui(ui).p.text_faint;
    for s in SHORTCUTS {
        ui.horizontal(|ui| {
            ui.label(g.texts.t(s.label_key));
            ui.label(RichText::new(keys::label(&g.texts, s, mac)).color(faint));
        });
    }
}
