//! Galerie: zeigt alle Grundbausteine von `daedyx-ui` in hell und dunkel, mit Desk-, Tasks- und
//! Board-Akzent und in Deutsch, Englisch, Französisch. Nur erfundene Beispieltexte.
//!
//! Umgebungsvariablen (zum Prüfen und für Bildschirmfotos):
//! `DAEDYX_THEME=dark|light`, `DAEDYX_APP=desk|tasks|board`, `DAEDYX_LANG=de|en|fr`,
//! `DAEDYX_DIALOG=1` (Dialog offen), `DAEDYX_SCREENSHOT=datei.png` (nur das Fenster, dann Ende).
mod app;
mod controls;
mod demo;
mod sections;

use daedyx_ui::i18n::Texts;
use eframe::egui;

fn main() -> eframe::Result {
    let settings = app::Settings::from_env();
    let title = Texts::for_language(settings.language).t("galerie.titel");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 860.0])
            .with_min_inner_size([640.0, 480.0])
            .with_title(title),
        // Spike S1: wgpu hält im Leerlauf einen Kern auf 100 %, glow nicht.
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "daedyx-galerie",
        options,
        Box::new(move |cc| Ok(Box::new(app::Galerie::new(&cc.egui_ctx, settings)))),
    )
}
