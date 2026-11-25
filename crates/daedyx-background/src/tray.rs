//! Symbol im Infobereich mit Menü „Öffnen“ und „Beenden“; Linksklick holt das Fenster (aus Spike S5).
//! Die Ereignisse kommen auf dem Haupt-Thread an (Nachrichtenschleife von winit) und gehen über [`Sink`] in
//! denselben Kanal wie die Hintergrund-Ereignisse; so läuft `logic` auch bei verstecktem Fenster.
use crate::sink::{Event, Sink};
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

/// Seitenlänge des Symbols in Pixeln.
pub const ICON_SIZE: u32 = 32;

/// Muss am Leben bleiben, sonst verschwindet das Symbol.
pub struct Tray {
    _icon: TrayIcon,
}

/// Texte des Infobereichs (aus der Sprachdatei).
pub struct TrayTexts {
    pub tooltip: String,
    pub open: String,
    pub quit: String,
}

/// Reine Zuordnung, damit sie ohne Infobereich testbar ist.
pub fn map_click(button: MouseButton, state: MouseButtonState) -> Option<Event> {
    (button == MouseButton::Left && state == MouseButtonState::Up).then_some(Event::Show)
}

pub fn map_menu(id: &MenuId, open: &MenuId, quit: &MenuId) -> Option<Event> {
    if id == open {
        Some(Event::Show)
    } else if id == quit {
        Some(Event::Quit)
    } else {
        None
    }
}

/// Gefüllter Kreis in der Akzentfarbe (aus den Tokens), außen durchsichtig. Kein Bild aus fremder Quelle.
pub fn icon_rgba(size: u32, rgba: [u8; 4]) -> Vec<u8> {
    let side = usize::try_from(size).unwrap_or(0);
    let mut px = Vec::with_capacity(side.saturating_mul(side).saturating_mul(4));
    let c = size as f32 / 2.0;
    let r2 = (c - 1.0) * (c - 1.0);
    for y in 0..size {
        for x in 0..size {
            let (dx, dy) = (x as f32 + 0.5 - c, y as f32 + 0.5 - c);
            let inside = dx * dx + dy * dy <= r2;
            px.extend_from_slice(if inside { &rgba } else { &[0, 0, 0, 0] });
        }
    }
    px
}

/// Fehler ohne Text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayError {
    Icon,
    Build,
}

pub fn create(sink: &Sink, texts: &TrayTexts, rgba: [u8; 4]) -> Result<Tray, TrayError> {
    let open = MenuItem::new(&texts.open, true, None);
    let quit = MenuItem::new(&texts.quit, true, None);
    let menu = Menu::new();
    // Ein Fehler beim Menü ist kein Grund, ohne Symbol zu laufen: dann eben ohne Menü.
    let _ = menu.append_items(&[&open, &quit]);
    let (open_id, quit_id) = (open.id().clone(), quit.id().clone());
    let s = sink.clone();
    MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
        if let Some(ev) = map_menu(&e.id, &open_id, &quit_id) {
            s.emit(ev);
        }
    }));
    let s = sink.clone();
    TrayIconEvent::set_event_handler(Some(move |e: TrayIconEvent| {
        if let TrayIconEvent::Click {
            button,
            button_state,
            ..
        } = e
            && let Some(ev) = map_click(button, button_state)
        {
            s.emit(ev);
        }
    }));
    let icon = Icon::from_rgba(icon_rgba(ICON_SIZE, rgba), ICON_SIZE, ICON_SIZE)
        .map_err(|_| TrayError::Icon)?;
    let tray = TrayIconBuilder::new()
        .with_tooltip(&texts.tooltip)
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .build()
        .map_err(|_| TrayError::Build)?;
    Ok(Tray { _icon: tray })
}
