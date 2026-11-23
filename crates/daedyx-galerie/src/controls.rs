//! Kopfzeile der Galerie: Titel und Umschalter für Darstellung, App-Farbe und Sprache.
use crate::app::Galerie;
use daedyx_ui::fonts;
use daedyx_ui::i18n::Language;
use daedyx_ui::theme::look_ui;
use daedyx_ui::tokens::{AppAccent, TOKENS};
use daedyx_ui::widgets::{ButtonKind, button};
use eframe::egui::{FontId, RichText, Theme, Ui};

/// Umschalter-Knopf: gewählt = wichtig, sonst leise.
fn choice(ui: &mut Ui, text: &str, active: bool) -> bool {
    let kind = if active {
        ButtonKind::Primary
    } else {
        ButtonKind::Quiet
    };
    button(ui, kind, text).clicked()
}

fn group_label(ui: &mut Ui, text: &str) {
    let muted = look_ui(ui).p.text_muted;
    ui.label(RichText::new(text).small().color(muted));
}

pub fn header(ui: &mut Ui, g: &mut Galerie) {
    let look = look_ui(ui);
    let title_font = FontId::new(TOKENS.font.steps.large.size, fonts::semibold_in(ui.ctx()));
    ui.label(
        RichText::new(g.texts.t("galerie.titel"))
            .font(title_font)
            .color(look.p.text),
    );
    ui.label(RichText::new(g.texts.t("galerie.untertitel")).color(look.p.text_muted));
    ui.add_space(f32::from(TOKENS.spacing.s3));
    ui.horizontal_wrapped(|ui| {
        group_label(ui, &g.texts.t("galerie.thema"));
        let dark = ui.visuals().dark_mode;
        if choice(ui, &g.texts.t("galerie.hell"), !dark) {
            ui.ctx().set_theme(Theme::Light);
        }
        if choice(ui, &g.texts.t("galerie.dunkel"), dark) {
            ui.ctx().set_theme(Theme::Dark);
        }
        ui.add_space(f32::from(TOKENS.spacing.s6));
        group_label(ui, &g.texts.t("galerie.app"));
        for (accent, key) in [
            (AppAccent::Desk, "app.desk"),
            (AppAccent::Tasks, "app.tasks"),
            (AppAccent::Board, "app.board"),
        ] {
            if choice(ui, &g.texts.t(key), g.accent == accent) {
                let ctx = ui.ctx().clone();
                g.set_accent(&ctx, accent);
            }
        }
        ui.add_space(f32::from(TOKENS.spacing.s6));
        group_label(ui, &g.texts.t("galerie.sprache"));
        for language in Language::ALL {
            if choice(ui, &language.native_name(), g.language == language) {
                g.set_language(language);
            }
        }
    });
}
