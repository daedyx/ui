//! Maße aus `assets/tokens.json` (Schrift, Abstände, Radien, Linien, Größen, Bewegung).
//! Wie `schema.rs` auch von `build.rs` eingebunden, darum ohne egui.
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Font {
    #[serde(rename = "stufen_px")]
    pub steps: TypeSteps,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeSteps {
    #[serde(rename = "klein")]
    pub small: TypeStep,
    pub text: TypeStep,
    #[serde(rename = "text_fett")]
    pub text_bold: TypeStep,
    #[serde(rename = "betont")]
    pub emphasis: TypeStep,
    #[serde(rename = "titel")]
    pub title: TypeStep,
    #[serde(rename = "gross")]
    pub large: TypeStep,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeStep {
    #[serde(rename = "groesse")]
    pub size: f32,
    #[serde(rename = "zeile")]
    pub line_height: f32,
    #[serde(rename = "gewicht")]
    pub weight: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spacing {
    #[serde(rename = "1")]
    pub s1: f32,
    #[serde(rename = "2")]
    pub s2: f32,
    #[serde(rename = "3")]
    pub s3: f32,
    #[serde(rename = "4")]
    pub s4: f32,
    #[serde(rename = "5")]
    pub s5: f32,
    #[serde(rename = "6")]
    pub s6: f32,
    #[serde(rename = "8")]
    pub s8: f32,
    #[serde(rename = "10")]
    pub s10: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Radius {
    #[serde(rename = "klein")]
    pub small: f32,
    #[serde(rename = "mittel")]
    pub medium: f32,
    #[serde(rename = "gross")]
    pub large: f32,
    #[serde(rename = "rund")]
    pub round: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Line {
    #[serde(rename = "fein")]
    pub fine: f32,
    #[serde(rename = "fokus")]
    pub focus: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Shadows<C> {
    #[serde(rename = "schwebend")]
    pub floating: ShadowPair<C>,
    pub dialog: ShadowPair<C>,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShadowPair<C> {
    #[serde(rename = "hell")]
    pub light: ShadowSpec<C>,
    #[serde(rename = "dunkel")]
    pub dark: ShadowSpec<C>,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShadowSpec<C> {
    #[serde(rename = "versatz_y")]
    pub offset_y: u8,
    #[serde(rename = "unschaerfe")]
    pub blur: u8,
    #[serde(rename = "farbe")]
    pub color: C,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sizes {
    #[serde(rename = "bedienelement_hoehe")]
    pub control: f32,
    #[serde(rename = "bedienelement_hoehe_gross")]
    pub control_large: f32,
    #[serde(rename = "zeile_liste")]
    pub list_row: f32,
    #[serde(rename = "zeile_liste_kompakt")]
    pub list_row_compact: f32,
    #[serde(rename = "leiste_schmal")]
    pub sidebar_narrow: f32,
    #[serde(rename = "leiste_breit")]
    pub sidebar_wide: f32,
    #[serde(rename = "kopfzeile")]
    pub header: f32,
    #[serde(rename = "symbol")]
    pub icon: f32,
    #[serde(rename = "symbol_klein")]
    pub icon_small: f32,
    #[serde(rename = "symbol_gross")]
    pub icon_large: f32,
    pub avatar: f32,
    #[serde(rename = "trefferflaeche_min")]
    pub hit_target_min: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Motion {
    pub hover_ms: u16,
    #[serde(rename = "auswahl_ms")]
    pub selection_ms: u16,
    pub dialog_ms: u16,
}
