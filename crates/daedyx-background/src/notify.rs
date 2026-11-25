//! Benachrichtigungen. Die Oberfläche kennt nur das Merkmal [`Notifier`]; unter Windows zeigt [`WinToast`] einen
//! Toast mit der App-Kennung der App. Tests nehmen [`Recorder`] (zeigt nichts, merkt sich nur Titel und Text).
use crate::sink::{Event, Sink};
use std::sync::{Mutex, MutexGuard};

/// Fehler ohne Text: Inhalte landen nie in Fehlermeldungen oder Protokollen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotifyError;

pub trait Notifier: Send + Sync {
    fn notify(&self, title: &str, body: &str) -> Result<(), NotifyError>;
}

/// Für Tests: zeigt nichts.
#[derive(Debug, Default)]
pub struct Recorder {
    pub sent: Mutex<Vec<(String, String)>>,
}

impl Recorder {
    /// Die bisher „gezeigten“ Benachrichtigungen. Der Takt-Thread und der Test greifen beide zu; das ist die einzige
    /// Sperre des Crates (Regel 2.2), gekapselt hier.
    #[allow(clippy::disallowed_methods, reason = "SICHERHEIT-AUSNAHMEN A-022")]
    pub fn sent(&self) -> Result<MutexGuard<'_, Vec<(String, String)>>, NotifyError> {
        self.sent.lock().map_err(|_| NotifyError)
    }
}

impl Notifier for Recorder {
    fn notify(&self, title: &str, body: &str) -> Result<(), NotifyError> {
        self.sent()?.push((title.to_owned(), body.to_owned()));
        Ok(())
    }
}

/// Windows-Toast. Klick auf den Toast holt das Fenster, solange der Prozess läuft (er läuft im Hintergrund).
pub struct WinToast {
    /// App-Kennung ([`crate::ids::AppIds::aumid`]).
    pub aumid: &'static str,
    pub on_click: Option<Sink>,
}

#[cfg(windows)]
impl Notifier for WinToast {
    fn notify(&self, title: &str, body: &str) -> Result<(), NotifyError> {
        use tauri_winrt_notification::{Duration, Toast};
        let mut toast = Toast::new(self.aumid)
            .title(title)
            .text1(body)
            .duration(Duration::Short);
        if let Some(sink) = self.on_click.clone() {
            toast = toast.on_activated(move |_| {
                sink.emit(Event::Show);
                Ok(())
            });
        }
        toast.show().map_err(|_| NotifyError)
    }
}

#[cfg(not(windows))]
impl Notifier for WinToast {
    fn notify(&self, _title: &str, _body: &str) -> Result<(), NotifyError> {
        let _ = (&self.on_click, self.aumid, Event::Show);
        Err(NotifyError)
    }
}
