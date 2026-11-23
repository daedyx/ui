use super::{AppAccent, TOKENS, TokenFile, palette};
use eframe::egui::Color32;
use serde::{Deserialize, Deserializer};
use std::fmt;

/// Farbe aus der JSON-Datei, schreibt sich als `Color32`-Konstante (nur für den Erzeuger).
struct Lit([u8; 4]);

impl<'de> Deserialize<'de> for Lit {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        let digits = text.strip_prefix('#').unwrap_or("?");
        let v = u32::from_str_radix(digits, 16).map_err(serde::de::Error::custom)?;
        match digits.len() {
            6 => {
                let [_, r, g, b] = v.to_be_bytes();
                Ok(Lit([r, g, b, 255]))
            }
            8 => Ok(Lit(v.to_be_bytes())),
            _ => Err(serde::de::Error::custom(format!("Farbe {text}"))),
        }
    }
}

impl fmt::Debug for Lit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [r, g, b, a] = self.0;
        write!(
            f,
            "Color32::from_rgba_unmultiplied_const({r}, {g}, {b}, {a})"
        )
    }
}

const HEADER: &str = "//! Erzeugt aus `assets/tokens.json` vom Test `tokens::tests::erzeugte_datei_passt_zur_json`.\n\
//! Nicht von Hand ändern. Neu schreiben: `DAEDYX_TOKENS_SCHREIBEN=1 cargo test -p daedyx-ui tokens`.\n\
use super::metrics::*;\nuse super::schema::*;\nuse eframe::egui::Color32;\n\n\
pub const TOKENS: TokenFile<Color32> = ";

fn json_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/tokens.json")
}

fn render() -> String {
    let json = std::fs::read_to_string(json_path()).unwrap();
    let file: TokenFile<Lit> = serde_json::from_str(&json).unwrap();
    format!("{HEADER}{file:#?};\n")
}

/// Die Rust-Datei `generated.rs` ist eine genaue Abschrift der JSON-Datei.
#[test]
fn erzeugte_datei_passt_zur_json() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tokens/generated.rs");
    let want = render();
    if std::env::var("DAEDYX_TOKENS_SCHREIBEN").is_ok() {
        std::fs::write(&path, &want).unwrap();
    }
    let have = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        have == want,
        "generated.rs passt nicht zu tokens.json: DAEDYX_TOKENS_SCHREIBEN=1 cargo test -p daedyx-ui tokens"
    );
}

#[test]
fn stichproben_kommen_an() {
    let light = palette(false);
    let dark = palette(true);
    assert_eq!(light.window, Color32::from_rgb(0xF4, 0xF5, 0xF9));
    assert_eq!(dark.text, Color32::from_rgb(0xE7, 0xE9, 0xF2));
    assert_eq!(
        light.accent(AppAccent::Tasks).text,
        Color32::from_rgb(0x7F, 0x4B, 0x00)
    );
    assert_eq!(
        dark.accent(AppAccent::Board).full,
        Color32::from_rgb(0x4F, 0xB8, 0xC8)
    );
    assert_eq!(dark.overlay.a(), 0x99);
    assert_eq!(light.calendar[4], Color32::from_rgb(0xA3, 0x18, 0x5E));
    assert_eq!(TOKENS.size.control, 32.0);
    assert_eq!(TOKENS.radius.large, 12);
    assert_eq!(TOKENS.font.steps.text_bold.weight, 600);
    assert_eq!(TOKENS.shadow.dialog.dark.blur, 48);
}

#[test]
fn kaputte_oder_unbekannte_farben_werden_abgelehnt() {
    let json = std::fs::read_to_string(json_path()).unwrap();
    let extra = json.replacen(
        "\"fenster\": \"#F4F5F9\",",
        "\"fenster\": \"#F4F5F9\", \"neu\": \"#000000\",",
        1,
    );
    assert!(
        serde_json::from_str::<TokenFile<Lit>>(&extra).is_err(),
        "unbekannte Farbe"
    );
    let bad = json.replacen("#F4F5F9", "#F4F5", 1);
    assert!(
        serde_json::from_str::<TokenFile<Lit>>(&bad).is_err(),
        "falsche Länge"
    );
}
