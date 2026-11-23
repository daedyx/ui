use super::{SEMIBOLD, definitions, semibold};
use eframe::egui::FontFamily;

#[test]
fn ohne_systemschrift_bleibt_die_eingebaute_schrift_als_ersatz() {
    let d = definitions(|_| None);
    let prop = &d.families[&FontFamily::Proportional];
    let bold = &d.families[&semibold()];
    assert!(!prop.is_empty(), "eingebaute Schrift fehlt");
    assert_eq!(prop, bold, "ohne Systemschrift sind beide Familien gleich");
    assert_eq!(semibold(), FontFamily::Name(SEMIBOLD.into()));
}

#[test]
fn systemschrift_steht_vorn_und_halbfett_vor_normal() {
    let d = definitions(|f| (f == "segoeui.ttf" || f == "seguisb.ttf").then(|| vec![0u8; 4]));
    let prop = &d.families[&FontFamily::Proportional];
    let bold = &d.families[&semibold()];
    assert_eq!(prop.first().map(String::as_str), Some("system-regular"));
    assert_eq!(bold.first().map(String::as_str), Some("system-semibold"));
    assert_eq!(bold.get(1).map(String::as_str), Some("system-regular"));
    assert!(prop.len() > 1, "Ersatzschrift bleibt dahinter");
}

#[test]
fn mac_schrift_wird_genommen_wenn_segoe_fehlt() {
    let d = definitions(|f| (f == "SFNS.ttf").then(|| vec![0u8; 4]));
    assert!(d.font_data.contains_key("system-regular"));
    assert!(!d.font_data.contains_key("system-semibold"));
}

#[test]
fn ohne_geladene_schriften_faellt_halbfett_auf_normal_zurueck() {
    let ctx = eframe::egui::Context::default();
    let mut out = ctx.run_ui(Default::default(), |_| {});
    out.textures_delta.clear();
    assert_eq!(super::semibold_in(&ctx), FontFamily::Proportional);
    ctx.set_fonts(definitions(|_| None));
    let mut out = ctx.run_ui(Default::default(), |_| {});
    out.textures_delta.clear();
    assert_eq!(super::semibold_in(&ctx), semibold());
}
