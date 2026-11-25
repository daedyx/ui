use super::lexer::lex;
use super::{check_workspace, color_values, fixed_texts, missing_keys, source_files};
use crate::i18n::Texts;

#[test]
fn zerleger_trennt_kommentare_zeichenketten_und_zeichen() {
    let src = "// \"kein\" Text\nlet a = \"eins\"; /* \"auch kein\" */ let b = r#\"zwei \"x\"\"#;\nlet c = '\"'; fn f<'a>() {}";
    let l = lex(src);
    let s: Vec<&str> = l.strings.iter().map(|(_, s)| s.as_str()).collect();
    assert_eq!(s, ["eins", "zwei \"x\""]);
    assert_eq!(l.strings[1].0, 2, "Zeilennummer");
    assert!(l.code.contains("fn f<'a>"));
    assert!(!l.code.contains("kein"));
}

#[test]
fn regel_schluessel_findet_fehlende() {
    let texts = Texts::from_json(r#"{"knopf.ok":"OK"}"#).unwrap();
    let src = "t.t(\"knopf.ok\"); t.t(\"knopf.fehlt\"); let f = \"de.json\";";
    let found = missing_keys("x.rs", src, &texts);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("knopf.fehlt"));
}

#[test]
fn regel_feste_texte_findet_saetze_umlaute_und_woerter() {
    for bad in [
        "ui.label(\"Neue Mail\")",
        "\"Löschen\"",
        "\"Speichern\"",
        "\"{a} ist leer\"",
    ] {
        assert_eq!(fixed_texts("x.rs", bad).len(), 1, "{bad}");
    }
    for ok in [
        "\"knopf.ok\"",
        "\"daedyx-ui-accent\"",
        "\"segoeui.ttf\"",
        "\"WINDIR\"",
        "\"{name}\"",
        "\"/System/Library/Fonts\"",
    ] {
        assert!(fixed_texts("x.rs", ok).is_empty(), "{ok}");
    }
}

#[test]
fn regel_farben_findet_werte_im_code() {
    for bad in [
        "Color32::from_rgb(1, 2, 3)",
        "Color32::WHITE",
        "let c = \"#AABBCC\";",
        "Rgba::from_gray(0.5)",
    ] {
        assert_eq!(color_values("x.rs", bad).len(), 1, "{bad}");
    }
    for ok in [
        "Color32::TRANSPARENT",
        "p.text",
        "// Color32::RED im Kommentar",
        "\"#1\"",
    ] {
        assert!(color_values("x.rs", ok).is_empty(), "{ok}");
    }
}

#[test]
fn quelltexte_werden_gefunden() {
    let files = source_files();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert!(
        names
            .iter()
            .any(|n| n.ends_with("daedyx-ui/src/theme/mod.rs"))
    );
    assert!(
        names
            .iter()
            .any(|n| n.ends_with("daedyx-galerie/src/main.rs"))
    );
    assert!(
        !names
            .iter()
            .any(|n| n.ends_with("tests.rs") || n.contains("/lint/"))
    );
}

/// Pflichttest: jeder benutzte Schlüssel existiert, keine festen Texte, keine Farbwerte außerhalb der Tokens.
#[test]
fn workspace_haelt_die_ui_regeln_ein() {
    let found = check_workspace();
    assert!(found.is_empty(), "Verstöße:\n{}", found.join("\n"));
}

/// Review T1a N-4: Übersprungen wird nur, wer fest benannt ist und einen eigenen Wächter hat. Ein neues Crate mit
/// eigenen Sprachdateien fällt nicht still heraus, sondern wird mit den Texten von daedyx-ui geprüft (und rot).
#[test]
fn nur_benannte_crates_mit_eigenem_waechter_werden_uebersprungen() {
    let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for name in super::OWN_GUARD {
        let guard = crates.join(name).join("src").join("guard_tests.rs");
        let src = std::fs::read_to_string(&guard).unwrap_or_default();
        assert!(src.contains("check_files"), "{name}: eigener Wächter fehlt");
    }
    let files = source_files();
    assert!(files.iter().all(|f| {
        !super::OWN_GUARD
            .iter()
            .any(|n| f.to_string_lossy().contains(n))
    }));
    assert!(
        files
            .iter()
            .any(|f| f.to_string_lossy().contains("daedyx-galerie")),
        "Galerie wird geprüft"
    );
    // Tasks T1b: daedyx-sys hat keine Texte und wird mit denen von daedyx-ui geprüft.
    assert!(
        files
            .iter()
            .any(|f| f.to_string_lossy().contains("daedyx-sys")),
        "daedyx-sys wird geprüft"
    );
}
