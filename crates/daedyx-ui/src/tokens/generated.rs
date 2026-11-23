//! Erzeugt aus `assets/tokens.json` vom Test `tokens::tests::erzeugte_datei_passt_zur_json`.
//! Nicht von Hand ändern. Neu schreiben: `DAEDYX_TOKENS_SCHREIBEN=1 cargo test -p daedyx-ui tokens`.
use super::metrics::*;
use super::schema::*;
use eframe::egui::Color32;

pub const TOKENS: TokenFile<Color32> = TokenFile {
    color: Themes {
        light: Palette {
            window: Color32::from_rgba_unmultiplied_const(244, 245, 249, 255),
            surface: Color32::from_rgba_unmultiplied_const(255, 255, 255, 255),
            surface_sunken: Color32::from_rgba_unmultiplied_const(236, 238, 244, 255),
            sidebar: Color32::from_rgba_unmultiplied_const(234, 236, 243, 255),
            line: Color32::from_rgba_unmultiplied_const(225, 227, 236, 255),
            line_strong: Color32::from_rgba_unmultiplied_const(198, 201, 216, 255),
            text: Color32::from_rgba_unmultiplied_const(23, 26, 42, 255),
            text_muted: Color32::from_rgba_unmultiplied_const(87, 92, 114, 255),
            text_faint: Color32::from_rgba_unmultiplied_const(98, 103, 126, 255),
            overlay: Color32::from_rgba_unmultiplied_const(23, 26, 42, 102),
            hover: Color32::from_rgba_unmultiplied_const(23, 26, 42, 10),
            pressed: Color32::from_rgba_unmultiplied_const(23, 26, 42, 20),
            focus_ring: Color32::from_rgba_unmultiplied_const(62, 61, 163, 255),
            accent: Accents {
                desk: Accent {
                    full: Color32::from_rgba_unmultiplied_const(62, 61, 163, 255),
                    on_full: Color32::from_rgba_unmultiplied_const(255, 255, 255, 255),
                    soft: Color32::from_rgba_unmultiplied_const(230, 230, 248, 255),
                    text: Color32::from_rgba_unmultiplied_const(62, 61, 163, 255),
                },
                tasks: Accent {
                    full: Color32::from_rgba_unmultiplied_const(232, 163, 61, 255),
                    on_full: Color32::from_rgba_unmultiplied_const(43, 28, 3, 255),
                    soft: Color32::from_rgba_unmultiplied_const(251, 235, 208, 255),
                    text: Color32::from_rgba_unmultiplied_const(127, 75, 0, 255),
                },
                board: Accent {
                    full: Color32::from_rgba_unmultiplied_const(31, 127, 142, 255),
                    on_full: Color32::from_rgba_unmultiplied_const(255, 255, 255, 255),
                    soft: Color32::from_rgba_unmultiplied_const(216, 238, 241, 255),
                    text: Color32::from_rgba_unmultiplied_const(23, 101, 115, 255),
                },
            },
            state: States {
                ok: Tone {
                    text: Color32::from_rgba_unmultiplied_const(23, 112, 58, 255),
                    soft: Color32::from_rgba_unmultiplied_const(220, 243, 228, 255),
                },
                warning: Tone {
                    text: Color32::from_rgba_unmultiplied_const(138, 83, 0, 255),
                    soft: Color32::from_rgba_unmultiplied_const(251, 235, 208, 255),
                },
                error: StrongTone {
                    text: Color32::from_rgba_unmultiplied_const(180, 35, 24, 255),
                    soft: Color32::from_rgba_unmultiplied_const(252, 228, 225, 255),
                    full: Color32::from_rgba_unmultiplied_const(180, 35, 24, 255),
                    on_full: Color32::from_rgba_unmultiplied_const(255, 255, 255, 255),
                },
                info: Tone {
                    text: Color32::from_rgba_unmultiplied_const(29, 95, 184, 255),
                    soft: Color32::from_rgba_unmultiplied_const(222, 233, 250, 255),
                },
            },
            kanban: Kanban {
                open: Lane {
                    dot: Color32::from_rgba_unmultiplied_const(107, 112, 136, 255),
                    soft: Color32::from_rgba_unmultiplied_const(236, 238, 244, 255),
                },
                planned: Lane {
                    dot: Color32::from_rgba_unmultiplied_const(62, 61, 163, 255),
                    soft: Color32::from_rgba_unmultiplied_const(230, 230, 248, 255),
                },
                in_progress: Lane {
                    dot: Color32::from_rgba_unmultiplied_const(194, 122, 18, 255),
                    soft: Color32::from_rgba_unmultiplied_const(251, 235, 208, 255),
                },
                review: Lane {
                    dot: Color32::from_rgba_unmultiplied_const(31, 127, 142, 255),
                    soft: Color32::from_rgba_unmultiplied_const(216, 238, 241, 255),
                },
                done: Lane {
                    dot: Color32::from_rgba_unmultiplied_const(46, 158, 91, 255),
                    soft: Color32::from_rgba_unmultiplied_const(220, 243, 228, 255),
                },
                stuck: Lane {
                    dot: Color32::from_rgba_unmultiplied_const(180, 35, 24, 255),
                    soft: Color32::from_rgba_unmultiplied_const(252, 228, 225, 255),
                },
            },
            priority: Priority {
                low: Color32::from_rgba_unmultiplied_const(98, 103, 126, 255),
                medium: Color32::from_rgba_unmultiplied_const(138, 83, 0, 255),
                high: Color32::from_rgba_unmultiplied_const(180, 35, 24, 255),
            },
            calendar: [
                Color32::from_rgba_unmultiplied_const(62, 61, 163, 255),
                Color32::from_rgba_unmultiplied_const(23, 112, 58, 255),
                Color32::from_rgba_unmultiplied_const(138, 83, 0, 255),
                Color32::from_rgba_unmultiplied_const(31, 127, 142, 255),
                Color32::from_rgba_unmultiplied_const(163, 24, 94, 255),
                Color32::from_rgba_unmultiplied_const(180, 35, 24, 255),
            ],
        },
        dark: Palette {
            window: Color32::from_rgba_unmultiplied_const(15, 17, 23, 255),
            surface: Color32::from_rgba_unmultiplied_const(23, 26, 35, 255),
            surface_sunken: Color32::from_rgba_unmultiplied_const(11, 13, 18, 255),
            sidebar: Color32::from_rgba_unmultiplied_const(18, 20, 28, 255),
            line: Color32::from_rgba_unmultiplied_const(38, 42, 55, 255),
            line_strong: Color32::from_rgba_unmultiplied_const(58, 63, 80, 255),
            text: Color32::from_rgba_unmultiplied_const(231, 233, 242, 255),
            text_muted: Color32::from_rgba_unmultiplied_const(163, 168, 188, 255),
            text_faint: Color32::from_rgba_unmultiplied_const(138, 144, 166, 255),
            overlay: Color32::from_rgba_unmultiplied_const(0, 0, 0, 153),
            hover: Color32::from_rgba_unmultiplied_const(255, 255, 255, 13),
            pressed: Color32::from_rgba_unmultiplied_const(255, 255, 255, 23),
            focus_ring: Color32::from_rgba_unmultiplied_const(157, 156, 242, 255),
            accent: Accents {
                desk: Accent {
                    full: Color32::from_rgba_unmultiplied_const(157, 156, 242, 255),
                    on_full: Color32::from_rgba_unmultiplied_const(18, 18, 58, 255),
                    soft: Color32::from_rgba_unmultiplied_const(38, 38, 79, 255),
                    text: Color32::from_rgba_unmultiplied_const(157, 156, 242, 255),
                },
                tasks: Accent {
                    full: Color32::from_rgba_unmultiplied_const(242, 180, 80, 255),
                    on_full: Color32::from_rgba_unmultiplied_const(43, 28, 3, 255),
                    soft: Color32::from_rgba_unmultiplied_const(58, 42, 14, 255),
                    text: Color32::from_rgba_unmultiplied_const(242, 180, 80, 255),
                },
                board: Accent {
                    full: Color32::from_rgba_unmultiplied_const(79, 184, 200, 255),
                    on_full: Color32::from_rgba_unmultiplied_const(6, 36, 42, 255),
                    soft: Color32::from_rgba_unmultiplied_const(18, 56, 64, 255),
                    text: Color32::from_rgba_unmultiplied_const(92, 195, 211, 255),
                },
            },
            state: States {
                ok: Tone {
                    text: Color32::from_rgba_unmultiplied_const(88, 200, 135, 255),
                    soft: Color32::from_rgba_unmultiplied_const(20, 50, 34, 255),
                },
                warning: Tone {
                    text: Color32::from_rgba_unmultiplied_const(242, 180, 80, 255),
                    soft: Color32::from_rgba_unmultiplied_const(58, 42, 14, 255),
                },
                error: StrongTone {
                    text: Color32::from_rgba_unmultiplied_const(242, 139, 130, 255),
                    soft: Color32::from_rgba_unmultiplied_const(64, 26, 23, 255),
                    full: Color32::from_rgba_unmultiplied_const(242, 139, 130, 255),
                    on_full: Color32::from_rgba_unmultiplied_const(42, 9, 7, 255),
                },
                info: Tone {
                    text: Color32::from_rgba_unmultiplied_const(127, 174, 240, 255),
                    soft: Color32::from_rgba_unmultiplied_const(23, 41, 74, 255),
                },
            },
            kanban: Kanban {
                open: Lane {
                    dot: Color32::from_rgba_unmultiplied_const(138, 144, 166, 255),
                    soft: Color32::from_rgba_unmultiplied_const(31, 35, 48, 255),
                },
                planned: Lane {
                    dot: Color32::from_rgba_unmultiplied_const(157, 156, 242, 255),
                    soft: Color32::from_rgba_unmultiplied_const(38, 38, 79, 255),
                },
                in_progress: Lane {
                    dot: Color32::from_rgba_unmultiplied_const(242, 180, 80, 255),
                    soft: Color32::from_rgba_unmultiplied_const(58, 42, 14, 255),
                },
                review: Lane {
                    dot: Color32::from_rgba_unmultiplied_const(79, 184, 200, 255),
                    soft: Color32::from_rgba_unmultiplied_const(18, 56, 64, 255),
                },
                done: Lane {
                    dot: Color32::from_rgba_unmultiplied_const(88, 200, 135, 255),
                    soft: Color32::from_rgba_unmultiplied_const(20, 50, 34, 255),
                },
                stuck: Lane {
                    dot: Color32::from_rgba_unmultiplied_const(242, 139, 130, 255),
                    soft: Color32::from_rgba_unmultiplied_const(64, 26, 23, 255),
                },
            },
            priority: Priority {
                low: Color32::from_rgba_unmultiplied_const(163, 168, 188, 255),
                medium: Color32::from_rgba_unmultiplied_const(242, 180, 80, 255),
                high: Color32::from_rgba_unmultiplied_const(242, 139, 130, 255),
            },
            calendar: [
                Color32::from_rgba_unmultiplied_const(157, 156, 242, 255),
                Color32::from_rgba_unmultiplied_const(88, 200, 135, 255),
                Color32::from_rgba_unmultiplied_const(242, 180, 80, 255),
                Color32::from_rgba_unmultiplied_const(79, 184, 200, 255),
                Color32::from_rgba_unmultiplied_const(240, 143, 192, 255),
                Color32::from_rgba_unmultiplied_const(242, 139, 130, 255),
            ],
        },
    },
    font: Font {
        steps: TypeSteps {
            small: TypeStep {
                size: 12.0,
                line_height: 16.0,
                weight: 400,
            },
            text: TypeStep {
                size: 14.0,
                line_height: 20.0,
                weight: 400,
            },
            text_bold: TypeStep {
                size: 14.0,
                line_height: 20.0,
                weight: 600,
            },
            emphasis: TypeStep {
                size: 16.0,
                line_height: 24.0,
                weight: 600,
            },
            title: TypeStep {
                size: 20.0,
                line_height: 28.0,
                weight: 600,
            },
            large: TypeStep {
                size: 28.0,
                line_height: 36.0,
                weight: 600,
            },
        },
    },
    spacing: Spacing {
        s1: 4,
        s2: 8,
        s3: 12,
        s4: 16,
        s5: 20,
        s6: 24,
        s8: 32,
        s10: 40,
    },
    radius: Radius {
        small: 4,
        medium: 8,
        large: 12,
        round: 999,
    },
    line: Line {
        fine: 1.0,
        focus: 2.0,
    },
    shadow: Shadows {
        floating: ShadowPair {
            light: ShadowSpec {
                offset_y: 4,
                blur: 16,
                color: Color32::from_rgba_unmultiplied_const(23, 26, 42, 41),
            },
            dark: ShadowSpec {
                offset_y: 6,
                blur: 20,
                color: Color32::from_rgba_unmultiplied_const(0, 0, 0, 166),
            },
        },
        dialog: ShadowPair {
            light: ShadowSpec {
                offset_y: 12,
                blur: 40,
                color: Color32::from_rgba_unmultiplied_const(23, 26, 42, 61),
            },
            dark: ShadowSpec {
                offset_y: 16,
                blur: 48,
                color: Color32::from_rgba_unmultiplied_const(0, 0, 0, 204),
            },
        },
    },
    size: Sizes {
        control: 32.0,
        control_large: 40.0,
        list_row: 56.0,
        list_row_compact: 36.0,
        sidebar_narrow: 56.0,
        sidebar_wide: 208.0,
        header: 48.0,
        icon: 20.0,
        icon_small: 16.0,
        icon_large: 24.0,
        avatar: 32.0,
        hit_target_min: 32.0,
    },
    motion: Motion {
        hover_ms: 90,
        selection_ms: 120,
        dialog_ms: 150,
    },
};
