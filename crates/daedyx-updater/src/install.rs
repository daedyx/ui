//! Geprüftes Setup starten (plan-update-bibliothek 6). [`Prepared`] entsteht nur im Ablauf nach bestandener Prüfung
//! und hält die gemeinsame Sperre bis zum Start. Die Argumente stehen hier fest, nichts kommt aus `entry.json`:
//! `/P` (passiv: Fortschritt, keine Fragen), `/UPDATE` (bisheriger Ordner), `/R` (App danach neu starten), bei
//! verstecktem Fenster dazu `/ARGS --hidden`. Scheitert der Start, wird die Datei gelöscht.
use daedyx_sys::StartError;
use daedyx_update::lock::Lock;
use daedyx_update::version::Version;
use std::path::{Path, PathBuf};

/// Immer übergebene Schalter des Setups.
pub const ARGS: [&str; 3] = ["/P", "/UPDATE", "/R"];
/// Zusatz, wenn die App versteckt im Infobereich lief: Sie startet danach wieder versteckt.
pub const HIDDEN_ARGS: [&str; 2] = ["/ARGS", "--hidden"];

/// Argumente des Setups.
pub fn setup_args(hidden: bool) -> Vec<&'static str> {
    let mut args = ARGS.to_vec();
    if hidden {
        args.extend(HIDDEN_ARGS);
    }
    args
}

/// Kann diese Plattform ihr Setup selbst starten? Bisher nur Windows; macOS folgt mit dem Mac-Block (U-F3).
pub fn supported() -> bool {
    cfg!(windows)
}

/// Startet ein Programm nach Prüfung der Prüfsumme (echt: `daedyx_sys::start_verified`).
pub type Starter<'a> = &'a dyn Fn(&Path, &[u8; 32], &[&str]) -> Result<(), StartError>;

/// Geladenes und geprüftes Setup mit gehaltener Sperre.
#[derive(Debug)]
pub struct Prepared {
    path: PathBuf,
    sha256: [u8; 32],
    version: Version,
    lock: Lock,
}

impl Prepared {
    pub(crate) fn new(path: PathBuf, sha256: [u8; 32], version: Version, lock: Lock) -> Prepared {
        Prepared {
            path,
            sha256,
            version,
            lock,
        }
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Startet das Setup. Danach muss die App sich beenden (das Setup wartet darauf).
    pub fn start(self, hidden: bool) -> Result<(), StartError> {
        if !supported() {
            return Err(StartError::Failed);
        }
        self.start_with(hidden, &daedyx_sys::start_verified)
    }

    /// Wie [`Prepared::start`] mit eigenem Starter (Tests).
    pub fn start_with(self, hidden: bool, starter: Starter<'_>) -> Result<(), StartError> {
        let result = starter(&self.path, &self.sha256, &setup_args(hidden));
        if result.is_err() {
            let _ = std::fs::remove_file(&self.path);
        }
        // Die Sperre fällt erst jetzt, nach dem Start (oder nach dem Aufräumen).
        drop(self.lock);
        result
    }
}
