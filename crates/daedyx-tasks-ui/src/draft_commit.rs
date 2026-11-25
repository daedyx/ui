//! Entwurf vor dem Beenden übernehmen (Tasks T4): Titel, Notiz und Fälligkeit im Detailbereich gelten sonst erst
//! beim Verlassen des Feldes. Wie das Sichern des Mail-Entwurfs in Desk (Review int2 M-2): Klappt es nicht, bleibt
//! die App offen und zeigt den Fehler am Feld.
use crate::TasksUi;
use crate::detail_fields::{draft_due, parse_due};
use daedyx_ui::i18n::Texts;
use serde_json::json;

impl TasksUi {
    /// Übernimmt geänderte Felder des Entwurfs. `false`: etwas ließ sich nicht übernehmen (Datum unbrauchbar, Kern
    /// lehnt ab); der Fehler steht dann in `error`. Nur dieser Versuch zählt: Ein Fehler aus einer früheren Aktion
    /// wird vorher gelöscht und hält das Beenden nicht auf (Review T4, M4-1).
    pub fn commit_draft(&mut self, texts: &Texts) -> bool {
        self.error = None;
        if self.draft.id.is_empty() {
            return true;
        }
        let id = self.draft.id.clone();
        let Some(t) = self.store.as_deref().and_then(|s| s.task(&id).ok()) else {
            return true;
        };
        let title = self.draft.title.trim().to_owned();
        if title != t.title {
            self.set(&id, &[("title", json!(title))]);
            if self.error.is_some() {
                return false;
            }
        }
        if self.draft.notes != t.notes {
            let notes = self.draft.notes.clone();
            self.set(&id, &[("notes", json!(notes))]);
            if self.error.is_some() {
                return false;
            }
        }
        if draft_due(self, texts, &t) != (self.draft.date.clone(), self.draft.time.clone()) {
            match parse_due(self, texts) {
                Some(fields) => self.set(&id, &fields),
                None => self.error = Some("aufgaben.fehler.datum".into()),
            }
        }
        self.error.is_none()
    }
}
