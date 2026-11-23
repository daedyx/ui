//! Die Abschnitte der Galerie, je einer pro Baustein-Gruppe.
use crate::app::Galerie;
use daedyx_ui::keys::{self, Action, SHORTCUTS};
use daedyx_ui::theme::look_ui;
use daedyx_ui::tokens::TOKENS;
use daedyx_ui::widgets::{
    BadgeTone, ButtonKind, ListRow, MenuItem, badge, button, button_enabled, card, empty_state,
    error_state, list, menu_button, text_input,
};
use eframe::egui::{Frame, Margin, RichText, Stroke, Ui};

/// Rahmen eines Abschnitts: Fläche, feine Linie, großer Radius.
fn section(ui: &mut Ui, title: &str, add: impl FnOnce(&mut Ui)) {
    let look = look_ui(ui);
    let pad = i8::try_from(TOKENS.spacing.s4).unwrap_or(i8::MAX);
    let radius = u8::try_from(TOKENS.radius.large).unwrap_or(u8::MAX);
    Frame::new()
        .fill(look.p.surface)
        .stroke(Stroke::new(TOKENS.line.fine, look.p.line))
        .corner_radius(radius)
        .inner_margin(Margin::same(pad))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.heading(RichText::new(title).color(look.p.text));
            ui.add_space(f32::from(TOKENS.spacing.s2));
            add(ui);
        });
    ui.add_space(f32::from(TOKENS.spacing.s4));
}

fn buttons(ui: &mut Ui, g: &Galerie) {
    ui.horizontal_wrapped(|ui| {
        let _ = button(ui, ButtonKind::Primary, &g.texts.t("knopf.speichern"));
        let _ = button(ui, ButtonKind::Secondary, &g.texts.t("knopf.weiter"));
        let _ = button(ui, ButtonKind::Quiet, &g.texts.t("knopf.abbrechen"));
        let _ = button(ui, ButtonKind::Danger, &g.texts.t("knopf.loeschen"));
        let _ = button_enabled(ui, ButtonKind::Primary, &g.texts.t("knopf.aus"), false);
    });
}

fn inputs(ui: &mut Ui, g: &mut Galerie) {
    let (label, hint, error) = (
        g.texts.t("eingabe.betreff"),
        g.texts.t("eingabe.betreff_hinweis"),
        g.texts.t("eingabe.pflicht"),
    );
    let missing = g.subject.trim().is_empty();
    let _ = text_input(
        ui,
        &mut g.subject,
        &label,
        &hint,
        missing.then_some(error.as_str()),
    );
}

fn lists(ui: &mut Ui, g: &mut Galerie) {
    let titles = [
        g.texts.t("liste.eintrag1"),
        g.texts.t("liste.eintrag2"),
        g.texts.t("liste.eintrag3"),
    ];
    let details = [
        g.texts.t("marke.warnung"),
        g.texts.t("marke.neutral"),
        g.texts.t("marke.ok"),
    ];
    let rows: Vec<ListRow<'_>> = titles
        .iter()
        .zip(details.iter())
        .enumerate()
        .map(|(i, (title, detail))| ListRow {
            title,
            detail,
            unread: i == 0,
        })
        .collect();
    let _ = list(ui, &g.texts.t("liste.name"), &rows, &mut g.list_selected);
}

fn cards(ui: &mut Ui, g: &mut Galerie) {
    if card(
        ui,
        &g.texts.t("karte.titel"),
        &g.texts.t("karte.text"),
        g.card_selected,
    )
    .clicked()
    {
        g.card_selected = !g.card_selected;
    }
    let _ = card(
        ui,
        &g.texts.t("karte.titel2"),
        &g.texts.t("karte.text2"),
        false,
    );
}

fn badges(ui: &mut Ui, g: &Galerie) {
    ui.horizontal_wrapped(|ui| {
        let _ = badge(ui, BadgeTone::Neutral, &g.texts.t("marke.neutral"));
        let _ = badge(
            ui,
            BadgeTone::Accent,
            &g.texts.tf("marke.akzent", &[("anzahl", "3")]),
        );
        let _ = badge(ui, BadgeTone::Ok, &g.texts.t("marke.ok"));
        let _ = badge(ui, BadgeTone::Warning, &g.texts.t("marke.warnung"));
        let _ = badge(ui, BadgeTone::Error, &g.texts.t("marke.fehler"));
        let _ = badge(ui, BadgeTone::Info, &g.texts.t("marke.info"));
    });
}

fn dialog_and_menu(ui: &mut Ui, g: &mut Galerie) {
    let mac = cfg!(target_os = "macos");
    let delete_keys = keys::find(Action::Delete).map(|s| keys::label(&g.texts, s, mac));
    let items = [
        MenuItem {
            label: &g.texts.t("menue.umbenennen"),
            shortcut: None,
            danger: false,
        },
        MenuItem {
            label: &g.texts.t("menue.kopieren"),
            shortcut: None,
            danger: false,
        },
        MenuItem {
            label: &g.texts.t("menue.loeschen"),
            shortcut: delete_keys,
            danger: true,
        },
    ];
    ui.horizontal(|ui| {
        if button(ui, ButtonKind::Secondary, &g.texts.t("dialog.oeffnen")).clicked() {
            g.dialog_open = true;
        }
        if let Some(i) = menu_button(ui, &g.texts.t("menue.oeffnen"), &items) {
            g.last_menu = Some(i);
        }
    });
}

fn states(ui: &mut Ui, g: &Galerie) {
    ui.columns(2, |cols| {
        if let [left, right] = cols {
            let _ = empty_state(
                left,
                &g.texts.t("leer.titel"),
                &g.texts.t("leer.text"),
                Some(&g.texts.t("leer.knopf")),
            );
            let _ = error_state(
                right,
                &g.texts.t("fehler.titel"),
                &g.texts.t("fehler.text"),
                &g.texts.t("fehler.erneut"),
                Some(&g.texts.t("fehler.konto")),
            );
        }
    });
}

fn bridge(ui: &mut Ui, g: &mut Galerie) {
    ui.horizontal(|ui| {
        if button(ui, ButtonKind::Primary, &g.texts.t("bruecke.fragen")).clicked() {
            g.waiting = g.bridge.as_ref().is_some_and(|b| b.send(7).is_ok());
        }
        let status = match (g.bridge.is_some(), g.waiting, g.answer) {
            (false, _, _) => g.texts.t("bruecke.getrennt"),
            (true, true, _) => g.texts.t("bruecke.wartet"),
            (true, false, Some(n)) => g.texts.tf("bruecke.antwort", &[("wert", &n.to_string())]),
            (true, false, None) => String::new(),
        };
        ui.label(status);
    });
}

fn shortcuts(ui: &mut Ui, g: &Galerie) {
    let mac = cfg!(target_os = "macos");
    let faint = look_ui(ui).p.text_faint;
    for s in SHORTCUTS {
        ui.horizontal(|ui| {
            ui.label(g.texts.t(s.label_key));
            ui.label(RichText::new(keys::label(&g.texts, s, mac)).color(faint));
        });
    }
}

pub fn all(ui: &mut Ui, g: &mut Galerie) {
    ui.columns(2, |cols| {
        if let [left, right] = cols {
            section(left, &g.texts.t("bereich.knoepfe"), |ui| buttons(ui, g));
            section(left, &g.texts.t("bereich.eingabe"), |ui| inputs(ui, g));
            section(left, &g.texts.t("bereich.liste"), |ui| lists(ui, g));
            section(left, &g.texts.t("bereich.karten"), |ui| cards(ui, g));
            section(right, &g.texts.t("bereich.marken"), |ui| badges(ui, g));
            section(right, &g.texts.t("bereich.dialog_menue"), |ui| {
                dialog_and_menu(ui, g)
            });
            section(right, &g.texts.t("bereich.zustaende"), |ui| states(ui, g));
            section(right, &g.texts.t("bereich.bruecke"), |ui| bridge(ui, g));
            section(right, &g.texts.t("kuerzel.tabelle"), |ui| shortcuts(ui, g));
        }
    });
}
