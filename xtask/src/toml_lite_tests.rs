use super::*;

#[test]
fn sections_keys_comments_and_multi_line_values() {
    let text = "a = 1 # Kommentar\n[sec]\n# ganz\nb = \"x # kein Kommentar\"\nc = [\n  \"p\", # eins\n  { n = \"q\" },\n]\n[\"t\".u]\nd = true\n";
    let got: Vec<(String, String)> = items(text)
        .into_iter()
        .map(|i| (i.key, compact(&i.value)))
        .collect();
    let want = [
        ("a", "1"),
        ("sec.b", "\"x#keinKommentar\""),
        ("sec.c", "[\"p\",{n=\"q\"},]"),
        ("t.u.d", "true"),
    ];
    let want: Vec<(String, String)> = want
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    assert_eq!(got, want);
}

#[test]
fn values_are_split_into_strings_elements_and_fields() {
    let v = "[ \"a\", { crate = \"b\", allow = [\"x\", \"y\"] }, 'c' ]";
    assert_eq!(strings(v), vec!["a", "b", "x", "y", "c"]);
    let e = elements(v);
    assert_eq!(e.len(), 3);
    assert_eq!(
        field(&e[1], "crate").map(|f| unquote(&f)),
        Some("b".to_string())
    );
    assert_eq!(field(&e[1], "fehlt"), None);
    assert_eq!(unquote(" \"deny\" "), "deny");
    assert_eq!(unquote(" true "), "true");
}

#[test]
fn multi_line_strings_and_duplicate_keys_are_refused() {
    let hidden = "[workspace.package]\ndescription = \"\"\"x\n[workspace.lints.clippy]\nunwrap_used = \"deny\"\n[workspace.package]\ny\"\"\"\n[workspace.lints.clippy]\nunwrap_used = \"allow\"\n";
    assert!(!crate::toml_check::problems(hidden).is_empty());
    assert!(!crate::toml_check::problems("a = '''\nx\n'''\n").is_empty());
    assert!(
        !crate::toml_check::problems("[s]\nk = 1\n[s]\nk = 2\n").is_empty(),
        "doppelter Schlüssel"
    );
    assert!(
        !crate::toml_check::problems("[s]\nk = 1\nk = 2\n").is_empty(),
        "doppelter Schlüssel im selben Abschnitt"
    );
    // Listen von Tabellen ([[bin]]) dürfen dieselben Schlüssel je Eintrag haben.
    assert!(
        crate::toml_check::problems("[[bin]]\nname = \"a\"\n[[bin]]\nname = \"b\"\n").is_empty()
    );
    assert!(
        crate::toml_check::problems("a = \"x \\\"\\\"\\\" y\"\nb = 'z'\n").is_empty(),
        "drei Anführungszeichen in einem normalen Text"
    );
}
