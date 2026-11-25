//! Mindestzahl bestandener Tests je Testprogramm (Review Prüflücken H-2, M-5). Liest das Protokoll von
//! `cargo test` und vergleicht es mit einer festgehaltenen Liste (`xtask/testzahlen.txt`). Rot, wenn ein Programm
//! weniger Tests hat als festgehalten, wenn eins fehlt, wenn ein neues mit Tests ohne Eintrag auftaucht oder wenn ein
//! Test fehlschlägt. So fällt auf, wenn Tests still wegfallen (`test = false`, `--exclude`, gelöschte Module).
use std::collections::BTreeMap;

pub const FLOOR: &str = "xtask/testzahlen.txt";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Outcome {
    pub passed: u64,
    pub failed: u64,
}

/// `unittests src\lib.rs (D:\…\deps\name-hash.exe)` → `unittests src/lib.rs (name)`.
fn program_key(running: &str) -> Option<String> {
    let (what, bin) = running.rsplit_once(" (")?;
    let file = bin.trim_end_matches(')').rsplit(['/', '\\']).next()?;
    let stem = file.trim_end_matches(".exe");
    let name = stem.rsplit_once('-').map_or(stem, |(n, _)| n);
    Some(format!("{} ({name})", what.replace('\\', "/")))
}

fn number_before(line: &str, word: &str) -> Option<u64> {
    let i = line.find(word)?;
    line.get(..i)?.split_whitespace().next_back()?.parse().ok()
}

/// Ergebnis je Testprogramm aus dem Protokoll.
pub fn counts(log: &str) -> BTreeMap<String, Outcome> {
    let mut out = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in log.lines().map(str::trim) {
        if let Some(rest) = line.strip_prefix("Running ") {
            current = program_key(rest);
        } else if let Some(name) = line.strip_prefix("Doc-tests ") {
            current = Some(format!("doc-tests ({})", name.trim()));
        } else if line.starts_with("test result:") {
            let Some(key) = current.take() else { continue };
            let passed = number_before(line, " passed").unwrap_or(0);
            let failed = number_before(line, " failed").unwrap_or(0);
            out.insert(key, Outcome { passed, failed });
        }
    }
    out
}

/// Probleme (leer = grün).
pub fn check(log: &str, floor: &str) -> Vec<String> {
    let found = counts(log);
    let mut problems = Vec::new();
    let mut listed = std::collections::BTreeSet::new();
    for line in floor
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let Some((n, key)) = line
            .split_once(' ')
            .and_then(|(n, k)| Some((n.parse::<u64>().ok()?, k.trim())))
        else {
            problems.push(format!(
                "{FLOOR}: Zeile ist nicht „<Zahl> <Testprogramm>“: {line}"
            ));
            continue;
        };
        listed.insert(key.to_string());
        match found.get(key) {
            None => problems.push(format!(
                "{key}: Testprogramm fehlt im Protokoll (festgehalten: {n})"
            )),
            Some(o) if o.passed < n => problems.push(format!(
                "{key}: {} Tests bestanden, festgehalten sind {n}",
                o.passed
            )),
            Some(_) => {}
        }
    }
    for (key, o) in &found {
        if o.failed > 0 {
            problems.push(format!("{key}: {} Test(s) fehlgeschlagen", o.failed));
        }
        if o.passed > 0 && !listed.contains(key) {
            problems.push(format!(
                "{key}: {} Tests, aber keine Mindestzahl in {FLOOR}",
                o.passed
            ));
        }
    }
    problems
}

#[cfg(test)]
#[path = "testcount_tests.rs"]
mod tests;
