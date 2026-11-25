//! Feinstellung des Dunkelmodus (Tasks T-F3): Helligkeit, Farbton und Tönung, sechs Vorlagen. Alle Farben werden aus
//! der dunklen Token-Palette abgeleitet; nur Gründe und Linien bekommen Farbton und Tönung, Gründe und Schrift die
//! Helligkeit. Die Regler sind so begrenzt, dass jede Stellung die Kontrastregeln der Tokens erfüllt (Test für alle
//! Vorlagen und Reglerstellungen). Akzent, Zustände, Kanban und Kalender bleiben wie in den Tokens.
use crate::tokens::{Palette, palette};
use eframe::egui::Color32;
use eframe::egui::ecolor::Hsva;

/// Eine Stellung der drei Regler (je 0–100, Farbton 0–359 Grad).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DarkTuning {
    pub level: u8,
    pub hue: u16,
    pub tint: u8,
}

/// Eine Vorlage; der Name kommt aus der Sprachdatei der App (`id`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preset {
    pub id: &'static str,
    pub hue: u16,
    pub tint: u8,
}

/// Die sechs Vorlagen der Tauri-Fassung (gleiche Werte).
pub const PRESETS: [Preset; 6] = [
    Preset {
        id: "neutral",
        hue: 220,
        tint: 0,
    },
    Preset {
        id: "blau",
        hue: 220,
        tint: 30,
    },
    Preset {
        id: "petrol",
        hue: 185,
        tint: 40,
    },
    Preset {
        id: "gruen",
        hue: 140,
        tint: 35,
    },
    Preset {
        id: "lila",
        hue: 270,
        tint: 40,
    },
    Preset {
        id: "warm",
        hue: 30,
        tint: 35,
    },
];

/// Tönung 30 entspricht der Sättigung der Tokens (Faktor 1), 0 ist neutral grau, 100 etwa dreifach.
const TINT_UNIT: f32 = 30.0;
/// Größter Anteil Schwarz (Regler ganz links) und Weiß (ganz rechts). Gemischt wird im linearen HSV-Raum; dort hellen
/// 2,5 % Weiß die Fläche ungefähr von (23, 26, 35) auf (52, 54, 60) auf. Mehr ließe den Desk-Akzent auf der Fläche
/// unter 4,5:1 fallen (Test `jede_stellung_erfuellt_die_kontrastregeln`).
pub const BLACK_MAX: f32 = 0.8;
pub const WHITE_MAX: f32 = 0.025;
/// Die Schrift hellt stärker auf als der Grund, damit der Abstand bleibt.
const TEXT_LIFT: f32 = 10.0;

impl Default for DarkTuning {
    fn default() -> Self {
        DarkTuning {
            level: 50,
            hue: 220,
            tint: 30,
        }
    }
}

impl DarkTuning {
    /// Aus beliebigen Zahlen (gespeicherte Werte können kaputt sein): Helligkeit und Tönung auf 0–100 begrenzt,
    /// Farbton im Kreis 0–359.
    pub fn new(level: i64, hue: i64, tint: i64) -> DarkTuning {
        let pct = |v: i64| u8::try_from(v.clamp(0, 100)).unwrap_or(50);
        DarkTuning {
            level: pct(level),
            hue: u16::try_from(hue.rem_euclid(360)).unwrap_or(220),
            tint: pct(tint),
        }
    }

    /// Farbton und Tönung einer Vorlage, Helligkeit bleibt.
    pub fn with_preset(self, p: &Preset) -> DarkTuning {
        DarkTuning {
            hue: p.hue,
            tint: p.tint.min(100),
            ..self
        }
    }

    /// Welche Vorlage gerade eingestellt ist (Farbton und Tönung gleich).
    pub fn preset(&self) -> Option<&'static Preset> {
        PRESETS
            .iter()
            .find(|p| p.hue == self.hue && p.tint == self.tint)
    }

    /// Anteil Schwarz und Weiß zur Helligkeit: links der Mitte Schwarz, rechts Weiß.
    pub fn mix(&self) -> (f32, f32) {
        let l = f32::from(self.level.min(100));
        let black = ((50.0 - l) / 50.0).max(0.0) * BLACK_MAX;
        let white = ((l - 50.0) / 50.0).max(0.0) * WHITE_MAX;
        (black, white)
    }

    /// Die dunkle Palette mit dieser Stellung.
    pub fn palette(&self) -> Palette<Color32> {
        let mut p = *palette(true);
        let (black, white) = self.mix();
        let hue = f32::from(self.hue % 360) / 360.0;
        let factor = f32::from(self.tint.min(100)) / TINT_UNIT;
        let ground = |c: Color32| {
            let mut h = Hsva::from(c);
            h.h = hue;
            h.s = (h.s * factor).clamp(0.0, 1.0);
            shade(Color32::from(h), black, white)
        };
        for c in [
            &mut p.window,
            &mut p.surface,
            &mut p.surface_sunken,
            &mut p.sidebar,
            &mut p.line,
            &mut p.line_strong,
        ] {
            *c = ground(*c);
        }
        // Die Schrift wird mit aufgehellt, damit der Abstand zum helleren Grund bleibt.
        for c in [&mut p.text, &mut p.text_muted, &mut p.text_faint] {
            *c = shade(*c, 0.0, (white * TEXT_LIFT).min(1.0));
        }
        p
    }
}

/// Mischt erst Schwarz, dann Weiß bei (Anteile 0–1), im HSV-Raum: dunkler über den Hellwert, heller über Hellwert
/// und Sättigung. Ohne feste Farbwerte im Code (Wächter „Farben nur aus den Tokens“).
fn shade(c: Color32, black: f32, white: f32) -> Color32 {
    let mut h = Hsva::from(c);
    h.v *= 1.0 - black.clamp(0.0, 1.0);
    let white = white.clamp(0.0, 1.0);
    h.s *= 1.0 - white;
    h.v += (1.0 - h.v) * white;
    Color32::from(h)
}
