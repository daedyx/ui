//! Einzelinstanz auf anderen Systemen (Modul `instance::other`): wird auch unter Windows übersetzt, von Clippy
//! geprüft und hier getestet (Review Desk-Tor, A-029). Heute ein Platzhalter: jeder Start ist der erste.
//! Mit Desk umgezogen (Tasks T1b).
use crate::Sink;
use crate::instance::{Role, other};

fn run<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

#[test]
fn other_systems_start_as_primary_and_accept_every_word_quietly() {
    let role: Role<other::Listener> = other::acquire("daedyx-background-test-other").unwrap();
    let Role::Primary(listener) = role else {
        panic!("ein Start auf anderen Systemen ist immer der erste")
    };
    run(other::notify_primary(
        "daedyx-background-test-other",
        b"SHOW",
    ))
    .unwrap();
    let (sink, rx) = Sink::new();
    run(listener.serve(sink));
    assert!(
        rx.try_iter().next().is_none(),
        "der Platzhalter meldet nie ein Ereignis"
    );
}
