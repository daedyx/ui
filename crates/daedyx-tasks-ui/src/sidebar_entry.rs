//! Ein Eintrag der Seitenleiste (Smart-Liste oder eigene Liste): Name links, Zähler oder Farbpunkt, ausgewählt mit
//! weicher Akzentfläche. Eigene Listen lassen sich mit `Features::lists_order` ziehen.
use super::Sel;
use super::sidebar::view_key;
use daedyx_ui::a11y::mark_selected;
use daedyx_ui::fonts;
use daedyx_ui::i18n::Texts;
use daedyx_ui::theme::look_ui;
use daedyx_ui::tokens::TOKENS;
use daedyx_ui::widgets::focus;
use eframe::egui::{
    Align2, Color32, FontFamily, FontId, Id, Sense, Ui, WidgetInfo, accesskit::Role, pos2, vec2,
};

const ROW: f32 = 32.0;

pub fn row_id(sel: &Sel) -> Id {
    match sel {
        Sel::View(v) => Id::unique("aufgaben_ansicht").with(view_key(*v)),
        Sel::List(id) => Id::unique("aufgaben_liste").with(id.as_str()),
    }
}

/// Ein Eintrag: Name links, Zähler rechts; ausgewählt = weiche Akzentfläche.
/// Ein Eintrag der Leiste: Auswahl, Name, Zähler (Smart-Listen) oder Farbpunkt (eigene Listen).
pub struct Entry {
    pub sel: Sel,
    pub name: String,
    pub count: Option<usize>,
    pub dot: Option<Color32>,
}

pub fn entry(
    ui: &mut Ui,
    e: &Entry,
    active: bool,
    drag: bool,
    texts: &Texts,
) -> eframe::egui::Response {
    let (sel, name, count) = (&e.sel, e.name.as_str(), e.count);
    let look = look_ui(ui);
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::hover());
    let sense = if drag {
        Sense::click_and_drag()
    } else {
        Sense::click()
    };
    let resp = ui.interact(rect, row_id(sel), sense);
    let label = match count {
        Some(n) => texts.tf(
            "aufgaben.leiste.eintrag",
            &[("name", name), ("anzahl", &n.to_string())],
        ),
        None => name.to_owned(),
    };
    resp.widget_info(|| WidgetInfo::labeled(Role::ListItem, true, &label));
    mark_selected(ui, resp.id, active);
    if ui.is_rect_visible(rect) {
        let radius = u8::try_from(TOKENS.radius.medium).unwrap_or(u8::MAX);
        let painter = ui.painter();
        if active {
            painter.rect_filled(rect, radius, look.accent.soft);
        } else if resp.hovered() {
            painter.rect_filled(rect, radius, look.p.hover);
        }
        let (family, color) = if active {
            (fonts::semibold_in(ui.ctx()), look.accent.text)
        } else {
            (FontFamily::Proportional, look.p.text)
        };
        let pad = f32::from(TOKENS.spacing.s3);
        let font = FontId::new(TOKENS.font.steps.text.size, family);
        let mut x = rect.left() + pad;
        if let Some(dot) = e.dot {
            painter.circle_filled(pos2(x + 4.0, rect.center().y), 4.0, dot);
            x += 16.0;
        }
        painter.text(
            pos2(x, rect.center().y),
            Align2::LEFT_CENTER,
            name,
            font.clone(),
            color,
        );
        if let Some(n) = count.filter(|n| *n > 0) {
            painter.text(
                pos2(rect.right() - pad, rect.center().y),
                Align2::RIGHT_CENTER,
                n.to_string(),
                font,
                look.p.text_muted,
            );
        }
        if resp.has_focus() {
            focus::ring(painter, rect, &look);
        }
    }
    resp
}
