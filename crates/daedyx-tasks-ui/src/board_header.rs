//! Kopf einer Board-Spalte: Farbpunkt, Name, Zähler, Knopf „Aktionen“; Umbenennen direkt im Kopf.
//! Kopf mit Fokus: F2 benennt um, Entf fragt vor dem Löschen, Alt+Pfeil links/rechts verschiebt die Spalte.
use super::{Ask, TasksUi};
use daedyx_ui::fonts;
use daedyx_ui::i18n::Texts;
use daedyx_ui::theme::look_ui;
use daedyx_ui::tokens::{Lane, TOKENS};
use daedyx_ui::widgets::{MenuItem, focus, menu_button};
use eframe::egui::{
    Align2, Color32, FontId, Id, Key, Modifiers, Response, Sense, TextEdit, Ui, WidgetInfo,
    accesskit::Role, pos2, vec2,
};

/// Was beim Ziehen eines Spaltenkopfs mitgetragen wird.
#[derive(Debug, Clone)]
pub struct SectionDrag(pub String);

pub fn header_id(section: Option<&str>) -> Id {
    Id::unique("abschnitt").with(section.unwrap_or(""))
}

pub fn header(
    ui: &mut Ui,
    tasks: &mut TasksUi,
    texts: &Texts,
    sid: Option<&str>,
    name: &str,
    n: usize,
    lane: Lane<Color32>,
) -> (Response, Option<isize>) {
    let look = look_ui(ui);
    if let Some((rid, draft)) = tasks
        .renaming
        .as_mut()
        .filter(|(r, _)| Some(r.as_str()) == sid)
    {
        let id = Id::unique("abschnitt_name").with(rid.as_str());
        let resp = ui.add(
            TextEdit::singleline(draft)
                .id(id)
                .char_limit(200)
                .desired_width(f32::INFINITY),
        );
        ui.ctx()
            .accesskit_node_builder(id, |b| b.set_label(texts.t("aufgaben.abschnitt.name")));
        if !resp.has_focus() && !resp.lost_focus() {
            resp.request_focus();
        }
        if resp.lost_focus() {
            let escape = ui.input(|i| i.key_pressed(Key::Escape));
            if let Some((rid, draft)) = tasks.renaming.take().filter(|_| !escape) {
                tasks.rename_section(&rid, &draft);
            }
        }
        return (resp, None);
    }
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::hover());
    let resp = ui.interact(rect, header_id(sid), Sense::click_and_drag());
    let label = texts.tf(
        "aufgaben.abschnitt.kopf",
        &[("name", name), ("anzahl", &n.to_string())],
    );
    resp.widget_info(|| WidgetInfo::labeled(Role::ColumnHeader, true, &label));
    if let Some(s) = sid {
        resp.dnd_set_drag_payload(SectionDrag(s.to_owned()));
    }
    let painter = ui.painter();
    painter.circle_filled(pos2(rect.left() + 6.0, rect.center().y), 4.0, lane.dot);
    let font = FontId::new(
        TOKENS.font.steps.text_bold.size,
        fonts::semibold_in(ui.ctx()),
    );
    painter.text(
        pos2(rect.left() + 18.0, rect.center().y),
        Align2::LEFT_CENTER,
        name,
        font,
        look.p.text,
    );
    let small = FontId::proportional(TOKENS.font.steps.small.size);
    painter.text(
        pos2(rect.right() - 4.0, rect.center().y),
        Align2::RIGHT_CENTER,
        n.to_string(),
        small,
        look.p.text_muted,
    );
    if resp.has_focus() {
        focus::ring(painter, rect, &look);
    }
    let mut step = None;
    if let (Some(s), true) = (sid, resp.has_focus()) {
        let [left, right] = [Key::ArrowLeft, Key::ArrowRight]
            .map(|k| ui.input_mut(|i| i.consume_key(Modifiers::ALT, k)));
        let [f2, del] =
            [Key::F2, Key::Delete].map(|k| ui.input_mut(|i| i.consume_key(Modifiers::NONE, k)));
        step = if left {
            Some(-1)
        } else if right {
            Some(1)
        } else {
            None
        };
        if f2 {
            tasks.renaming = Some((s.to_owned(), name.to_owned()));
        }
        if del {
            tasks.ask = Some(Ask::Section(s.to_owned()));
        }
    }
    (resp, step)
}

pub fn menu(
    ui: &mut Ui,
    tasks: &mut TasksUi,
    texts: &Texts,
    sid: &str,
    name: &str,
) -> Option<isize> {
    let labels = [
        "aufgaben.abschnitt.umbenennen",
        "aufgaben.abschnitt.links",
        "aufgaben.abschnitt.rechts",
        "aufgaben.abschnitt.loeschen",
    ]
    .map(|k| texts.t(k));
    let items: Vec<MenuItem<'_>> = labels
        .iter()
        .enumerate()
        .map(|(i, l)| MenuItem {
            label: l,
            shortcut: None,
            danger: i == 3,
        })
        .collect();
    let title = texts.t("aufgaben.abschnitt.aktionen");
    let _ = name;
    match menu_button(ui, &title, &items)? {
        0 => tasks.renaming = Some((sid.to_owned(), name.to_owned())),
        1 => return Some(-1),
        2 => return Some(1),
        _ => tasks.ask = Some(Ask::Section(sid.to_owned())),
    }
    None
}
