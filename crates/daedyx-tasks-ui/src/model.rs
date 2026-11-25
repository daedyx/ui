//! Reine Rechenregeln des Aufgabenbereichs (ohne Zeichnen): Zeilenfolge mit Unteraufgaben, Spalten des
//! Boards, Position beim Einfügen zwischen zwei Karten, Verschieben eines Abschnitts.
use daedyx_tasks_core::types::{Section, Task};

/// Eine Zeile der Liste: Index in `items` und Einrückung (0 = Aufgabe, 1 = Unteraufgabe).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Row {
    pub index: usize,
    pub depth: usize,
}

/// Offene vor erledigten Aufgaben, jede Aufgabe mit ihren Unteraufgaben direkt darunter. Unteraufgaben,
/// deren Aufgabe nicht in `items` steht (z. B. in „Heute“), erscheinen als eigene Zeile.
pub fn rows(items: &[Task]) -> Vec<Row> {
    let has_parent = |t: &Task| {
        t.parent_id
            .as_deref()
            .is_some_and(|p| items.iter().any(|x| x.id == p))
    };
    let mut tops: Vec<usize> = (0..items.len())
        .filter(|&i| items.get(i).is_some_and(|t| !has_parent(t)))
        .collect();
    tops.sort_by_key(|&i| items.get(i).is_some_and(|t| t.done_at.is_some()));
    let mut out = Vec::new();
    for i in tops {
        out.push(Row { index: i, depth: 0 });
        let Some(id) = items.get(i).map(|t| t.id.as_str()) else {
            continue;
        };
        let mut kids: Vec<usize> = (0..items.len())
            .filter(|&k| {
                items
                    .get(k)
                    .is_some_and(|t| t.parent_id.as_deref() == Some(id))
            })
            .collect();
        kids.sort_by_key(|&k| items.get(k).is_some_and(|t| t.done_at.is_some()));
        out.extend(kids.into_iter().map(|k| Row { index: k, depth: 1 }));
    }
    out
}

/// Spalten des Boards: zuerst „Ohne Abschnitt“ (`None`), dann die Abschnitte in ihrer Reihenfolge.
pub fn columns(sections: &[Section]) -> Vec<Option<String>> {
    std::iter::once(None)
        .chain(sections.iter().map(|s| Some(s.id.clone())))
        .collect()
}

/// Spalte einer Karte: ihr Abschnitt, wenn es ihn in `sections` gibt, sonst „Ohne Abschnitt“ (`None`). So bleibt eine
/// Karte sichtbar, deren Abschnitt anderswo gelöscht wurde (Abgleich), statt aus allen Spalten zu fallen.
pub fn column_of<'a>(t: &'a Task, sections: &[Section]) -> Option<&'a str> {
    t.section_id
        .as_deref()
        .filter(|s| sections.iter().any(|x| x.id == *s))
}

/// Karten einer Spalte (nur Aufgaben ohne Oberaufgabe): offene nach Position, erledigte darunter.
pub fn cards<'a>(items: &'a [Task], sections: &[Section], column: Option<&str>) -> Vec<&'a Task> {
    let mut v: Vec<&Task> = items
        .iter()
        .filter(|t| t.parent_id.is_none() && column_of(t, sections) == column)
        .collect();
    v.sort_by(|a, b| {
        a.done_at
            .is_some()
            .cmp(&b.done_at.is_some())
            .then(a.position.total_cmp(&b.position))
    });
    v
}

/// Position für eine Karte, die an Stelle `index` zwischen die übrigen Karten (`others`, sortiert) kommt.
pub fn drop_position(others: &[f64], index: usize) -> f64 {
    let index = index.min(others.len());
    let before = index.checked_sub(1).and_then(|i| others.get(i)).copied();
    let after = others.get(index).copied();
    match (before, after) {
        (Some(a), Some(b)) => (a + b) / 2.0,
        (Some(a), None) => a + 1.0,
        (None, Some(b)) => b - 1.0,
        (None, None) => 1.0,
    }
}

/// Abschnitt `id` um `step` (−1 links, +1 rechts) verschoben; `None`, wenn es nicht geht.
pub fn moved_sections(sections: &[Section], id: &str, step: isize) -> Option<Vec<String>> {
    let mut ids: Vec<String> = sections.iter().map(|s| s.id.clone()).collect();
    let from = ids.iter().position(|x| x == id)?;
    let to = from.checked_add_signed(step).filter(|t| *t < ids.len())?;
    ids.swap(from, to);
    Some(ids)
}

/// Abschnitt `id` an die Stelle von `target` (Ziehen), ohne gültiges Ziel ans Ende. `None`, wenn nichts zu tun ist.
pub fn dropped_sections(sections: &[Section], id: &str, target: &str) -> Option<Vec<String>> {
    let mut ids: Vec<String> = sections.iter().map(|s| s.id.clone()).collect();
    let from = ids.iter().position(|x| x == id)?;
    // Ohne gültiges Ziel ans Ende, wie die alte Oberfläche (Entscheid T3-1).
    let to = ids
        .iter()
        .position(|x| x == target)
        .unwrap_or(ids.len().saturating_sub(1));
    if from == to {
        return None;
    }
    let moved = ids.remove(from);
    ids.insert(to.min(ids.len()), moved);
    Some(ids)
}
