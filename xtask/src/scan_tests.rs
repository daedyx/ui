use super::*;

fn rules(path: &str, content: &str) -> Vec<(usize, &'static str)> {
    scan_file(path, content.as_bytes(), &[])
        .into_iter()
        .map(|f| (f.line, f.rule))
        .collect()
}

#[test]
fn clean_code_has_no_findings() {
    assert!(rules("src/lib.rs", "fn main() {}\n// Test mit anna@example.com\n").is_empty());
}

#[test]
fn findings_name_line_and_rule_but_not_the_text() {
    let content = concat!("ok\n", "mail = \"anna", "@", "beispiel.de\"\n");
    let f = scan_file("a.rs", content.as_bytes(), &[]);
    assert_eq!(
        f,
        vec![Finding {
            path: "a.rs".into(),
            line: 2,
            rule: "mailadresse"
        }]
    );
}

#[test]
fn forbidden_files_are_found_except_invented_fixtures() {
    assert_eq!(
        rules("daten/postfach.mbox", ""),
        vec![(0, "verbotener-dateityp")]
    );
    assert_eq!(
        rules("export/KONTAKTE.VCF", ""),
        vec![(0, "verbotener-dateityp")]
    );
    assert!(rules("crates/x/testdata/erfunden/termin.ics", "").is_empty());
    assert_eq!(rules("CLAUDE.md", ""), vec![(0, "claude-md-im-git")]);
    assert_eq!(rules("app/.env.local", ""), vec![(0, "env-datei")]);
}

#[test]
fn large_and_binary_files() {
    let big = vec![b'a'; MAX_FILE_BYTES + 1];
    assert_eq!(scan_file("gross.txt", &big, &[]).len(), 1);
    let binary = [0x89, b'P', b'N', b'G', 0, 1, 2, b'@', b'x'];
    assert!(scan_file("bild.png", &binary, &[]).is_empty());
}

#[test]
fn workflows_may_only_use_the_github_token() {
    let ok = "token: ${{ secrets.GITHUB_TOKEN }}";
    let bad = concat!("key: ${{ secrets.", "SIGNING_KEY }}");
    assert!(rules(".github/workflows/ci.yml", ok).is_empty());
    assert_eq!(
        rules(".github/workflows/ci.yml", bad),
        vec![(1, "secret-im-workflow")]
    );
    assert!(rules("docs/x.md", bad).is_empty());
}

#[test]
fn local_words_are_checked() {
    let words = vec!["musterstadt".to_string()];
    let f = scan_file("README.md", "Wir in Musterstadt".as_bytes(), &words);
    assert_eq!(f.len(), 1);
    assert_eq!(f.first().map(|x| x.rule), Some("lokale-wortliste"));
}

/// Erfundene Adresse außerhalb der Test-Domains, zusammengesetzt, damit diese Datei sich nicht selbst meldet.
const MAIL_LINE: &str = concat!("kontakt = anna", "@", "beispiel.de");

fn utf16(text: &str, big_endian: bool) -> Vec<u8> {
    let mut out = if big_endian {
        vec![0xFE, 0xFF]
    } else {
        vec![0xFF, 0xFE]
    };
    for unit in text.encode_utf16() {
        let bytes = if big_endian {
            unit.to_be_bytes()
        } else {
            unit.to_le_bytes()
        };
        out.extend_from_slice(&bytes);
    }
    out
}

#[test]
fn utf16_with_bom_is_read_line_by_line() {
    // So schreibt Windows PowerShell 5.1 mit `>` oder `Out-File`.
    let text = format!("Ausgabe\r\n{MAIL_LINE}\r\n");
    for big_endian in [false, true] {
        let f = scan_file("docs/log.txt", &utf16(&text, big_endian), &[]);
        assert_eq!(
            f,
            vec![Finding {
                path: "docs/log.txt".into(),
                line: 2,
                rule: "mailadresse"
            }],
            "BE={big_endian}"
        );
    }
}

#[test]
fn invalid_utf8_like_windows_1252_is_still_read() {
    let mut content = b"Gr\xfc\xdfe aus M\xfcnchen\n".to_vec();
    content.extend_from_slice(MAIL_LINE.as_bytes());
    content.extend_from_slice(b" (M\xfcller)\n");
    let f = scan_file("testdata/probe.txt", &content, &[]);
    assert_eq!(
        f,
        vec![Finding {
            path: "testdata/probe.txt".into(),
            line: 2,
            rule: "mailadresse"
        }]
    );
}

#[test]
fn unknown_binary_files_are_reported_not_skipped() {
    let content = [0x7F, b'E', b'L', b'F', 0, 0, 1, 2];
    let f = scan_file("werkzeug/hilfe.bin", &content, &[]);
    assert_eq!(
        f,
        vec![Finding {
            path: "werkzeug/hilfe.bin".into(),
            line: 0,
            rule: "unbekannte-binaerdatei"
        }]
    );
    // UTF-16 ohne BOM sieht aus wie eine Binärdatei: gemeldet, und der Inhalt wird trotzdem gelesen.
    let no_bom: Vec<u8> = utf16(MAIL_LINE, false).into_iter().skip(2).collect();
    let rules_found: Vec<_> = scan_file("probe.txt", &no_bom, &[])
        .into_iter()
        .map(|f| f.rule)
        .collect();
    assert!(
        rules_found.contains(&"unbekannte-binaerdatei"),
        "{rules_found:?}"
    );
    assert!(rules_found.contains(&"mailadresse"), "{rules_found:?}");
}

#[test]
fn image_and_font_endings_without_their_file_signature_are_reported() {
    let binary = [0u8, 2, 2, 0xFF, 0xD8];
    for name in [
        "a.png", "b.JPG", "c.jpeg", "d.gif", "e.ico", "f.icns", "g.webp", "h.bmp", "i.ttf",
        "j.otf", "k.woff", "l.woff2",
    ] {
        let got: Vec<_> = scan_file(name, &binary, &[])
            .into_iter()
            .map(|f| f.rule)
            .collect();
        assert_eq!(got, vec!["unbekannte-binaerdatei"], "{name}");
    }
    assert_eq!(
        rules("bild.png.txt", "\u{0}x"),
        vec![(0, "unbekannte-binaerdatei")]
    );
}

#[test]
fn text_disguised_as_image_or_font_is_read_and_reported() {
    let text = format!("{MAIL_LINE}\n");
    let got: Vec<_> = scan_file("docs/probe/notiz.png", text.as_bytes(), &[])
        .into_iter()
        .map(|f| f.rule)
        .collect();
    assert!(
        got.contains(&"unbekannte-binaerdatei") && got.contains(&"mailadresse"),
        "{got:?}"
    );
    let real: [(&str, &[u8]); 12] = [
        ("a.png", b"\x89PNG\r\n\x1a\n\0"),
        ("a.jpg", b"\xFF\xD8\xFF\xE0\0"),
        ("a.jpeg", b"\xFF\xD8\xFF\xDB\0"),
        ("a.gif", b"GIF89a\0"),
        ("a.bmp", b"BM\0\0"),
        ("a.webp", b"RIFF\0\0\0\0WEBP"),
        ("a.ico", b"\0\0\x01\0\x01\0"),
        ("a.icns", b"icns\0\0"),
        ("a.ttf", b"\0\x01\0\0\0"),
        ("a.otf", b"OTTO\0"),
        ("a.woff", b"wOFF\0"),
        ("a.woff2", b"wOF2\0"),
    ];
    for (name, bytes) in real {
        assert!(scan_file(name, bytes, &[]).is_empty(), "{name}");
    }
}
