use super::*;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

/// Wegwerf-Ordner unter `%TEMP%` (bzw. `TMPDIR`), räumt sich am Testende selbst weg.
struct Dir(PathBuf);

impl Dir {
    fn new(name: &str) -> Dir {
        let base = std::env::var_os("TEMP")
            .or_else(|| std::env::var_os("TMPDIR"))
            .map(PathBuf::from)
            .unwrap();
        let dir = base.join(format!("daedyx-sys-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Dir(dir)
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn sha(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

#[test]
fn a_missing_setup_is_not_started() {
    let dir = Dir::new("fehlt");
    let end = start_verified(&dir.0.join("gibt-es-nicht-setup.exe"), &[0; 32], &[]);
    assert_eq!(end, Err(StartError::Missing));
}

#[test]
fn a_setup_that_changed_after_the_check_is_not_started() {
    let dir = Dir::new("geaendert");
    let file = dir.0.join("app-0.0.2-setup.exe");
    std::fs::write(&file, b"geprueft").unwrap();
    let checked = sha(b"geprueft");
    std::fs::write(&file, b"getauscht").unwrap();
    assert_eq!(
        start_verified(&file, &checked, &[]),
        Err(StartError::Changed)
    );
}

#[test]
fn a_held_setup_cannot_be_written_renamed_or_deleted() {
    let dir = Dir::new("gehalten");
    let file = dir.0.join("app-0.0.2-setup.exe");
    std::fs::write(&file, b"geprueft").unwrap();
    let held = Held::open(&file, &sha(b"geprueft")).unwrap();
    if cfg!(windows) {
        assert!(std::fs::OpenOptions::new().write(true).open(&file).is_err());
        assert!(std::fs::rename(&file, dir.0.join("anders.exe")).is_err());
        assert!(std::fs::remove_file(&file).is_err());
    }
    // Lesen bleibt erlaubt (das Betriebssystem muss das Programm laden können).
    assert_eq!(std::fs::read(&file).unwrap(), b"geprueft");
    drop(held);
    assert!(std::fs::remove_file(&file).is_ok());
}

#[test]
fn a_file_that_is_not_a_program_fails_to_start_and_is_reported() {
    let dir = Dir::new("kein-programm");
    let file = dir.0.join("app-0.0.2-setup.exe");
    std::fs::write(&file, b"kein Programm").unwrap();
    assert_eq!(
        start_verified(&file, &sha(b"kein Programm"), &["/P"]),
        Err(StartError::Failed)
    );
}

#[cfg(windows)]
#[test]
fn a_checked_program_is_started_without_waiting() {
    // Die Kommandozeile von Windows als „Setup“: geprüft, gestartet, sofort beendet.
    let cmd = PathBuf::from(std::env::var_os("ComSpec").unwrap());
    let bytes = std::fs::read(&cmd).unwrap();
    assert_eq!(
        start_verified(&cmd, &sha(&bytes), &["/C", "exit", "0"]),
        Ok(())
    );
}
