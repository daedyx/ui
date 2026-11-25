//! Schutzregel beim Beenden: Ein hängender Takt-Durchgang blockiert das Ende der App nicht (Nachtrag M7b).
//! Mit Desk umgezogen (Tasks T1b).
use crate::Sink;
use crate::notify::Recorder;
use crate::ticker::{Jobs, NoticeStyle, SharedStyle, Tick, Ticker};
use daedyx_ui::i18n::Language;
use std::sync::Arc;
use std::time::Duration;

/// Ein Durchgang, der hängt (z. B. weil eine Benachrichtigung nicht zurückkehrt).
struct Stuck;

impl Jobs for Stuck {
    fn run(&self) -> Tick {
        std::thread::sleep(Duration::from_secs(6));
        Tick::default()
    }
}

/// Schutzregel: Das Beenden der App wartet höchstens kurz auf einen hängenden Takt und lässt ihn sonst auslaufen.
#[test]
fn haengender_takt_blockiert_das_beenden_nicht() {
    let (sink, _rx) = Sink::new();
    let ticker = Ticker::spawn(
        Duration::from_secs(60),
        Arc::new(Stuck),
        Arc::new(Recorder::default()),
        sink,
        Arc::new(SharedStyle::new(NoticeStyle::new(Language::De))),
    );
    std::thread::sleep(Duration::from_millis(100));
    let t0 = std::time::Instant::now();
    drop(ticker);
    assert!(
        t0.elapsed() < Duration::from_secs(4),
        "Beenden wartete {:?}",
        t0.elapsed()
    );
}
