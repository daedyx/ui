//! Setup-Start: feste Argumente, versteckter Neustart, Aufräumen bei Fehlschlag, Sperre bis zum Start.
use crate::install::{ARGS, Prepared, setup_args};
use crate::test_support::{SETUP, TestDir, sha};
use daedyx_sys::StartError;
use daedyx_update::lock::{Lock, LockError, shared_lock_dir};
use daedyx_update::version::Version;

#[test]
fn argumente_stehen_fest() {
    assert_eq!(ARGS, ["/P", "/UPDATE", "/R"]);
    assert_eq!(setup_args(false), ["/P", "/UPDATE", "/R"]);
    assert_eq!(
        setup_args(true),
        ["/P", "/UPDATE", "/R", "/ARGS", "--hidden"]
    );
}

fn prepared(dir: &TestDir) -> Prepared {
    let file = dir.0.join("probe-0.2.1-setup.exe");
    std::fs::write(&file, SETUP).unwrap();
    let lock = Lock::acquire(&shared_lock_dir(&dir.0), "probe").unwrap();
    Prepared::new(file, sha(SETUP), Version::new(0, 2, 1), lock)
}

#[test]
fn versteckt_gestartete_app_startet_versteckt_neu() {
    let dir = TestDir::new("versteckt");
    let (tx, rx) = std::sync::mpsc::channel();
    let starter = move |_: &std::path::Path, _: &[u8; 32], a: &[&str]| {
        let _ = tx.send(a.join(" "));
        Ok(())
    };
    prepared(&dir).start_with(true, &starter).unwrap();
    assert_eq!(rx.recv().unwrap(), "/P /UPDATE /R /ARGS --hidden");
}

#[test]
fn scheitert_der_start_wird_die_datei_geloescht_und_die_sperre_frei() {
    let dir = TestDir::new("startfehler");
    let p = prepared(&dir);
    let file = p.path().to_path_buf();
    assert_eq!(
        Lock::acquire(&shared_lock_dir(&dir.0), "probe").unwrap_err(),
        LockError::Busy
    );
    let fail = |_: &std::path::Path, _: &[u8; 32], _: &[&str]| Err(StartError::Changed);
    assert_eq!(p.start_with(false, &fail), Err(StartError::Changed));
    assert!(!file.exists());
    assert!(Lock::acquire(&shared_lock_dir(&dir.0), "probe").is_ok());
}

#[cfg(windows)]
#[test]
fn echter_start_prueft_die_pruefsumme_vorher() {
    // Kein Programm, das gestartet würde: Die Prüfsumme passt nicht, also kein Start und die Datei ist weg.
    let dir = TestDir::new("echt");
    let file = dir.0.join("probe-0.2.1-setup.exe");
    std::fs::write(&file, b"getauscht").unwrap();
    let lock = Lock::acquire(&shared_lock_dir(&dir.0), "probe").unwrap();
    let p = Prepared::new(file.clone(), sha(SETUP), Version::new(0, 2, 1), lock);
    assert_eq!(p.start(false), Err(StartError::Changed));
    assert!(!file.exists());
}
