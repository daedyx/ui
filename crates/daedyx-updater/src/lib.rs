//! Update in der App (plan-update-bibliothek, Auftrag U2), gemeinsam für Tasks und Desk. Die Prüfung selbst steckt in
//! `daedyx-update` (Kern): Nachweis, Eintrag, Version, Download, Sperre, Zustand. Hier liegt, was eine App dazu
//! braucht:
//! - [`net::UreqTransport`]: echtes Netz, nur `https`, folgt keiner Weiterleitung selbst (das prüft der Kern je
//!   Schritt gegen die erlaubten Rechner), feste Zeitgrenzen, fester User-Agent.
//! - [`worker::Updater`]: Ablauf im eigenen Thread (Suche, Download nach Klick, Schalter „Beim Start suchen“,
//!   höchstens einmal am Tag), Meldungen über einen Kanal, keine Sperren im Speicher.
//! - [`install::Prepared`]: geprüftes Setup mit gehaltener Sperre, Start mit festen Argumenten über `daedyx-sys`.
//! - [`panel`]: Bauteil „Updates“ für die Einstellungen der App und den Hinweis in der Statuszeile; Texte aus `lang/*.json`.
//!
//! Kein stilles Installieren: Das Setup startet nur nach dem Klick auf „Jetzt aktualisieren“.
pub mod config;
pub mod install;
pub mod net;
pub mod panel;
pub mod status;
pub mod texts;
pub mod worker;
mod worker_run;
mod worker_steps;

/// Die Kern-Bibliothek in genau der Fassung, die dieses Crate benutzt (Policy, Version, Sperrgründe), damit die App
/// keine zweite Kern-Quelle einträgt.
pub use daedyx_update;

pub use config::{Backend, Config};
pub use install::Prepared;
pub use status::Status;
pub use worker::Updater;

#[cfg(test)]
mod download_tests;
#[cfg(test)]
mod guard_tests;
#[cfg(test)]
mod install_tests;
#[cfg(test)]
mod net_tests;
#[cfg(test)]
mod panel_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod worker_tests;
