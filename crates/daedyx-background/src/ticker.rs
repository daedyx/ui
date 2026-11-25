//! Hintergrund-Takt: läuft auf einem eigenen Thread, unabhängig davon, ob das Fenster sichtbar ist. Je Durchgang:
//! fällige Erinnerungen holen (was genau, bestimmt die App mit [`Jobs`]; Desk: Termine und Aufgaben), als
//! Benachrichtigung zeigen und der Oberfläche melden; dazu der Abgleich-Check (welche Quellen wären fällig).
use crate::notify::Notifier;
use crate::sink::{Event, Notice, NoticeKind, Sink};
pub use crate::style::{NoticeStyle, SharedStyle};
use daedyx_ui::i18n::Texts;
use std::sync::Arc;
use std::sync::mpsc::{RecvTimeoutError, Sender, channel};
use std::thread::JoinHandle;
use std::time::Duration;

/// Ergebnis eines Durchgangs.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Tick {
    pub notices: Vec<Notice>,
    pub sync_due: usize,
}

/// Was je Durchgang zu tun ist (Tests können es ersetzen).
pub trait Jobs: Send + Sync {
    fn run(&self) -> Tick;
}

/// Titel und Text der Benachrichtigung (aus der Sprachdatei). Ohne `show_titles` neutral, nur mit der Uhrzeit.
pub fn render(texts: &Texts, n: &Notice, show_titles: bool) -> (String, String) {
    let (title, at, none) = match n.kind {
        NoticeKind::TaskLead { minutes_until } => {
            let title = if show_titles {
                "hintergrund.aufgabe.titel"
            } else {
                "hintergrund.aufgabe.neutral"
            };
            return (
                texts.tf(title, &[("titel", &n.title)]),
                lead_text(texts, minutes_until),
            );
        }
        NoticeKind::Event => (
            if show_titles {
                "hintergrund.termin.titel"
            } else {
                "hintergrund.termin.neutral"
            },
            "hintergrund.termin.um",
            "hintergrund.termin.ganztags",
        ),
        NoticeKind::Task => (
            if show_titles {
                "hintergrund.aufgabe.titel"
            } else {
                "hintergrund.aufgabe.neutral"
            },
            "hintergrund.aufgabe.um",
            "hintergrund.aufgabe.heute",
        ),
    };
    let body = if n.time.is_empty() {
        texts.t(none)
    } else {
        texts.tf(at, &[("zeit", &n.time)])
    };
    (texts.tf(title, &[("titel", &n.title)]), body)
}

/// Ganzzahlige Division mit Runden zur Hälfte nach oben (wie `Math.round` der Tauri-Fassung), ohne Überlauf.
fn rounded(m: i64, unit: i64) -> i64 {
    let whole = m.checked_div(unit).unwrap_or(0);
    let rest = m.checked_rem(unit).unwrap_or(0);
    if rest.saturating_mul(2) >= unit {
        whole.saturating_add(1)
    } else {
        whole
    }
}

/// Text zur Restzeit einer Erinnerung (Tasks): seit n Minuten, jetzt, oder Minuten, Stunden, Tage bis zur Fälligkeit.
fn lead_text(texts: &Texts, m: i64) -> String {
    const HOUR: i64 = 60;
    const DAY: i64 = 1440;
    const TWO_HOURS: i64 = 120;
    const TWO_DAYS: i64 = 2880;
    let n = |v: i64| v.to_string();
    if m == -1 {
        texts.t("hintergrund.aufgabe.seit_einer_minute")
    } else if m < 0 {
        texts.tf(
            "hintergrund.aufgabe.seit_minuten",
            &[("n", &n(m.saturating_neg()))],
        )
    } else if m == 0 {
        texts.t("hintergrund.aufgabe.jetzt")
    } else if m == 1 {
        texts.t("hintergrund.aufgabe.in_einer_minute")
    } else if m < HOUR {
        texts.tf("hintergrund.aufgabe.in_minuten", &[("n", &n(m))])
    } else if m < TWO_HOURS {
        texts.t("hintergrund.aufgabe.in_einer_stunde")
    } else if m < DAY {
        texts.tf(
            "hintergrund.aufgabe.in_stunden",
            &[("n", &n(rounded(m, HOUR)))],
        )
    } else if m < TWO_DAYS {
        texts.t("hintergrund.aufgabe.in_einem_tag")
    } else {
        texts.tf(
            "hintergrund.aufgabe.in_tagen",
            &[("n", &n(rounded(m, DAY)))],
        )
    }
}

/// Ein Durchgang mit Benachrichtigungen und Meldungen an die Oberfläche.
pub fn run_once(jobs: &dyn Jobs, notifier: &dyn Notifier, sink: &Sink, style: NoticeStyle) {
    let tick = jobs.run();
    let texts = crate::texts::texts(style.language);
    for notice in tick.notices {
        let (title, body) = render(&texts, &notice, style.show_titles);
        let shown = notifier.notify(&title, &body).is_ok();
        sink.emit(Event::Notice { notice, shown });
    }
    sink.emit(Event::Tick {
        sync_due: tick.sync_due,
    });
}

/// Läuft, bis es fallen gelassen wird (dann endet der Thread beim nächsten Warten sofort).
pub struct Ticker {
    stop: Option<Sender<()>>,
    handle: Option<JoinHandle<()>>,
}

impl Ticker {
    pub fn spawn(
        every: Duration,
        jobs: Arc<dyn Jobs>,
        notifier: Arc<dyn Notifier>,
        sink: Sink,
        style: Arc<SharedStyle>,
    ) -> Ticker {
        let (stop, wait) = channel::<()>();
        let handle = std::thread::Builder::new()
            .name("daedyx-takt".into())
            .spawn(move || {
                loop {
                    let now = style.get();
                    run_once(jobs.as_ref(), notifier.as_ref(), &sink, now);
                    match wait.recv_timeout(every) {
                        Err(RecvTimeoutError::Timeout) => {}
                        _ => break,
                    }
                }
            })
            .ok();
        Ticker {
            stop: Some(stop),
            handle,
        }
    }
}

/// So lange wartet das Beenden höchstens auf einen laufenden Durchgang.
const JOIN_LIMIT: Duration = Duration::from_secs(2);

impl Drop for Ticker {
    /// Hält den Takt an. Hängt ein Durchgang (z. B. eine Benachrichtigung kehrt nicht zurück), wird nach
    /// `JOIN_LIMIT` nicht weiter gewartet: Der Thread läuft aus, das Ende des Prozesses beendet ihn.
    fn drop(&mut self) {
        drop(self.stop.take());
        if let Some(h) = self.handle.take() {
            let start = std::time::Instant::now();
            while !h.is_finished() && start.elapsed() < JOIN_LIMIT {
                std::thread::sleep(Duration::from_millis(10));
            }
            if h.is_finished() {
                let _ = h.join();
            }
        }
    }
}
