use super::{Bridge, BridgeError, MAX_EVENTS_PER_POLL, repaint_waker};
use crate::widgets::{ButtonKind, button};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

fn doubling(wakes: Arc<AtomicUsize>) -> Bridge<u32, u32> {
    Bridge::start(
        |n: u32| async move { n.saturating_mul(2) },
        move || {
            wakes.fetch_add(1, Ordering::SeqCst);
        },
    )
    .unwrap()
}

#[test]
fn antworten_kommen_als_ereignisse_und_wecken_die_oberflaeche() {
    let wakes = Arc::new(AtomicUsize::new(0));
    let mut bridge = doubling(wakes.clone());
    for n in [1, 2, 3] {
        bridge.send(n).unwrap();
    }
    let mut got: Vec<u32> = (0..3)
        .filter_map(|_| bridge.wait(Duration::from_secs(5)))
        .collect();
    got.sort_unstable();
    assert_eq!(got, [2, 4, 6]);
    assert_eq!(wakes.load(Ordering::SeqCst), 3, "je Antwort ein Weckruf");
    assert!(bridge.poll().is_empty(), "nichts doppelt");
}

#[test]
fn poll_liefert_hoechstens_eine_begrenzte_menge_je_bild() {
    let wakes = Arc::new(AtomicUsize::new(0));
    let mut bridge = doubling(wakes.clone());
    let total = MAX_EVENTS_PER_POLL + 10;
    for n in 0..total {
        bridge.send(u32::try_from(n).unwrap()).unwrap();
    }
    while wakes.load(Ordering::SeqCst) < total {
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(bridge.poll().len(), MAX_EVENTS_PER_POLL);
    assert_eq!(bridge.poll().len(), 10);
}

#[test]
fn fehler_sind_unterscheidbar() {
    assert_ne!(BridgeError::Start, BridgeError::Closed);
    assert_ne!(BridgeError::Busy, BridgeError::Closed);
}

struct Demo {
    bridge: Option<Bridge<u32, u32>>,
    answer: Option<u32>,
}

/// Bewusst ohne `theme::install`: Bausteine dürfen dann nicht abstürzen.
/// Muster für die Apps: Knopf schickt Anfrage, Antwort kommt als Ereignis, Oberfläche zeigt sie.
#[test]
fn oberflaeche_fragt_den_kern_und_zeigt_die_antwort() {
    let mut h = Harness::builder().build_ui_state(
        |ui, demo: &mut Demo| {
            let bridge = demo.bridge.get_or_insert_with(|| {
                Bridge::start(
                    |n: u32| async move { n.saturating_add(40) },
                    repaint_waker(ui.ctx()),
                )
                .unwrap()
            });
            for answer in bridge.poll() {
                demo.answer = Some(answer);
            }
            if button(ui, ButtonKind::Primary, "Fragen").clicked() {
                let _ = bridge.send(2);
            }
            if let Some(a) = demo.answer {
                ui.label(format!("Antwort {a}"));
            }
        },
        Demo {
            bridge: None,
            answer: None,
        },
    );
    h.run();
    h.get_by_label("Fragen").click();
    for _ in 0..200 {
        h.run();
        if h.query_by_label("Antwort 42").is_some() {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("Antwort kam nicht an");
}
