use super::contrast::{GRAPHIC_MIN, TEXT_MIN, ratio};
use super::{AppAccent, Palette, palette};
use eframe::egui::Color32;

/// Prüft ein Paar und sammelt Verstöße, damit der Test alle auf einmal meldet.
fn check(out: &mut Vec<String>, name: &str, fg: Color32, bg: Color32, min: f64) {
    let r = ratio(fg, bg);
    if r < min {
        out.push(format!("{name}: {r:.2} < {min}"));
    }
}

pub(crate) fn pairs(p: &Palette<Color32>, mode: &str, out: &mut Vec<String>) {
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
