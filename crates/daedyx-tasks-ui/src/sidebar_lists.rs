//! Eigene Listen in der Seitenleiste: direkt in der Zeile umbenennen (F2, Enter übernimmt, Esc bricht
//! ab), Entf fragt vor dem Löschen, unten das Feld „Neue Liste“.
use super::{Ask, TasksUi};
use daedyx_ui::i18n::Texts;
use daedyx_ui::tokens::TOKENS;
use daedyx_ui::widgets::text_input;
use eframe::egui::{Id, Key, Modifiers, TextEdit, Ui};

pub fn rename_id(list: &str) -> Id {
    Id::unique("liste_name").with(list)
}

/// Eingabefeld statt der Zeile, solange die Liste umbenannt wird. `true`, wenn es gezeigt wurde.
pub fn rename_field(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts, list: &str) -> bool {
    let Some((rid, draft)) = tasks.renaming_list.as_mut().filter(|(r, _)| r == list) else {
        return false;
    };
    let id = rename_id(rid);
    let resp = ui.add(
        TextEdit::singleline(draft)
            .id(id)
            .char_limit(200)
            .desired_width(f32::INFINITY),
    );
    ui.ctx()
        .accesskit_node_builder(id, |b| b.set_label(texts.t("aufgaben.liste.name_feld")));
    if !resp.has_focus() && !resp.lost_focus() {
        resp.request_focus();
    }
    if resp.lost_focus() {
        let escape = ui.input(|i| i.key_pressed(Key::Escape));
        if let Some((rid, draft)) = tasks.renaming_list.take().filter(|_| !escape) {
            tasks.rename_list(&rid, &draft);
        }
    }
    true
}

/// Tasten auf einer Listenzeile mit Fokus: F2 benennt um, Entf fragt vor dem Löschen.
pub fn keys(ui: &mut Ui, tasks: &mut TasksUi, list: &str, name: &str) {
    let [f2, del] =
        [Key::F2, Key::Delete].map(|k| ui.input_mut(|i| i.consume_key(Modifiers::NONE, k)));
    if f2 {
        tasks.renaming_list = Some((list.to_owned(), name.to_owned()));
    }
    if del {
        tasks.ask = Some(Ask::List(list.to_owned()));
    }
}

/// Feld „Neue Liste“ unter den Listen.
pub fn new_list(ui: &mut Ui, tasks: &mut TasksUi, texts: &Texts) {
    ui.add_space(f32::from(TOKENS.spacing.s2));
    let r = text_input(
        ui,
        &mut tasks.new_list,
        &texts.t("aufgaben.liste.neu"),
        &texts.t("aufgaben.liste.neu.hinweis"),
        None,
    );
    if r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
        tasks.add_list();
    }
}
