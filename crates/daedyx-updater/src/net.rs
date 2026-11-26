//! Echtes Netz für die Update-Suche (`daedyx_update::fetch::Transport`): ureq mit rustls. Der Transport beantwortet
//! genau **eine** Anfrage und folgt Weiterleitungen nie selbst; der Kern prüft jede neue Adresse gegen die Liste der
//! erlaubten Rechner, bevor sie angefragt wird (Review U1-R, H-U1-5). Nur `https`, fester User-Agent, keine Cookies,
//! keine Kompression. Zeitgrenzen: 15 s für Namensauflösung, Verbindung und Antwortkopf; beim Laden 60 s ohne
//! Datenfluss, aber keine Gesamtgrenze (plan-update-bibliothek 5.4). Ein Proxy aus der Umgebung wird benutzt; TLS
//! geht trotzdem bis zu GitHub.
use daedyx_update::fetch::{HttpsUrl, Reply, Transport, TransportError};
use std::io::ErrorKind;
use std::time::Duration;

/// Einziger User-Agent (plan-update-bibliothek 4): keine Version, keine Plattform.
pub const USER_AGENT: &str = "daedyx-update";
/// Namensauflösung, Verbindung (mit TLS) und Antwortkopf je Anfrage.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
/// Leerlauf beim Laden: so lange darf zwischen zwei Datenpaketen nichts kommen.
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(60);

/// Einstellungen des Agenten (prüfbar ohne Netz).
pub fn config() -> ureq::config::Config {
    ureq::Agent::config_builder()
        .https_only(true)
        .max_redirects(0)
        .http_status_as_error(false)
        .save_redirect_history(false)
        .user_agent(USER_AGENT)
        .timeout_resolve(Some(CONNECT_TIMEOUT))
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_send_request(Some(CONNECT_TIMEOUT))
        .timeout_recv_response(Some(CONNECT_TIMEOUT))
        .timeout_per_read(Some(IDLE_TIMEOUT))
        .timeout_per_write(Some(CONNECT_TIMEOUT))
        .build()
}

/// Transport über ureq.
pub struct UreqTransport {
    agent: ureq::Agent,
}

impl UreqTransport {
    pub fn new() -> UreqTransport {
        UreqTransport {
            agent: ureq::Agent::new_with_config(config()),
        }
    }
}

impl Default for UreqTransport {
    fn default() -> UreqTransport {
        UreqTransport::new()
    }
}

/// Fehler von ureq als Transportfehler: kein Netz, Zeitgrenze oder sonst gescheitert (TLS, Protokoll).
pub fn map_error(e: &ureq::Error) -> TransportError {
    match e {
        ureq::Error::Timeout(_) | ureq::Error::BodyStalled => TransportError::Timeout,
        ureq::Error::HostNotFound | ureq::Error::ConnectionFailed => TransportError::Offline,
        ureq::Error::Io(io) => match io.kind() {
            ErrorKind::TimedOut | ErrorKind::WouldBlock => TransportError::Timeout,
            ErrorKind::ConnectionRefused
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::NotConnected
            | ErrorKind::AddrNotAvailable
            | ErrorKind::NetworkUnreachable
            | ErrorKind::HostUnreachable
            | ErrorKind::NetworkDown => TransportError::Offline,
            _ => TransportError::Failed,
        },
        _ => TransportError::Failed,
    }
}

/// Art der Antwort aus Status und `Location`-Kopf. 3xx ohne `Location` ist ein gewöhnlicher Status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Body,
    Redirect(String),
    Status(u16),
}

pub fn kind(status: u16, location: Option<&str>) -> Kind {
    match status {
        200..=299 => Kind::Body,
        300..=399 => location.map_or(Kind::Status(status), |l| Kind::Redirect(l.to_owned())),
        _ => Kind::Status(status),
    }
}

impl Transport for UreqTransport {
    fn get(&mut self, url: &HttpsUrl) -> Result<Reply<'_>, TransportError> {
        let response = self
            .agent
            .get(url.as_str())
            .call()
            .map_err(|e| map_error(&e))?;
        let status = response.status().as_u16();
        let location = response
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        Ok(match kind(status, location.as_deref()) {
            Kind::Redirect(to) => Reply::Redirect(to),
            Kind::Status(code) => Reply::Status(code),
            Kind::Body => {
                let body = response.into_body();
                let length = body.content_length();
                Reply::Body {
                    length,
                    reader: Box::new(body.into_reader()),
                }
            }
        })
    }
}
