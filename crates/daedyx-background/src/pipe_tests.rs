//! Einzelinstanz über die benannte Pipe (nur Windows): zweiter Start ist sekundär und weckt den ersten, `QUIT`
//! kommt an, fremde Wörter werden verworfen, nach dem Ende der ersten Instanz ist der nächste Start wieder primär.
//! Jeder Test hat einen eigenen Pipe-Namen (nie den einer laufenden App).
//! Mit Desk umgezogen (Tasks T1b).
use crate::instance::{QUIT, Role, SHOW, acquire, notify_primary, word_to_event};
use crate::{Event, Sink};
use std::sync::mpsc::Receiver;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

fn test_name(tag: &str) -> String {
    format!(
        r"\\.\pipe\daedyx-background-test-{tag}-{}",
        std::process::id()
    )
}

async fn collect(rx: &Receiver<Event>, want: usize) -> Vec<Event> {
    let mut got = Vec::new();
    for _ in 0..100 {
        got.extend(rx.try_iter());
        if got.len() >= want {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    got
}

#[test]
fn nur_feste_woerter() {
    assert_eq!(word_to_event(b"SHOW"), Some(Event::Show));
    assert_eq!(word_to_event(b"QUIT"), Some(Event::Quit));
    assert_eq!(word_to_event(b"show"), None);
    assert_eq!(word_to_event(b""), None);
    assert_eq!(word_to_event(b"SHOWX"), None);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn zweiter_start_weckt_den_ersten_und_quit_kommt_an() {
    let name = test_name("weck");
    let Role::Primary(listener) = acquire(&name).unwrap() else {
        panic!("erster Start muss primär sein")
    };
    assert!(matches!(acquire(&name).unwrap(), Role::Secondary));
    let (sink, rx) = Sink::new();
    let serve = tokio::spawn(listener.serve(sink));
    notify_primary(&name, SHOW).await.unwrap();
    notify_primary(&name, QUIT).await.unwrap();
    let got = collect(&rx, 2).await;
    serve.abort();
    // Zwei Verbindungen werden nebenläufig bedient: die Reihenfolge ist nicht festgelegt.
    assert_eq!(got.len(), 2);
    assert!(got.contains(&Event::Show) && got.contains(&Event::Quit));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fremdes_wort_wird_verworfen() {
    let name = test_name("fremd");
    let Role::Primary(listener) = acquire(&name).unwrap() else {
        panic!("erster Start muss primär sein")
    };
    let (sink, rx) = Sink::new();
    let serve = tokio::spawn(listener.serve(sink));
    let mut c = tokio::net::windows::named_pipe::ClientOptions::new()
        .open(&name)
        .unwrap();
    c.write_all(b"DELETE").await.unwrap();
    drop(c);
    tokio::time::sleep(Duration::from_millis(200)).await;
    serve.abort();
    assert!(rx.try_iter().next().is_none());
}

#[tokio::test]
async fn nach_ende_der_ersten_instanz_ist_der_naechste_start_primaer() {
    let name = test_name("ende");
    let first = acquire(&name).unwrap();
    drop(first);
    assert!(matches!(acquire(&name).unwrap(), Role::Primary(_)));
}

/// Schutzregel: Eine erste Instanz, die die Pipe hält, aber nie liest (z. B. weil sie selbst hängt), darf den
/// zweiten Start nicht blockieren. Er kehrt in begrenzter Zeit zurück (Erfolg oder Fehler, nie endlos).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn haengende_erste_instanz_blockiert_den_zweiten_start_nicht() {
    let name = test_name("liest-nie");
    let Role::Primary(listener) = acquire(&name).unwrap() else {
        panic!("erster Start muss primär sein")
    };
    let t0 = std::time::Instant::now();
    let r = tokio::time::timeout(Duration::from_secs(10), notify_primary(&name, SHOW)).await;
    drop(listener);
    assert!(r.is_ok(), "zweiter Start wartet endlos");
    assert!(t0.elapsed() < Duration::from_secs(5));
}

/// Schutzregel: Ist die einzige Pipe-Instanz belegt (ein anderer Client hängt daran), gibt der zweite Start nach
/// der Zeitgrenze mit Fehler auf, statt zu warten.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn belegte_pipe_fuehrt_zu_fehler_nach_zeitgrenze() {
    let name = test_name("belegt");
    let Role::Primary(listener) = acquire(&name).unwrap() else {
        panic!("erster Start muss primär sein")
    };
    let _blocker = tokio::net::windows::named_pipe::ClientOptions::new()
        .open(&name)
        .unwrap();
    let t0 = std::time::Instant::now();
    let r = tokio::time::timeout(Duration::from_secs(10), notify_primary(&name, SHOW)).await;
    drop(listener);
    assert!(matches!(r, Ok(Err(_))), "Fehler nach Zeitgrenze erwartet");
    assert!(t0.elapsed() < Duration::from_secs(5));
}

/// Review M-1: Ein Programm verbindet sich und legt sofort wieder auf (auch wiederholt). Die Annahmeschleife darf
/// sich daran nicht festlaufen; danach muss `SHOW` noch ankommen.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn abgebrochene_verbindungen_legen_die_pipe_nicht_lahm() {
    let name = test_name("abbruch");
    let Role::Primary(listener) = acquire(&name).unwrap() else {
        panic!("erster Start muss primär sein")
    };
    // Vor dem ersten `connect`: verbinden und auflegen (Windows meldet dann ERROR_NO_DATA).
    drop(
        tokio::net::windows::named_pipe::ClientOptions::new()
            .open(&name)
            .unwrap(),
    );
    let (sink, rx) = Sink::new();
    let serve = tokio::spawn(listener.serve(sink));
    for _ in 0..20 {
        if let Ok(c) = tokio::net::windows::named_pipe::ClientOptions::new().open(&name) {
            drop(c);
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let sent = notify_primary(&name, SHOW).await;
    let got = collect(&rx, 1).await;
    serve.abort();
    assert!(
        sent.is_ok(),
        "zweiter Start erreicht die erste Instanz nicht mehr"
    );
    assert_eq!(got, vec![Event::Show]);
}

/// Eine in Stücken geschickte Nachricht kommt trotzdem an; eine zu lange wird verworfen.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn geteilte_nachricht_kommt_an_zu_lange_nicht() {
    let name = test_name("stuecke");
    let Role::Primary(listener) = acquire(&name).unwrap() else {
        panic!("erster Start muss primär sein")
    };
    let (sink, rx) = Sink::new();
    let serve = tokio::spawn(listener.serve(sink));
    let open = || tokio::net::windows::named_pipe::ClientOptions::new().open(&name);
    let mut c = open().unwrap();
    c.write_all(b"SH").await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;
    let _ = c.write_all(b"OW").await;
    drop(c);
    let first = collect(&rx, 1).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    let mut long = open().unwrap();
    // Der Server legt nach seiner Grenze auf; das Schreiben darf dann scheitern.
    let _ = long.write_all(b"SHOWSHOWSHOWSHOWSHOW").await;
    drop(long);
    tokio::time::sleep(Duration::from_millis(300)).await;
    let rest: Vec<Event> = rx.try_iter().collect();
    serve.abort();
    assert_eq!(first, vec![Event::Show], "geteilte Nachricht");
    assert!(rest.is_empty(), "zu lange Nachricht verworfen: {rest:?}");
}
