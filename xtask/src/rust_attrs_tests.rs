use super::*;

fn s(src: &str) -> Vec<String> {
    sites(src)
}

#[test]
fn plain_and_spaced_attributes_are_found() {
    assert_eq!(
        s("#[allow(clippy::unwrap_used)]\nfn f() {}"),
        vec!["clippy::unwrap_used"]
    );
    assert_eq!(
        s("# [ allow ( clippy :: unwrap_used ) ]\nfn f() {}"),
        vec!["clippy::unwrap_used"]
    );
    assert_eq!(
        s("#! [expect(clippy::panic, reason = \"x\")]"),
        vec!["clippy::panic"]
    );
}

#[test]
fn cfg_attr_hides_nothing() {
    assert_eq!(
        s("#[cfg_attr(all(), allow(clippy::unwrap_used))]\nfn f() {}"),
        vec!["clippy::unwrap_used"]
    );
    let nested = "#[cfg_attr(test, cfg_attr(all(), expect(clippy::panic, reason = \"x\")), allow(dead_code))]";
    assert_eq!(s(nested), vec!["clippy::panic", "dead_code"]);
    assert_eq!(
        s("#[cfg_attr(windows, ignore)]\n#[test]\nfn t() {}"),
        vec!["ignore"]
    );
}

#[test]
fn multi_line_attributes_list_all_rules_sorted() {
    let src = "#![\n  allow(\n    clippy::print_stdout,\n    clippy::exit,\n    reason = \"A-001\"\n  )\n]\n";
    assert_eq!(s(src), vec!["clippy::exit,clippy::print_stdout"]);
}

#[test]
fn comments_and_text_do_not_count() {
    let src = r##"
// #[allow(clippy::a)]
/* #[allow(clippy::b)] /* verschachtelt */ #[allow(clippy::c)] */
/// Beispiel: #[allow(clippy::d)]
let a = "#[allow(clippy::e)] \" #[allow(clippy::f)]";
let b = r#"#[allow(clippy::g)] " unsafe { }"#;
let c = '"';
let d = b'#';
fn g<'a>(x: &'a str) -> &'a str { x }
#[allow(clippy::echt)]
fn h() {}
"##;
    assert_eq!(s(src), vec!["clippy::echt"]);
}

#[test]
fn ignore_and_unsafe_are_found_in_every_spelling() {
    assert_eq!(
        s("#[ignore]\n#[ignore = \"langsam\"]\n#[ ignore ]"),
        vec!["ignore", "ignore", "ignore"]
    );
    let src =
        "unsafe {}\nunsafe\n{}\nunsafe fn a() {}\nunsafe impl X for Y {}\nunsafe extern \"C\" {}\n";
    assert_eq!(s(src).len(), 5);
    assert!(s("#![forbid(unsafe_code)]\nlet unsafe_x = 1;").is_empty());
}

#[test]
fn other_attributes_are_no_exceptions() {
    assert!(
        s("#[derive(Debug)]\n#[cfg(test)]\n#[test]\n#[deny(clippy::x)]\n#[warn(clippy::y)]")
            .is_empty()
    );
    assert!(s("macro_rules! m { ($(#[$a:meta])*) => {} }").is_empty());
}

#[test]
fn attributes_from_macro_placeholders_are_counted_for_a_hard_stop() {
    let hidden = "macro_rules! with { ($m:meta, $i:item) => { #[$m] $i }; }\nwith!(allow(clippy::panic), pub fn f() {});";
    assert_eq!(macro_attributes(hidden), 1);
    assert_eq!(
        macro_attributes("macro_rules! m { ($($a:tt)*) => { #![$($a)*] }; }"),
        1
    );
    assert_eq!(
        macro_attributes("macro_rules! m { ($x:meta) => { #[cfg_attr(test, $x)] fn f() {} }; }"),
        1
    );
    assert_eq!(
        macro_attributes("#[allow(clippy::x)]\n#[doc = \"#[$m]\"]\n// #[$m]\nfn f() {}"),
        0
    );
}

#[test]
fn cfg_that_hides_code_from_clippy_needs_an_entry() {
    // Die CI prüft mit Clippy unter Windows, ohne `--release`, mit allen Merkmalen, mit und ohne `test`.
    for (src, want) in [
        ("#[cfg(any())]\n#[test]\nfn t() {}", "cfg:any()"),
        ("#[cfg(all(test, any()))]\nfn t() {}", "cfg:all(test,any())"),
        ("#![cfg(not(all()))]", "cfg:not(all())"),
        ("#[cfg(not(clippy))]\nfn f() {}", "cfg:not(clippy)"),
        (
            "#[cfg(not(debug_assertions))]\nfn f() {}",
            "cfg:not(debug_assertions)",
        ),
        ("#[cfg(not(windows))]\nfn f() {}", "cfg:not(windows)"),
        (
            "#[cfg(target_os = \"macos\")]\nfn f() {}",
            "cfg:target_os=\"macos\"",
        ),
        (
            "#[cfg(eigener_schalter)]\nfn f() {}",
            "cfg:eigener_schalter",
        ),
    ] {
        assert_eq!(s(src), vec![want], "{src}");
    }
    for src in [
        "#[cfg(test)]\nfn t() {}",
        "#[cfg(not(test))]\nfn t() {}",
        "#[cfg(windows)]\nfn f() {}",
        "#[cfg(debug_assertions)]\nfn f() {}",
        "#[cfg(target_os = \"windows\")]\nfn f() {}",
        "#[cfg(any(test, target_os = \"macos\"))]\nfn f() {}",
        "#![cfg(all(windows, feature = \"test-helpers\"))]",
        "#[cfg(not(any()))]\nfn t() {}",
        "#[cfg(all())]\nfn t() {}",
        "#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = \"windows\")]",
    ] {
        assert!(s(src).is_empty(), "{src}");
    }
}
