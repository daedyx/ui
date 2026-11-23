use super::{accent_of, install, look, style};
use crate::tokens::{AppAccent, TOKENS, palette};
use eframe::egui::{self, Color32, Context, CornerRadius, TextStyle, Theme};

#[test]
fn visuals_kommen_aus_den_tokens_hell_und_dunkel() {
    for dark in [false, true] {
        for app in AppAccent::ALL {
            let p = palette(dark);
            let a = p.accent(app);
            let v = style(dark, app).visuals;
            assert_eq!(v.dark_mode, dark);
            assert_eq!(v.panel_fill, p.window);
            assert_eq!(v.window_fill, p.surface);
            assert_eq!(v.window_stroke.color, p.line);
            assert_eq!(v.extreme_bg_color, p.surface_sunken);
            assert_eq!(v.faint_bg_color, p.surface_sunken);
            assert_eq!(v.hyperlink_color, a.text);
            assert_eq!(v.selection.bg_fill, a.soft);
            assert_eq!(v.selection.stroke.color, a.text);
            assert_eq!(v.warn_fg_color, p.state.warning.text);
            assert_eq!(v.error_fg_color, p.state.error.text);
            assert_eq!(v.window_corner_radius, CornerRadius::same(12));
            assert_eq!(v.menu_corner_radius, CornerRadius::same(8));
            assert_eq!(v.window_shadow.color, TOKENS.shadow.dialog.of(dark).color);
            assert_eq!(v.popup_shadow.blur, TOKENS.shadow.floating.of(dark).blur);
            assert_eq!(v.widgets.noninteractive.fg_stroke.color, p.text_muted);
            assert_eq!(v.widgets.inactive.weak_bg_fill, Color32::TRANSPARENT);
            assert_eq!(v.widgets.hovered.weak_bg_fill, p.hover);
            assert_eq!(v.widgets.active.weak_bg_fill, p.pressed);
            assert_eq!(v.widgets.hovered.expansion, 0.0);
            assert!(!v.striped);
        }
    }
}

#[test]
fn abstaende_und_schriftstufen_kommen_aus_den_tokens() {
    let s = style(false, AppAccent::Desk);
    assert_eq!(s.spacing.item_spacing, egui::vec2(8.0, 8.0));
    assert_eq!(s.spacing.button_padding, egui::vec2(12.0, 6.0));
    assert_eq!(s.spacing.interact_size.y, TOKENS.size.control);
    assert_eq!(s.spacing.indent, 16.0);
    let size = |t: TextStyle| s.text_styles.get(&t).map(|f| f.size);
    assert_eq!(size(TextStyle::Small), Some(12.0));
    assert_eq!(size(TextStyle::Body), Some(14.0));
    assert_eq!(size(TextStyle::Button), Some(14.0));
    assert_eq!(size(TextStyle::Heading), Some(20.0));
    assert!((s.animation_time - 0.12).abs() < 1e-6);
}

#[test]
fn install_setzt_beide_themen_und_merkt_die_app() {
    let ctx = Context::default();
    install(&ctx, AppAccent::Tasks);
    assert_eq!(accent_of(&ctx), AppAccent::Tasks);
    assert_eq!(
        ctx.style_of(Theme::Dark).visuals.panel_fill,
        palette(true).window
    );
    assert_eq!(
        ctx.style_of(Theme::Light).visuals.panel_fill,
        palette(false).window
    );
    let l = look(&ctx, true);
    assert_eq!(l.accent.full, palette(true).accent.tasks.full);
}
