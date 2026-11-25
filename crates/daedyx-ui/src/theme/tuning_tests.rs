//! Feinstellung des Dunkelmodus (Tasks T-F3): Kontrast in jeder Stellung, Grenzen der Regler, Wirkung auf Stil und
//! `look`, helles Thema unberührt.
use super::{DarkTuning, PRESETS, dark_tuning, install, look, set_dark_tuning};
use crate::tokens::contrast_tests::pairs;
use crate::tokens::{AppAccent, palette};
use eframe::egui::{Context, Theme};

/// Alle Vorlagen und die Ränder der Regler: jede Kontrastregel der Tokens gilt weiter.
#[test]
fn jede_stellung_erfuellt_die_kontrastregeln() {
    let mut out = Vec::new();
    let mut seen = 0;
    let levels = (0..=100).step_by(5);
    for level in levels {
        for p in &PRESETS {
            let t = DarkTuning::new(level, p.hue.into(), p.tint.into());
            pairs(&t.palette(), &format!("{} {level}", p.id), &mut out);
            seen += 1;
        }
        for hue in (0..360).step_by(15) {
            for tint in [0, 50, 100] {
                let t = DarkTuning::new(level, hue, tint);
                pairs(&t.palette(), &format!("{level}/{hue}/{tint}"), &mut out);
                seen += 1;
            }
        }
    }
    assert!(seen > 1500);
    out.dedup();
    assert!(out.is_empty(), "Kontrast zu schwach:\n{}", out.join("\n"));
}

#[test]
fn regler_bleiben_in_ihren_grenzen() {
    let t = DarkTuning::new(-5, 725, 300);
    assert_eq!((t.level, t.hue, t.tint), (0, 5, 100));
    let t = DarkTuning::new(i64::MIN, i64::MIN, i64::MAX);
    assert!(t.level == 0 && t.hue < 360 && t.tint == 100);
    let t = DarkTuning::new(i64::MAX, -1, i64::MIN);
    assert_eq!((t.level, t.hue, t.tint), (100, 359, 0));
    assert_eq!(DarkTuning::default(), DarkTuning::new(50, 220, 30));
    let (black, white) = DarkTuning::new(0, 0, 0).mix();
    assert!((black - super::BLACK_MAX).abs() < 1e-6 && white == 0.0);
    let (black, white) = DarkTuning::new(100, 0, 0).mix();
    assert!(black == 0.0 && (white - super::WHITE_MAX).abs() < 1e-6);
    assert_eq!(DarkTuning::default().mix(), (0.0, 0.0));
}

#[test]
fn vorlagen_setzen_farbton_und_toenung() {
    let ids: Vec<_> = PRESETS.iter().map(|p| p.id).collect();
    assert_eq!(ids, ["neutral", "blau", "petrol", "gruen", "lila", "warm"]);
    let t = DarkTuning::new(70, 0, 0).with_preset(&PRESETS[3]);
    assert_eq!((t.level, t.hue, t.tint), (70, 140, 35));
    assert_eq!(t.preset().map(|p| p.id), Some("gruen"));
    assert_eq!(DarkTuning::new(50, 1, 1).preset(), None);
    // Neutral ist grau: kein Farbton in den Gründen.
    let w = DarkTuning::default()
        .with_preset(&PRESETS[0])
        .palette()
        .window;
    assert!(w.r() == w.g() && w.g() == w.b(), "{w:?}");
    // Dunkler und heller als die Mitte.
    let mid = DarkTuning::default().palette().surface;
    let dark = DarkTuning::new(0, 220, 30).palette().surface;
    let light = DarkTuning::new(100, 220, 30).palette().surface;
    assert!(dark.r() <= mid.r() && light.r() >= mid.r() && light.g() > dark.g());
}

#[test]
fn feinstellung_wirkt_nur_im_dunkelmodus() {
    let ctx = Context::default();
    install(&ctx, AppAccent::Tasks);
    let light = look(&ctx, false).p.window;
    assert_eq!(look(&ctx, true).p.window, palette(true).window);
    let t = DarkTuning::new(0, 140, 60);
    set_dark_tuning(&ctx, Some(t));
    assert_eq!(dark_tuning(&ctx), Some(t));
    assert_eq!(look(&ctx, true).p.window, t.palette().window);
    assert_eq!(
        ctx.style_of(Theme::Dark).visuals.panel_fill,
        t.palette().window
    );
    assert_eq!(look(&ctx, false).p.window, light, "hell unverändert");
    assert_eq!(ctx.style_of(Theme::Light).visuals.panel_fill, light);
    assert_eq!(
        look(&ctx, true).accent.full,
        palette(true).accent.tasks.full
    );
    set_dark_tuning(&ctx, None);
    assert_eq!(dark_tuning(&ctx), None);
    assert_eq!(look(&ctx, true).p.window, palette(true).window);
    assert_eq!(
        ctx.style_of(Theme::Dark).visuals.panel_fill,
        palette(true).window
    );
}
