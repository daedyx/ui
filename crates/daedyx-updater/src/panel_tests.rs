//! Bauteil „Updates“ (kittest): jede Lage hat ihren Text, Knöpfe sind während des Ladens gesperrt, die Notiz ist
//! reiner Text, „von Hand laden“ nennt die Quelle ohne Link, die Statuszeile meldet nur, was es zu sagen gibt.
use crate::panel::{show, status_badge, status_text};
use crate::status::Status;
use crate::texts::texts;
use crate::worker::Updater;
use daedyx_ui::i18n::Language;
use daedyx_ui::theme;
use daedyx_ui::tokens::AppAccent;
use daedyx_update::reason::BlockKind;
use eframe::egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

fn harness(status: Status) -> Harness<'static, Updater> {
    let t = texts(Language::De);
    let mut u = Updater::disabled();
    u.set_status(status);
    let mut h = Harness::builder()
        .with_size((700.0, 500.0))
        .with_context_setup(|ctx| theme::install(ctx, AppAccent::Tasks))
        .build_ui_state(move |ui, u: &mut Updater| show(ui, u, &t), u);
    h.run();
    h
}

fn available() -> Status {
    Status::Available {
        version: "0.2.1".into(),
        notes: "Neu: [Link](https://example.invalid) bleibt Text.".into(),
        important: true,
    }
}

#[test]
fn angebot_zeigt_version_notiz_als_text_und_den_knopf() {
    let h = harness(available());
    let _ = h.get_by_label("Version 0.2.1 ist verfügbar.");
    let _ = h.get_by_label("Wichtiges Sicherheits-Update");
    let _ = h.get_by_label("Neu: [Link](https://example.invalid) bleibt Text.");
    assert!(
        h.query_by_role(Role::Link).is_none(),
        "keine Links aus der Notiz"
    );
    if crate::install::supported() {
        let _ = h.get_by_role_and_label(Role::Button, "Jetzt aktualisieren");
    }
    let _ = h.get_by_role_and_label(Role::Button, "Nach Updates suchen");
    let _ = h.get_by_label("Beim Start nach Updates suchen");
}

#[test]
fn waehrend_des_ladens_sind_die_knoepfe_gesperrt() {
    let h = harness(Status::Downloading {
        done: 50,
        total: 200,
    });
    let _ = h.get_by_label("Update wird geladen … 25 %");
    let b = h.get_by_role_and_label(Role::Button, "Nach Updates suchen");
    assert!(b.accesskit_node().is_disabled());
    assert!(h.query_by_label("Jetzt aktualisieren").is_none());
}

#[test]
fn klick_auf_suchen_startet_die_suche() {
    let mut h = harness(Status::UpToDate);
    let _ = h.get_by_label("Du hast die neueste Version.");
    h.get_by_role_and_label(Role::Button, "Nach Updates suchen")
        .click();
    h.run();
    assert_eq!(h.state().status(), &Status::Checking);
}

#[test]
fn von_hand_laden_nennt_nur_die_quelle_ohne_link() {
    let t = texts(Language::De);
    let s = Status::Blocked(BlockKind::NeedsManualUpdate);
    let text = status_text(&s, &t, "github.com/daedyx/tasks").unwrap();
    assert!(text.contains("github.com/daedyx/tasks") && !text.contains("https://"));
    assert!(
        !text.to_lowercase().contains("sicher"),
        "nie „sicher“ (Review M-U1-1)"
    );
    let h = harness(s);
    assert!(h.query_by_role(Role::Link).is_none());
}

#[test]
fn jede_lage_hat_einen_text_in_allen_sprachen() {
    let all = [
        Status::Checking,
        Status::UpToDate,
        available(),
        Status::Downloading { done: 1, total: 2 },
        Status::Ready {
            version: "0.2.1".into(),
        },
        Status::Installing,
        Status::Busy,
        Status::NotSaved,
        Status::StartFailed,
        Status::Blocked(BlockKind::Offline),
        Status::Blocked(BlockKind::SourceUnavailable),
        Status::Blocked(BlockKind::NotTrusted),
        Status::Blocked(BlockKind::NeedsManualUpdate),
        Status::Blocked(BlockKind::SourceWentBack),
        Status::Blocked(BlockKind::Interrupted),
        Status::Blocked(BlockKind::NoSpace),
        Status::Blocked(BlockKind::LocalFile),
    ];
    for lang in Language::ALL {
        let t = texts(lang);
        assert_eq!(status_text(&Status::Idle, &t, "q"), None);
        for s in &all {
            let text = status_text(s, &t, "q").unwrap();
            assert!(
                !text.contains("[[") && !text.contains('{'),
                "{lang:?} {s:?}: {text}"
            );
        }
    }
}

#[test]
fn statuszeile_meldet_angebot_und_still_keine_verbindung() {
    let t = texts(Language::De);
    // Ob ein Hinweis kommt und ob er den erwarteten Text trägt.
    let badge = |status: Status, text: &str| {
        let t = t.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        let mut u = Updater::disabled();
        u.set_status(status);
        let mut h = Harness::builder()
            .with_context_setup(|ctx| theme::install(ctx, AppAccent::Tasks))
            .build_ui_state(
                move |ui, u: &mut Updater| {
                    let _ = tx.send(status_badge(ui, u, &t).is_some());
                },
                u,
            );
        h.run();
        (
            rx.try_iter().last().unwrap(),
            h.query_by_label(text).is_some(),
        )
    };
    assert_eq!(badge(available(), "Update 0.2.1 verfügbar"), (true, true));
    let offline = "Keine Verbindung. Wir versuchen es später erneut.";
    assert_eq!(
        badge(Status::Blocked(BlockKind::Offline), offline),
        (true, true)
    );
    assert_eq!(
        badge(Status::UpToDate, "Du hast die neueste Version."),
        (false, false)
    );
    assert!(!badge(Status::Idle, "Update").0);
}
