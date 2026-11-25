//! Zeichnen einer Aufgabenzeile: Haken-Kreis (eigenes Kontrollkästchen), Titel, Marken rechts
//! (Fälligkeit, Priorität, Erinnerung, Wiederholung) und der Name für Screenreader.
use super::TasksUi;
use daedyx_tasks_core::types::Task;
use daedyx_ui::i18n::Texts;
use daedyx_ui::theme::{Look, look_ui};
use daedyx_ui::tokens::TOKENS;
use daedyx_ui::widgets::{badge, focus};
use eframe::egui::{
    Align, FontId, Id, Layout, Rect, Response, Sense, Stroke, TextFormat, Ui, UiBuilder,
    WidgetInfo, accesskit::Role, pos2, text::LayoutJob, vec2,
};

pub const CIRCLE: f32 = 18.0;

pub fn check_id(task: &str) -> Id {
    Id::unique("aufgabe_haken").with(task)
}

pub fn priority_key(p: u8) -> &'static str {
    match p {
        1 => "aufgaben.prioritaet.niedrig",
        2 => "aufgaben.prioritaet.mittel",
        3 => "aufgaben.prioritaet.hoch",
        _ => "aufgaben.prioritaet.keine",
    }
}

/// Name für Screenreader: Titel, Fälligkeit, Priorität, Zustand.
pub fn spoken(tasks: &TasksUi, texts: &Texts, t: &Task) -> String {
    let due = tasks
        .due_label(texts, t)
        .map(|(s, _)| s)
        .unwrap_or_else(|| texts.t("aufgaben.faellig.ohne"));
    let state = texts.t(if t.done_at.is_some() {
        "aufgaben.zustand.erledigt"
    } else {
        "aufgaben.zustand.offen"
    });
    texts.tf(
        "aufgaben.zeile",
        &[
            ("titel", &t.title),
            ("faellig", &due),
            ("prioritaet", &texts.t(priority_key(t.priority))),
            ("zustand", &state),
        ],
    )
}

/// Haken-Kreis als eigenes Kontrollkästchen. `true`, wenn geklickt.
pub fn check(
    ui: &mut Ui,
    look: &Look,
    t: &Task,
    center: eframe::egui::Pos2,
    texts: &Texts,
) -> bool {
    let rect = Rect::from_center_size(center, vec2(CIRCLE, CIRCLE));
    let resp = ui.interact(rect.expand(4.0), check_id(&t.id), Sense::click());
    let done = t.done_at.is_some();
    let label = texts.tf("aufgaben.haken", &[("titel", &t.title)]);
    resp.widget_info(|| WidgetInfo::selected(Role::CheckBox, true, done, &label));
    let painter = ui.painter();
    let ring = if done {
        look.p.state.ok.text
    } else {
        look.p.line_strong
    };
    painter.circle_stroke(center, CIRCLE / 2.0, Stroke::new(1.5, ring));
    if done {
        painter.circle_filled(center, CIRCLE / 2.0 - 3.0, look.p.state.ok.text);
    }
    if resp.has_focus() {
        focus::ring(painter, rect.expand(2.0), look);
    }
    resp.clicked()
}

fn title_job(look: &Look, t: &Task) -> LayoutJob {
    let done = t.done_at.is_some();
    let mut fmt = TextFormat::simple(
        FontId::proportional(TOKENS.font.steps.text.size),
        if done { look.p.text_faint } else { look.p.text },
    );
    if done {
        fmt.strikethrough = Stroke::new(1.0, look.p.text_faint);
    }
    let mut job = LayoutJob::single_section(t.title.clone(), fmt);
    job.wrap.max_rows = 1;
    job
}

/// Marken rechts in der Zeile (von rechts nach links).
fn marks(ui: &mut Ui, tasks: &TasksUi, texts: &Texts, look: &Look, t: &Task, rect: Rect) {
    let inner = rect.shrink2(vec2(f32::from(TOKENS.spacing.s3), 0.0));
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::right_to_left(Align::Center)),
    );
    if let Some((text, tone)) = tasks.due_label(texts, t) {
        let _ = badge(&mut child, tone, &text);
    }
    let p = &look.p.priority;
    if let Some(color) = [None, Some(p.low), Some(p.medium), Some(p.high)]
        .get(usize::from(t.priority))
        .copied()
        .flatten()
    {
        let (dot, _) = child.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
        child.painter().circle_filled(dot.center(), 4.0, color);
    }
    for (on, key) in [
        (t.remind_minutes.is_some(), "aufgaben.marke.erinnerung"),
        (t.repeat.is_some(), "aufgaben.marke.wiederholung"),
    ] {
        if on {
            child.label(
                eframe::egui::RichText::new(texts.t(key))
                    .small()
                    .color(look.p.text_muted),
            );
        }
    }
}

pub fn paint(
    ui: &mut Ui,
    tasks: &TasksUi,
    texts: &Texts,
    t: &Task,
    rect: Rect,
    resp: &Response,
    selected: bool,
) {
    let look = look_ui(ui);
    let radius = u8::try_from(TOKENS.radius.medium).unwrap_or(u8::MAX);
    if selected {
        ui.painter().rect_filled(rect, radius, look.accent.soft);
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, radius, look.p.hover);
    }
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(TOKENS.line.fine, look.p.line),
    );
    let galley = ui.painter().layout_job(title_job(&look, t));
    let x = rect.left() + CIRCLE + f32::from(TOKENS.spacing.s5);
    ui.painter().galley(
        pos2(x, rect.center().y - galley.size().y / 2.0),
        galley,
        look.p.text,
    );
    marks(
        ui,
        tasks,
        texts,
        &look,
        t,
        Rect::from_min_max(pos2(rect.center().x, rect.top()), rect.max),
    );
    if resp.has_focus() {
        focus::ring(ui.painter(), rect, &look);
    }
}
