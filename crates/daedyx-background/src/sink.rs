//! Ereignisse aus Hintergrund, Infobereich, Pipe und Benachrichtigung in die Oberfläche. Jedes Ereignis weckt die
//! Oberfläche (`request_repaint`), damit `logic` auch bei verstecktem Fenster läuft.
use eframe::egui::Context;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, OnceLock};

/// Woher eine Erinnerung kommt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeKind {
    Event,
    Task,
    /// Aufgabe mit Vorlauf (Tasks): „Fällig in 10 Minuten“ statt einer Uhrzeit; `time` bleibt leer.
    TaskLead {
        minutes_before: i64,
    },
}

/// Eine fällige Erinnerung (Titel aus den Vorschau-Daten, Zeit schon in Ortszeit).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub kind: NoticeKind,
    pub title: String,
    /// Beginn bzw. Fälligkeit, z. B. „14:30“; leer bei ganztägig oder ohne Uhrzeit.
    pub time: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Fenster nach vorn holen (Infobereich, zweiter Start, Klick auf die Benachrichtigung).
    Show,
    /// Programm wirklich beenden (Menü im Infobereich, `--quit-running`).
    Quit,
    /// Eine Erinnerung wurde gemeldet; `shown` = die Benachrichtigung hat geklappt.
    Notice { notice: Notice, shown: bool },
    /// Ein Durchgang des Takts ist fertig (Zahl der abzugleichenden Quellen).
    Tick { sync_due: usize },
}

#[derive(Clone)]
pub struct Sink {
    tx: Sender<Event>,
    ctx: Arc<OnceLock<Context>>,
}

impl Sink {
    pub fn new() -> (Sink, Receiver<Event>) {
        let (tx, rx) = channel();
        let sink = Sink {
            tx,
            ctx: Arc::new(OnceLock::new()),
        };
        (sink, rx)
    }

    /// Ab jetzt weckt jedes Ereignis die Oberfläche.
    pub fn attach(&self, ctx: &Context) {
        let _ = self.ctx.set(ctx.clone());
    }

    pub fn emit(&self, e: Event) {
        // Kein Empfänger mehr heißt: Das Programm endet gerade; das Ereignis ist dann bedeutungslos.
        let _ = self.tx.send(e);
        if let Some(ctx) = self.ctx.get() {
            ctx.request_repaint();
        }
    }
}
