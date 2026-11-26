//! Netz ohne Netz geprüft: Einstellungen des Agenten, Fehlerzuordnung und Antwortarten (Review U1-R, H-U1-5).
use crate::net::{CONNECT_TIMEOUT, IDLE_TIMEOUT, Kind, USER_AGENT, config, kind, map_error};
use daedyx_update::fetch::TransportError;
use std::io;

#[test]
fn der_transport_folgt_keiner_weiterleitung_selbst() {
    let c = config();
    assert_eq!(
        c.max_redirects(),
        0,
        "Weiterleitungen prüft der Kern je Schritt"
    );
    assert!(c.https_only());
    assert!(
        !c.http_status_as_error(),
        "404/500 kommen als Status zurück"
    );
    assert!(!c.save_redirect_history());
}

#[test]
fn zeitgrenzen_und_user_agent_sind_fest() {
    let c = config();
    let t = c.timeouts();
    assert_eq!(
        c.timeout_per_read(),
        Some(IDLE_TIMEOUT),
        "60 s ohne Datenfluss"
    );
    assert_eq!(IDLE_TIMEOUT.as_secs(), 60);
    assert_eq!(t.connect, Some(CONNECT_TIMEOUT));
    assert_eq!(t.resolve, Some(CONNECT_TIMEOUT));
    assert_eq!(t.recv_response, Some(CONNECT_TIMEOUT));
    assert_eq!(CONNECT_TIMEOUT.as_secs(), 15);
    assert_eq!(
        t.global, None,
        "keine Gesamtgrenze, damit langsame Leitungen klappen"
    );
    assert_eq!(t.recv_body, None);
    assert_eq!(USER_AGENT, "daedyx-update");
}

#[test]
fn weiterleitung_nur_mit_location() {
    let to = "https://objects.githubusercontent.com/x";
    assert_eq!(kind(302, Some(to)), Kind::Redirect(to.to_owned()));
    assert_eq!(kind(301, None), Kind::Status(301));
    assert_eq!(kind(200, Some(to)), Kind::Body);
    assert_eq!(kind(204, None), Kind::Body);
    assert_eq!(kind(404, None), Kind::Status(404));
    assert_eq!(kind(500, Some(to)), Kind::Status(500));
    assert_eq!(kind(101, None), Kind::Status(101));
}

#[test]
fn fehler_werden_als_offline_zeitgrenze_oder_gescheitert_gemeldet() {
    use ureq::Error as E;
    assert_eq!(
        map_error(&E::Timeout(ureq::Timeout::Connect)),
        TransportError::Timeout
    );
    assert_eq!(
        map_error(&E::Timeout(ureq::Timeout::PerRead)),
        TransportError::Timeout
    );
    assert_eq!(map_error(&E::BodyStalled), TransportError::Timeout);
    assert_eq!(map_error(&E::HostNotFound), TransportError::Offline);
    assert_eq!(map_error(&E::ConnectionFailed), TransportError::Offline);
    let io_err = |k| E::Io(io::Error::from(k));
    assert_eq!(
        map_error(&io_err(io::ErrorKind::ConnectionRefused)),
        TransportError::Offline
    );
    assert_eq!(
        map_error(&io_err(io::ErrorKind::NetworkUnreachable)),
        TransportError::Offline
    );
    assert_eq!(
        map_error(&io_err(io::ErrorKind::TimedOut)),
        TransportError::Timeout
    );
    assert_eq!(
        map_error(&io_err(io::ErrorKind::InvalidData)),
        TransportError::Failed
    );
    assert_eq!(map_error(&E::Tls("kaputt")), TransportError::Failed);
    assert_eq!(
        map_error(&E::RequireHttpsOnly("http://x".into())),
        TransportError::Failed
    );
    assert_eq!(map_error(&E::TooManyRedirects), TransportError::Failed);
}
