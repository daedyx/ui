//! Liste als Baum (Tasks T3a): Unteraufgaben beliebig tief direkt unter ihrer Oberaufgabe, vier Sortierungen für
//! die obersten Aufgaben (der Unterbaum folgt in Handreihenfolge), Zähler über alle Nachfahren. Reine Rechnung.
use super::model::Row;
use chrono_tz::Tz;
use daedyx_tasks_core::types::Task;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

/// Reihenfolge der obersten Aufgaben.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortMode {
    #[default]
    Hand,
    Due,
    Priority,
    Title,
}

impl SortMode {
    pub const ALL: [SortMode; 4] = [
        SortMode::Hand,
        SortMode::Due,
        SortMode::Priority,
        SortMode::Title,
    ];

    pub fn key(self) -> &'static str {
        match self {
            SortMode::Hand => "aufgaben.sortierung.hand",
            SortMode::Due => "aufgaben.sortierung.faellig",
            SortMode::Priority => "aufgaben.sortierung.prioritaet",
            SortMode::Title => "aufgaben.sortierung.titel",
        }
    }
}

/// Tag der Fälligkeit in Ortszeit (Kern `smart::due_day`; `None` ohne Fälligkeit oder bei unlesbaren Sekunden).
pub use daedyx_tasks_core::smart::due_day;

/// Titel zum Vergleichen: ohne Groß-/Kleinschreibung, Umlaute wie ihr Grundbuchstabe (ä wie a, ß wie ss).
pub fn title_key(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .flat_map(|c| match c {
            'ä' | 'à' | 'â' | 'á' => vec!['a'],
            'ö' | 'ô' | 'ó' => vec!['o'],
            'ü' | 'û' | 'ú' => vec!['u'],
            'é' | 'è' | 'ê' | 'ë' => vec!['e'],
            'ç' => vec!['c'],
            'ß' => vec!['s', 's'],
            c => vec![c],
        })
        .collect()
}

fn compare(a: &Task, b: &Task, mode: SortMode, tz: Tz) -> Ordering {
    let pos = a.position.total_cmp(&b.position);
    match mode {
        SortMode::Hand => pos,
        // Ohne Fälligkeit ans Ende.
        SortMode::Due => match (due_day(a, tz), due_day(b, tz)) {
            (Some(x), Some(y)) => x.cmp(&y),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
        .then(pos),
        SortMode::Priority => b.priority.cmp(&a.priority).then(pos),
        SortMode::Title => title_key(&a.title).cmp(&title_key(&b.title)).then(pos),
    }
}

/// Zeilen der Liste: offene vor erledigten, oberste nach `mode`, darunter jede Ebene in Handreihenfolge. Aufgaben,
/// deren Oberaufgabe nicht in `items` steht (z. B. in „Heute“), sind oberste. Jede Aufgabe kommt genau einmal vor.
pub fn tree_rows(items: &[Task], mode: SortMode, tz: Tz) -> Vec<Row> {
    let ids: HashSet<&str> = items.iter().map(|t| t.id.as_str()).collect();
    let mut kids: HashMap<&str, Vec<usize>> = HashMap::new();
    let mut tops = Vec::new();
    for (i, t) in items.iter().enumerate() {
        match t.parent_id.as_deref().filter(|p| ids.contains(p)) {
            Some(p) => kids.entry(p).or_default().push(i),
            None => tops.push(i),
        }
    }
    let task = |i: &usize| items.get(*i);
    let done = |i: &usize| task(i).is_some_and(|t| t.done_at.is_some());
    tops.sort_by(|a, b| match (task(a), task(b)) {
        (Some(x), Some(y)) => done(a).cmp(&done(b)).then(compare(x, y, mode, tz)),
        _ => Ordering::Equal,
    });
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    // Nach den obersten noch alle übrigen: Aufgaben in einem Kreis (kaputte Daten) gehen nicht verloren.
    for root in tops.into_iter().chain(0..items.len()) {
        let mut stack = vec![(root, 0usize)];
        while let Some((i, depth)) = stack.pop() {
            let Some(t) = items.get(i).filter(|_| seen.insert(i)) else {
                continue;
            };
            out.push(Row { index: i, depth });
            let mut below = kids.get(t.id.as_str()).cloned().unwrap_or_default();
            below.sort_by(|a, b| match (task(a), task(b)) {
                (Some(x), Some(y)) => done(a)
                    .cmp(&done(b))
                    .then(x.position.total_cmp(&y.position)),
                _ => Ordering::Equal,
            });
            stack.extend(
                below
                    .into_iter()
                    .rev()
                    .map(|k| (k, depth.saturating_add(1))),
            );
        }
    }
    out
}

/// (erledigt, gesamt) über alle Nachfahren in `items`, nur für Aufgaben mit Unteraufgaben.
pub fn progress(items: &[Task]) -> HashMap<String, (usize, usize)> {
    let parent: HashMap<&str, &str> = items
        .iter()
        .filter_map(|t| t.parent_id.as_deref().map(|p| (t.id.as_str(), p)))
        .collect();
    let mut out: HashMap<String, (usize, usize)> = HashMap::new();
    for t in items.iter().filter(|t| t.parent_id.is_some()) {
        // Jede Oberaufgabe höchstens einmal je Aufgabe zählen; ein Kreis (kaputte Daten) endet beim ersten Wiedersehen.
        let mut seen: HashSet<&str> = HashSet::from([t.id.as_str()]);
        let mut at = t.id.as_str();
        while let Some(&p) = parent.get(at) {
            if !seen.insert(p) {
                break;
            }
            let e = out.entry(p.to_owned()).or_default();
            e.0 = e.0.saturating_add(usize::from(t.done_at.is_some()));
            e.1 = e.1.saturating_add(1);
            at = p;
        }
    }
    out
}
