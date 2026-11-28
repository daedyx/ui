//! Tasks 0.2.1 (Björns Meldung, 2000×1240 px bei 150 %): Der Detailbereich bleibt in seinen 300 px, und die Mitte
//! (Liste oder Board) endet vor ihm. War eine Zeile im Detail breiter als sein Inhalt (Priorität: 288 statt 268 px),
//! maß egui den Bereich zu breit, die Mitte malte über seinen linken Rand („ufgabe“) und Board-Spalten liefen
//! darunter. Geprüft für jede Aufgabe der ersten Liste, in Liste und Board, für Desk und Tasks, in allen Sprachen
//! und bei mehreren Fensterbreiten.
use crate::test_support::harness_in;
use crate::texts::texts;
use crate::{Features, TasksUi};
use daedyx_ui::i18n::Language;
use daedyx_ui::tokens::TOKENS;
use eframe::egui::accesskit::Role;
use eframe::egui::{Rect, vec2};
use egui_kittest::kittest::{By, NodeT, Queryable};
use egui_kittest::{Harness, Node};

/// Breite des Detailbereichs (`layout::DETAIL`; mit Rand und Strich zeichnet egui ihn etwas breiter).
const DETAIL: f32 = 300.0;
/// Fensterbreiten: 1333 wie Björns Bildschirm in logischen Punkten, dazu breiter und schmaler.
const WIDTHS: [f32; 6] = [2000.0, 1333.0, 1100.0, 900.0, 760.0, 640.0];
/// Ab hier passt auch die Mitte ganz; darunter ist sie so schmal, dass ein einzelner Knopf übersteht. Den schneidet
/// sie an ihrer eigenen Kante ab (Clip der Mitte), er malt nicht über den Detailbereich.
const CENTER_FITS: f32 = 900.0;

type H = Harness<'static, TasksUi>;

/// Rechteck des Knotens in Punkten; Knoten ohne eigenes Rechteck (Behälter) fallen weg.
fn rect(n: &Node<'_>) -> Option<Rect> {
    n.accesskit_node().bounding_box().map(|_| n.rect())
}

/// Waagerechte Ausdehnung aller Knoten im Detailbereich der gewählten Aufgabe.
fn detail_span(h: &H, lang: Language) -> (f32, f32, String) {
    let start = texts(lang).tf("aufgaben.detail.bereich", &[("titel", "")]);
    let group = h
        .query_all_by_role(Role::Group)
        .find(|n| {
            n.accesskit_node()
                .label()
                .is_some_and(|l| l.starts_with(&start))
        })
        .expect("Detailbereich");
    let nodes: Vec<(Rect, String)> = group
        .query_all(By::new().predicate(|_| true))
        .filter_map(|n| {
            Some((
                rect(&n)?,
                format!(
                    "{:?} {}",
                    n.accesskit_node().role(),
                    n.accesskit_node().label().unwrap_or_default()
                ),
            ))
        })
        .collect();
    let lo = nodes.iter().map(|(r, _)| r.min.x).fold(f32::MAX, f32::min);
    let (hi, widest) =
        nodes
            .iter()
            .map(|(r, l)| (r.max.x, l.clone()))
            .fold(
                (f32::MIN, String::new()),
                |a, b| if b.0 > a.0 { b } else { a },
            );
    (lo, hi, widest)
}

/// Rechte Kante des Felds „Neue Aufgabe“ (füllt die Mitte in Liste und Board).
fn center_right(h: &H, lang: Language) -> f32 {
    let name = texts(lang).t("aufgaben.neu.feld");
    rect(&h.get_by_role_and_label(Role::TextInput, &name))
        .expect("Feld Neue Aufgabe")
        .max
        .x
}

fn check(h: &mut H, width: f32, lang: Language, what: &str) {
    let margin = f32::from(TOKENS.spacing.s4);
    let left = width - DETAIL;
    let ids: Vec<String> = h.state().items.iter().map(|t| t.id.clone()).collect();
    assert!(!ids.is_empty(), "{what}");
    for id in ids {
        h.state_mut().selected = Some(id.clone());
        h.run();
        h.run();
        let at = format!("{what}, Breite {width}, Aufgabe {id}");
        let (lo, hi, widest) = detail_span(h, lang);
        assert!(
            lo >= left - margin,
            "{at}: Detail beginnt bei {lo} ({widest} bis {hi})"
        );
        assert!(
            hi <= width - margin + 0.5,
            "{at}: Detail endet bei {hi} ({widest})"
        );
        if width < CENTER_FITS {
            continue;
        }
        let center = center_right(h, lang);
        let wide: Vec<String> = h
            .query_all(By::new().predicate(|_| true))
            .filter_map(|n| {
                let r = rect(&n)?;
                (r.min.x < lo - margin && r.max.x > lo - 2.0 * margin - 6.0).then(|| {
                    format!(
                        "{:?} {} {r:?}",
                        n.accesskit_node().role(),
                        n.accesskit_node().label().unwrap_or_default()
                    )
                })
            })
            .collect();
        assert!(
            center + 2.0 * margin <= lo + 0.5,
            "{at}: Mitte endet bei {center}, Detail beginnt bei {lo}; {wide:#?}"
        );
    }
}

#[test]
fn detailbereich_bleibt_in_seinem_rahmen_und_die_mitte_endet_davor() {
    for (features, name) in [(Features::DESK, "Desk"), (Features::TASKS, "Tasks")] {
        for lang in Language::ALL {
            for board in [false, true] {
                let mut h = harness_in(Some((0, board)), lang);
                h.state_mut().features = features;
                for width in WIDTHS {
                    h.set_size(vec2(width, 1400.0));
                    h.run();
                    check(
                        &mut h,
                        width,
                        lang,
                        &format!("{name} {lang:?} board={board}"),
                    );
                }
            }
        }
    }
}
