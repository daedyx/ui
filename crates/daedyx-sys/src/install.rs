//! Setup eines geprüften Updates starten (plan-update-bibliothek 6, Auftrag U2). Die Datei wird zum Prüfen so
//! geöffnet, dass andere sie nur noch lesen dürfen (kein Schreiben, Umbenennen oder Löschen), und der Griff bleibt
//! offen, bis der Start zurückkommt. So kann niemand die Datei zwischen Prüfung und Start tauschen. Argumente gibt
//! der Aufrufer fest vor; nichts davon kommt aus dem Netz. Auf das Ende wird nicht gewartet: Das Setup beendet die
//! App über `--quit-running` und startet sie danach neu.
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::Path;
use std::process::{Command, Stdio};

/// Warum das Setup nicht gestartet wurde.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartError {
    /// Die Datei fehlt.
    Missing,
    /// Inhalt passt nicht mehr zur geprüften Prüfsumme (nach der Prüfung getauscht oder beschädigt).
    Changed,
    /// Öffnen, Lesen oder Starten scheiterte.
    Failed,
}

/// Geprüfte, gegen Schreiben gesperrte Datei.
#[derive(Debug)]
pub(crate) struct Held {
    _file: File,
}

#[cfg(windows)]
fn open_read_only_shared(path: &Path) -> std::io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_SHARE_READ: u32 = 0x1;
    std::fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(path)
}

#[cfg(not(windows))]
fn open_read_only_shared(path: &Path) -> std::io::Result<File> {
    File::open(path)
}

fn digest(file: &mut File) -> Option<[u8; 32]> {
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        match file.read(&mut buf) {
            Ok(0) => return Some(hasher.finalize().into()),
            Ok(n) => hasher.update(buf.get(..n).unwrap_or_default()),
            Err(e) if e.kind() == ErrorKind::Interrupted => {}
            Err(_) => return None,
        }
    }
}

/// SHA-256 einer Datei (`None`, wenn sie sich nicht lesen lässt).
pub fn file_sha256(path: &Path) -> Option<[u8; 32]> {
    digest(&mut File::open(path).ok()?)
}

impl Held {
    /// Öffnet `path` gesperrt und prüft die Prüfsumme über den gesperrten Griff.
    pub(crate) fn open(path: &Path, sha256: &[u8; 32]) -> Result<Held, StartError> {
        let mut file = open_read_only_shared(path).map_err(|e| match e.kind() {
            ErrorKind::NotFound => StartError::Missing,
            _ => StartError::Failed,
        })?;
        let got = digest(&mut file).ok_or(StartError::Failed)?;
        if &got == sha256 {
            Ok(Held { _file: file })
        } else {
            Err(StartError::Changed)
        }
    }
}

/// Startet das Setup `exe`, wenn sein Inhalt noch `sha256` hat, mit den festen Argumenten `args`.
#[allow(clippy::disallowed_methods, reason = "SICHERHEIT-AUSNAHMEN A-034")]
pub fn start_verified(exe: &Path, sha256: &[u8; 32], args: &[&str]) -> Result<(), StartError> {
    let held = Held::open(exe, sha256)?;
    let started = Command::new(exe)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    drop(held);
    // Nicht warten: Der Prozess läuft weiter, wenn `Child` hier fallen gelassen wird.
    started.map(|_| ()).map_err(|_| StartError::Failed)
}

#[cfg(test)]
#[path = "install_tests.rs"]
mod tests;
