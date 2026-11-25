//! Fenster: Schließen versteckt nur, „Beenden“ beendet wirklich, Start mit `--hidden` bleibt unsichtbar.
//! Zwei eframe-0.37-Eigenheiten (Spike S5): Ein per `Visible(false)` verstecktes Fenster hält egui nicht für
//! unsichtbar und zeichnet weiter – deshalb ein eigener Zustand `hidden`, bei dem das Zeichnen entfällt. Und
//! `with_visible(false)` wird beim Start übergangen – deshalb nach dem ersten Bild noch einmal verstecken.
use crate::sink::Event;
use eframe::egui::{Context, ViewportCommand};

#[derive(Debug, Default)]
pub struct WindowState {
    pub hidden: bool,
    pub quitting: bool,
    /// „Beenden“ ist angefragt; die App sichert erst den Entwurf und ruft dann [`WindowState::finish_quit`]
    /// (Review int2 M-2).
    pub quit_wanted: bool,
    /// Versteckter Start: 0 = erstes Bild abwarten, 1 = jetzt verstecken.
    start_pending: Option<u8>,
}

impl WindowState {
    /// Für den Start mit `--hidden` (Autostart).
    pub fn start_hidden(&mut self) {
        self.hidden = true;
        self.start_pending = Some(0);
    }

    pub fn hide(&mut self) -> Vec<ViewportCommand> {
        self.hidden = true;
        vec![ViewportCommand::Visible(false)]
    }

    /// Schließen-Knopf: verstecken statt beenden, außer „Beenden“ wurde gewählt.
    pub fn on_close_requested(&mut self) -> Vec<ViewportCommand> {
        if self.quitting {
            return Vec::new();
        }
        let mut cmds = vec![ViewportCommand::CancelClose];
        cmds.extend(self.hide());
        cmds
    }

    /// Fensterbefehle zu einem Ereignis aus Infobereich, Pipe oder Benachrichtigung.
    pub fn apply(&mut self, e: &Event) -> Vec<ViewportCommand> {
        match e {
            Event::Show => {
                self.hidden = false;
                vec![
                    ViewportCommand::Visible(true),
                    ViewportCommand::Minimized(false),
                    ViewportCommand::Focus,
                ]
            }
            Event::Quit => {
                self.quit_wanted = true;
                Vec::new()
            }
            Event::Notice { .. } | Event::Tick { .. } => Vec::new(),
        }
    }

    /// Wirklich beenden (nach dem Sichern des Entwurfs).
    pub fn finish_quit(&mut self) -> Vec<ViewportCommand> {
        self.quit_wanted = false;
        self.quitting = true;
        vec![ViewportCommand::Close]
    }

    /// Erster Aufruf: neues Bild anfordern. Zweiter Aufruf (nach dem ersten Bild): verstecken.
    pub fn start_step(&mut self, ctx: &Context) {
        match self.start_pending {
            Some(0) => {
                self.start_pending = Some(1);
                ctx.request_repaint();
            }
            Some(_) => {
                self.start_pending = None;
                if self.hidden {
                    ctx.send_viewport_cmd(ViewportCommand::Visible(false));
                }
            }
            None => {}
        }
    }
}
