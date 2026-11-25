//! Tasks T4: Schließen ohne Infobereich beendet (Schalter „Im Infobereich weiterlaufen“, T-F9) und Erinnerungen mit
//! Vorlauf („Fällig in 10 Minuten“, T-F6 neutral ohne Titel).
use crate::ticker::render;
use crate::window::WindowState;
use crate::{Event, Notice, NoticeKind};
use daedyx_ui::i18n::Language;
use eframe::egui::ViewportCommand;

#[test]
fn schliessen_ohne_infobereich_beendet_nach_dem_sichern() {
    let mut w = WindowState::default();
    assert!(w.keep_running(), "Standard: im Infobereich weiterlaufen");
    w.set_keep_running(false);
    let cmds = w.on_close_requested();
    assert_eq!(cmds, vec![ViewportCommand::CancelClose]);
    assert!(
        w.quit_wanted && !w.hidden && !w.quitting,
        "erst sichert die App den Entwurf"
    );
    assert_eq!(w.finish_quit(), vec![ViewportCommand::Close]);
    assert!(w.on_close_requested().is_empty(), "danach wirklich zu");
    w.set_keep_running(true);
    assert!(w.keep_running());
}

#[test]
fn schalter_wieder_an_versteckt_wie_bisher() {
    let mut w = WindowState::default();
    w.set_keep_running(false);
    w.set_keep_running(true);
    assert_eq!(
        w.on_close_requested(),
        vec![
            ViewportCommand::CancelClose,
            ViewportCommand::Visible(false)
        ]
    );
    assert!(w.hidden && !w.quit_wanted);
    assert!(!w.apply(&Event::Show).is_empty());
}

fn lead(minutes_until: i64) -> Notice {
    Notice {
        kind: NoticeKind::TaskLead { minutes_until },
        title: "Wasserzähler ablesen".into(),
        time: String::new(),
    }
}

fn body(language: Language, minutes: i64) -> String {
    render(&crate::texts::texts(language), &lead(minutes), false).1
}

#[test]
fn faellig_in_minuten_stunden_tagen() {
    let de = Language::De;
    for (m, want) in [
        (i64::MIN, "Seit 9223372036854775807 Minuten fällig"),
        (-90, "Seit 90 Minuten fällig"),
        (-5, "Seit 5 Minuten fällig"),
        (-1, "Seit 1 Minute fällig"),
        (0, "Jetzt fällig"),
        (1, "Fällig in 1 Minute"),
        (5, "Fällig in 5 Minuten"),
        (59, "Fällig in 59 Minuten"),
        (60, "Fällig in 1 Stunde"),
        (119, "Fällig in 1 Stunde"),
        (120, "Fällig in 2 Stunden"),
        (1439, "Fällig in 24 Stunden"),
        (1440, "Fällig in 1 Tag"),
        (2880, "Fällig in 2 Tagen"),
        (10_080, "Fällig in 7 Tagen"),
        (i64::MAX, "Fällig in 6405119470038039 Tagen"),
    ] {
        assert_eq!(body(de, m), want, "{m}");
    }
    assert_eq!(body(Language::En, 10), "Due in 10 minutes");
    assert_eq!(body(Language::En, 60), "Due in 1 hour");
    assert_eq!(body(Language::Fr, 2880), "À faire dans 2 jours");
    assert_eq!(body(Language::Fr, 0), "À faire maintenant");
    assert_eq!(body(Language::En, -3), "Overdue by 3 minutes");
    assert_eq!(body(Language::En, 1), "Due in 1 minute");
    assert_eq!(body(Language::Fr, -1), "En retard de 1 minute");
    assert_eq!(body(Language::Fr, -7), "En retard de 7 minutes");
}

#[test]
fn vorlauf_standard_ohne_titel() {
    let t = crate::texts::texts(Language::De);
    let (title, body) = render(&t, &lead(10), false);
    assert_eq!(title, "Erinnerung an eine Aufgabe");
    assert!(!title.contains("Wasserzähler") && !body.contains("Wasserzähler"));
    assert_eq!(body, "Fällig in 10 Minuten");
    let (title, _) = render(&t, &lead(10), true);
    assert_eq!(title, "Aufgabe: Wasserzähler ablesen");
}

#[test]
fn pipe_dienst_versucht_mit_pause_erneut() {
    // Review T4, H4-1: Ein Fehler beim Anlegen beendet den Dienst nicht, es folgt ein neuer Versuch mit Pause.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    let mut tries = 0;
    let started = std::time::Instant::now();
    let got: u32 = rt.block_on(crate::instance::retry_with_pause(|| {
        tries += 1;
        if tries < 4 { Err(()) } else { Ok(7) }
    }));
    assert_eq!((got, tries), (7, 4));
    assert!(
        started.elapsed() >= std::time::Duration::from_millis(50 + 100 + 200 + 400),
        "wachsende Pause"
    );
}
