//! Windows-Aufrufe der Pipe-Härtung (SICHERHEIT-AUSNAHMEN A-027). `unsafe` ist je Funktion freigegeben (Review
//! H-P3), jeder `unsafe`-Block enthält genau einen Aufruf. Handles und vom System zugeteilter Speicher gehören sofort
//! einem Typ, der sie beim Verlassen freigibt.
use std::ffi::c_void;
use std::io;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle, RawHandle};
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};

const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
const TOKEN_QUERY: u32 = 0x0008;
const TOKEN_USER_CLASS: i32 = 1;
const SDDL_REVISION_1: u32 = 1;
const SE_KERNEL_OBJECT: i32 = 6;
/// `OWNER_SECURITY_INFORMATION`: nur der Besitzer.
pub(super) const OWNER: u32 = 0x1;
/// Besitzer und Zugriffsliste (`OWNER_` und `DACL_SECURITY_INFORMATION`).
pub(super) const OWNER_AND_DACL: u32 = 0x1 | 0x4;

#[repr(C)]
struct SecurityAttributes {
    length: u32,
    descriptor: *mut c_void,
    inherit: i32,
}

#[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
#[link(name = "kernel32")]
#[link(name = "advapi32")]
unsafe extern "system" {
    fn OpenProcess(access: u32, inherit: i32, pid: u32) -> RawHandle;
    fn OpenProcessToken(process: RawHandle, access: u32, token: *mut RawHandle) -> i32;
    fn GetTokenInformation(
        token: RawHandle,
        class: i32,
        buf: *mut c_void,
        len: u32,
        ret: *mut u32,
    ) -> i32;
    fn ConvertSidToStringSidW(sid: *mut c_void, text: *mut *mut u16) -> i32;
    fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
        text: *const u16,
        revision: u32,
        sd: *mut *mut c_void,
        len: *mut u32,
    ) -> i32;
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
    fn LocalFree(mem: *mut c_void) -> *mut c_void;
    fn lstrlenW(text: *const u16) -> i32;
}

/// Vom System mit `LocalAlloc` zugeteilter Speicher; wird beim Verlassen freigegeben.
pub(super) struct Local(pub(super) *mut c_void);

impl Drop for Local {
    #[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: Der Zeiger stammt aus einer Windows-Funktion, die `LocalAlloc`-Speicher liefert, und wird nur
            // hier, genau einmal, freigegeben.
            let _ = unsafe { LocalFree(self.0) };
        }
    }
}

impl Local {
    /// Liest den Speicher als nullterminierten UTF-16-Text.
    #[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
    pub(super) fn wide_text(&self) -> io::Result<String> {
        let text = self.0.cast::<u16>();
        // SAFETY: `text` zeigt auf einen nullterminierten Text, den das System gerade geschrieben hat.
        let len = unsafe { lstrlenW(text) };
        let len = usize::try_from(len).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
        // SAFETY: `len` Zeichen ab `text` gehören zu diesem Speicher, der bis zum Ende von `self` lebt.
        let units = unsafe { std::slice::from_raw_parts(text, len) };
        String::from_utf16(units).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

pub(super) fn check(ok: i32) -> io::Result<()> {
    if ok != 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
fn owned(raw: RawHandle) -> io::Result<OwnedHandle> {
    if raw.is_null() {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `raw` ist ein gerade geöffnetes, gültiges Handle, das sonst niemandem gehört.
    Ok(unsafe { OwnedHandle::from_raw_handle(raw) })
}

/// SID (Textform `S-1-…`) des Benutzers, unter dem der Prozess `pid` läuft.
#[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
pub(super) fn process_user_sid(pid: u32) -> io::Result<String> {
    // SAFETY: einfacher Aufruf mit Zahlen; das Ergebnis übernimmt sofort `owned`.
    let process = owned(unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) })?;
    let mut raw = std::ptr::null_mut();
    // SAFETY: `process` ist gültig, `raw` ist ein lokaler Ausgabewert.
    check(unsafe { OpenProcessToken(process.as_raw_handle(), TOKEN_QUERY, &mut raw) })?;
    let token = owned(raw)?;
    // TOKEN_USER (16 Byte) plus SID (höchstens 68 Byte) passt in 128 Byte; u64 sorgt für die Ausrichtung.
    let mut buf = [0u64; 16];
    let mut ret = 0u32;
    let size = u32::try_from(std::mem::size_of_val(&buf)).unwrap_or(0);
    // SAFETY: `buf` ist `size` Byte groß und beschreibbar, `ret` ist lokal.
    check(unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TOKEN_USER_CLASS,
            buf.as_mut_ptr().cast(),
            size,
            &mut ret,
        )
    })?;
    // SAFETY: Das erste Feld von TOKEN_USER ist der SID-Zeiger; `buf` ist ausgerichtet und vom System gefüllt.
    let sid = unsafe { buf.as_ptr().cast::<*mut c_void>().read() };
    let mut text = std::ptr::null_mut();
    // SAFETY: `sid` zeigt in `buf`, das noch lebt; `text` ist ein lokaler Ausgabewert.
    check(unsafe { ConvertSidToStringSidW(sid, &mut text) })?;
    Local(text.cast()).wide_text()
}

/// Teile `what` (`OWNER` oder `OWNER_AND_DACL`) der Sicherheitsbeschreibung eines Kernel-Objekts als SDDL-Text.
#[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
pub(super) fn security_text(object: &impl AsRawHandle, what: u32) -> io::Result<String> {
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
    let sd = Local(sd);
    let mut text = null_mut();
    // SAFETY: `sd` ist eine gültige Beschreibung, `text` ist ein lokaler Ausgabewert.
    check(unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd.0,
            SDDL_REVISION_1,
            what,
            &mut text,
            null_mut(),
        )
    })?;
    Local(text.cast()).wide_text()
}

/// Legt eine Pipe-Instanz mit der Sicherheitsbeschreibung `sddl` an. Gibt es den Namen schon, erbt die Instanz die
/// Rechte der bestehenden Pipe; das prüft erst der Aufrufer (`pipe::create_server`).
#[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
pub(super) fn create_with_sddl(
    options: &ServerOptions,
    name: &str,
    sddl: &str,
) -> io::Result<NamedPipeServer> {
    let text = wide(sddl);
    let mut sd = std::ptr::null_mut();
    // SAFETY: `text` ist nullterminiert und lebt über den Aufruf; `sd` ist ein lokaler Ausgabewert.
    check(unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            text.as_ptr(),
            SDDL_REVISION_1,
            &mut sd,
            std::ptr::null_mut(),
        )
    })?;
    let sd = Local(sd);
    let mut attrs = SecurityAttributes {
        length: u32::try_from(std::mem::size_of::<SecurityAttributes>()).unwrap_or(0),
        descriptor: sd.0,
        inherit: 0,
    };
    // SAFETY: `attrs` ist eine gültige SECURITY_ATTRIBUTES-Struktur, deren Beschreibung (`sd`) über den Aufruf lebt.
    unsafe { options.create_with_security_attributes_raw(name, (&raw mut attrs).cast()) }
}
