//! Was die App dem Updater mitgibt: ihre feste Identität (`UpdatePolicy` als Konstante im Programm), die laufende
//! Version und ihre Ordner. Dazu die austauschbaren Teile ([`Backend`]): Netz, Nachweis-Prüfer, freier Platz und Uhr,
//! damit die Tests ohne Netz und ohne echten Nachweis laufen.
use daedyx_update::attest::{AttestError, GithubAttestation, Provenance};
use daedyx_update::fetch::Transport;
use daedyx_update::policy::UpdatePolicy;
use daedyx_update::version::Version;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Größte zufällige Verzögerung der automatischen Suche nach dem Start (plan-update-bibliothek 4).
pub const MAX_AUTO_DELAY: Duration = Duration::from_secs(10 * 60);

/// Feste Angaben der App.
#[derive(Debug, Clone)]
pub struct Config {
    pub policy: UpdatePolicy,
    /// Laufende Version (`CARGO_PKG_VERSION` der App).
    pub installed: Version,
    /// Datenordner der App: dort liegt `updates.json`.
    pub data_dir: PathBuf,
    /// Eigener Ordner für geladene Setups (`<Datenordner>\updates`, nicht `%TEMP%`).
    pub update_dir: PathBuf,
    /// Gemeinsamer Sperrordner mit dem Launcher (`daedyx_update::lock::shared_lock_dir`).
    pub lock_dir: PathBuf,
    /// Verzögerung der automatischen Suche nach dem Start (App: zufällig bis [`MAX_AUTO_DELAY`], Tests: 0).
    pub auto_delay: Duration,
}

impl Config {
    /// Ordner aus dem Datenordner der App und dem lokalen Datenordner des Benutzers (`%LOCALAPPDATA%`).
    pub fn new(
        policy: UpdatePolicy,
        installed: Version,
        data_dir: &Path,
        local_data: &Path,
    ) -> Config {
        Config {
            policy,
            installed,
            data_dir: data_dir.to_path_buf(),
            update_dir: data_dir.join("updates"),
            lock_dir: daedyx_update::lock::shared_lock_dir(local_data),
            auto_delay: random_delay(MAX_AUTO_DELAY),
        }
    }
}

/// Zufällige Dauer bis `max` (ohne eigene Abhängigkeit: `RandomState` ist je Aufruf neu gesät).
pub fn random_delay(max: Duration) -> Duration {
    use std::hash::{BuildHasher, Hasher};
    let random = std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish();
    let secs = max.as_secs().max(1);
    Duration::from_secs(random.checked_rem(secs).unwrap_or(0))
}

/// Prüfer für den Nachweis einer Quelle.
pub type MakeProvenance =
    Box<dyn Fn(&UpdatePolicy) -> Result<Box<dyn Provenance>, AttestError> + Send>;

/// Freier Platz eines Ordners, falls messbar.
pub type FreeSpace = Box<dyn Fn(&Path) -> Option<u64> + Send>;

/// Austauschbare Teile des Ablaufs.
pub struct Backend {
    pub transport: Box<dyn Transport + Send>,
    pub provenance: MakeProvenance,
    /// Freier Platz im Update-Ordner, falls messbar (`daedyx_sys::available_space`).
    pub free_space: FreeSpace,
    /// Jetzt als Unix-Sekunden.
    pub now: Box<dyn Fn() -> i64 + Send>,
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

impl Backend {
    /// Echtes Netz, echter GitHub-Nachweis mit der eingebauten Wurzel, Platz über `daedyx-sys`, Systemuhr.
    pub fn real() -> Backend {
        Backend {
            transport: Box::new(crate::net::UreqTransport::new()),
            provenance: Box::new(|p| {
                Ok(Box::new(GithubAttestation::new(p)?) as Box<dyn Provenance>)
            }),
            free_space: Box::new(daedyx_sys::available_space),
            now: Box::new(unix_now),
        }
    }
}
