//! Ablauf im eigenen Thread (plan-update-bibliothek 4 und 6). Die Oberfläche schickt Befehle über einen Kanal und
//! holt Meldungen in jedem Bild ab ([`Updater::poll`]); keine geteilten Sperren im Speicher. Suche und Speichern von
//! `updates.json` laufen unter der gemeinsamen Sperre mit dem Launcher. Automatisch wird höchstens einmal am Tag
//! gesucht (Schalter „Beim Start suchen“, U-F2), installiert wird nie ohne Klick.
use crate::config::{Backend, Config};
use crate::install::Prepared;
use crate::status::Status;
use crate::worker_run::run;
use daedyx_update::reason::BlockKind;
use eframe::egui::Context;
use std::sync::mpsc::{Receiver, Sender, channel};

pub(crate) enum Cmd {
    Check,
    Download,
    SetAuto(bool),
}

pub(crate) enum Msg {
    Status(Status),
    Auto(bool),
    /// Geprüftes Setup, zugleich der Status „bereit“; immer die letzte Meldung eines Downloads.
    Ready(Box<Prepared>),
}

/// Griff der Oberfläche auf den Ablauf.
pub struct Updater {
    tx: Option<Sender<Cmd>>,
    rx: Option<Receiver<Msg>>,
    status: Status,
    auto_check: bool,
    ready: Option<Prepared>,
    /// Erwartete Quelle als Text für den Hinweis „bitte von Hand laden“ (ohne Link, Review M-U1-1).
    source: String,
}

impl Updater {
    /// Startet den Ablauf. `ctx` bekommt nach jeder Meldung ein neues Bild.
    pub fn start(config: Config, backend: Backend, ctx: Option<Context>) -> Updater {
        let (tx, cmd_rx) = channel();
        let (msg_tx, rx) = channel();
        let auto_check = config.initial_state().auto_check;
        let source = config.policy.expected_source();
        let spawned = std::thread::Builder::new()
            .name("daedyx-update".into())
            .spawn(move || run(&config, backend, &cmd_rx, &msg_tx, ctx.as_ref()));
        let ok = spawned.is_ok();
        Updater {
            tx: ok.then_some(tx),
            rx: ok.then_some(rx),
            status: Status::Idle,
            auto_check,
            ready: None,
            source,
        }
    }

    /// Ohne Ablauf (Tests der Oberfläche, Bildschirmfotos): Knöpfe tun nichts.
    pub fn disabled() -> Updater {
        Updater {
            tx: None,
            rx: None,
            status: Status::Idle,
            auto_check: true,
            ready: None,
            source: String::new(),
        }
    }

    /// Meldungen abholen (jedes Bild).
    pub fn poll(&mut self) {
        let Some(rx) = &self.rx else { return };
        let msgs: Vec<Msg> = rx.try_iter().collect();
        for msg in msgs {
            self.apply(msg);
        }
    }

    fn apply(&mut self, msg: Msg) {
        match msg {
            Msg::Status(s) => self.status = s,
            Msg::Auto(on) => self.auto_check = on,
            // Bereit setzt den Status mit; danach kommt keine Statusmeldung mehr, die ein StartFailed der App
            // überschreiben könnte (Review U2-R N-U2-3).
            Msg::Ready(p) => {
                self.status = Status::Ready {
                    version: p.version().to_string(),
                };
                self.ready = Some(*p);
            }
        }
    }

    /// Meldungen ohne Kanal verarbeiten (Tests der Reihenfolge).
    #[cfg(test)]
    pub(crate) fn feed(&mut self, msgs: Vec<Msg>) {
        for m in msgs {
            self.apply(m);
        }
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    /// Lage von außen setzen (die App nach dem Startversuch des Setups).
    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn auto_check(&self) -> bool {
        self.auto_check
    }

    fn send(&self, cmd: Cmd) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(cmd);
        }
    }

    /// Knopf „Nach Updates suchen“.
    pub fn check_now(&mut self) {
        if !self.status.working() {
            self.status = Status::Checking;
            self.send(Cmd::Check);
        }
    }

    /// Knopf „Jetzt aktualisieren“ (lädt, prüft; danach holt die App das Setup mit [`Updater::take_ready`]).
    pub fn update_now(&mut self) {
        // „Erneut“ nur, wo das Angebot noch gilt (abgebrochen, Platz, eigene Datei, Setup startete nicht: das Angebot
        // liegt im Ablauf noch vor, Review U2-R N-U2-2); sonst neu suchen.
        let retry = matches!(
            self.status,
            Status::Blocked(BlockKind::Interrupted | BlockKind::NoSpace | BlockKind::LocalFile)
                | Status::StartFailed
        );
        if retry || matches!(self.status, Status::Available { .. }) {
            self.status = Status::Downloading { done: 0, total: 0 };
            self.send(Cmd::Download);
        }
    }

    pub fn set_auto_check(&mut self, on: bool) {
        self.auto_check = on;
        self.send(Cmd::SetAuto(on));
    }

    /// Geprüftes Setup, bereit zum Start. Die App sichert vorher ihren Zustand und beendet sich danach.
    pub fn take_ready(&mut self) -> Option<Prepared> {
        self.ready.take()
    }
}
