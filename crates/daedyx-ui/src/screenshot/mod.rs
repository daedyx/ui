//! Bildschirmfoto-Hilfe für Demo-Apps (SICHERHEIT 3.4): egui liefert das Bild des eigenen
//! Bildpuffers, also **nur das App-Fenster**, nie den Bildschirm, keine anderen Fenster.
//! Eingeschaltet nur über `DAEDYX_SCREENSHOT=datei.png`; danach schließt sich das Fenster.
//! Gedacht für die Galerie und Demo-Modi mit erfundenen Daten, nie für echte Profile.
use eframe::egui::{ColorImage, Context, ScreenshotCallback, ViewportCommand};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

/// Name der Umgebungsvariable.
pub const ENV: &str = "DAEDYX_SCREENSHOT";
/// So viele Bilder warten, bis Schriften und Anordnung stehen.
const WARMUP_FRAMES: u32 = 6;
/// Und so lange (Sekunden seit Start), damit Einblendungen (Dialog, höchstens 150 ms) fertig sind.
const WARMUP_SECONDS: f64 = 0.8;
const PENDING: u8 = 0;
const WRITTEN: u8 = 1;
const FAILED: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShotError {
    Encode,
    Write,
}

/// Wandelt ein egui-Bild in PNG-Bytes (RGBA, 8 Bit).
pub fn encode_png(image: &ColorImage) -> Result<Vec<u8>, ShotError> {
    let [w, h] = image.size;
    let width = u32::try_from(w).map_err(|_| ShotError::Encode)?;
    let height = u32::try_from(h).map_err(|_| ShotError::Encode)?;
    let raw: Vec<u8> = image
        .pixels
        .iter()
        .flat_map(|p| p.to_srgba_unmultiplied())
        .collect();
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|_| ShotError::Encode)?;
    writer
        .write_image_data(&raw)
        .map_err(|_| ShotError::Encode)?;
    writer.finish().map_err(|_| ShotError::Encode)?;
    Ok(out)
}

/// Nur `.png` in einem schon vorhandenen Ordner ist erlaubt.
pub fn valid_target(path: &Path) -> bool {
    let png = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("png"));
    let parent_ok = path
        .parent()
        .is_some_and(|p| p.as_os_str().is_empty() || p.is_dir());
    png && parent_ok
}

/// Ablauf eines Fotos: warten, anfordern, schreiben, Fenster schließen.
pub struct Shot {
    path: PathBuf,
    frames: u32,
    requested: bool,
    state: Arc<AtomicU8>,
}

impl Shot {
    /// Foto-Auftrag aus der Umgebung, falls gesetzt und gültig.
    pub fn from_env() -> Option<Shot> {
        let path = PathBuf::from(std::env::var_os(ENV)?);
        Shot::new(path)
    }

    pub fn new(path: PathBuf) -> Option<Shot> {
        valid_target(&path).then(|| Shot {
            path,
            frames: 0,
            requested: false,
            state: Arc::new(AtomicU8::new(PENDING)),
        })
    }

    /// Einmal je Bild aufrufen.
    pub fn tick(&mut self, ctx: &Context) {
        if self.state.load(Ordering::SeqCst) != PENDING {
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }
        self.frames = self.frames.saturating_add(1);
        let settled = ctx.input(|i| i.time) >= WARMUP_SECONDS;
        if self.frames >= WARMUP_FRAMES && settled && !self.requested {
            self.requested = true;
            let (path, state) = (self.path.clone(), self.state.clone());
            let callback = ScreenshotCallback::new(move |image: Arc<ColorImage>| {
                let ok = encode_png(&image)
                    .ok()
                    .is_some_and(|bytes| std::fs::write(&path, bytes).is_ok());
                state.store(if ok { WRITTEN } else { FAILED }, Ordering::SeqCst);
            });
            ctx.send_viewport_cmd(ViewportCommand::Screenshot(callback));
        }
        ctx.request_repaint();
    }

    /// Ergebnis, sobald das Foto geschrieben (oder gescheitert) ist.
    pub fn result(&self) -> Option<Result<(), ShotError>> {
        match self.state.load(Ordering::SeqCst) {
            WRITTEN => Some(Ok(())),
            FAILED => Some(Err(ShotError::Write)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
