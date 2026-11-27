//! Prüfhilfen für die Tests der Pipe-Härtung (SICHERHEIT-AUSNAHMEN A-027): als fremdes (anonymes) Konto handeln und
//! Sicherheitsbeschreibungen als SDDL-Text lesen. Nur in Tests übersetzt; `unsafe` je Funktion freigegeben (Review H-P3).
use super::ffi::{Local, check};
use std::ffi::c_void;
use std::io;
use std::os::windows::io::{AsRawHandle, RawHandle};

const SDDL_REVISION_1: u32 = 1;
const SE_KERNEL_OBJECT: i32 = 6;

/// Pseudo-Handle des eigenen Threads (Wert von `GetCurrentThread`, dokumentiert als `(HANDLE)-2`).
const CURRENT_THREAD: RawHandle = std::ptr::without_provenance_mut(!1usize);

#[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
#[link(name = "advapi32")]
unsafe extern "system" {
    fn ImpersonateAnonymousToken(thread: RawHandle) -> i32;
    fn RevertToSelf() -> i32;
    fn GetSecurityInfo(
        handle: RawHandle,
        kind: i32,
        what: u32,
        owner: *mut *mut c_void,
        group: *mut *mut c_void,
        dacl: *mut *mut c_void,
        sacl: *mut *mut c_void,
        sd: *mut *mut c_void,
    ) -> u32;
    fn ConvertSecurityDescriptorToStringSecurityDescriptorW(
        sd: *mut c_void,
        revision: u32,
        what: u32,
        text: *mut *mut u16,
        len: *mut u32,
    ) -> i32;
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

/// Teile `what` (`OWNER` oder `OWNER_AND_DACL`) der Sicherheitsbeschreibung eines Kernel-Objekts als SDDL-Text. Nur
/// für Tests: SDDL kürzt bekannte SIDs ab („LA“), verglichen wird im Programm mit `pipe_acl` (Release-Lauf 1).
#[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
pub fn security_text(object: &impl AsRawHandle, what: u32) -> io::Result<String> {
    use std::ptr::null_mut;
    let mut sd = null_mut();
    // SAFETY: Das Handle gehört dem lebenden `object`, `sd` ist ein lokaler Ausgabewert.
    let err = unsafe {
        GetSecurityInfo(
            object.as_raw_handle(),
            SE_KERNEL_OBJECT,
            what,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            &mut sd,
        )
    };
    if err != 0 {
        return Err(io::Error::from_raw_os_error(
            i32::try_from(err).unwrap_or(i32::MAX),
        ));
    }
    descriptor_text(&Local(sd), what)
}

/// Teile `what` einer Sicherheitsbeschreibung als SDDL-Text (nur Tests, wie `security_text`).
#[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
pub fn descriptor_text(sd: &Local, what: u32) -> io::Result<String> {
    let mut text = std::ptr::null_mut();
    // SAFETY: `sd` ist eine gültige Beschreibung, `text` ist ein lokaler Ausgabewert.
    check(unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd.0,
            SDDL_REVISION_1,
            what,
            &mut text,
            std::ptr::null_mut(),
        )
    })?;
    Local(text.cast()).wide_text()
}
