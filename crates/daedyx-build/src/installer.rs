//! Die Installer-Vorlage (NSIS) als Text, damit Apps in Tests prüfen können, was ihr Setup mitliefert. Gebaut wird
//! mit den Dateien im Ordner [`dir`] (`installer-bauen.ps1`, Probe `installer-probe.ps1`).
use std::path::PathBuf;

/// Ablauf des Setups (`app-vorschau.nsi`); die App setzt PRODUCT, AUMID, EXE und LICENSES.
pub const NSI: &str = include_str!("../vorlage/installer/app-vorschau.nsi");

/// Ordner der Vorlage in diesem Crate.
pub fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("vorlage")
        .join("installer")
}
