//! Der Thread des Ablaufs: wartet auf Befehle oder den nächsten automatischen Termin (höchstens einmal am Tag), sucht,
//! lädt nach Klick (Schritte in `worker_steps.rs`).
use crate::config::{Backend, Config};
use crate::status::Status;
use crate::worker::{Cmd, Msg};
use daedyx_update::check::Offer;
use daedyx_update::lock::{Lock, LockError};
use daedyx_update::state::{AUTO_CHECK_INTERVAL_SECS, UpdateState};
use eframe::egui::Context;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

const DAY: Duration = Duration::from_secs(AUTO_CHECK_INTERVAL_SECS.unsigned_abs());

pub(crate) struct Run<'a> {
    pub(crate) config: &'a Config,
    pub(crate) backend: Backend,
    pub(crate) out: &'a Sender<Msg>,
    pub(crate) ctx: Option<&'a Context>,
    pub(crate) state: UpdateState,
    pub(crate) offer: Option<Offer>,
}

impl Run<'_> {
    pub(crate) fn send(&self, msg: Msg) {
        let _ = self.out.send(msg);
        if let Some(ctx) = self.ctx {
            ctx.request_repaint();
        }
    }

    pub(crate) fn status(&self, s: Status) {
        self.send(Msg::Status(s));
    }

    pub(crate) fn lock(&self) -> Result<Lock, LockError> {
        Lock::acquire(&self.config.lock_dir, self.config.policy.app_id())
    }

    /// Nächster automatischer Termin: `None`, wenn der Schalter aus ist. Beim Start mit zufälliger Verzögerung.
    fn next_auto(&self, first: bool) -> Option<Instant> {
        if !self.state.auto_check {
            return None;
        }
        let now = Instant::now();
        if self.state.due_for_auto_check((self.backend.now)()) {
            let delay = if first {
                self.config.auto_delay
            } else {
                Duration::ZERO
            };
            return now.checked_add(delay);
        }
        now.checked_add(DAY)
    }

    pub(crate) fn save(&mut self) {
        let _ = self.state.save(&self.config.data_dir);
    }

    fn set_auto(&mut self, on: bool) {
        // Speichern unter der gemeinsamen Sperre; hält sie gerade der Launcher, trotzdem speichern (die höchste
        // gesehene Version führt `save` zusammen, beim Schalter gewinnt der letzte Schreiber, Review U1-R H-U1-7).
        let _lock = self.lock();
        self.state = UpdateState::load_or_default(&self.config.data_dir).with_auto(on);
        self.save();
        self.send(Msg::Auto(on));
    }
}

pub(crate) trait WithAuto {
    fn with_auto(self, on: bool) -> Self;
}

impl WithAuto for UpdateState {
    fn with_auto(mut self, on: bool) -> UpdateState {
        self.auto_check = on;
        self
    }
}

impl Config {
    /// Update-Ordner zum Messen des freien Platzes; fehlt er noch, der Datenordner.
    pub(crate) fn update_dir_existing(&self) -> std::path::PathBuf {
        if self.update_dir.is_dir() {
            self.update_dir.clone()
        } else {
            self.data_dir.clone()
        }
    }
}

/// Hauptschleife des Threads; endet, wenn die Oberfläche ihren Griff fallen lässt.
pub(crate) fn run(
    config: &Config,
    backend: Backend,
    cmds: &Receiver<Cmd>,
    out: &Sender<Msg>,
    ctx: Option<&Context>,
) {
    let state = config.initial_state();
    let mut run = Run {
        config,
        backend,
        out,
        ctx,
        state,
        offer: None,
    };
    // Reste alter Setups und abgebrochener Downloads, nur unter der Sperre (Review U1-R, N-U1-4).
    if let Ok(_lock) = run.lock() {
        let _ = daedyx_update::download::clean_leftovers(&config.update_dir);
    }
    let mut next = run.next_auto(true);
    loop {
        let cmd = match next {
            Some(at) => match cmds.recv_timeout(at.saturating_duration_since(Instant::now())) {
                Ok(c) => Some(c),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => return,
            },
            None => match cmds.recv() {
                Ok(c) => Some(c),
                Err(_) => return,
            },
        };
        match cmd {
            None | Some(Cmd::Check) => run.check(),
            Some(Cmd::Download) => run.download(),
            Some(Cmd::SetAuto(on)) => run.set_auto(on),
        }
        next = run.next_auto(false);
    }
}
