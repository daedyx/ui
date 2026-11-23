//! Aufbau von `assets/tokens.json` als Rust-Strukturen. Generisch über den Farbtyp `C`:
//! `build.rs` liest die Datei mit einem Text-Farbtyp und schreibt daraus die Konstante
//! `TOKENS` mit `egui::Color32`. Diese Datei wird darum auch von `build.rs` eingebunden und
//! darf nichts aus egui benutzen. Farb-Strukturen lehnen unbekannte Schlüssel ab: Kommt in
//! der Datei eine Farbe dazu, bricht der Bau, bis sie hier einen Platz hat.
use super::metrics::{Font, Line, Motion, Radius, Shadows, Sizes, Spacing};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct TokenFile<C> {
    #[serde(rename = "farbe")]
    pub color: Themes<C>,
    #[serde(rename = "schrift")]
    pub font: Font,
    #[serde(rename = "abstand_px")]
    pub spacing: Spacing,
    #[serde(rename = "radius_px")]
    pub radius: Radius,
    #[serde(rename = "linie_px")]
    pub line: Line,
    #[serde(rename = "schatten")]
    pub shadow: Shadows<C>,
    #[serde(rename = "groessen_px")]
    pub size: Sizes,
    #[serde(rename = "bewegung")]
    pub motion: Motion,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Themes<C> {
    #[serde(rename = "hell")]
    pub light: Palette<C>,
    #[serde(rename = "dunkel")]
    pub dark: Palette<C>,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Palette<C> {
    #[serde(rename = "fenster")]
    pub window: C,
    #[serde(rename = "flaeche")]
    pub surface: C,
    #[serde(rename = "flaeche_vertieft")]
    pub surface_sunken: C,
    #[serde(rename = "leiste")]
    pub sidebar: C,
    #[serde(rename = "linie")]
    pub line: C,
    #[serde(rename = "linie_stark")]
    pub line_strong: C,
    pub text: C,
    #[serde(rename = "text_gedaempft")]
    pub text_muted: C,
    #[serde(rename = "text_schwach")]
    pub text_faint: C,
    #[serde(rename = "ueberlagerung")]
    pub overlay: C,
    pub hover: C,
    #[serde(rename = "gedrueckt")]
    pub pressed: C,
    #[serde(rename = "fokusring")]
    pub focus_ring: C,
    #[serde(rename = "akzent")]
    pub accent: Accents<C>,
    #[serde(rename = "zustand")]
    pub state: States<C>,
    pub kanban: Kanban<C>,
    #[serde(rename = "prioritaet")]
    pub priority: Priority<C>,
    #[serde(rename = "kalender_farben")]
    pub calendar: [C; 6],
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Accents<C> {
    pub desk: Accent<C>,
    pub tasks: Accent<C>,
    pub board: Accent<C>,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Accent<C> {
    #[serde(rename = "voll")]
    pub full: C,
    #[serde(rename = "auf_voll")]
    pub on_full: C,
    #[serde(rename = "weich")]
    pub soft: C,
    pub text: C,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct States<C> {
    pub ok: Tone<C>,
    #[serde(rename = "warnung")]
    pub warning: Tone<C>,
    #[serde(rename = "fehler")]
    pub error: StrongTone<C>,
    pub info: Tone<C>,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tone<C> {
    pub text: C,
    #[serde(rename = "weich")]
    pub soft: C,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrongTone<C> {
    pub text: C,
    #[serde(rename = "weich")]
    pub soft: C,
    #[serde(rename = "voll")]
    pub full: C,
    #[serde(rename = "auf_voll")]
    pub on_full: C,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kanban<C> {
    #[serde(rename = "offen")]
    pub open: Lane<C>,
    #[serde(rename = "geplant")]
    pub planned: Lane<C>,
    #[serde(rename = "arbeit")]
    pub in_progress: Lane<C>,
    #[serde(rename = "pruefen")]
    pub review: Lane<C>,
    #[serde(rename = "fertig")]
    pub done: Lane<C>,
    #[serde(rename = "stehen")]
    pub stuck: Lane<C>,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lane<C> {
    #[serde(rename = "punkt")]
    pub dot: C,
    #[serde(rename = "weich")]
    pub soft: C,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Priority<C> {
    #[serde(rename = "niedrig")]
    pub low: C,
    #[serde(rename = "mittel")]
    pub medium: C,
    #[serde(rename = "hoch")]
    pub high: C,
}
