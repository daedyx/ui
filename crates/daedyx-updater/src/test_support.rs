//! Testhilfen ohne Netz: gefälschter Server (meldet jede Anfrage über einen Kanal), Attrappe für den Nachweis (nimmt
//! nur bekannte Prüfsummen an), erfundenes Release 0.2.1 einer erfundenen App „probe“, Wegwerf-Ordner.
use crate::config::{Backend, Config};
use daedyx_update::attest::{AttestError, AttestKind, Provenance};
use daedyx_update::fetch::{HttpsUrl, Reply, Transport, TransportError};
use daedyx_update::policy::{Platform, UpdatePolicy};
use daedyx_update::version::Version;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

pub const SETUP_URL: &str =
    "https://github.com/daedyx/tasks/releases/download/v0.2.1/probe-0.2.1-setup.exe";
pub const SETUP: &[u8] = b"erfundenes Setup 0.2.1";

pub fn sha(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

pub fn policy() -> UpdatePolicy {
    UpdatePolicy::new("daedyx", "tasks", 1001, 2002, "probe", Platform::WindowsX64).unwrap()
}

pub fn entry_json(version: &str, setup: &[u8]) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schema": 1, "app": "probe", "version": version, "date": "2026-12-01",
        "notes": "Neu: erfundene Suche.\nBehoben: erfundener Fehler.",
        "packages": {"windows-x64": {
            "url": SETUP_URL.replace("0.2.1", version), "sha256": hex::encode(sha(setup)),
            "size": setup.len(), "kind": "nsis-silent"}}
    }))
    .unwrap()
}

/// Antwort je Adresse; fehlende Adressen ergeben 404.
#[derive(Clone)]
pub enum Fake {
    Body(Vec<u8>),
    Fail(TransportError),
}

pub struct FakeServer {
    pub routes: BTreeMap<String, Fake>,
    pub seen: Sender<String>,
}

impl Transport for FakeServer {
    fn get(&mut self, url: &HttpsUrl) -> Result<Reply<'_>, TransportError> {
        let _ = self.seen.send(url.as_str().to_owned());
        match self.routes.get(url.as_str()).cloned() {
            Some(Fake::Body(b)) => Ok(Reply::Body {
                length: Some(b.len() as u64),
                reader: Box::new(std::io::Cursor::new(b)),
            }),
            Some(Fake::Fail(e)) => Err(e),
            None => Ok(Reply::Status(404)),
        }
    }
}

/// Nimmt nur diese Prüfsummen an (Bündel-Inhalt egal).
pub struct FakeProvenance(pub BTreeSet<[u8; 32]>);

impl Provenance for FakeProvenance {
    fn verify(&self, sha256: &[u8; 32], _bundle: &[u8]) -> Result<(), AttestError> {
        if self.0.contains(sha256) {
            Ok(())
        } else {
            Err(AttestError {
                kind: AttestKind::Invalid,
                detail: "fake".into(),
            })
        }
    }
}

/// Release `version` mit `setup` (der Server liefert `served` als Setup).
pub struct Release {
    pub entry: Vec<u8>,
    pub served: Vec<u8>,
    pub trusted: BTreeSet<[u8; 32]>,
}

impl Release {
    pub fn new(version: &str) -> Release {
        let entry = entry_json(version, SETUP);
        let trusted = BTreeSet::from([sha(&entry), sha(SETUP)]);
        Release {
            entry,
            served: SETUP.to_vec(),
            trusted,
        }
    }

    pub fn routes(&self) -> BTreeMap<String, Fake> {
        let p = policy();
        let version = serde_json::from_slice::<serde_json::Value>(&self.entry).unwrap()["version"]
            .as_str()
            .unwrap()
            .to_owned();
        BTreeMap::from([
            (
                p.latest_asset_url("entry.json"),
                Fake::Body(self.entry.clone()),
            ),
            (
                p.latest_asset_url("provenance.sigstore.json"),
                Fake::Body(b"{}".to_vec()),
            ),
            (
                SETUP_URL.replace("0.2.1", &version),
                Fake::Body(self.served.clone()),
            ),
        ])
    }
}

/// Wegwerf-Ordner, räumt sich am Testende selbst weg.
pub struct TestDir(pub PathBuf);

impl TestDir {
    pub fn new(name: &str) -> TestDir {
        let base = std::env::var_os("TEMP")
            .or_else(|| std::env::var_os("TMPDIR"))
            .map(PathBuf::from)
            .unwrap();
        let dir = base.join(format!("daedyx-updater-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        TestDir(dir)
    }

    pub fn config(&self, installed: &str) -> Config {
        let mut c = Config::new(
            policy(),
            Version::parse(installed).unwrap(),
            &self.data(),
            &self.0,
        );
        c.auto_delay = Duration::ZERO;
        c
    }

    /// Datenordner (wie `%APPDATA%\<Kennung>`) und erwarteter lokaler Ordner der Setups (`%LOCALAPPDATA%\…\updates`).
    pub fn data(&self) -> PathBuf {
        self.0.join("roaming").join("daten")
    }
    pub fn updates(&self) -> PathBuf {
        self.0.join("daten").join("updates")
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub const NOW: i64 = 1_800_000_000;

pub fn backend(
    routes: BTreeMap<String, Fake>,
    trusted: BTreeSet<[u8; 32]>,
) -> (Backend, Receiver<String>) {
    let (seen, rx) = channel();
    let backend = Backend {
        transport: Box::new(FakeServer { routes, seen }),
        provenance: Box::new(move |_| {
            Ok(Box::new(FakeProvenance(trusted.clone())) as Box<dyn Provenance>)
        }),
        free_space: Box::new(|_: &Path| Some(u64::MAX / 4)),
        now: Box::new(|| NOW),
    };
    (backend, rx)
}

/// Holt Meldungen ab, bis `done` gilt (höchstens 10 s).
pub fn wait(u: &mut crate::Updater, done: impl Fn(&crate::Updater) -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        u.poll();
        if done(u) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    false
}

/// Zustand in `updates.json` des Test-Datenordners: lesen und setzen.
pub fn saved(dir: &TestDir) -> daedyx_update::state::UpdateState {
    daedyx_update::state::UpdateState::load(&dir.data()).unwrap()
}
pub fn set_state(dir: &TestDir, auto: bool, last: Option<i64>) {
    let mut s = daedyx_update::state::UpdateState::default();
    s.auto_check = auto;
    s.last_check = last;
    s.save(&dir.data()).unwrap();
}
