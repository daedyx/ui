//! Backend-Brücke: Die Oberfläche schickt Anfragen an einen eigenen Tokio-Thread (dort laufen
//! Kern-Aufrufe, Netz, Datenbank). Antworten kommen als Ereignisse zurück; jede Antwort weckt die
//! Oberfläche (`request_repaint`), sonst zeichnet egui im Leerlauf nichts. Die Oberfläche
//! wartet nie: `poll` holt nur, was schon da ist. Muster wie ArtCraft (nur die Idee).
use eframe::egui::Context;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Höchstens so viele offene Anfragen; mehr wird mit [`BridgeError::Busy`] abgelehnt.
pub const MAX_PENDING: usize = 1024;
/// Höchstens so viele Ereignisse je Bild, damit ein Schwall die Oberfläche nicht aufhält.
pub const MAX_EVENTS_PER_POLL: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeError {
    /// Laufzeit oder Thread ließen sich nicht starten.
    Start,
    /// Der Hintergrund-Thread läuft nicht mehr.
    Closed,
    /// Zu viele offene Anfragen.
    Busy,
}

/// Verbindung der Oberfläche zum Hintergrund. Fällt sie weg, endet der Thread von selbst.
pub struct Bridge<Req, Ev> {
    requests: mpsc::Sender<Req>,
    events: mpsc::Receiver<Ev>,
}

impl<Req: Send + 'static, Ev: Send + 'static> Bridge<Req, Ev> {
    /// Startet den Hintergrund-Thread. `handler` beantwortet eine Anfrage (darf warten),
    /// `wake` wird nach jeder Antwort gerufen (in Apps: [`repaint_waker`]).
    pub fn start<H, Fut, W>(handler: H, wake: W) -> Result<Self, BridgeError>
    where
        H: Fn(Req) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Ev> + Send + 'static,
        W: Fn() + Send + Sync + 'static,
    {
        let (req_tx, mut req_rx) = mpsc::channel::<Req>(MAX_PENDING);
        let (ev_tx, ev_rx) = mpsc::channel::<Ev>(MAX_PENDING);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .map_err(|_| BridgeError::Start)?;
        let handler = Arc::new(handler);
        let wake = Arc::new(wake);
        std::thread::Builder::new()
            .name("daedyx-bridge".into())
            .spawn(move || {
                runtime.block_on(async move {
                    while let Some(req) = req_rx.recv().await {
                        let (handler, events, wake) =
                            (handler.clone(), ev_tx.clone(), wake.clone());
                        tokio::spawn(async move {
                            let event = handler(req).await;
                            if events.send(event).await.is_ok() {
                                wake();
                            }
                        });
                    }
                });
            })
            .map_err(|_| BridgeError::Start)?;
        Ok(Bridge {
            requests: req_tx,
            events: ev_rx,
        })
    }

    /// Schickt eine Anfrage, ohne zu warten.
    pub fn send(&self, req: Req) -> Result<(), BridgeError> {
        self.requests.try_send(req).map_err(|e| match e {
            mpsc::error::TrySendError::Full(_) => BridgeError::Busy,
            mpsc::error::TrySendError::Closed(_) => BridgeError::Closed,
        })
    }

    /// Alle schon eingetroffenen Antworten (höchstens [`MAX_EVENTS_PER_POLL`]). Einmal je Bild.
    pub fn poll(&mut self) -> Vec<Ev> {
        let mut out = Vec::new();
        while out.len() < MAX_EVENTS_PER_POLL {
            match self.events.try_recv() {
                Ok(event) => out.push(event),
                Err(_) => break,
            }
        }
        out
    }

    /// Wartet höchstens `timeout` auf die nächste Antwort (für Tests und Hilfsprogramme,
    /// nie im Zeichen-Code der Oberfläche).
    pub fn wait(&mut self, timeout: Duration) -> Option<Ev> {
        let deadline = Instant::now().checked_add(timeout)?;
        loop {
            if let Ok(event) = self.events.try_recv() {
                return Some(event);
            }
            if Instant::now() >= deadline {
                return None;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}

/// Weckruf für egui: zeichnet nach einer Antwort ein neues Bild.
pub fn repaint_waker(ctx: &Context) -> impl Fn() + Send + Sync + 'static {
    let ctx = ctx.clone();
    move || ctx.request_repaint()
}

#[cfg(test)]
mod tests;
