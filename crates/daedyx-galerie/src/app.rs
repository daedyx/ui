//! Zustand und Bildaufbau der Galerie.
use crate::{controls, sections};
use daedyx_ui::bridge::{Bridge, repaint_waker};
use daedyx_ui::i18n::{Language, Texts};
use daedyx_ui::keys::{self, Action};
use daedyx_ui::screenshot::Shot;
use daedyx_ui::theme::{self, look_ui};
use daedyx_ui::tokens::{AppAccent, TOKENS};
use daedyx_ui::widgets::{ConfirmDialog, confirm_dialog};
use eframe::egui::{self, Context, Frame, Margin, ScrollArea, Theme};

/// Startwerte aus der Umgebung (alles optional).
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub language: Language,
    pub accent: AppAccent,
    pub dark: Option<bool>,
    pub dialog: bool,
}

fn var(name: &str) -> String {
    std::env::var(name).unwrap_or_default().to_ascii_lowercase()
}

impl Settings {
    pub fn from_env() -> Settings {
        let language = match var("DAEDYX_LANG").as_str() {
            "en" => Language::En,
            "fr" => Language::Fr,
            _ => Language::De,
        };
        let accent = match var("DAEDYX_APP").as_str() {
            "tasks" => AppAccent::Tasks,
            "board" => AppAccent::Board,
            _ => AppAccent::Desk,
        };
        let dark = match var("DAEDYX_THEME").as_str() {
            "dark" => Some(true),
            "light" => Some(false),
            _ => None,
        };
        Settings {
            language,
            accent,
            dark,
            dialog: var("DAEDYX_DIALOG") == "1",
        }
    }
}

pub struct Galerie {
    pub texts: Texts,
    pub language: Language,
    pub accent: AppAccent,
    pub subject: String,
    pub list_selected: Option<usize>,
    pub card_selected: bool,
    pub dialog_open: bool,
    pub last_menu: Option<usize>,
    pub bridge: Option<Bridge<u32, u32>>,
    pub answer: Option<u32>,
    pub waiting: bool,
    shot: Option<Shot>,
}

/// Erfundener „Kern“ für die Brücken-Vorführung: rechnet nur.
async fn demo_core(question: u32) -> u32 {
    question.saturating_mul(6)
}

impl Galerie {
    pub fn new(ctx: &Context, s: Settings) -> Galerie {
        theme::install(ctx, s.accent);
        if let Some(dark) = s.dark {
            ctx.set_theme(if dark { Theme::Dark } else { Theme::Light });
        }
        Galerie {
            texts: Texts::for_language(s.language),
            language: s.language,
            accent: s.accent,
            subject: String::new(),
            list_selected: Some(1),
            card_selected: true,
            dialog_open: s.dialog,
            last_menu: None,
            bridge: Bridge::start(demo_core, repaint_waker(ctx)).ok(),
            answer: None,
            waiting: false,
            shot: Shot::from_env(),
        }
    }

    pub fn set_language(&mut self, language: Language) {
        self.language = language;
        self.texts = Texts::for_language(language);
    }

    pub fn set_accent(&mut self, ctx: &Context, accent: AppAccent) {
        self.accent = accent;
        theme::install(ctx, accent);
    }

    fn dialog(&mut self, ctx: &Context) {
        let t = &self.texts;
        let (title, text, confirm, cancel) = (
            t.t("dialog.titel"),
            t.t("dialog.text"),
            t.t("knopf.loeschen"),
            t.t("knopf.abbrechen"),
        );
        let d = ConfirmDialog {
            id_salt: "galerie-dialog",
            title: &title,
            text: &text,
            confirm: &confirm,
            cancel: &cancel,
            danger: true,
        };
        if confirm_dialog(ctx, &d).is_some() {
            self.dialog_open = false;
        }
    }
}

impl eframe::App for Galerie {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if let Some(shot) = &mut self.shot {
            shot.tick(&ctx);
        }
        if keys::pressed(&ctx) == Some(Action::Close) {
            self.dialog_open = false;
        }
        if let Some(bridge) = &mut self.bridge {
            for answer in bridge.poll() {
                self.answer = Some(answer);
                self.waiting = false;
            }
        }
        let look = look_ui(ui);
        let margin = i8::try_from(TOKENS.spacing.s6).unwrap_or(i8::MAX);
        Frame::new()
            .fill(look.p.window)
            .inner_margin(Margin::same(margin))
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                controls::header(ui, self);
                ui.add_space(f32::from(TOKENS.spacing.s4));
                ScrollArea::vertical().show(ui, |ui| sections::all(ui, self));
            });
        if self.dialog_open {
            self.dialog(&ctx);
        }
    }
}
