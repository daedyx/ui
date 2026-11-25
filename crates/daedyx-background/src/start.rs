//! Hilfen für `main`: Tokio-Laufzeit für die Pipe, Aufräumen (`--cleanup`), Toast-Probe (`--probe-toast`),
//! Autostart des Systems. Rückgabe als Programm-Codes ohne Text. Die Kennungen gibt die App herein ([`AppIds`]).
use crate::aumid;
use crate::autostart::{AutostartStore, FakeStore};
use crate::ids::{self, AppIds};
use crate::notify::{Notifier, WinToast};
use std::process::ExitCode;
use std::time::Duration;

pub mod code {
    pub const RUNTIME: u8 = 2;
    pub const SECOND_INSTANCE_FAILED: u8 = 3;
    pub const TOAST_SHOW_FAILED: u8 = 4;
    pub const TOAST_NOT_IN_HISTORY: u8 = 5;
    pub const GUI: u8 = 6;
    pub const CLEANUP_INCOMPLETE: u8 = 7;
    pub const PIPE: u8 = 8;
}

/// Eine kleine Laufzeit nur für die Pipe der Einzelinstanz.
pub fn runtime() -> std::io::Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .thread_name("daedyx-pipe")
        .enable_all()
        .build()
}

/// Autostart über die Registry (Windows), sonst ein Ersatz, der immer scheitert (Schalter zeigt dann den Fehler).
pub fn autostart_store(app: &AppIds) -> Box<dyn AutostartStore> {
    #[cfg(windows)]
    if let Ok(s) = crate::autostart::RegistryStore::for_current_exe(app.run_value) {
        return Box::new(s);
    }
    let _ = app;
    Box::new(FakeStore {
        on: false,
        fail: true,
    })
}

/// Entfernt Autostart-Wert, App-Kennung und die eigenen Benachrichtigungen im Info-Center.
pub fn cleanup(app: &AppIds) -> ExitCode {
    let mut ok = autostart_store(app).set(false).is_ok();
    let _ = aumid::clear_history(app.aumid);
    ok &= aumid::unregister(app.aumid).is_ok();
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(code::CLEANUP_INCOMPLETE)
    }
}

/// Zeigt eine Benachrichtigung mit erfundenem Inhalt und prüft, ob Windows sie ins Info-Center übernommen hat.
/// Anzeigename, Titel und Text kommen aus der Sprachdatei der App.
pub fn probe_toast(app: &AppIds, display_name: &str, title: &str, body: &str) -> ExitCode {
    let _ = aumid::register(app.aumid, display_name);
    let before = aumid::history_count(app.aumid).unwrap_or(0);
    let toast = WinToast {
        aumid: app.aumid,
        on_click: None,
    };
    if toast.notify(title, body).is_err() {
        return ExitCode::from(code::TOAST_SHOW_FAILED);
    }
    std::thread::sleep(Duration::from_millis(1500));
    match aumid::history_count(app.aumid) {
        Some(after) if after > before => ExitCode::SUCCESS,
        _ => ExitCode::from(code::TOAST_NOT_IN_HISTORY),
    }
}

/// Pipe-Name dieses Benutzers.
pub fn pipe(app: &AppIds) -> String {
    ids::pipe_name(app.pipe_prefix, &ids::user_scope())
}
