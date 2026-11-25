//! Startfehler-Fenster (Tasks T2, gemeinsam für die egui-Apps): Kann eine App nicht starten (Datenbank, Tresor,
//! Speicherplatz …), zeigt sie statt ihrer Oberfläche ein kleines Fenster mit Titel, Erklärung und „Beenden“.
//! Ersetzt das Meldungsfenster des Betriebssystems (kein `unsafe`). Texte und Akzent gibt die App herein; mit
//! `DAEDYX_SCREENSHOT` macht es ein Foto nur von sich selbst und schließt sich.
use crate::screenshot::Shot;
use crate::theme::{self, look_ui};
use crate::tokens::{AppAccent, TOKENS};
use crate::widgets::fatal_state;
use eframe::egui::{self, Frame, Margin, ViewportCommand};

/// Inhalt und Zustand des Fensters.
pub struct StartupError {
    title: String,
    text: String,
    quit: String,
    quit_chosen: bool,
    shot: Option<Shot>,
}

impl StartupError {
    pub fn new(title: &str, text: &str, quit: &str) -> StartupError {
        StartupError {
            title: title.to_owned(),
            text: text.to_owned(),
            quit: quit.to_owned(),
            quit_chosen: false,
            shot: None,
        }
    }

    /// Wurde „Beenden“ gewählt?
    pub fn quit_chosen(&self) -> bool {
        self.quit_chosen
    }

    /// Zeichnet den Inhalt (auch in Tests ohne Fenster).
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let look = look_ui(ui);
        let margin = i8::try_from(TOKENS.spacing.s6).unwrap_or(i8::MAX);
        Frame::new()
            .fill(look.p.window)
            .inner_margin(Margin::same(margin))
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                if fatal_state(ui, &self.title, &self.text, &self.quit) {
                    self.quit_chosen = true;
                }
            });
    }
}

impl eframe::App for StartupError {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if let Some(shot) = &mut self.shot {
            shot.tick(&ctx);
        }
        self.show(ui);
        if self.quit_chosen {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
}

/// Öffnet das Fenster und kehrt zurück, wenn es geschlossen ist. `window_title` steht in der Titelleiste.
pub fn run(
    app_id: &str,
    window_title: &str,
    accent: AppAccent,
    mut error: StartupError,
) -> Result<(), eframe::Error> {
    error.shot = Shot::from_env();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 320.0])
            .with_resizable(false)
            .with_title(window_title),
        // Spike S1: wgpu hält im Leerlauf einen Kern auf 100 %, glow nicht.
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        app_id,
        options,
        Box::new(move |cc| {
            theme::install(&cc.egui_ctx, accent);
            Ok(Box::new(error))
        }),
    )
}

#[cfg(test)]
mod tests;
