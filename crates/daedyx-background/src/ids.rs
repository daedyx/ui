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
    /// Anfang des Pipe-Namens der Einzelinstanz (dahinter die SID des Benutzers).
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

/// Name der benannten Pipe je Benutzer: `scope` ist die SID (siehe `instance::user_scope`), kein Name und kein Pfad.
/// Aus `scope` gelangen nur ASCII-Buchstaben, Ziffern und Bindestriche in den Namen.
pub fn pipe_name(prefix: &str, scope: &str) -> String {
    let scope: String = scope
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    format!(r"\\.\pipe\{prefix}-{scope}")
}
