//! Befehlszeile des Hintergrundbetriebs. Unbekanntes wird still übergangen (die App startet dann normal); eigene
//! Schalter der App (Desk: `--echt`, `--attrappe`) bekommt sie über [`Args::parse_with`].
use crate::ids::HIDDEN_ARG;

/// Kürzester und längster Abstand des Hintergrund-Takts in Sekunden.
pub const TAKT_RANGE: (u64, u64) = (5, 3600);
pub const TAKT_DEFAULT: u64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Args {
    /// Versteckt starten (Autostart bei der Anmeldung).
    pub hidden: bool,
    /// Eine laufende Instanz sauber beenden (für Updater/Installer) und selbst gleich enden.
    pub quit_running: bool,
    /// Autostart-Wert, App-Kennung und eigene Benachrichtigungen entfernen und enden.
    pub cleanup: bool,
    /// Eine Benachrichtigung mit erfundenem Inhalt zeigen, im Info-Center nachsehen und enden.
    pub probe_toast: bool,
    /// Startprobe: Programm laden und sofort mit 0 enden (keine Pipe, keine Registry, kein Fenster). Belegt, dass
    /// Windows das Programm laden kann (Ladefehler zeigen sonst einen Dialog und der Start hängt).
    pub start_probe: bool,
    /// Abstand des Hintergrund-Takts in Sekunden.
    pub takt_secs: u64,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            hidden: false,
            quit_running: false,
            cleanup: false,
            probe_toast: false,
            start_probe: false,
            takt_secs: TAKT_DEFAULT,
        }
    }
}

impl Args {
    pub fn parse<I: IntoIterator<Item = String>>(items: I) -> Args {
        Args::parse_with(items, |_| {})
    }

    /// Wie [`Args::parse`]; jedes hier unbekannte Wort geht an `extra` (eigene Schalter der App).
    pub fn parse_with<I, F>(items: I, mut extra: F) -> Args
    where
        I: IntoIterator<Item = String>,
        F: FnMut(&str),
    {
        let mut a = Args::default();
        let mut it = items.into_iter();
        while let Some(item) = it.next() {
            match item.as_str() {
                HIDDEN_ARG => a.hidden = true,
                "--quit-running" => a.quit_running = true,
                "--cleanup" => a.cleanup = true,
                "--probe-toast" => a.probe_toast = true,
                "--start-probe" => a.start_probe = true,
                "--takt" => {
                    if let Some(n) = it.next().and_then(|v| v.parse::<u64>().ok()) {
                        a.takt_secs = n.clamp(TAKT_RANGE.0, TAKT_RANGE.1);
                    }
                }
                other => extra(other),
            }
        }
        a
    }
}
