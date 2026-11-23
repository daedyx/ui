//! Kontrast nach WCAG 2.x (relative Leuchtdichte, Verhältnis 1:1 bis 21:1).
use eframe::egui::Color32;

/// Mindestwert für normalen Text (WCAG AA).
pub const TEXT_MIN: f64 = 4.5;
/// Mindestwert für große Schrift, Symbole und Bedienelement-Grenzen (WCAG AA).
pub const GRAPHIC_MIN: f64 = 3.0;

fn channel(value: u8) -> f64 {
    let c = f64::from(value) / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Relative Leuchtdichte einer deckenden Farbe (0 = schwarz, 1 = weiß).
pub fn luminance(color: Color32) -> f64 {
    let [r, g, b, _] = color.to_srgba_unmultiplied();
    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

/// Kontrastverhältnis zweier deckender Farben, unabhängig von der Reihenfolge.
pub fn ratio(a: Color32, b: Color32) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}
