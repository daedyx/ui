//! Besitzer und Zugriffsliste einer Sicherheitsbeschreibung, jede SID in voller Textform `S-1-…` (SICHERHEIT-AUSNAHMEN
//! A-027). `ConvertSidToStringSidW` schreibt jede SID aus; SDDL-Text kürzt dagegen bekannte SIDs ab, etwa das
//! eingebaute Administratorkonto (`…-500`) zu „LA“. Ein Textvergleich mit SDDL lehnte deshalb unter diesem Konto die
//! eigene Pipe ab (Release-Lauf 1 auf dem GitHub-Läufer). `unsafe` ist je Funktion freigegeben, jeder Block enthält
//! genau einen Aufruf oder Lesezugriff.
use super::ffi::{Local, check};
use std::ffi::c_void;
use std::io;
use std::os::windows::io::AsRawHandle;
use std::ptr::null_mut;

const SE_KERNEL_OBJECT: i32 = 6;
const ACL_SIZE_INFORMATION: i32 = 2;
/// `SE_DACL_PROTECTED`, `SE_DACL_AUTO_INHERITED`, `SE_DACL_AUTO_INHERIT_REQ`: die Schalter, die SDDL hinter `D:`
/// als `P`, `AI` und `AR` schreibt.
pub(super) const DACL_FLAGS: u16 = 0x1000 | 0x0400 | 0x0100;
pub(super) const DACL_PROTECTED: u16 = 0x1000;
pub(super) const ACCESS_ALLOWED: u8 = 0;

/// Eine Regel der Zugriffsliste. `mask` und `sid` nur bei „erlauben“ (`ACCESS_ALLOWED`), sonst 0 und leer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Rule {
    pub(super) kind: u8,
    pub(super) flags: u8,
    pub(super) mask: u32,
    pub(super) sid: String,
}

/// Besitzer, Schalter der Zugriffsliste und ihre Regeln (`None`: keine Zugriffsliste, also Zugriff für jeden).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Security {
    pub(super) owner: String,
    pub(super) dacl_flags: u16,
    pub(super) rules: Option<Vec<Rule>>,
}

#[repr(C)]
#[derive(Default)]
struct AclSize {
    ace_count: u32,
    bytes_in_use: u32,
    bytes_free: u32,
}

#[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
#[link(name = "advapi32")]
unsafe extern "system" {
    fn GetSecurityInfo(
        handle: std::os::windows::io::RawHandle,
        kind: i32,
        what: u32,
        owner: *mut *mut c_void,
        group: *mut *mut c_void,
        dacl: *mut *mut c_void,
        sacl: *mut *mut c_void,
        sd: *mut *mut c_void,
    ) -> u32;
    fn GetSecurityDescriptorOwner(
        sd: *mut c_void,
        owner: *mut *mut c_void,
        defaulted: *mut i32,
    ) -> i32;
    fn GetSecurityDescriptorControl(sd: *mut c_void, control: *mut u16, revision: *mut u32) -> i32;
    fn GetSecurityDescriptorDacl(
        sd: *mut c_void,
        present: *mut i32,
        dacl: *mut *mut c_void,
        defaulted: *mut i32,
    ) -> i32;
    fn GetAclInformation(acl: *mut c_void, info: *mut c_void, len: u32, class: i32) -> i32;
    fn GetAce(acl: *mut c_void, index: u32, ace: *mut *mut c_void) -> i32;
    fn ConvertSidToStringSidW(sid: *mut c_void, text: *mut *mut u16) -> i32;
}

/// Teile `what` (`ffi::OWNER` oder `ffi::OWNER_AND_DACL`) der Sicherheitsbeschreibung eines Kernel-Objekts.
#[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
pub(super) fn of_object(object: &impl AsRawHandle, what: u32) -> io::Result<Security> {
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
    of_descriptor(&Local(sd))
}

/// Liest eine Sicherheitsbeschreibung, die das System zugeteilt hat (`GetSecurityInfo` oder eine SDDL-Vorlage).
#[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
pub(super) fn of_descriptor(sd: &Local) -> io::Result<Security> {
    let (mut owner, mut defaulted) = (null_mut(), 0i32);
    // SAFETY: `sd` ist eine gültige Beschreibung; der Besitzer-Zeiger zeigt in sie hinein und lebt mit ihr.
    check(unsafe { GetSecurityDescriptorOwner(sd.0, &mut owner, &mut defaulted) })?;
    if owner.is_null() {
        return Err(io::Error::from(io::ErrorKind::InvalidData));
    }
    let owner = sid_text(owner)?;
    let (mut control, mut revision) = (0u16, 0u32);
    // SAFETY: wie oben; beide Ausgabewerte sind lokal.
    check(unsafe { GetSecurityDescriptorControl(sd.0, &mut control, &mut revision) })?;
    let (mut present, mut dacl) = (0i32, null_mut());
    // SAFETY: wie oben; die Zugriffsliste zeigt in `sd` hinein und lebt mit ihr.
    check(unsafe { GetSecurityDescriptorDacl(sd.0, &mut present, &mut dacl, &mut defaulted) })?;
    let rules = if present != 0 && !dacl.is_null() {
        Some(rules(dacl)?)
    } else {
        None
    };
    Ok(Security {
        owner,
        dacl_flags: control & DACL_FLAGS,
        rules,
    })
}

/// Regeln einer Zugriffsliste, die in einer lebenden Sicherheitsbeschreibung liegt.
#[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
fn rules(acl: *mut c_void) -> io::Result<Vec<Rule>> {
    let mut size = AclSize::default();
    let len = u32::try_from(std::mem::size_of::<AclSize>()).unwrap_or(0);
    // SAFETY: `acl` ist gültig, `size` ist eine lokale, passend große Struktur.
    check(unsafe { GetAclInformation(acl, (&raw mut size).cast(), len, ACL_SIZE_INFORMATION) })?;
    let mut out = Vec::new();
    for index in 0..size.ace_count {
        let mut ace = null_mut();
        // SAFETY: `index` liegt unter der Anzahl der Regeln, `ace` ist ein lokaler Ausgabewert.
        check(unsafe { GetAce(acl, index, &mut ace) })?;
        let ace = ace.cast::<u8>();
        // SAFETY: Jede Regel beginnt mit ihrem Kopf (Art, Schalter, Größe; 4 Byte).
        let head = unsafe { ace.cast::<[u8; 4]>().read_unaligned() };
        let [kind, flags, lo, hi] = head;
        let mut rule = Rule {
            kind,
            flags,
            mask: 0,
            sid: String::new(),
        };
        // „Erlauben“: Kopf, Maske (4 Byte), dann die SID (Revision, Anzahl der Unterkennungen, 6 Byte Herkunft, je
        // Unterkennung 4 Byte); nur, wenn die Regel dafür groß genug ist.
        let size = usize::from(u16::from_le_bytes([lo, hi]));
        if kind == ACCESS_ALLOWED && size >= 16 {
            // SAFETY: Die Regel ist mindestens 16 Byte groß; Maske und SID-Anfang liegen in Byte 4 bis 11.
            let body = unsafe { ace.wrapping_add(4).cast::<[u8; 8]>().read_unaligned() };
            let [m0, m1, m2, m3, _revision, count, ..] = body;
            // Die SID muss ganz in der Regel liegen (Review N-PS-1), sonst gilt die Regel als fremd.
            if 16usize.saturating_add(usize::from(count).saturating_mul(4)) <= size {
                rule.mask = u32::from_le_bytes([m0, m1, m2, m3]);
                rule.sid = sid_text(ace.wrapping_add(8).cast())?;
            }
        }
        out.push(rule);
    }
    Ok(out)
}

/// SID als volle Textform `S-1-…`, nie abgekürzt.
#[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-027")]
fn sid_text(sid: *mut c_void) -> io::Result<String> {
    let mut text = null_mut();
    // SAFETY: `sid` zeigt auf eine SID in einer lebenden Beschreibung, `text` ist ein lokaler Ausgabewert.
    check(unsafe { ConvertSidToStringSidW(sid, &mut text) })?;
    Local(text.cast()).wide_text()
}
