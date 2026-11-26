//! Ablauf ohne Netz (plan-update-bibliothek 10): Suche, Zustand in `updates.json`, Rücksprung, automatische Suche,
//! Aufräumen beim Start. Der Download steht in `download_tests.rs`.
use crate::Updater;
use crate::status::Status;
use crate::test_support::{
    Fake, NOW, Release, SETUP_URL, TestDir, backend, policy, saved, set_state, wait,
};
use crate::worker::Msg;
use crate::worker_run::Run;
use daedyx_update::fetch::TransportError;
use daedyx_update::reason::BlockKind;
use daedyx_update::state::UpdateState;
use daedyx_update::version::Version;
use std::collections::BTreeMap;
use std::time::Duration;

#[test]
fn ohne_netz_bleibt_es_still_und_nichts_wird_vermerkt() {
    let dir = TestDir::new("offline");
    set_state(&dir, false, None);
    let url = policy().latest_asset_url("entry.json");
    let (b, _seen) = backend(
        BTreeMap::from([(url, Fake::Fail(TransportError::Offline))]),
        Default::default(),
    );
    let mut u = Updater::start(dir.config("0.2.0"), b, None);
    u.check_now();
    assert!(wait(&mut u, |u| matches!(u.status(), Status::Blocked(_))));
    assert_eq!(u.status(), &Status::Blocked(BlockKind::Offline));
    let s = saved(&dir);
    assert_eq!(s.last_check, Some(NOW));
    assert_eq!(
        s.highest_seen(),
        None,
        "nur geprüfte Versionen zählen als gesehen"
    );
}

#[test]
fn gleiche_version_ist_aktuell() {
    let dir = TestDir::new("aktuell");
    set_state(&dir, false, None);
    let r = Release::new("0.2.1");
    let (b, _seen) = backend(r.routes(), r.trusted.clone());
    let mut u = Updater::start(dir.config("0.2.1"), b, None);
    u.check_now();
    assert!(wait(&mut u, |u| u.status() == &Status::UpToDate));
    assert_eq!(saved(&dir).highest_seen(), Some(Version::new(0, 2, 1)));
}

#[test]
fn zuruecksprung_der_quelle_wird_gemeldet() {
    let dir = TestDir::new("zurueck");
    let mut s = UpdateState::default();
    s.auto_check = false;
    s.note_seen(&Version::new(0, 3, 0));
    s.save(&dir.data()).unwrap();
    let r = Release::new("0.2.1");
    let (b, _seen) = backend(r.routes(), r.trusted.clone());
    let mut u = Updater::start(dir.config("0.2.0"), b, None);
    u.check_now();
    assert!(wait(&mut u, |u| matches!(u.status(), Status::Blocked(_))));
    assert_eq!(u.status(), &Status::Blocked(BlockKind::SourceWentBack));
}

#[test]
fn automatische_suche_nur_wenn_an_und_faellig() {
    let r = Release::new("0.2.1");
    // Fällig: nie gesucht.
    let dir = TestDir::new("auto-faellig");
    let (b, seen) = backend(r.routes(), r.trusted.clone());
    let mut u = Updater::start(dir.config("0.2.0"), b, None);
    assert!(
        wait(&mut u, |u| matches!(u.status(), Status::Available { .. })),
        "ohne Klick gesucht"
    );
    assert!(
        seen.try_iter().all(|url| url != SETUP_URL),
        "aber nichts geladen"
    );
    // Nicht fällig (vor einer Stunde gesucht) und Schalter aus: keine Anfrage.
    for (name, auto, last) in [
        ("auto-frisch", true, Some(NOW - 3600)),
        ("auto-aus", false, None),
    ] {
        let dir = TestDir::new(name);
        set_state(&dir, auto, last);
        let (b, seen) = backend(r.routes(), r.trusted.clone());
        let mut u = Updater::start(dir.config("0.2.0"), b, None);
        std::thread::sleep(Duration::from_millis(300));
        u.poll();
        assert_eq!(u.status(), &Status::Idle, "{name}");
        assert_eq!(seen.try_iter().count(), 0, "{name}");
        assert_eq!(u.auto_check(), auto);
    }
}

#[test]
fn schalter_wird_in_updates_json_gespeichert() {
    let dir = TestDir::new("schalter");
    set_state(&dir, true, Some(NOW));
    let (b, _seen) = backend(BTreeMap::new(), Default::default());
    let mut u = Updater::start(dir.config("0.2.0"), b, None);
    u.set_auto_check(false);
    assert!(wait(&mut u, |_| UpdateState::load(&dir.data())
        .is_ok_and(|s| !s.auto_check)));
    assert!(!u.auto_check());
}

#[test]
fn alte_reste_im_update_ordner_werden_beim_start_geraeumt() {
    let dir = TestDir::new("reste");
    set_state(&dir, false, None);
    let updates = dir.updates();
    std::fs::create_dir_all(&updates).unwrap();
    for f in ["probe-0.1.0-setup.exe", "probe-0.2.1-setup.exe.part"] {
        std::fs::write(updates.join(f), b"alt").unwrap();
    }
    std::fs::write(updates.join("fremd.txt"), b"bleibt").unwrap();
    let (b, _seen) = backend(BTreeMap::new(), Default::default());
    let mut u = Updater::start(dir.config("0.2.0"), b, None);
    assert!(wait(&mut u, |_| std::fs::read_dir(&updates)
        .unwrap()
        .count()
        == 1));
    assert!(updates.join("fremd.txt").exists());
}

#[test]
fn nach_dem_bereit_kommt_keine_statusmeldung_mehr() {
    // Review U2-R N-U2-3: Eine späte Statusmeldung darf das StartFailed der App nicht überschreiben.
    let dir = TestDir::new("reihenfolge");
    set_state(&dir, false, None);
    let r = Release::new("0.2.1");
    let (b, _seen) = backend(r.routes(), r.trusted.clone());
    let config = dir.config("0.2.0");
    let (out, rx) = std::sync::mpsc::channel();
    let mut run = Run {
        config: &config,
        backend: b,
        out: &out,
        ctx: None,
        state: UpdateState::default(),
        offer: None,
    };
    run.check();
    run.download();
    let msgs: Vec<Msg> = rx.try_iter().collect();
    assert!(
        matches!(msgs.last(), Some(Msg::Ready(_))),
        "Bereit ist die letzte Meldung"
    );
    let mut u = Updater::disabled();
    u.feed(msgs);
    assert_eq!(
        u.status(),
        &Status::Ready {
            version: "0.2.1".into()
        }
    );
    assert!(u.take_ready().is_some());
    u.set_status(Status::StartFailed);
    u.feed(Vec::new());
    assert_eq!(u.status(), &Status::StartFailed);
}

#[test]
fn ohne_updates_json_gilt_der_standard_der_app() {
    // Entscheid F-U2-1: Die Vorschau sucht nicht von selbst, bis jemand den Schalter einschaltet.
    let r = Release::new("0.2.1");
    let dir = TestDir::new("standard-aus");
    let mut config = dir.config("0.2.0");
    config.auto_check_default = false;
    let (b, seen) = backend(r.routes(), r.trusted.clone());
    let mut u = Updater::start(config, b, None);
    std::thread::sleep(Duration::from_millis(300));
    u.poll();
    assert!(!u.auto_check());
    assert_eq!(u.status(), &Status::Idle);
    assert_eq!(seen.try_iter().count(), 0, "keine Anfrage");
    // Gespeichertes gewinnt: eingeschaltet ist eingeschaltet.
    let dir = TestDir::new("standard-gespeichert");
    set_state(&dir, true, None);
    let mut config = dir.config("0.2.0");
    config.auto_check_default = false;
    let (b, _seen) = backend(r.routes(), r.trusted.clone());
    let mut u = Updater::start(config, b, None);
    assert!(u.auto_check());
    assert!(wait(&mut u, |u| matches!(
        u.status(),
        Status::Available { .. }
    )));
}
