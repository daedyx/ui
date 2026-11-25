//! Abschnitte (Board-Spalten) einer Liste, über den Kern.
use crate::store::TaskStore;
use daedyx_tasks_core::TaskError;
use daedyx_tasks_core::edit_sections as sections;
use daedyx_tasks_core::store;
use daedyx_tasks_core::types::Section;

impl TaskStore {
    pub fn sections(&self, list: &str) -> Result<Vec<Section>, TaskError> {
        self.with(|c| store::sections(c, list))
    }

    pub fn create_section(&self, list: &str, name: &str) -> Result<Section, TaskError> {
        let now = self.now_ms();
        self.with(|c| sections::create_section(c, list, name, now))
    }

    pub fn rename_section(&self, id: &str, name: &str) -> Result<(), TaskError> {
        let now = self.now_ms();
        self.with(|c| sections::rename_section(c, id, name, now))
    }

    pub fn reorder_sections(&self, list: &str, ids: &[String]) -> Result<(), TaskError> {
        let now = self.now_ms();
        self.with(|c| sections::reorder_sections(c, list, ids, now))
    }

    pub fn delete_section(&self, id: &str) -> Result<(), TaskError> {
        let now = self.now_ms();
        self.with(|c| sections::delete_section(c, id, now))
    }
}
