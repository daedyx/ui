//! Hintergrund-Zustand der App: Ereigniskanal, Fenster, Benachrichtigung, Autostart, Takt. `logic` läuft vor jedem
//! Bild und – über `eframe::App::logic` – auch bei verstecktem Fenster.
use crate::autostart::{AutostartStore, FakeStore, State};
use crate::notify::{Notifier, Recorder};
use crate::sink::{Event, Notice, Sink};
use crate::ticker::{Jobs, NoticeStyle, SharedStyle, Ticker};
use crate::window::WindowState;
use daedyx_ui::i18n::Language;
use eframe::egui::Context;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::Duration;

pub struct Background {
    pub sink: Sink,
    rx: Receiver<Event>,
    pub window: WindowState,
    pub notifier: Arc<dyn Notifier>,
    pub autostart: Box<dyn AutostartStore>,
    pub autostart_state: Option<State>,
    pub autostart_error: bool,
    /// Zuletzt gemeldete Erinnerung (Statusleiste).
    pub last: Option<Notice>,
    /// Durchgänge des Takts seit dem Start und zuletzt abzugleichende Quellen.
    pub ticks: u64,
    pub sync_due: usize,
    /// Ergebnis der Testbenachrichtigung (Einstellungen).
    pub test_sent: Option<bool>,
    style: Arc<SharedStyle>,
    ticker: Option<Ticker>,
}

impl Background {
    /// Ohne Benachrichtigungen des Systems und ohne echten Autostart (Tests); das Programm setzt beides.
    pub fn new(language: Language) -> Background {
        let (sink, rx) = Sink::new();
        let autostart: Box<dyn AutostartStore> = Box::new(FakeStore::default());
        Background {
            sink,
            rx,
            window: WindowState::default(),
            notifier: Arc::new(Recorder::default()),
            autostart_state: autostart.state().ok(),
            autostart,
            autostart_error: false,
            last: None,
            ticks: 0,
            sync_due: 0,
            test_sent: None,
            style: Arc::new(SharedStyle::new(NoticeStyle::new(language))),
            ticker: None,
        }
    }

    pub fn set_language(&mut self, language: Language) {
        self.style.set_language(language);
    }

    /// Titel von Terminen und Aufgaben in Benachrichtigungen zeigen (Standard: aus, Review M-2).
    pub fn set_show_titles(&mut self, on: bool) {
        self.style.set_show_titles(on);
    }

    pub fn show_titles(&self) -> bool {
        self.style.get().show_titles
    }

    pub fn set_autostart_store(&mut self, store: Box<dyn AutostartStore>) {
        self.autostart = store;
        self.autostart_state = self.autostart.state().ok();
    }

    pub fn set_autostart(&mut self, on: bool) {
        self.autostart_error = self.autostart.set(on).is_err();
        self.autostart_state = self.autostart.state().ok();
    }

    /// Startet (oder ersetzt) den Takt.
    pub fn start_ticker(&mut self, every: Duration, jobs: Arc<dyn Jobs>) {
        self.ticker = None;
        self.ticker = Some(Ticker::spawn(
            every,
            jobs,
            Arc::clone(&self.notifier),
            self.sink.clone(),
            Arc::clone(&self.style),
        ));
    }

    /// Testbenachrichtigung mit erfundenem Inhalt (Titel und Text aus der Sprachdatei).
    pub fn send_test(&mut self, title: &str, body: &str) {
        self.test_sent = Some(self.notifier.notify(title, body).is_ok());
    }

    /// Ereignisse anwenden, Schließen abfangen, versteckten Start zu Ende führen.
    pub fn logic(&mut self, ctx: &Context) {
        self.window.start_step(ctx);
        let events: Vec<Event> = self.rx.try_iter().collect();
        for e in events {
            for cmd in self.window.apply(&e) {
                ctx.send_viewport_cmd(cmd);
            }
            match e {
                Event::Notice { notice, .. } => self.last = Some(notice),
                Event::Tick { sync_due } => {
                    self.ticks = self.ticks.saturating_add(1);
                    self.sync_due = sync_due;
                }
                Event::Show | Event::Quit => {}
            }
        }
        if ctx.input(|i| i.viewport().close_requested()) {
            for cmd in self.window.on_close_requested() {
                ctx.send_viewport_cmd(cmd);
            }
        }
    }
}
