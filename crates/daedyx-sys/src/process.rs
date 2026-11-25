//! Ein Programm starten und höchstens eine Zeitgrenze lang auf sein Ende warten (SICHERHEIT 2.7: Prozesse nur hier).
//! Gedacht für Startproben: Hängt das Programm (z. B. weil Windows beim Laden einen Fehlerdialog zeigt), wird es
//! nach der Zeitgrenze beendet, statt den Aufrufer endlos zu blockieren. Ein-/Ausgaben werden nicht verbunden.
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Wie ein Lauf endete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunEnd {
    /// Beendet mit diesem Code (`None`: ohne Code, etwa von außen beendet).
    Exited(Option<i32>),
    /// Nach der Zeitgrenze noch nicht fertig; der Prozess wurde beendet.
    TimedOut,
    /// Konnte nicht gestartet oder nicht beobachtet werden.
    Failed,
}

#[allow(clippy::disallowed_methods, reason = "SICHERHEIT-AUSNAHMEN A-021")]
pub fn run_with_limit(exe: &Path, args: &[&str], limit: Duration) -> RunEnd {
    let spawned = Command::new(exe)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    let Ok(mut child) = spawned else {
        return RunEnd::Failed;
    };
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return RunEnd::Exited(status.code()),
            Ok(None) => {}
            Err(_) => return RunEnd::Failed,
        }
        if start.elapsed() >= limit {
            let _ = child.kill();
            let _ = child.wait();
            return RunEnd::TimedOut;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}
