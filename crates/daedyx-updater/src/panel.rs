//! Bauteil „Updates“ für den Abschnitt der App (Tasks: Einstellungen › Updates; Stil 1) und der Hinweis in der Statuszeile. Die Notiz erscheint als reiner
//! Text, ohne Links und ohne Markdown. Farben nur aus den Tokens, Texte nur aus `lang/*.json`.
use crate::install::supported;
use crate::status::{Status, block_key};
use crate::worker::Updater;
use daedyx_ui::i18n::Texts;
use daedyx_ui::theme::look_ui;
use daedyx_ui::tokens::TOKENS;
use daedyx_ui::widgets::{BadgeTone, ButtonKind, badge, button_enabled};
use daedyx_update::reason::BlockKind;
use eframe::egui::{Color32, ProgressBar, Response, RichText, ScrollArea, Ui};

const NOTES_H: f32 = 160.0;

/// Text der Lage (ohne Notiz). `None` vor der ersten Suche.
pub fn status_text(status: &Status, t: &Texts, source: &str) -> Option<String> {
    Some(match status {
        Status::Idle => return None,
        Status::Checking => t.t("update.pruefe"),
        Status::UpToDate => t.t("update.aktuell"),
        Status::Available { version, .. } => t.tf("update.verfuegbar", &[("version", version)]),
        Status::Downloading { done, total } => {
            let pct = Status::percent(*done, *total).to_string();
            t.tf("update.laedt", &[("prozent", &pct)])
        }
        Status::Ready { version } => t.tf("update.bereit", &[("version", version)]),
        Status::Installing => t.t("update.startet"),
        Status::Busy => t.t("update.belegt"),
        Status::Blocked(BlockKind::NeedsManualUpdate) => {
            t.tf("update.von_hand", &[("quelle", source)])
        }
        Status::Blocked(kind) => t.t(block_key(*kind)),
        Status::NotSaved => t.t("update.nicht_gesichert"),
        Status::StartFailed => t.t("update.start_fehler"),
    })
}

fn tone_color(ui: &Ui, status: &Status) -> Color32 {
    let look = look_ui(ui);
    let s = &look.p.state;
    match status {
        Status::UpToDate => s.ok.text,
        Status::Available { .. } => s.info.text,
        Status::Blocked(BlockKind::Offline | BlockKind::SourceUnavailable) => look.p.text_muted,
        Status::Blocked(BlockKind::NotTrusted | BlockKind::SourceWentBack) => s.error.text,
        Status::Blocked(_) | Status::Busy | Status::NotSaved | Status::StartFailed => {
            s.warning.text
        }
        _ => look.p.text,
    }
}

fn gap(ui: &mut Ui) {
    ui.add_space(f32::from(TOKENS.spacing.s3));
}

/// Inhalt des Abschnitts „Updates“ (die Überschrift setzt die App).
pub fn show(ui: &mut Ui, updater: &mut Updater, t: &Texts) {
    let look = look_ui(ui);
    let status = updater.status().clone();
    if let Some(text) = status_text(&status, t, updater.source()) {
        ui.label(RichText::new(text).color(tone_color(ui, &status)));
        gap(ui);
    }
    if let Status::Downloading { done, total } = status {
        let frac = Status::percent(done, total) as f32 / 100.0;
        ui.add(ProgressBar::new(frac).desired_width(ui.available_width().min(360.0)));
        gap(ui);
    }
    if let Status::Available {
        notes, important, ..
    } = &status
    {
        if *important {
            let _ = badge(ui, BadgeTone::Warning, &t.t("update.wichtig"));
            gap(ui);
        }
        if !notes.trim().is_empty() {
            ui.label(RichText::new(t.t("update.notiz")).strong());
            ScrollArea::vertical()
                .id_salt("update_notiz")
                .max_height(NOTES_H)
                .show(ui, |ui| {
                    ui.label(RichText::new(notes.as_str()).color(look.p.text));
                });
            gap(ui);
        }
    }
    let busy = status.working();
    let offer = matches!(status, Status::Available { .. });
    let retry = matches!(
        status,
        Status::Blocked(BlockKind::Interrupted | BlockKind::NoSpace | BlockKind::LocalFile)
            | Status::StartFailed
    );
    ui.horizontal(|ui| {
        if (offer || retry) && supported() {
            let label = if offer {
                t.t("update.jetzt")
            } else {
                t.t("update.erneut")
            };
            if button_enabled(ui, ButtonKind::Primary, &label, !busy).clicked() {
                updater.update_now();
            }
        }
        if button_enabled(ui, ButtonKind::Secondary, &t.t("update.pruefen"), !busy).clicked() {
            updater.check_now();
        }
    });
    gap(ui);
    if offer {
        let key = if supported() {
            "update.jetzt.hinweis"
        } else {
            "update.nur_windows"
        };
        ui.label(RichText::new(t.t(key)).small().color(look.p.text_muted));
        gap(ui);
    }
    let mut auto = updater.auto_check();
    if ui.checkbox(&mut auto, t.t("update.auto")).changed() {
        updater.set_auto_check(auto);
    }
    ui.label(
        RichText::new(t.t("update.auto.hinweis"))
            .small()
            .color(look.p.text_muted),
    );
}

/// Hinweis in der Statuszeile, solange es etwas zu sagen gibt (Angebot, Laden, Sperre). Klick: Die App öffnet den Abschnitt.
pub fn status_badge(ui: &mut Ui, updater: &Updater, t: &Texts) -> Option<Response> {
    let (tone, text) = match updater.status() {
        Status::Idle | Status::Checking | Status::UpToDate => return None,
        Status::Available { version, .. } => (
            BadgeTone::Info,
            t.tf("update.status.verfuegbar", &[("version", version)]),
        ),
        Status::Downloading { .. } | Status::Ready { .. } => {
            (BadgeTone::Info, t.t("update.status.laedt"))
        }
        Status::Installing => (BadgeTone::Info, t.t("update.status.startet")),
        // Still und ohne Dialog (plan-update-bibliothek 7): nur der kurze Satz.
        Status::Blocked(kind @ (BlockKind::Offline | BlockKind::SourceUnavailable)) => {
            (BadgeTone::Neutral, t.t(block_key(*kind)))
        }
        _ => (BadgeTone::Warning, t.t("update.status.hinweis")),
    };
    Some(badge(ui, tone, &text).interact(eframe::egui::Sense::click()))
}
