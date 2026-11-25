use super::*;

/// Grundstand wie im Repo (Lizenzliste gekürzt auf Einträge der Grundliste).
const BASE: &str = r#"
[graph]
all-features = true

[advisories]
yanked = "deny"
unmaintained = "all"
ignore = [
  { id = "RUSTSEC-2026-0001", reason = "x" },
  # { id = "RUSTSEC-2026-0002", reason = "auskommentiert" },
]

[licenses]
allow = ["MIT", "Apache-2.0", "Apache-2.0 WITH LLVM-exception"]
confidence-threshold = 0.9

[bans]
multiple-versions = "warn"
wildcards = "deny"
deny = [
  { name = "openssl", reason = "x" },
  { name = "native-tls", reason = "x" },
]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = ["https://github.com/daedyx/core"]
"#;

fn with(from: &str, to: &str) -> Vec<String> {
    assert!(BASE.contains(from), "{from}");
    sites(&BASE.replace(from, to))
}

#[test]
fn base_has_only_the_listed_advisory() {
    assert_eq!(sites(BASE), vec!["RUSTSEC-2026-0001"]);
}

#[test]
fn loosened_fixed_values_are_sites() {
    assert!(
        with("yanked = \"deny\"", "yanked = \"allow\"")
            .contains(&"advisories.yanked=allow".to_string())
    );
    assert!(with("yanked = \"deny\"\n", "").contains(&"advisories.yanked=fehlt".to_string()));
    assert!(
        with("unmaintained = \"all\"", "unmaintained = \"none\"")
            .contains(&"advisories.unmaintained=none".to_string())
    );
    assert!(
        with("unknown-git = \"deny\"", "unknown-git = \"allow\"")
            .contains(&"sources.unknown-git=allow".to_string())
    );
    assert!(
        with("wildcards = \"deny\"", "wildcards = \"allow\"")
            .contains(&"bans.wildcards=allow".to_string())
    );
    assert!(with("= 0.9", "= 0.5").contains(&"licenses.confidence-threshold=0.5".to_string()));
}

#[test]
fn additions_beyond_the_base_lists_are_sites() {
    let git = with(
        "allow-git = [\"https://github.com/daedyx/core\"]",
        "allow-git = [\"https://github.com/daedyx/core\", \"https://git.example/x\"]",
    );
    assert!(
        git.contains(&"sources.allow-git=https://git.example/x".to_string()),
        "{git:?}"
    );
    let lic = with("\"MIT\",", "\"MIT\", \"GPL-3.0\",");
    assert!(
        lic.contains(&"licenses.allow=GPL-3.0".to_string()),
        "{lic:?}"
    );
    let exc = with(
        "confidence-threshold",
        "exceptions = [{ crate = \"f\", allow = [\"OFL-1.1\", \"A\"] }]\nconfidence-threshold",
    );
    assert!(
        exc.contains(&"licenses.exceptions=f:A+OFL-1.1".to_string()),
        "{exc:?}"
    );
}

#[test]
fn bans_skip_and_unknown_keys_are_sites() {
    let skip = with(
        "wildcards = \"deny\"",
        "wildcards = \"deny\"\nskip = [\"a@1.0\", { crate = \"b\" }]\nskip-tree = [\"c\"]",
    );
    for want in ["bans.skip=a@1.0", "bans.skip=b", "bans.skip-tree=c"] {
        assert!(skip.contains(&want.to_string()), "{want} {skip:?}");
    }
    let unknown = with(
        "wildcards = \"deny\"",
        "wildcards = \"deny\"\nallow-wildcard-paths = true",
    );
    assert!(
        unknown.contains(&"bans.allow-wildcard-paths=true".to_string()),
        "{unknown:?}"
    );
    let removed = with("{ name = \"openssl\", reason = \"x\" },", "");
    assert!(
        removed.contains(&"bans.deny-fehlt=openssl".to_string()),
        "{removed:?}"
    );
    let wrapped = with(
        "{ name = \"openssl\", reason = \"x\" }",
        "{ name = \"openssl\", wrappers = [\"w\"] }",
    );
    assert!(
        wrapped.contains(&"bans.deny-feld=openssl:wrappers".to_string()),
        "{wrapped:?}"
    );
    let section = sites(&format!(
        "{BASE}\n[bans.build]\nallow-build-scripts = [\"x\"]\n"
    ));
    assert!(
        section
            .iter()
            .any(|s| s.starts_with("bans.build.allow-build-scripts=")),
        "{section:?}"
    );
}

#[test]
fn ignore_by_crate_is_a_site_too() {
    let by_crate = with(
        "{ id = \"RUSTSEC-2026-0001\", reason = \"x\" },",
        "{ crate = \"bad@1.0\" },\n\"RUSTSEC-2026-0003\",",
    );
    assert!(
        by_crate.contains(&"advisories.ignore=bad@1.0".to_string()),
        "{by_crate:?}"
    );
    assert!(
        by_crate.contains(&"RUSTSEC-2026-0003".to_string()),
        "{by_crate:?}"
    );
}

#[test]
fn audit_toml_may_only_list_ignores() {
    let base = "[advisories]\nignore = [\"RUSTSEC-2026-0001\"]\n";
    assert!(audit_sites(base).is_empty());
    let loose = format!("{base}informational_warnings = []\n[yanked]\nenabled = false\n");
    assert_eq!(
        audit_sites(&loose),
        vec![
            "advisories.informational_warnings=[]",
            "yanked.enabled=false"
        ]
    );
}

#[test]
fn banned_crates_may_not_be_narrowed() {
    for (from, to, want) in [
        (
            "{ name = \"openssl\", reason = \"x\" }",
            "{ name = \"openssl\", version = \"=0.0.1\", reason = \"x\" }",
            "bans.deny-feld=openssl:version",
        ),
        (
            "{ name = \"openssl\", reason = \"x\" }",
            "{ name = \"openssl@0.0.1\", reason = \"x\" }",
            "bans.deny-version=openssl@0.0.1",
        ),
        (
            "{ name = \"native-tls\", reason = \"x\" }",
            "{ name = \"native-tls\", use-instead = \"rustls\", reason = \"x\" }",
            "bans.deny-feld=native-tls:use-instead",
        ),
        (
            "{ name = \"native-tls\", reason = \"x\" }",
            "\"native-tls@0.1\"",
            "bans.deny-version=native-tls@0.1",
        ),
    ] {
        let got = with(from, to);
        assert!(got.contains(&want.to_string()), "{want} {got:?}");
    }
    assert_eq!(
        with(
            "{ name = \"openssl\", reason = \"x\" }",
            "{ crate = \"openssl\", reason = \"x\" }"
        ),
        vec!["RUSTSEC-2026-0001"]
    );
}
