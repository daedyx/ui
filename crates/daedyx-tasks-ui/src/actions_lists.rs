//! Änderungen aus Leiste und Board für Tasks T3b: eigene Listen umsortieren (Ziehen, Alt+↑/↓), eine Aufgabe in eine
//! andere Liste verschieben (Ziehen auf die Liste oder Menü „Verschieben nach …“), Karte direkt in einer Spalte
//! anlegen („+ Karte“). Alles über den Kern.
use super::TasksUi;
use super::model::dropped_sections;
use daedyx_tasks_core::smart::View;
use daedyx_tasks_core::types::Section;

/// Neue Reihenfolge der Listen, wenn `id` an die Stelle von `target` kommt (wie bei den Abschnitten).
pub fn dropped_lists(lists: &[(String, f64)], id: &str, target: &str) -> Option<Vec<String>> {
    let as_sections: Vec<Section> = lists
        .iter()
        .map(|(l, p)| Section {
            id: l.clone(),
            list_id: String::new(),
            name: String::new(),
            position: *p,
            color: None,
        })
        .collect();
    dropped_sections(&as_sections, id, target)
}

impl TasksUi {
    fn list_order(&self) -> Vec<(String, f64)> {
        self.lists
            .iter()
            .map(|l| (l.id.clone(), l.position))
            .collect()
    }

    /// Liste `id` an die Stelle von `target` (Ziehen in der Leiste).
    pub fn drop_list(&mut self, id: &str, target: &str) {
        if let Some(ids) = dropped_lists(&self.list_order(), id, target) {
            let _ = self.run(|s| s.reorder_lists(&ids));
        }
    }

    /// Liste `id` eine Stelle nach oben (`-1`) oder unten (`+1`), Alt+↑/↓.
    pub fn list_step(&mut self, id: &str, step: isize) {
        let ids: Vec<String> = self.lists.iter().map(|l| l.id.clone()).collect();
        let Some(from) = ids.iter().position(|x| x == id) else {
            return;
        };
        if let Some(target) = from
            .checked_add_signed(step)
            .and_then(|to| ids.get(to))
            .cloned()
        {
            self.drop_list(id, &target);
        }
    }

    /// Aufgabe samt Unterbaum ans Ende der obersten Aufgaben von `list` (ohne Abschnitt, ohne Oberaufgabe).
    /// Auf die eigene Liste gezogen: immer Abbruch, auch bei Unteraufgaben (keine stille Umwandlung zur obersten
    /// Aufgabe; Master-Entscheid zu Review T3b, N3b-3).
    pub fn move_to_list(&mut self, id: &str, list: &str) {
        let own = self
            .items
            .iter()
            .find(|t| t.id == id)
            .map(|t| t.list_id.clone())
            .or_else(|| {
                self.store
                    .as_deref()
                    .and_then(|s| s.task(id).ok())
                    .map(|t| t.list_id)
            });
        if own.as_deref().is_none_or(|l| l == list) {
            return;
        }
        let depth = self.features.max_depth;
        let _ = self.run(|s| {
            let last = s
                .query(View::List, Some(list))?
                .iter()
                .filter(|t| t.parent_id.is_none())
                .map(|t| t.position)
                .fold(0.0, f64::max);
            s.move_deep(id, (list, None, None), last + 1.0, depth)
        });
        if self.selected.as_deref() == Some(id) && self.list_id().is_some_and(|l| l != list) {
            self.selected = None;
        }
    }

    /// „+ Karte“: Schnelleingabe in die Spalte (`None` = „Ohne Abschnitt“) der gewählten Liste.
    pub fn add_card(&mut self) {
        let Some((section, text)) = self.adding_card.take() else {
            return;
        };
        let Some(list) = self
            .list_id()
            .map(str::to_owned)
            .filter(|_| !text.trim().is_empty())
        else {
            return;
        };
        let made = self.run(|s| s.quick(&list, section.as_deref(), &text));
        // Feld bleibt für die nächste Karte offen; der Text geht nur weg, wenn die Karte entstanden ist (N3b-2).
        let keep = if matches!(made, Some(Some(_))) {
            String::new()
        } else {
            text
        };
        self.adding_card = Some((section, keep));
    }
}
