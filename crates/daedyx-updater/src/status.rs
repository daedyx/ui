//! Was die Oberfläche über das Update zeigt. Jede Lage hat genau einen Text (`lang/*.json`, plan-update-bibliothek 7);
//! Fehlergründe der Bibliothek (englisch) kommen nie in die Oberfläche.
use daedyx_update::reason::BlockKind;

/// Lage des Updates.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Status {
    /// In dieser Sitzung noch nicht gesucht.
    #[default]
    Idle,
    Checking,
    UpToDate,
    /// Geprüft und neuer. `notes` ist reiner Text (vom Kern ohne Steuer- und Formatzeichen geprüft).
    Available {
        version: String,
        notes: String,
        important: bool,
    },
    Downloading {
        done: u64,
        total: u64,
    },
    /// Geladen und geprüft; die App startet das Setup ([`crate::Updater::take_ready`]).
    Ready {
        version: String,
    },
    /// Setup gestartet, die App beendet sich.
    Installing,
    /// Die gemeinsame Sperre hält schon jemand (z. B. der Launcher aktualisiert dieselbe App).
    Busy,
    /// Suche oder Download haben nichts gebracht oder wurden abgelehnt.
    Blocked(BlockKind),
    /// Der eigene Zustand ließ sich nicht sichern (Entwurf): kein Update gestartet.
    NotSaved,
    /// Das geprüfte Setup ließ sich nicht starten.
    StartFailed,
}

/// Textschlüssel einer Sperre.
pub fn block_key(kind: BlockKind) -> &'static str {
    match kind {
        BlockKind::Offline => "update.offline",
        BlockKind::SourceUnavailable => "update.quelle",
        BlockKind::NotTrusted => "update.unsicher",
        BlockKind::NeedsManualUpdate => "update.von_hand",
        BlockKind::SourceWentBack => "update.zurueck",
        BlockKind::Interrupted => "update.unterbrochen",
        BlockKind::NoSpace => "update.kein_platz",
        BlockKind::LocalFile => "update.datei",
    }
}

impl Status {
    /// Läuft gerade etwas (Knöpfe gesperrt)?
    pub fn working(&self) -> bool {
        matches!(
            self,
            Status::Checking
                | Status::Downloading { .. }
                | Status::Ready { .. }
                | Status::Installing
        )
    }

    /// Fortschritt des Downloads in ganzen Prozent.
    pub fn percent(done: u64, total: u64) -> u64 {
        if total == 0 {
            return 0;
        }
        done.saturating_mul(100)
            .checked_div(total)
            .unwrap_or(0)
            .min(100)
    }
}
