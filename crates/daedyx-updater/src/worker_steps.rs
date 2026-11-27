//! Die beiden Schritte des Ablaufs: Suche (unter der Sperre, danach `updates.json` sichern) und Download nach Klick
//! (Sperre bleibt bis zum Start des Setups beim [`Prepared`]).
use crate::install::Prepared;
use crate::status::Status;
use crate::worker::Msg;
use crate::worker_run::{Run, WithAuto};
use daedyx_update::attest::Provenance;
use daedyx_update::check::{self, Outcome};
use daedyx_update::reason::{BlockKind, Blocked};
use daedyx_update::state::UpdateState;

impl Run<'_> {
    /// Prüfer für den Nachweis; scheitert schon das (Wurzel unlesbar), eine Sperre mit Grund.
    fn provenance(&self) -> Result<Box<dyn Provenance>, Status> {
        (self.backend.provenance)(&self.config.policy)
            .map_err(|e| Status::Blocked(Blocked::from(e).kind))
    }

    /// Die Sperre wird vor dem letzten Status freigegeben: Wer ihn sieht (Oberfläche, Launcher, Test), findet sie
    /// frei (Review fix-pipe-sid, M-PS-1).
    pub(crate) fn check(&mut self) {
        let Ok(lock) = self.lock() else {
            return self.status(Status::Busy);
        };
        self.status(Status::Checking);
        let provenance = match self.provenance() {
            Ok(p) => p,
            Err(s) => {
                drop(lock);
                return self.status(s);
            }
        };
        let seen = self.state.highest_seen();
        let outcome = check::check(
            self.backend.transport.as_mut(),
            provenance.as_ref(),
            &self.config.policy,
            &self.config.installed,
            seen.as_ref(),
        );
        // Frisch von der Platte (ein zweiter Schreiber wie der Launcher), dann nur aufwärts vermerken.
        self.state =
            UpdateState::load_or_default(&self.config.data_dir).with_auto(self.state.auto_check);
        self.state.note_check((self.backend.now)());
        if let Some(latest) = outcome.verified_latest() {
            self.state.note_seen(latest);
        }
        self.save();
        self.offer = None;
        let status = match outcome {
            Outcome::UpToDate { .. } => Status::UpToDate,
            Outcome::Available(offer) => {
                let e = offer.entry();
                let s = Status::Available {
                    version: e.version.to_string(),
                    notes: e.notes.clone(),
                    important: e.important,
                };
                self.offer = Some(*offer);
                s
            }
            Outcome::Blocked(b) => Status::Blocked(b.kind),
        };
        drop(lock);
        self.status(status);
    }

    pub(crate) fn download(&mut self) {
        let Some(offer) = self.offer.clone() else {
            return self.status(Status::Idle);
        };
        let Ok(lock) = self.lock() else {
            return self.status(Status::Busy);
        };
        let provenance = match self.provenance() {
            Ok(p) => p,
            Err(s) => {
                drop(lock);
                return self.status(s);
            }
        };
        let free = (self.backend.free_space)(&self.config.update_dir_existing());
        let mut last = u64::MAX;
        let (out, ctx) = (self.out, self.ctx);
        let mut progress = |done: u64, total: u64| {
            let pct = Status::percent(done, total);
            if pct != last {
                last = pct;
                let _ = out.send(Msg::Status(Status::Downloading { done, total }));
                if let Some(c) = ctx {
                    c.request_repaint();
                }
            }
        };
        let result = check::fetch_setup(
            self.backend.transport.as_mut(),
            provenance.as_ref(),
            &self.config.policy,
            &offer,
            &self.config.update_dir,
            free,
            &mut progress,
        );
        match result {
            Ok(path) => {
                let e = offer.entry();
                let prepared = Prepared::new(path, e.package.sha256, e.version.clone(), lock);
                self.send(Msg::Ready(Box::new(prepared)));
            }
            Err(b) => {
                drop(lock);
                // Nicht vertrauenswürdig: Angebot verwerfen (nur eine neue Suche bringt ein neues).
                if b.kind == BlockKind::NotTrusted {
                    self.offer = None;
                }
                self.status(Status::Blocked(b.kind));
            }
        }
    }
}
