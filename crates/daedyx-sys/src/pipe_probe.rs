//! Prüfhilfe für die Tests der Pipe-Härtung (SICHERHEIT-AUSNAHMEN A-027): als fremdes (anonymes) Konto handeln.
//! Nur in Tests übersetzt; `unsafe` je Funktion freigegeben (Review H-P3).
use super::ffi::check;
use std::io;
use std::os::windows::io::RawHandle;

/// Pseudo-Handle des eigenen Threads (Wert von `GetCurrentThread`, dokumentiert als `(HANDLE)-2`).
const CURRENT_THREAD: RawHandle = std::ptr::without_provenance_mut(!1usize);

#[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
#[link(name = "advapi32")]
unsafe extern "system" {
    fn ImpersonateAnonymousToken(thread: RawHandle) -> i32;
    fn RevertToSelf() -> i32;
}

/// Beendet beim Verlassen die Identitätsübernahme dieses Threads, auch wenn ein Test mittendrin scheitert.
struct Revert;

impl Drop for Revert {
    #[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
    fn drop(&mut self) {
        // SAFETY: einfacher Aufruf ohne Zeiger; setzt nur die Identität dieses Threads zurück.
        let _ = unsafe { RevertToSelf() };
    }
}

/// Führt `f` im eigenen Thread als anonymes Konto aus (ein fremder Benutzer, nur für Tests).
#[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
pub fn as_anonymous<T>(f: impl FnOnce() -> T) -> io::Result<T> {
    // SAFETY: Pseudo-Handle des eigenen Threads; `Revert` setzt die Identität danach in jedem Fall zurück.
    check(unsafe { ImpersonateAnonymousToken(CURRENT_THREAD) })?;
    let _revert = Revert;
    Ok(f())
}
