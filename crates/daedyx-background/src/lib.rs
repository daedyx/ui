//! Hintergrundbetrieb der egui-Apps (aus Desk M7b und Spike S5, Tasks T1b): Schließen versteckt nur, ein Takt meldet
//! Erinnerungen auch ohne Fenster als Windows-Benachrichtigung, Symbol im Infobereich mit Menü, Autostart (versteckt),
//! Einzelinstanz über eine benannte Pipe, Befehlszeile, Aufräumen. Die Kennungen (App-Kennung, Autostart-Wert, Pipe)
//! gibt die App herein ([`ids::AppIds`]); was der Takt prüft, bestimmt sie mit [`ticker::Jobs`]. Kein Netz.
pub mod args;
pub mod aumid;
pub mod autostart;
pub mod ids;
pub mod instance;
pub mod notify;
mod sink;
pub mod start;
mod state;
mod style;
pub mod texts;
pub mod ticker;
pub mod tray;
pub mod window;

pub use sink::{Event, Notice, NoticeKind, Sink};
pub use state::Background;

#[cfg(test)]
mod exit_tests;
#[cfg(test)]
mod guard_tests;
#[cfg(test)]
mod other_tests;
#[cfg(test)]
mod parts_tests;
#[cfg(all(test, windows))]
mod pipe_tests;
