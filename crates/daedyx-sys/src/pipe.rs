//! Gehärtete benannte Pipe der Einzelinstanz (nur Windows; Entscheid T4-7, Review Tasks T4). Gilt für Desk und Tasks
//! über `daedyx-background`:
//! - Zugriff nur für den eigenen Benutzer: Besitzer und einzige Zugriffsregel sind seine SID, kein Eintrag für
//!   „Jeder“, anonym, Netzwerk, Administratoren oder SYSTEM.
//! - Keine Fernverbindung (`PIPE_REJECT_REMOTE_CLIENTS`).
//! - Beim Verbinden prüft der Client, dass das Pipe-Objekt dem eigenen Benutzer gehört (Besitzer der
//!   Sicherheitsbeschreibung, Review N-P1); sonst sendet er nichts und meldet `PermissionDenied`. Dagegen hilft es,
//!   wenn ein anderes Konto den Namen vorher belegt.
use std::io;
use std::os::windows::io::AsRawHandle;
use tokio::net::windows::named_pipe::{
    ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions,
};

#[path = "pipe_ffi.rs"]
mod ffi;
#[cfg(test)]
#[path = "pipe_probe.rs"]
mod probe;

/// SID des eigenen Benutzers in Textform (`S-1-5-21-…`). Hängt nicht an Umgebungsvariablen.
pub fn user_sid() -> io::Result<String> {
    let sid = ffi::process_user_sid(std::process::id())?;
    if is_sid(&sid) {
        Ok(sid)
    } else {
        Err(io::Error::from(io::ErrorKind::InvalidData))
    }
}

/// Prüft die Textform einer SID: `S-1-` und danach nur Zahlengruppen, durch genau einen Bindestrich getrennt.
/// So kann eine SID weder den Pipe-Namen noch die Sicherheitsbeschreibung verändern.
pub fn is_sid(text: &str) -> bool {
    let Some(rest) = text.strip_prefix("S-1-") else {
        return false;
    };
    let mut groups = 0usize;
    for group in rest.split('-') {
        if group.is_empty() || group.len() > 10 || !group.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
        groups = groups.saturating_add(1);
    }
    (2..=16).contains(&groups)
}

/// Sicherheitsbeschreibung: Besitzer und Gruppe sind `sid`, einzige Regel „voller Zugriff für `sid`“, geschützt gegen
/// geerbte Regeln.
pub fn owner_only_sddl(sid: &str) -> io::Result<String> {
    if !is_sid(sid) {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }
    Ok(format!("O:{sid}G:{sid}D:P(A;;GA;;;{sid})"))
}

/// Legt eine Pipe-Instanz an. `first`: nur, wenn es die Pipe noch nicht gibt (sonst `PermissionDenied`). Auch eine
/// Instanz, die nicht genau die eigenen Rechte trägt, ergibt `PermissionDenied`.
/// Muss innerhalb einer Tokio-Laufzeit aufgerufen werden.
pub fn create_server(name: &str, first: bool) -> io::Result<NamedPipeServer> {
    let sid = user_sid()?;
    let sddl = owner_only_sddl(&sid)?;
    let mut options = ServerOptions::new();
    options
        .first_pipe_instance(first)
        .reject_remote_clients(true);
    let server = ffi::create_with_sddl(&options, name, &sddl)?;
    // Gibt es den Namen schon, erbt die Instanz die Rechte der bestehenden Pipe, nicht `sddl` (Review N-P2). Trägt
    // sie nicht genau unsere Rechte, wird sie verworfen: nie auf einer fremden Pipe dienen.
    match ffi::security_text(&server, ffi::OWNER_AND_DACL) {
        Ok(text) if text == format!("O:{sid}D:P(A;;FA;;;{sid})") => Ok(server),
        _ => Err(io::Error::from(io::ErrorKind::PermissionDenied)),
    }
}

/// Verbindet sich mit der Pipe, wenn der Server dem eigenen Benutzer gehört.
pub fn open_client(name: &str) -> io::Result<NamedPipeClient> {
    open_client_for(name, &user_sid()?)
}

/// Verbindet sich mit der Pipe, wenn das Pipe-Objekt dem Benutzer `expected` gehört; sonst wird die Verbindung
/// sofort wieder geschlossen, ohne etwas zu senden.
pub fn open_client_for(name: &str, expected: &str) -> io::Result<NamedPipeClient> {
    let client = ClientOptions::new().open(name)?;
    match pipe_owner_sid(&client) {
        Ok(sid) if sid == expected => Ok(client),
        // Fremder Besitzer oder nicht feststellbar: nichts senden.
        _ => Err(io::Error::from(io::ErrorKind::PermissionDenied)),
    }
}

/// Besitzer des Pipe-Objekts hinter `pipe` (Client- oder Server-Seite). Ein anderes Konto ohne Adminrechte kann den
/// Besitzer nicht auf eine fremde SID setzen; anders als die Prozess-ID hängt das nicht am Server-Prozess.
pub fn pipe_owner_sid(pipe: &impl AsRawHandle) -> io::Result<String> {
    let text = ffi::security_text(pipe, ffi::OWNER)?;
    match text.strip_prefix("O:") {
        Some(sid) if is_sid(sid) => Ok(sid.to_string()),
        _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
    }
}

#[cfg(test)]
#[path = "pipe_tests.rs"]
mod tests;
