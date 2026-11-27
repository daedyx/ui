//! Download erst nach Klick (plan-update-bibliothek 4 und 6): geprüftes Setup im eigenen Ordner, Sperre bis zum Start,
//! getauschtes oder nicht nachgewiesenes Setup wird abgelehnt und nichts bleibt liegen, belegte Sperre.
use crate::Updater;
use crate::status::Status;
use crate::test_support::{
    Release, SETUP, SETUP_URL, TestDir, backend, saved, set_state, sha, wait,
};
use daedyx_update::lock::{Lock, LockError};
use daedyx_update::reason::BlockKind;
use daedyx_update::version::Version;

#[test]
fn neuere_version_wird_angeboten_aber_erst_nach_klick_geladen() {
    let dir = TestDir::new("angebot");
    set_state(&dir, false, None);
    let r = Release::new("0.2.1");
    let (b, seen) = backend(r.routes(), r.trusted.clone());
    let mut u = Updater::start(dir.config("0.2.0"), b, None);
    u.check_now();
    assert!(wait(&mut u, |u| matches!(
        u.status(),
        Status::Available { .. }
    )));
    let Status::Available {
        version,
        notes,
        important,
    } = u.status().clone()
    else {
        panic!("kein Angebot")
    };
    assert_eq!((version.as_str(), important), ("0.2.1", false));
    assert!(notes.starts_with("Neu: erfundene Suche."));
    assert!(
        seen.try_iter().all(|url| url != SETUP_URL),
        "kein Download ohne Klick"
    );
    assert_eq!(saved(&dir).highest_seen(), Some(Version::new(0, 2, 1)));
    assert!(u.take_ready().is_none());

    u.update_now();
    assert!(wait(&mut u, |u| matches!(u.status(), Status::Ready { .. })));
    let ready = u.take_ready().unwrap();
    assert_eq!(std::fs::read(ready.path()).unwrap(), SETUP);
    assert!(
        ready.path().starts_with(dir.updates()),
        "lokal, nicht im Datenordner (Roaming) und nicht in %TEMP%"
    );
    let lock_dir = daedyx_update::lock::shared_lock_dir(&dir.0);
    assert_eq!(
        Lock::acquire(&lock_dir, "probe").unwrap_err(),
        LockError::Busy,
        "Sperre bis zum Start"
    );
    let (tx, rx) = std::sync::mpsc::channel();
    let starter = move |p: &std::path::Path, s: &[u8; 32], a: &[&str]| {
        let _ = tx.send((
            p.to_path_buf(),
            *s,
            a.iter().map(|x| (*x).to_owned()).collect::<Vec<_>>(),
        ));
        Ok(())
    };
    ready.start_with(false, &starter).unwrap();
    let (_, s, args) = rx.recv().unwrap();
    assert_eq!(s, sha(SETUP));
    assert_eq!(args, ["/P", "/UPDATE", "/R"]);
    assert!(
        Lock::acquire(&lock_dir, "probe").is_ok(),
        "Sperre nach dem Start frei"
    );
}

fn offered(name: &str, release: &Release) -> (TestDir, Updater) {
    let dir = TestDir::new(name);
    set_state(&dir, false, None);
    let (b, _seen) = backend(release.routes(), release.trusted.clone());
    let mut u = Updater::start(dir.config("0.2.0"), b, None);
    u.check_now();
    assert!(wait(&mut u, |u| matches!(
        u.status(),
        Status::Available { .. }
    )));
    (dir, u)
}

fn no_setup_left(dir: &TestDir) -> bool {
    std::fs::read_dir(dir.updates())
        .map(|l| l.filter_map(Result::ok).count() == 0)
        .unwrap_or(true)
}

#[test]
fn getauschtes_setup_wird_abgelehnt_und_nichts_bleibt_liegen() {
    let mut r = Release::new("0.2.1");
    r.served = b"getauschtes Setup 0.21".to_vec();
    let (dir, mut u) = offered("getauscht", &r);
    u.update_now();
    assert!(wait(&mut u, |u| matches!(u.status(), Status::Blocked(_))));
    assert_eq!(u.status(), &Status::Blocked(BlockKind::NotTrusted));
    assert!(u.take_ready().is_none() && no_setup_left(&dir));
}

#[test]
fn setup_ohne_nachweis_wird_abgelehnt() {
    let mut r = Release::new("0.2.1");
    let only_entry = std::collections::BTreeSet::from([sha(&r.entry)]);
    // Eintrag geprüft, Setup-Subjekt fehlt im Bündel: schon die Suche lehnt ab.
    r.trusted = only_entry;
    let dir = TestDir::new("ohne-nachweis");
    set_state(&dir, false, None);
    let (b, _seen) = backend(r.routes(), r.trusted.clone());
    let mut u = Updater::start(dir.config("0.2.0"), b, None);
    u.check_now();
    assert!(wait(&mut u, |u| matches!(u.status(), Status::Blocked(_))));
    assert_eq!(u.status(), &Status::Blocked(BlockKind::NotTrusted));
    assert!(no_setup_left(&dir));
}

/// Review M-PS-1: Wer das Ergebnis der Suche sieht, findet die Sperre frei. Vorher gab der Ablauf sie erst nach dem
/// letzten Status frei, und ein Test oder der Launcher bekam gelegentlich „läuft bereits“ (hier 2 von 16 Läufen).
#[test]
fn nach_der_suche_ist_die_sperre_schon_frei() {
    let r = Release::new("0.2.1");
    for n in 0..20 {
        let (dir, _u) = offered(&format!("frei-{n}"), &r);
        let lock = Lock::acquire(&daedyx_update::lock::shared_lock_dir(&dir.0), "probe");
        assert!(lock.is_ok(), "Lauf {n}");
    }
}

#[test]
fn belegte_sperre_meldet_update_laeuft_bereits() {
    let r = Release::new("0.2.1");
    let (dir, mut u) = offered("belegt", &r);
    let held = Lock::acquire(&daedyx_update::lock::shared_lock_dir(&dir.0), "probe").unwrap();
    u.update_now();
    assert!(wait(&mut u, |u| u.status() == &Status::Busy));
    assert!(no_setup_left(&dir));
    drop(held);
}

#[test]
fn erneut_versuchen_nach_gescheitertem_start_laedt_neu() {
    // Review U2-R N-U2-2: Der Knopf erscheint bei StartFailed und muss dann wirklich laden.
    let r = Release::new("0.2.1");
    let (dir, mut u) = offered("start-fehler", &r);
    u.update_now();
    assert!(wait(&mut u, |u| matches!(u.status(), Status::Ready { .. })));
    let first = u.take_ready().unwrap();
    let starter =
        |_: &std::path::Path, _: &[u8; 32], _: &[&str]| Err(daedyx_sys::StartError::Failed);
    assert!(first.start_with(false, &starter).is_err());
    u.set_status(Status::StartFailed);
    u.update_now();
    assert!(matches!(u.status(), Status::Downloading { .. }));
    assert!(wait(&mut u, |u| matches!(u.status(), Status::Ready { .. })));
    assert!(
        u.take_ready()
            .is_some_and(|p| p.path().starts_with(dir.updates()))
    );
}
