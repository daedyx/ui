//! Kennungen des Hintergrundbetriebs. Die App gibt ihre eigenen als [`AppIds`] herein (Desk: die der **Vorschau**,
//! nie die der echten App); hier stehen nur die festen Windows-Schlüssel und der Schalter auf der Befehlszeile.
//! Nur technische Kennungen, keine Oberflächentexte (der Wächter prüft diese Datei nur auf Farbwerte).

/// Kennungen einer App.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppIds {
    /// App-Kennung (AppUserModelID) für Windows-Benachrichtigungen.
    pub aumid: &'static str,
    /// Name des Werts unter `HKCU\...\Run`.
    pub run_value: &'static str,
    /// Anfang des Pipe-Namens der Einzelinstanz (dahinter ein Hash des Benutzers).
    pub pipe_prefix: &'static str,
}

pub const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
/// Hier schreibt der Task-Manager „Deaktiviert“ hinein; die App liest nur.
pub const APPROVED_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
pub const AUMID_KEY_BASE: &str = r"Software\Classes\AppUserModelId";
/// Name des Werts mit dem Anzeigenamen der App-Kennung.
pub const AUMID_NAME_VALUE: &str = "DisplayName";
/// Versteckt starten (Autostart).
pub const HIDDEN_ARG: &str = "--hidden";

/// FNV-1a: stabil über Programmversionen hinweg (anders als `DefaultHasher`).
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Name der benannten Pipe je Benutzer. Der Benutzername steht nicht im Klartext darin.
pub fn pipe_name(prefix: &str, scope: &str) -> String {
    format!(r"\\.\pipe\{prefix}-{:016x}", fnv1a(scope.as_bytes()))
}

/// Unterscheidet Benutzer auf demselben Rechner (Profilpfad, nur gehasht verwendet).
pub fn user_scope() -> String {
    std::env::var("USERPROFILE").unwrap_or_default()
}
