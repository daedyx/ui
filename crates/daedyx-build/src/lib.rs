//! Bau-Helfer für die `build.rs` der egui-Apps (aus Desk M7b/M8, Tasks T1b). Nur Windows/MSVC bekommt etwas:
//!
//! 1. **Anwendungsmanifest mit Common Controls 6.** Warum (Bericht M7b): Das Menü im Infobereich (`muda`) bekommt im
//!    Workspace-Bau über Tauri das Feature `common-controls-v6` und importiert dann `TaskDialogIndirect`, das es nur
//!    in Common Controls 6 gibt. Ohne Manifest zeigt Windows beim Start „Einsprungpunkt nicht gefunden“ (0xC0000139)
//!    und der Start hängt, bis jemand den Dialog schließt – auch beim Autostart. Nur Linker-Optionen.
//! 2. **Programmsymbol:** [`res::icon_res`] schreibt aus einer `.ico` eine `.res`-Datei, der Linker bindet sie ein.
//!    Kein `rc.exe`, kein Prozessstart.
//!
//! Die Installer-Vorlage (NSIS, Bauskript, Installationsprobe) liegt im Ordner `vorlage/installer` dieses Crates;
//! die App findet ihn über `cargo metadata` (Desk: `scripts/lib/vorlage-installer.ps1`).
pub mod installer;
mod link;
pub mod res;

pub use link::{COMMON_CONTROLS_6, Symbol, windows_app};

#[cfg(test)]
mod guard_tests;
#[cfg(test)]
mod res_tests;
