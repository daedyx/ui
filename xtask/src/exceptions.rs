//! Ausnahmenzähler (SICHERHEIT Anhang B.4): Ausnahme-Stellen (siehe `policy`) gegen `docs/SICHERHEIT-AUSNAHMEN.md`.
//! Verglichen wird Datei **und Regel**, nicht nur die Anzahl. Rot, wenn eine Stelle keinen passenden Eintrag hat oder
//! umgekehrt, wenn ein Datum fehlt, in der Zukunft liegt, abgelaufen ist oder mehr als 12 Monate Laufzeit hat.
use std::collections::BTreeMap;

pub const TABLE: &str = "docs/SICHERHEIT-AUSNAHMEN.md";

/// Ein Eintrag der Tabelle: Datei, Regel und Ablaufdatum `JJJJ-MM-TT`. Regeln `scan:<regel>` erlauben einen
/// Scan-Fund in genau dieser Datei (z. B. eine Altlast, deren Ablösung beschlossen ist) und zählen nicht als Code-Stelle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: String,
    pub rule: String,
    pub date: String,
    pub expires: String,
}

/// Höchstlaufzeit einer Ausnahme ab dem Eintragsdatum, in Jahren (SICHERHEIT 5.7, Review M-2 Nr. 5).
const MAX_YEARS: u32 = 1;

/// `JJJJ-MM-TT` mit gültigem Monat und Tag.
fn is_date(d: &str) -> bool {
    let b = d.as_bytes();
    let num = |r: std::ops::Range<usize>| {
        d.get(r)
            .filter(|x| x.bytes().all(|c| c.is_ascii_digit()))
            .and_then(|x| x.parse::<u32>().ok())
    };
    b.len() == 10
        && b.get(4) == Some(&b'-')
        && b.get(7) == Some(&b'-')
        && num(0..4).is_some()
        && num(5..7).is_some_and(|m| (1..=12).contains(&m))
        && num(8..10).is_some_and(|t| (1..=31).contains(&t))
}

/// Spätestes erlaubtes Ablaufdatum: gleicher Tag `MAX_YEARS` Jahre nach dem Eintrag.
fn latest_expiry(date: &str) -> Option<String> {
    let year: u32 = date.get(0..4)?.parse().ok()?;
    Some(format!(
        "{:04}{}",
        year.checked_add(MAX_YEARS)?,
        date.get(4..)?
    ))
}

fn valid_until(e: &Entry, today: &str) -> bool {
    is_date(&e.expires) && e.expires.as_str() >= today
}

/// Datumsfehler eines Eintrags (leer = gültig).
fn date_problems(e: &Entry, today: &str) -> Vec<String> {
    let p = &e.path;
    if !is_date(&e.date) || !is_date(&e.expires) {
        return vec![format!(
            "{p}: Datum oder Ablauf fehlt oder ist kein JJJJ-MM-TT ({})",
            e.rule
        )];
    }
    let mut out = Vec::new();
    if e.date.as_str() > today {
        out.push(format!(
            "{p}: Eintragsdatum {} liegt in der Zukunft",
            e.date
        ));
    }
    if e.expires.as_str() < today {
        out.push(format!("{p}: Ausnahme ist am {} abgelaufen", e.expires));
    }
    if latest_expiry(&e.date).is_none_or(|max| e.expires > max) {
        out.push(format!(
            "{p}: Ablauf {} liegt mehr als 12 Monate nach dem Eintrag {}",
            e.expires, e.date
        ));
    }
    out
}

/// Erlaubte Scan-Funde (Datei, Regel) aus gültigen `scan:`-Zeilen.
pub fn scan_allowances(
    entries: &[Entry],
    today: &str,
) -> std::collections::BTreeSet<(String, String)> {
    entries
        .iter()
        .filter(|e| valid_until(e, today))
        .filter_map(|e| Some((e.path.clone(), e.rule.strip_prefix("scan:")?.to_string())))
        .collect()
}

/// Zeilen `| Nr. | Datei | Regel | Grund | Entscheid | Datum | Ablauf |`; Kopf- und Trennzeilen zählen nicht.
pub fn parse_table(md: &str) -> Vec<Entry> {
    md.lines()
        .filter(|l| l.trim_start().starts_with('|'))
        .filter_map(|l| {
            let cells: Vec<&str> = l.split('|').map(str::trim).collect();
            let nr = cells.get(1)?;
            if nr.is_empty() || nr.starts_with("Nr") || nr.starts_with("---") {
                return None;
            }
            Some(Entry {
                path: cells.get(2)?.to_string(),
                rule: cells.get(3)?.to_string(),
                date: cells.get(6)?.to_string(),
                expires: cells.get(7)?.to_string(),
            })
        })
        .collect()
}

/// Alle Meldungs-Kennungen `RUSTSEC-JJJJ-NNNN` in nicht auskommentierten Zeilen.
fn advisory_ids(text: &str) -> std::collections::BTreeSet<String> {
    let prefix = concat!("RUST", "SEC-");
    text.lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .flat_map(|l| {
            l.match_indices(prefix)
                .map(move |(i, _)| l.get(i..).unwrap_or_default())
        })
        .map(|rest| {
            rest.chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect()
        })
        .collect()
}

/// `cargo audit` liest seine Ausnahmen aus `.cargo/audit.toml`; sie müssen genau denen in `deny.toml` entsprechen,
/// damit es nur eine Liste (mit Einträgen in der Tabelle) gibt.
pub fn mirror_problems(deny: &str, audit: &str) -> Vec<String> {
    if advisory_ids(deny) == advisory_ids(audit) {
        Vec::new()
    } else {
        vec!["Ausnahmen in .cargo/audit.toml und deny.toml unterscheiden sich".to_string()]
    }
}

/// Regel vergleichbar machen: Backticks und Erläuterung in Klammern (` (…)`) weg, Leerraum weg, klein geschrieben,
/// mehrere Regeln (durch Komma getrennt) sortiert. So passt die Tabellenzelle zur Stelle im Code.
pub fn normalize_rule(cell: &str) -> String {
    let plain = cell.replace('`', "");
    let core = plain.split(" (").next().unwrap_or_default();
    let mut parts: Vec<String> = core
        .split(',')
        .map(|p| {
            p.chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>()
                .to_lowercase()
        })
        .filter(|p| !p.is_empty())
        .collect();
    parts.sort();
    parts.join(",")
}

/// Probleme (leer = grün). `sites`: (Datei, Regel) je Ausnahme-Stelle.
pub fn check(sites: &[(String, String)], entries: &[Entry], today: &str) -> Vec<String> {
    let mut balance: BTreeMap<(String, String), i64> = BTreeMap::new();
    for (path, rule) in sites {
        let n = balance
            .entry((path.clone(), normalize_rule(rule)))
            .or_insert(0);
        *n = n.saturating_add(1);
    }
    let mut problems = Vec::new();
    for e in entries {
        if !e.rule.starts_with("scan:") {
            let n = balance
                .entry((e.path.clone(), normalize_rule(&e.rule)))
                .or_insert(0);
            *n = n.saturating_sub(1);
        }
        problems.extend(date_problems(e, today));
    }
    for ((path, rule), n) in balance {
        if n > 0 {
            problems.push(format!(
                "{path}: {n} Ausnahme-Stelle(n) „{rule}“ ohne passenden Eintrag in {TABLE}"
            ));
        } else if n < 0 {
            problems.push(format!(
                "{path}: {} Eintrag/Einträge „{rule}“ ohne passende Stelle",
                n.saturating_neg()
            ));
        }
    }
    problems
}

#[cfg(test)]
#[path = "exceptions_tests.rs"]
mod tests;
