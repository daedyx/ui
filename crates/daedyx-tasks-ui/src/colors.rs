//! Listenfarben nur über die Tokens. Gespeichert wird `#RRGGBB` der hellen Kalenderfarbe (so liest
//! auch die alte Oberfläche die Farbe); angezeigt wird immer eine der sechs Kalenderfarben des
//! aktuellen Modus, für fremde Werte die nächstgelegene. Leer oder kaputt: der Ersatz (Position).
use daedyx_ui::tokens::palette;
use eframe::egui::Color32;

/// Anzahl der Farben zur Auswahl (Kalenderfarben der Tokens).
pub const COLORS: usize = 6;

/// Textschlüssel der Farbnamen in derselben Reihenfolge wie die Tokens.
pub const NAMES: [&str; COLORS] = [
    "aufgaben.farbe.indigo",
    "aufgaben.farbe.gruen",
    "aufgaben.farbe.bernstein",
    "aufgaben.farbe.petrol",
    "aufgaben.farbe.beere",
    "aufgaben.farbe.rot",
];

/// Der Wert, der für Farbe `i` gespeichert wird.
pub fn stored(i: usize) -> String {
    let c = palette(false)
        .calendar
        .get(i % COLORS)
        .copied()
        .unwrap_or(Color32::TRANSPARENT);
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

fn parse(hex: &str) -> Option<[u8; 3]> {
    let digits = hex
        .strip_prefix('#')
        .filter(|d| d.len() == 6 && d.is_ascii())?;
    let part = |i: usize| u8::from_str_radix(digits.get(i..i.saturating_add(2))?, 16).ok();
    Some([part(0)?, part(2)?, part(4)?])
}

fn distance(a: [u8; 3], b: Color32) -> u32 {
    let d = |x: u8, y: u8| u32::from(x.abs_diff(y)).pow(2);
    d(a[0], b.r())
        .saturating_add(d(a[1], b.g()))
        .saturating_add(d(a[2], b.b()))
}

/// Index der Tokenfarbe für einen gespeicherten Wert; `fallback` (modulo Anzahl) für leer/kaputt.
pub fn color_index(hex: &str, fallback: usize) -> usize {
    let Some(rgb) = parse(hex) else {
        return fallback % COLORS;
    };
    let both = palette(false)
        .calendar
        .iter()
        .chain(palette(true).calendar.iter());
    both.enumerate()
        .min_by_key(|(_, c)| distance(rgb, **c))
        .map_or(fallback % COLORS, |(i, _)| i % COLORS)
}

/// Farbe einer Liste im aktuellen Modus.
pub fn list_color(dark: bool, hex: &str, position: usize) -> Color32 {
    palette(dark)
        .calendar
        .get(color_index(hex, position))
        .copied()
        .unwrap_or(Color32::TRANSPARENT)
}
