//! Betriebssystem-Aufrufe der Daedyx-Apps (aus `desk-sys`, Tasks T1b). In `daedyx/ui` und Desk ist das die einzige
//! Stelle mit `unsafe` (SICHERHEIT.md 2.1); jede Stelle trägt eine `// SAFETY:`-Begründung.
use std::path::Path;

mod install;
#[cfg(windows)]
pub mod pipe;
mod process;
pub use install::{StartError, file_sha256, start_verified};
pub use process::{RunEnd, run_with_limit};

/// Freier Platz auf dem Laufwerk eines Ordners (nur Windows; sonst keine Messung, die Kopie selbst meldet
/// dann den Fehler).
#[cfg(windows)]
#[allow(unsafe_code, reason = "SICHERHEIT-AUSNAHMEN A-020")]
pub fn available_space(dir: &Path) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetDiskFreeSpaceExW(
            dir: *const u16,
            free_to_caller: *mut u64,
            total: *mut u64,
            total_free: *mut u64,
        ) -> i32;
    }
    let wide: Vec<u16> = dir
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut free = 0u64;
    // SAFETY: `wide` ist nullterminiert und lebt über den Aufruf; die Zeiger zeigen auf lokale Werte.
    let ok = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    (ok != 0).then_some(free)
}

#[cfg(not(windows))]
pub fn available_space(_dir: &Path) -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn an_existing_folder_has_a_measurable_free_space() {
        assert!(available_space(&std::env::current_dir().unwrap()).is_some());
    }

    #[test]
    fn a_missing_program_cannot_be_started() {
        let end = run_with_limit(
            Path::new("gibt-es-nicht-daedyx.exe"),
            &[],
            std::time::Duration::from_secs(1),
        );
        assert_eq!(end, RunEnd::Failed);
    }

    #[test]
    fn a_missing_folder_gives_no_measurement_instead_of_a_crash() {
        // Ohne Laufwerksbuchstaben (der Datenscan des ui-Repos hält ihn für einen echten Pfad): ein relativer Ordner,
        // den es nicht gibt.
        let missing = Path::new("gibt-es-nicht-daedyx").join("ordner");
        assert_eq!(available_space(&missing), None);
    }
}
