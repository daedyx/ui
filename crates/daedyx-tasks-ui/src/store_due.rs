//! Was der Hintergrund-Takt die Aufgaben fragt: fällige Erinnerungen (Kern `reminders::tick`), gemeldet je
//! Erinnerung nur einmal.
use crate::store::TaskStore;
use chrono::TimeZone;
use daedyx_tasks_core::reminders::{self, DueReminder};

impl TaskStore {
    /// Jetzt in Sekunden seit 1970 (UTC), nach der Uhr der Oberfläche.
    pub fn now_secs(&self) -> i64 {
        self.zone()
            .from_local_datetime(&self.now())
            .earliest()
            .map_or(0, |d| d.timestamp())
    }

    /// Aufgaben, deren Erinnerung seit `now - 1 h` fällig wurde; danach gemerkt.
    pub fn due_reminders(&self, now: i64) -> Vec<DueReminder> {
        let tz = self.zone().name().to_owned();
        let mut out = Vec::new();
        let _ = self.with(|c| reminders::tick(c, now, &tz, |r| out.push(r.clone())));
        out
    }
}
