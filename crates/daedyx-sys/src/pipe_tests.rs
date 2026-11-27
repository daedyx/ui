//! Gehärtete Pipe der Einzelinstanz (Entscheid T4-7): Rechte nur für den eigenen Benutzer, keine Fernverbindung,
//! Besitzerprüfung beim Verbinden. Jede Härtung hat eine Gegenprobe mit einer ungehärteten Pipe, damit der Test
//! selbst beweist, dass er den Unterschied sieht. Jeder Test hat einen eigenen Pipe-Namen.
use super::probe::as_anonymous;
use super::probe::security_text;
use super::*;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{ClientOptions, ServerOptions};

fn name(tag: &str) -> String {
    format!(r"\\.\pipe\daedyx-sys-test-{tag}-{}", std::process::id())
}

/// Erfundene, gültig geformte SID eines anderen Kontos.
const FOREIGN_SID: &str = "S-1-5-21-1000000001-1000000002-1000000003-1001";

#[test]
fn die_eigene_sid_ist_gueltig_und_stabil() {
    let sid = user_sid().unwrap();
    assert!(sid.starts_with("S-1-5-"));
    assert!(is_sid(&sid));
    assert_eq!(user_sid().unwrap(), sid);
}

#[test]
fn nur_echte_sid_formen_gelten() {
    assert!(is_sid(FOREIGN_SID));
    for bad in [
        "",
        "S-1",
        "S-1-5-",
        "s-1-5-18",
        "S-1-5-18)(A;;GA;;;WD",
        "S-1-5-18 ",
        "S-1-5--18",
        "S-1-5-1x",
    ] {
        assert!(!is_sid(bad), "{bad}");
    }
}

#[test]
fn die_beschreibung_nennt_nur_den_eigenen_benutzer() {
    let sddl = owner_only_sddl(FOREIGN_SID).unwrap();
    assert_eq!(
        sddl,
        format!("O:{FOREIGN_SID}G:{FOREIGN_SID}D:P(A;;GA;;;{FOREIGN_SID})")
    );
    assert!(owner_only_sddl("S-1-5-18)(A;;GA;;;WD").is_err());
}

#[tokio::test]
async fn die_pipe_erlaubt_nur_dem_eigenen_benutzer_zugriff() {
    let sid = user_sid().unwrap();
    let hard = create_server(&name("dacl"), true).unwrap();
    // Über die SIDs, nicht über SDDL-Text: Der schreibt etwa das eingebaute Administratorkonto als „LA“ (pipe_acl).
    let found = acl::of_object(&hard, ffi::OWNER_AND_DACL).unwrap();
    assert!(owner_only(&found, &sid), "{found:?}");
    // Gegenprobe: Ohne eigene Beschreibung bekommt jeder (auch anonym) Lesezugriff.
    let plain = ServerOptions::new().create(name("dacl-plain")).unwrap();
    let open = security_text(&plain, ffi::OWNER_AND_DACL).unwrap();
    assert!(open.contains(";;;WD)") || open.contains(";;;AN)"), "{open}");
}

#[tokio::test]
async fn ein_fremdes_konto_kann_nicht_verbinden() {
    let hard_name = name("anon");
    let _hard = create_server(&hard_name, true).unwrap();
    let denied = as_anonymous(|| ClientOptions::new().write(false).open(&hard_name)).unwrap();
    assert_eq!(
        denied.unwrap_err().kind(),
        std::io::ErrorKind::PermissionDenied
    );
    // Gegenprobe: An eine ungehärtete Pipe kommt das anonyme Konto lesend heran.
    let plain_name = name("anon-plain");
    let _plain = ServerOptions::new().create(&plain_name).unwrap();
    let allowed = as_anonymous(|| ClientOptions::new().write(false).open(&plain_name)).unwrap();
    assert!(allowed.is_ok());
}

/// Über `\\127.0.0.1\pipe\…` (Loopback) läuft die Verbindung wie von fern (über den Server-Dienst).
/// Die Gegenprobe hat dieselben Rechte und unterscheidet sich nur im Merkmal (Review N-P3).
#[tokio::test]
async fn ein_entfernter_client_wird_abgewiesen() {
    const ERROR_ACCESS_DENIED: i32 = 5;
    let hard_name = name("remote");
    let _hard = create_server(&hard_name, true).unwrap();
    let remote = |n: &str| n.replacen(r"\\.\", r"\\127.0.0.1\", 1);
    let err = ClientOptions::new().open(remote(&hard_name)).unwrap_err();
    assert_eq!(err.raw_os_error(), Some(ERROR_ACCESS_DENIED), "{err}");
    // Gegenprobe: gleiche Beschreibung (nur der eigene Benutzer), nur ohne `PIPE_REJECT_REMOTE_CLIENTS`.
    let plain_name = name("remote-plain");
    let sddl = owner_only_sddl(&user_sid().unwrap()).unwrap();
    let mut open = ServerOptions::new();
    open.first_pipe_instance(true).reject_remote_clients(false);
    let _plain = ffi::create_with_sddl(&open, &plain_name, &sddl).unwrap();
    assert!(ClientOptions::new().open(remote(&plain_name)).is_ok());
}

/// Die Besitzerprüfung liest den Besitzer des Pipe-Objekts selbst, nicht den Server-Prozess (Review N-P1).
#[tokio::test]
async fn die_besitzerpruefung_liest_den_besitzer_der_pipe() {
    let n = name("owner");
    let server = create_server(&n, true).unwrap();
    let client = ClientOptions::new().open(&n).unwrap();
    let sid = user_sid().unwrap();
    assert_eq!(pipe_owner_sid(&client).unwrap(), sid);
    assert_eq!(pipe_owner_sid(&server).unwrap(), sid);
}

/// Gibt es den Namen schon mit fremden Rechten, erbt eine weitere Instanz diese Rechte. `create_server` verwirft sie,
/// statt auf der fremden Pipe zu dienen (Review N-P2).
#[tokio::test]
async fn auf_einer_fremden_pipe_wird_keine_instanz_angelegt() {
    let n = name("squat");
    let _foreign = ServerOptions::new().create(&n).unwrap();
    let err = create_server(&n, false).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    // Gegenprobe: Ohne die Prüfung entstünde eine Instanz mit den offenen Rechten der fremden Pipe.
    let sddl = owner_only_sddl(&user_sid().unwrap()).unwrap();
    let mut next = ServerOptions::new();
    next.first_pipe_instance(false).reject_remote_clients(true);
    let unchecked = ffi::create_with_sddl(&next, &n, &sddl).unwrap();
    let text = security_text(&unchecked, ffi::OWNER_AND_DACL).unwrap();
    assert!(text.contains(";;;WD)") || text.contains(";;;AN)"), "{text}");
}

#[tokio::test]
async fn der_eigene_server_wird_erkannt_und_bekommt_das_wort() {
    let n = name("own");
    let server = create_server(&n, true).unwrap();
    let mut client = open_client(&n).unwrap();
    assert_eq!(pipe_owner_sid(&client).unwrap(), user_sid().unwrap());
    server.connect().await.unwrap();
    client.write_all(b"SHOW").await.unwrap();
    drop(client);
    let mut got = Vec::new();
    let mut server = server;
    tokio::time::timeout(Duration::from_secs(2), server.read_to_end(&mut got))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got, b"SHOW");
}

/// Gehört der Server einem anderen Benutzer als erwartet, wird nichts gesendet und der Aufrufer erfährt es.
#[tokio::test]
async fn einem_fremden_server_wird_nichts_gesendet() {
    let n = name("foreign");
    let mut server = create_server(&n, true).unwrap();
    let err = open_client_for(&n, FOREIGN_SID).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    server.connect().await.unwrap();
    let mut got = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(2), server.read_to_end(&mut got)).await;
    assert!(got.is_empty());
}

#[tokio::test]
async fn eine_zweite_erste_instanz_gibt_es_nicht() {
    let n = name("first");
    let _a = create_server(&n, true).unwrap();
    let err = create_server(&n, true).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(create_server(&n, false).is_ok());
}
