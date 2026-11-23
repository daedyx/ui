//! Thema: füllt `egui::Visuals` und `egui::Style` aus den Tokens (hell und dunkel) und merkt sich,
//! welche App zeichnet (Akzentfarbe). Abbildung nach `tokens.json`, Abschnitt `abbildung_egui`.
use crate::fonts;
use crate::tokens::{Accent, AppAccent, Palette, ShadowSpec, TOKENS, palette};
use eframe::egui::{
    self, Color32, Context, CornerRadius, FontFamily, FontId, Shadow, Stroke, Style, TextStyle,
    Theme, Ui, Visuals,
};

/// Farben, die ein Baustein beim Zeichnen braucht.
#[derive(Debug, Clone, Copy)]
pub struct Look {
    pub dark: bool,
    pub p: &'static Palette<Color32>,
    pub accent: &'static Accent<Color32>,
}

fn accent_id() -> egui::Id {
    egui::Id::unique("daedyx-ui-accent")
}

fn radius(value: u16) -> CornerRadius {
    // „rund“ (999) wird auf das egui-Höchstmaß 255 begrenzt.
    CornerRadius::same(u8::try_from(value).unwrap_or(u8::MAX))
}

fn margin(value: u8) -> egui::Margin {
    egui::Margin::same(i8::try_from(value).unwrap_or(i8::MAX))
}

fn shadow(spec: &ShadowSpec<Color32>) -> Shadow {
    Shadow {
        offset: [0, i8::try_from(spec.offset_y).unwrap_or(i8::MAX)],
        blur: spec.blur,
        spread: 0,
        color: spec.color,
    }
}

/// Visuals für hell oder dunkel und eine App.
pub fn visuals(dark: bool, app: AppAccent) -> Visuals {
    let p = palette(dark);
    let a = p.accent(app);
    let mut v = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    let fine = TOKENS.line.fine;
    v.panel_fill = p.window;
    v.window_fill = p.surface;
    v.window_stroke = Stroke::new(fine, p.line);
    v.window_corner_radius = radius(TOKENS.radius.large);
    v.menu_corner_radius = radius(TOKENS.radius.medium);
    v.window_shadow = shadow(TOKENS.shadow.dialog.of(dark));
    v.popup_shadow = shadow(TOKENS.shadow.floating.of(dark));
    v.extreme_bg_color = p.surface_sunken;
    v.faint_bg_color = p.surface_sunken;
    v.code_bg_color = p.surface_sunken;
    v.hyperlink_color = a.text;
    v.warn_fg_color = p.state.warning.text;
    v.error_fg_color = p.state.error.text;
    v.weak_text_color = Some(p.text_muted);
    v.selection.bg_fill = a.soft;
    v.selection.stroke = Stroke::new(fine, a.text);
    v.striped = false;
    let w = &mut v.widgets;
    w.noninteractive.bg_fill = p.surface;
    w.noninteractive.weak_bg_fill = p.surface;
    w.noninteractive.bg_stroke = Stroke::new(fine, p.line);
    w.noninteractive.fg_stroke = Stroke::new(fine, p.text_muted);
    w.inactive.bg_fill = p.surface_sunken;
    w.inactive.weak_bg_fill = Color32::TRANSPARENT;
    w.inactive.bg_stroke = Stroke::new(fine, p.line_strong);
    w.inactive.fg_stroke = Stroke::new(fine, p.text);
    for (state, overlay) in [
        (&mut w.hovered, p.hover),
        (&mut w.active, p.pressed),
        (&mut w.open, p.hover),
    ] {
        state.bg_fill = p.surface_sunken;
        state.weak_bg_fill = overlay;
        state.bg_stroke = Stroke::new(fine, p.line_strong);
        state.fg_stroke = Stroke::new(fine, p.text);
    }
    for state in [
        &mut w.noninteractive,
        &mut w.inactive,
        &mut w.hovered,
        &mut w.active,
        &mut w.open,
    ] {
        state.corner_radius = radius(TOKENS.radius.medium);
        state.expansion = 0.0;
    }
    v
}

/// Vollständiger Stil (Visuals, Abstände, Schriftstufen) für hell oder dunkel.
pub fn style(dark: bool, app: AppAccent) -> Style {
    let mut s = Style {
        visuals: visuals(dark, app),
        ..Style::default()
    };
    let sp = &TOKENS.spacing;
    let gap = f32::from(sp.s2);
    s.spacing.item_spacing = egui::vec2(gap, gap);
    let pad_y = (TOKENS.size.control - TOKENS.font.steps.text.line_height) / 2.0;
    s.spacing.button_padding = egui::vec2(f32::from(sp.s3), pad_y);
    s.spacing.window_margin = margin(sp.s6);
    s.spacing.menu_margin = margin(sp.s1);
    s.spacing.interact_size = egui::vec2(TOKENS.size.control, TOKENS.size.control);
    s.spacing.indent = f32::from(sp.s4);
    s.animation_time = f32::from(TOKENS.motion.selection_ms) / 1000.0;
    let steps = &TOKENS.font.steps;
    for (style, size, family) in [
        (TextStyle::Small, steps.small.size, FontFamily::Proportional),
        (TextStyle::Body, steps.text.size, FontFamily::Proportional),
        (TextStyle::Button, steps.text_bold.size, fonts::semibold()),
        (TextStyle::Heading, steps.title.size, fonts::semibold()),
    ] {
        s.text_styles.insert(style, FontId::new(size, family));
    }
    s
}

/// Schriften laden, beide Themen setzen und die App merken. Einmal beim Start aufrufen.
pub fn install(ctx: &Context, app: AppAccent) {
    fonts::install(ctx);
    ctx.set_style_of(Theme::Light, style(false, app));
    ctx.set_style_of(Theme::Dark, style(true, app));
    ctx.data_mut(|d| d.insert_temp(accent_id(), app));
}

/// Welche App zeichnet (Standard: Desk).
pub fn accent_of(ctx: &Context) -> AppAccent {
    ctx.data(|d| d.get_temp(accent_id())).unwrap_or_default()
}

/// Farben für hell oder dunkel und die gemerkte App.
pub fn look(ctx: &Context, dark: bool) -> Look {
    let p = palette(dark);
    Look {
        dark,
        p,
        accent: p.accent(accent_of(ctx)),
    }
}

/// Farben passend zum Stil dieses Bereichs (eine Galerie kann hell und dunkel nebeneinander zeigen).
pub fn look_ui(ui: &Ui) -> Look {
    look(ui.ctx(), ui.visuals().dark_mode)
}

#[cfg(test)]
mod tests;
