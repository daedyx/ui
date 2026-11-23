use super::contrast::{GRAPHIC_MIN, TEXT_MIN, ratio};
use super::{AppAccent, Palette, TOKENS, TokenFile, palette};
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

/// Prüft ein Paar und sammelt Verstöße, damit der Test alle auf einmal meldet.
fn check(out: &mut Vec<String>, name: &str, fg: Color32, bg: Color32, min: f64) {
    let r = ratio(fg, bg);
    if r < min {
        out.push(format!("{name}: {r:.2} < {min}"));
    }
}

fn pairs(p: &Palette<Color32>, mode: &str, out: &mut Vec<String>) {
    let grounds = [
        ("fenster", p.window),
        ("flaeche", p.surface),
        ("vertieft", p.surface_sunken),
        ("leiste", p.sidebar),
    ];
    for (gname, g) in grounds {
        for (tname, t) in [
            ("text", p.text),
            ("gedaempft", p.text_muted),
            ("schwach", p.text_faint),
        ] {
            check(out, &format!("{mode} {tname} auf {gname}"), t, g, TEXT_MIN);
        }
        check(
            out,
            &format!("{mode} fokusring auf {gname}"),
            p.focus_ring,
            g,
            GRAPHIC_MIN,
        );
    }
    for app in AppAccent::ALL {
        let a = p.accent(app);
        check(
            out,
            &format!("{mode} {app:?} auf_voll"),
            a.on_full,
            a.full,
            TEXT_MIN,
        );
        for (gname, g) in [
            ("flaeche", p.surface),
            ("fenster", p.window),
            ("weich", a.soft),
        ] {
            check(
                out,
                &format!("{mode} {app:?} text auf {gname}"),
                a.text,
                g,
                TEXT_MIN,
            );
        }
        check(
            out,
            &format!("{mode} {app:?} text-farbe text auf weich"),
            p.text,
            a.soft,
            TEXT_MIN,
        );
    }
    let s = &p.state;
    for (name, text, soft) in [
        ("ok", s.ok.text, s.ok.soft),
        ("warnung", s.warning.text, s.warning.soft),
        ("fehler", s.error.text, s.error.soft),
        ("info", s.info.text, s.info.soft),
    ] {
        check(
            out,
            &format!("{mode} {name} auf flaeche"),
            text,
            p.surface,
            TEXT_MIN,
        );
        check(
            out,
            &format!("{mode} {name} auf weich"),
            text,
            soft,
            TEXT_MIN,
        );
    }
    check(
        out,
        &format!("{mode} fehler auf_voll"),
        s.error.on_full,
        s.error.full,
        TEXT_MIN,
    );
    for (name, c) in [
        ("niedrig", p.priority.low),
        ("mittel", p.priority.medium),
        ("hoch", p.priority.high),
    ] {
        check(
            out,
            &format!("{mode} prioritaet {name}"),
            c,
            p.surface,
            TEXT_MIN,
        );
    }
    let k = &p.kanban;
    for (name, lane) in [
        ("offen", k.open),
        ("geplant", k.planned),
        ("arbeit", k.in_progress),
        ("pruefen", k.review),
        ("fertig", k.done),
        ("stehen", k.stuck),
    ] {
        check(
            out,
            &format!("{mode} kanban {name} punkt"),
            lane.dot,
            p.surface,
            GRAPHIC_MIN,
        );
        check(
            out,
            &format!("{mode} kanban {name} text auf weich"),
            p.text,
            lane.soft,
            TEXT_MIN,
        );
    }
    for (i, c) in p.calendar.iter().enumerate() {
        check(
            out,
            &format!("{mode} kalender {i}"),
            *c,
            p.surface,
            GRAPHIC_MIN,
        );
    }
}

/// WCAG AA: Text mindestens 4,5:1, Zeichen und Fokusring mindestens 3:1, hell und dunkel.
#[test]
fn alle_farbpaare_erfuellen_wcag_aa() {
    let mut out = Vec::new();
    pairs(palette(false), "hell", &mut out);
    pairs(palette(true), "dunkel", &mut out);
    assert!(out.is_empty(), "Kontrast zu schwach:\n{}", out.join("\n"));
}

#[test]
fn kontrast_rechnet_wie_wcag() {
    let black = Color32::from_rgb(0, 0, 0);
    let white = Color32::from_rgb(255, 255, 255);
    assert!((ratio(black, white) - 21.0).abs() < 1e-9);
    assert!((ratio(white, white) - 1.0).abs() < 1e-9);
    // Wert aus plan-stil.md: weiße Schrift auf Desk-Indigo 8,75:1
    let r = ratio(white, Color32::from_rgb(0x3E, 0x3D, 0xA3));
    assert!((r - 8.75).abs() < 0.02, "{r}");
}
