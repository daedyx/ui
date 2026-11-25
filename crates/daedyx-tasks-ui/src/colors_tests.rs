//! Listenfarben nur über die Tokens: gespeichert als `#RRGGBB` (wie in der alten Oberfläche),
//! angezeigt als die nächstgelegene der sechs Kalenderfarben des aktuellen Modus.
use crate::colors::{COLORS, color_index, stored};
use daedyx_ui::tokens::palette;

#[test]
fn jede_tokenfarbe_wird_gespeichert_und_wiedererkannt() {
    for i in 0..COLORS {
        let hex = stored(i);
        assert!(hex.len() == 7 && hex.starts_with('#'), "{hex}");
        assert_eq!(color_index(&hex, 99), i);
    }
}

#[test]
fn auch_die_dunkle_fassung_und_fremde_farben_finden_die_naechste_tokenfarbe() {
    let dark = palette(true).calendar;
    for (i, c) in dark.iter().enumerate() {
        let hex = format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b());
        assert_eq!(color_index(&hex, 99), i, "dunkle Fassung {i}");
    }
    let light = palette(false).calendar;
    let near = |c: eframe::egui::Color32| {
        format!(
            "#{:02X}{:02X}{:02X}",
            c.r().saturating_add(3),
            c.g(),
            c.b().saturating_sub(2)
        )
    };
    assert_eq!(color_index(&near(light[3]), 99), 3);
}

#[test]
fn leer_oder_kaputt_nimmt_den_ersatz() {
    for bad in ["", "#12", "rot", "#GGGGGG", "#ÄÖÜÄÖÜ", "#1234567"] {
        assert_eq!(color_index(bad, 4), 4, "{bad}");
    }
    assert_eq!(color_index("", 13), 13 % COLORS);
}
