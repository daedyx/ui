//! Bausteine des Hintergrundbetriebs: Befehlszeile, Autostart (nur unter einem eigenen Testschlüssel),
//! Fenster-Zustand, Infobereich-Zuordnung. Mit Desk umgezogen (Tasks T1b); der Test der Desk-Kennungen blieb in Desk.
use crate::Event;
use crate::args::{Args, TAKT_DEFAULT};
use crate::autostart::{AutostartStore, State, approved, command_line};
use crate::tray::{ICON_SIZE, icon_rgba, map_click, map_menu};
use crate::window::WindowState;
use eframe::egui::ViewportCommand;
use std::path::Path;
use tray_icon::menu::MenuId;
use tray_icon::{MouseButton, MouseButtonState};

fn args(v: &[&str]) -> Args {
    Args::parse(v.iter().map(|s| (*s).to_owned()))
}

#[test]
fn befehlszeile_versteckt_takt_und_unsinn() {
    assert_eq!(args(&[]), Args::default());
    assert_eq!(args(&[]).takt_secs, TAKT_DEFAULT);
    let a = args(&["--hidden", "--takt", "10", "--unbekannt"]);
    assert!(a.hidden && !a.quit_running && a.takt_secs == 10);
    assert_eq!(args(&["--takt", "1"]).takt_secs, 5, "untere Grenze");
    assert_eq!(args(&["--takt", "999999"]).takt_secs, 3600, "obere Grenze");
    assert_eq!(args(&["--takt", "x"]).takt_secs, TAKT_DEFAULT);
    assert!(args(&["--quit-running"]).quit_running);
    assert!(args(&["--cleanup"]).cleanup && args(&["--probe-toast"]).probe_toast);
    assert!(args(&["--start-probe"]).start_probe && !args(&[]).start_probe);
}

#[test]
fn autostart_befehlszeile_und_sperre_im_task_manager() {
    let line = command_line(Path::new("/Programme/Daedyx Test/vorschau.exe")).unwrap();
    assert_eq!(line, r#""/Programme/Daedyx Test/vorschau.exe" --hidden"#);
    assert!(command_line(Path::new("/a\"b.exe")).is_err());
    assert!(approved(None) && approved(Some(&[2, 0])) && approved(Some(&[6])));
    assert!(!approved(Some(&[3, 0])) && !approved(Some(&[7])));
}

/// Schreibt nur unter `HKCU\Software\DaedyxBackgroundTest\<pid>`, nie unter `...\Run`, und löscht ihn danach.
#[cfg(windows)]
#[test]
fn autostart_ein_aus_und_blockiert_unter_testschluessel() {
    use crate::autostart::RegistryStore;
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, REG_BINARY};
    let base = format!(r"Software\DaedyxBackgroundTest\{}", std::process::id());
    let mut store = RegistryStore {
        run_key: format!(r"{base}\Run"),
        approved_key: format!(r"{base}\Approved"),
        value: "Probe".into(),
        command: r#""/x/vorschau.exe" --hidden"#.into(),
    };
    assert_eq!(store.state().unwrap(), State::Off);
    store.set(true).unwrap();
    assert_eq!(store.state().unwrap(), State::On);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (ap, _) = hkcu.create_subkey(&store.approved_key).unwrap();
    let blocked = winreg::RegValue {
        bytes: vec![3, 0, 0, 0].into(),
        vtype: REG_BINARY,
    };
    ap.set_raw_value("Probe", &blocked).unwrap();
    assert_eq!(store.state().unwrap(), State::Blocked);
    store.set(false).unwrap();
    store.set(false).unwrap();
    assert_eq!(store.state().unwrap(), State::Off);
    hkcu.delete_subkey_all(&base).unwrap();
    let _ = hkcu.delete_subkey(r"Software\DaedyxBackgroundTest");
}

#[test]
fn schliessen_versteckt_und_beenden_beendet() {
    let mut w = WindowState::default();
    let cmds = w.on_close_requested();
    assert_eq!(
        cmds,
        vec![
            ViewportCommand::CancelClose,
            ViewportCommand::Visible(false)
        ]
    );
    assert!(w.hidden);
    let show = w.apply(&Event::Show);
    assert!(!w.hidden && show.contains(&ViewportCommand::Visible(true)));
    assert!(show.contains(&ViewportCommand::Focus));
    // „Beenden“ wartet erst auf die App (Entwurf sichern, Review int2 M-2), dann schließt es wirklich.
    assert!(w.apply(&Event::Quit).is_empty());
    assert!(w.quit_wanted && !w.quitting);
    assert_eq!(w.finish_quit(), vec![ViewportCommand::Close]);
    assert!(w.quitting && !w.quit_wanted);
    assert!(
        w.on_close_requested().is_empty(),
        "nach „Beenden“ wird nicht mehr abgefangen"
    );
    let mut start = WindowState::default();
    start.start_hidden();
    assert!(start.hidden);
}

#[test]
fn infobereich_linksklick_und_menue() {
    assert_eq!(
        map_click(MouseButton::Left, MouseButtonState::Up),
        Some(Event::Show)
    );
    assert_eq!(map_click(MouseButton::Left, MouseButtonState::Down), None);
    assert_eq!(map_click(MouseButton::Right, MouseButtonState::Up), None);
    let (o, q, x) = (MenuId::new("o"), MenuId::new("q"), MenuId::new("x"));
    assert_eq!(map_menu(&o, &o, &q), Some(Event::Show));
    assert_eq!(map_menu(&q, &o, &q), Some(Event::Quit));
    assert_eq!(map_menu(&x, &o, &q), None);
    let px = icon_rgba(ICON_SIZE, [1, 2, 3, 255]);
    assert_eq!(px.len(), 32 * 32 * 4);
    let mid = (16 * 32 + 16) * 4;
    assert_eq!(
        &px[mid..mid + 4],
        &[1, 2, 3, 255],
        "Mitte in der Akzentfarbe"
    );
    assert_eq!(&px[0..4], &[0, 0, 0, 0], "Ecke durchsichtig");
}
