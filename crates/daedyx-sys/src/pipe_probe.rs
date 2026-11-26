//! Prüfhilfen für die Tests der Pipe-Härtung (SICHERHEIT-AUSNAHMEN A-027): Rechte eines Objekts als Text auslesen
//! und als fremdes (anonymes) Konto handeln. Nur in Tests übersetzt.
#![allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
use super::ffi::{Local, SDDL_REVISION_1, check};
use std::ffi::c_void;
use std::io;
use std::os::windows::io::{AsRawHandle, RawHandle};

const SE_KERNEL_OBJECT: i32 = 6;
const OWNER_AND_DACL: u32 = 0x1 | 0x4;
/// Pseudo-Handle des eigenen Threads (Wert von `GetCurrentThread`, dokumentiert als `(HANDLE)-2`).
const CURRENT_THREAD: RawHandle = std::ptr::without_provenance_mut(!1usize);

#[link(name = "advapi32")]
unsafe extern "system" {
    fn GetSecurityInfo(
        handle: RawHandle,
        kind: i32,
        what: u32,
        owner: *mut *mut c_void,
        group: *mut *mut c_void,
        dacl: *mut *mut c_void,
        sacl: *mut *mut c_void,
        sd: *mut *mut c_void,
    ) -> i32;
    fn ConvertSecurityDescriptorToStringSecurityDescriptorW(
        sd: *mut c_void,
        revision: u32,
        what: u32,
        text: *mut *mut u16,
        len: *mut u32,
    ) -> i32;
    fn ImpersonateAnonymousToken(thread: RawHandle) -> i32;
    fn RevertToSelf() -> i32;
}

/// Besitzer und Zugriffsliste eines Objekts als SDDL-Text.
pub fn security_text(object: &impl AsRawHandle) -> io::Result<String> {
    use std::ptr::null_mut;
    let mut sd = null_mut();
    // SAFETY: Das Handle gehört dem lebenden `object`, `sd` ist ein lokaler Ausgabewert.
    let err = unsafe {
        GetSecurityInfo(
            object.as_raw_handle(),
            SE_KERNEL_OBJECT,
            OWNER_AND_DACL,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            &mut sd,
        )
    };
    if err != 0 {
        return Err(io::Error::from_raw_os_error(err));
    }
    let sd = Local(sd);
    let mut text = std::ptr::null_mut();
    // SAFETY: `sd` ist eine gültige Beschreibung, `text` ist ein lokaler Ausgabewert.
    check(unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd.0,
            SDDL_REVISION_1,
            OWNER_AND_DACL,
            &mut text,
            null_mut(),
        )
    })?;
    Local(text.cast()).wide_text()
}

/// Beendet beim Verlassen die Identitätsübernahme dieses Threads, auch wenn ein Test mittendrin scheitert.
struct Revert;

impl Drop for Revert {
    fn drop(&mut self) {
        // SAFETY: einfacher Aufruf ohne Zeiger; setzt nur die Identität dieses Threads zurück.
        let _ = unsafe { RevertToSelf() };
    }
}

/// Führt `f` im eigenen Thread als anonymes Konto aus (ein fremder Benutzer, nur für Tests).
pub fn as_anonymous<T>(f: impl FnOnce() -> T) -> io::Result<T> {
    // SAFETY: Pseudo-Handle des eigenen Threads; `Revert` setzt die Identität danach in jedem Fall zurück.
    check(unsafe { ImpersonateAnonymousToken(CURRENT_THREAD) })?;
    let _revert = Revert;
    Ok(f())
}
