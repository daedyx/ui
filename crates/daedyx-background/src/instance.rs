//! Einzelinstanz über eine benannte Pipe (Tokio, ohne eigenes `unsafe`; übernommen aus Spike S5).
//! Der erste Start legt die Pipe an (`first_pipe_instance`), jeder weitere verbindet sich,
//! schickt das feste Wort `SHOW` und endet. `QUIT` beendet die laufende Instanz (für Updater/Installer).
//! Es wird nie ein Inhalt übertragen.
use std::io;

use crate::sink::{Event, Sink};

pub const SHOW: &[u8] = b"SHOW";
pub const QUIT: &[u8] = b"QUIT";

/// Nur diese zwei festen Wörter werden verstanden.
pub fn word_to_event(word: &[u8]) -> Option<Event> {
    match word {
        SHOW => Some(Event::Show),
        QUIT => Some(Event::Quit),
        _ => None,
    }
}

/// Ergebnis des Starts. `L` ist der Empfänger des Systems (Standard: der dieses Systems); so lässt sich der
/// Platzhalter anderer Systeme (`other`) auch unter Windows übersetzen, prüfen und testen.
#[derive(Debug)]
pub enum Role<L = Listener> {
    /// Dieser Prozess ist der erste: er hält die Pipe.
    Primary(L),
    /// Es läuft schon eine Instanz.
    Secondary,
}

/// Erste Pause vor einem neuen Versuch; sie verdoppelt sich bis `RETRY_MAX`.
pub const RETRY_FIRST: std::time::Duration = std::time::Duration::from_millis(50);
pub const RETRY_MAX: std::time::Duration = std::time::Duration::from_secs(2);

/// Ruft `attempt` mit wachsender Pause (50 ms bis 2 s), bis es gelingt. Der Pipe-Dienst endet so nie wegen eines
/// vorübergehenden Fehlers (Review T4, H4-1); sonst wäre der nächste Start selbst „primär“.
pub async fn retry_with_pause<T, E>(mut attempt: impl FnMut() -> Result<T, E>) -> T {
    let mut pause = RETRY_FIRST;
    loop {
        tokio::time::sleep(pause).await;
        if let Ok(v) = attempt() {
            return v;
        }
        pause = pause.saturating_mul(2).min(RETRY_MAX);
    }
}

#[cfg(windows)]
pub use win::{Listener, acquire, notify_primary};

#[cfg(windows)]
mod win {
    use super::*;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeServer, ServerOptions};

    const ERROR_ACCESS_DENIED: i32 = 5;
    const ERROR_PIPE_BUSY: i32 = 231;

    #[derive(Debug)]
    pub struct Listener {
        name: String,
        server: NamedPipeServer,
    }

    /// Muss innerhalb einer Tokio-Laufzeit aufgerufen werden.
    pub fn acquire(name: &str) -> io::Result<Role> {
        let created = ServerOptions::new()
            .first_pipe_instance(true)
            .reject_remote_clients(true)
            .create(name);
        match created {
            Ok(server) => Ok(Role::Primary(Listener {
                name: name.to_string(),
                server,
            })),
            // Pipe existiert schon: ein anderer Prozess hält sie.
            Err(e) if e.raw_os_error() == Some(ERROR_ACCESS_DENIED) => Ok(Role::Secondary),
            Err(e) => Err(e),
        }
    }

    /// Weckt die erste Instanz. Wartet höchstens etwa zwei Sekunden.
    pub async fn notify_primary(name: &str, word: &[u8]) -> io::Result<()> {
        let mut client = None;
        for _ in 0..40 {
            match ClientOptions::new().open(name) {
                Ok(c) => {
                    client = Some(c);
                    break;
                }
                Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY) => {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
                Err(e) => return Err(e),
            }
        }
        let mut client = client.ok_or_else(|| io::Error::from(io::ErrorKind::TimedOut))?;
        client.write_all(word).await?;
        client.flush().await
    }

    impl Listener {
        /// Nimmt Verbindungen an, bis die Laufzeit endet. Unbekannte Wörter werden still verworfen.
        pub async fn serve(self, sink: Sink) {
            let Listener { name, mut server } = self;
            loop {
                if server.connect().await.is_err() {
                    // Nie auf derselben Instanz im Kreis laufen (Review M-1): Instanz ersetzen, mit Pause.
                    server = super::retry_with_pause(|| create(&name)).await;
                    continue;
                }
                // Neue Instanz anlegen, bevor die alte bedient wird, damit die Pipe nie fehlt. Klappt das nicht,
                // wird die Verbindung trotzdem bedient und das Anlegen mit Pause wiederholt, statt den Dienst zu
                // beenden (Review T4, H4-1).
                match create(&name) {
                    Ok(next) => {
                        let conn = std::mem::replace(&mut server, next);
                        let s = sink.clone();
                        tokio::spawn(async move { handle(conn, s).await });
                    }
                    Err(_) => {
                        let s = sink.clone();
                        tokio::spawn(async move { handle(server, s).await });
                        server = super::retry_with_pause(|| create(&name)).await;
                    }
                }
            }
        }
    }

    /// Weitere Instanz derselben Pipe, nur für Verbindungen auf diesem Rechner.
    fn create(name: &str) -> io::Result<NamedPipeServer> {
        ServerOptions::new()
            .reject_remote_clients(true)
            .create(name)
    }

    /// Liest bis zum Auflegen des Clients (auch in Stücken), höchstens `MAX_MESSAGE` Bytes und `READ_LIMIT` lang.
    /// Nur genau `SHOW` oder `QUIT` zählt; alles andere (zu lang, kaputt, zu langsam) wird still verworfen.
    async fn handle(conn: NamedPipeServer, sink: Sink) {
        let mut buf = Vec::new();
        let limit = u64::try_from(MAX_MESSAGE).unwrap_or(16).saturating_add(1);
        let read = tokio::time::timeout(READ_LIMIT, conn.take(limit).read_to_end(&mut buf)).await;
        if matches!(read, Ok(Ok(_)))
            && buf.len() <= MAX_MESSAGE
            && let Some(e) = word_to_event(&buf)
        {
            sink.emit(e);
        }
    }

    /// Längste Nachricht, die überhaupt gelesen wird (die Wörter haben 4 Bytes).
    const MAX_MESSAGE: usize = 16;
    const READ_LIMIT: Duration = Duration::from_secs(2);
}

#[cfg(not(windows))]
pub use other::{Listener, acquire, notify_primary};

/// Auf anderen Systemen noch nicht gebaut (Mac: Unix-Socket im App-Ordner, siehe Bericht Spike S5). Das Modul wird
/// auf jedem System übersetzt, damit Clippy es auch in der Windows-CI prüft; benutzt wird es nur außerhalb von Windows.
pub mod other {
    use super::{Role, Sink, io};

    #[derive(Debug)]
    pub struct Listener;

    pub fn acquire(_name: &str) -> io::Result<Role<Listener>> {
        Ok(Role::Primary(Listener))
    }

    pub async fn notify_primary(_name: &str, _word: &[u8]) -> io::Result<()> {
        Ok(())
    }

    impl Listener {
        pub async fn serve(self, _sink: Sink) {}
    }
}
