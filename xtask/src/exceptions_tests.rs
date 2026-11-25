use super::*;

const TABLE_MD: &str = "\
| Nr. | Datei | Regel | Grund | Entscheid von | Datum | Ablauf |
|---|---|---|---|---|---|---|
| A-001 | src/a.rs | clippy::expect_used | Grund | Betreuer | 2026-11-23 | 2027-03-31 |
| A-002 | src/a.rs | `clippy::print_stdout, clippy::print_stderr` (Konsole) | Grund | Betreuer | 2026-11-23 | 2027-03-31 |
| A-003 | deny.toml | RUSTSEC-2026-0001 (x) | Grund | Betreuer | 2025-11-23 | 2026-01-01 |
";

fn site(path: &str, rule: &str) -> (String, String) {
    (path.to_string(), rule.to_string())
}

fn entry(path: &str, rule: &str, date: &str, expires: &str) -> Entry {
    Entry {
        path: path.into(),
        rule: rule.into(),
        date: date.into(),
        expires: expires.into(),
    }
}

#[test]
fn table_rows_are_parsed_without_header() {
    let e = parse_table(TABLE_MD);
    assert_eq!(e.len(), 3);
    assert_eq!(
        e.first(),
        Some(&entry(
            "src/a.rs",
            "clippy::expect_used",
            "2026-11-23",
            "2027-03-31"
        ))
    );
}

#[test]
fn rules_are_compared_not_only_counted() {
    let entries = parse_table(TABLE_MD);
    let ok = [
        site("src/a.rs", "clippy::expect_used"),
        site("src/a.rs", "clippy::print_stderr,clippy::print_stdout"),
    ];
    let p = check(&ok, &entries[..2], "2026-11-24");
    assert!(p.is_empty(), "{p:?}");
    // Gleiche Anzahl, andere Regel (Review M-2 Nr. 4): rot.
    let swapped = [
        site("src/a.rs", "clippy::indexing_slicing"),
        site("src/a.rs", "clippy::print_stderr,clippy::print_stdout"),
    ];
    let p = check(&swapped, &entries[..2], "2026-11-24");
    assert_eq!(p.len(), 2, "{p:?}");
    assert!(
        p.iter().any(|m| m.contains("clippy::indexing_slicing")),
        "{p:?}"
    );
}

#[test]
fn unlisted_or_surplus_or_expired_are_red() {
    let entries = parse_table(TABLE_MD);
    let too_many = [
        site("src/a.rs", "clippy::expect_used"),
        site("src/a.rs", "clippy::expect_used"),
    ];
    assert!(!check(&too_many, &entries[..1], "2026-11-24").is_empty());
    assert_eq!(
        check(&[site("src/b.rs", "unsafe")], &[], "2026-11-24").len(),
        1
    );
    let expired = check(
        &[site("deny.toml", "RUSTSEC-2026-0001")],
        &entries[2..],
        "2026-11-24",
    );
    assert!(
        expired.iter().any(|m| m.contains("abgelaufen")),
        "{expired:?}"
    );
    assert!(check(&[], &[], "2026-11-24").is_empty());
}

#[test]
fn normalized_rules_ignore_case_spaces_order_and_comments() {
    assert_eq!(
        normalize_rule("`clippy::b, clippy::a` (zwei)"),
        "clippy::a,clippy::b"
    );
    assert_eq!(
        normalize_rule("RUSTSEC-2026-0459 (libsqlite3-sys)"),
        "rustsec-2026-0459"
    );
    assert_eq!(
        normalize_rule("licenses.exceptions=f:A+OFL-1.1"),
        "licenses.exceptions=f:a+ofl-1.1"
    );
}

#[test]
fn broken_dates_are_red() {
    let s = [site("x.rs", "clippy::x")];
    for bad in [
        entry("x.rs", "clippy::x", "2026-11-23", "bald"),
        entry("x.rs", "clippy::x", "gestern", "2027-01-01"),
    ] {
        assert!(
            check(&s, std::slice::from_ref(&bad), "2026-11-24")
                .iter()
                .any(|m| m.contains("Datum")),
            "{bad:?}"
        );
    }
}

#[test]
fn expiry_is_at_most_twelve_months_after_the_entry() {
    let s = [site("x.rs", "clippy::x")];
    let ok = entry("x.rs", "clippy::x", "2026-11-23", "2027-11-23");
    assert!(check(&s, &[ok], "2026-11-24").is_empty());
    for far in ["2027-11-24", "2099-12-31"] {
        let p = check(
            &s,
            &[entry("x.rs", "clippy::x", "2026-11-23", far)],
            "2026-11-24",
        );
        assert!(p.iter().any(|m| m.contains("12 Monate")), "{far} {p:?}");
    }
    let future = check(
        &s,
        &[entry("x.rs", "clippy::x", "2027-01-01", "2027-06-30")],
        "2026-11-24",
    );
    assert!(future.iter().any(|m| m.contains("Zukunft")), "{future:?}");
}

#[test]
fn audit_ignores_must_mirror_deny_ignores() {
    let deny = concat!(
        "ignore = [\n { id = \"RUST",
        "SEC-2026-0001\", reason = \"x\" },\n]"
    );
    let same = concat!("ignore = [\"RUST", "SEC-2026-0001\"]");
    let more = concat!(
        "ignore = [\"RUST",
        "SEC-2026-0001\", \"RUST",
        "SEC-2026-0002\"]"
    );
    assert!(mirror_problems(deny, same).is_empty());
    assert_eq!(mirror_problems(deny, more).len(), 1);
    assert_eq!(mirror_problems(deny, "").len(), 1);
}

#[test]
fn scan_rows_allow_one_rule_in_one_file_until_they_expire() {
    let md = "\
| A-1 | .github/workflows/release.yml | scan:secret-im-workflow | alt | B | 2026-11-23 | 2027-01-31 |
| A-2 | docs/alt.md | scan:mailadresse | alt | B | 2025-11-23 | 2026-01-01 |
";
    let e = parse_table(md);
    assert_eq!(
        e.first().map(|x| x.rule.as_str()),
        Some("scan:secret-im-workflow")
    );
    let allowed = scan_allowances(&e, "2026-11-24");
    assert!(allowed.contains(&(
        ".github/workflows/release.yml".to_string(),
        "secret-im-workflow".to_string()
    )));
    assert_eq!(allowed.len(), 1, "abgelaufene Zeile erlaubt nichts mehr");
    // Scan-Zeilen zählen nicht als Code-Stellen; abgelaufen bleibt rot.
    let p = check(&[], &e, "2026-11-24");
    assert_eq!(p.len(), 1, "{p:?}");
}
