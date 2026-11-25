//! Cargo-Anweisungen für den Linker (aus Desk `build.rs`, Tasks T1b). Diese Datei enthält nur Linker-Optionen und
//! Cargo-Zeilen, keine Oberflächentexte; der Wächter des Crates prüft sie deshalb nur auf Farbwerte.
use std::io::Write;
use std::path::Path;

/// Manifest-Abhängigkeit auf Common Controls 6 (siehe Crate-Beschreibung).
pub const COMMON_CONTROLS_6: &str = "/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' \
     version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'";

/// Was mit dem Programmsymbol geschah.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    /// Eingebettet (Windows/MSVC).
    Embedded,
    /// Windows/MSVC, aber die `.ico` fehlt oder ist kaputt; die App meldet das als Bau-Warnung.
    Missing,
    /// Anderes Ziel: weder Manifest noch Symbol.
    NotWindows,
}

/// Für `build.rs`: Unter Windows/MSVC das Manifest mit Common Controls 6 einbetten und `icon` als Programmsymbol
/// (über `OUT_DIR/<res_name>`) einbinden. Meldet `icon` als Auslöser für einen neuen Bau.
pub fn windows_app(icon: &Path, res_name: &str) -> Symbol {
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let mut out = std::io::stdout();
    let _ = writeln!(out, "cargo:rerun-if-changed={}", icon.display());
    if os != "windows" || env != "msvc" {
        return Symbol::NotWindows;
    }
    let _ = writeln!(out, "cargo:rustc-link-arg-bins=/MANIFEST:EMBED");
    let _ = writeln!(out, "cargo:rustc-link-arg-bins={COMMON_CONTROLS_6}");
    let target = Path::new(&std::env::var("OUT_DIR").unwrap_or_default()).join(res_name);
    let written = std::fs::read(icon)
        .ok()
        .and_then(|ico| crate::res::icon_res(&ico))
        .and_then(|bytes| std::fs::write(&target, bytes).ok());
    if written.is_some() {
        let _ = writeln!(out, "cargo:rustc-link-arg-bins={}", target.display());
        Symbol::Embedded
    } else {
        Symbol::Missing
    }
}
