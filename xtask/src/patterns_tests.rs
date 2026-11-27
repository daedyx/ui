//! Testfälle werden zusammengesetzt, damit der Scan dieses Repos sie nicht selbst als Fund meldet.
use super::*;

#[test]
fn real_mail_addresses_are_found_test_domains_are_not() {
    assert!(has_real_mail(concat!(
        "Kontakt: anna.beispiel",
        "@",
        "beispiel.de"
    )));
    assert!(has_real_mail(concat!("<ben", "@", "firma.co.uk>")));
    for ok in [
        concat!("anna", "@", "example.com"),
        concat!("x", "@", "mail.example.org"),
        concat!("y", "@", "server.test"),
        concat!("z", "@", "nirgends.invalid"),
        concat!("123+bot", "@", "users.noreply.github.com"),
        concat!("git", "@", "github.com:daedyx/core.git"),
        "serde@1.0.229 und actions/checkout@v4.2.2",
        "@ allein, a@b, ann@localhost",
    ] {
        assert!(!has_real_mail(ok), "{ok}");
    }
}

#[test]
fn user_folders_are_found_placeholders_are_not() {
    assert!(has_user_path(concat!("C:", "\\Users\\", "anna\\x")));
    assert!(has_user_path(concat!("d:", "/users/", "ben/")));
    assert!(has_user_path(concat!("\"C:", "\\\\Users\\\\", "clara\\\\")));
    assert!(has_user_path(concat!(" /", "Users/", "dora/Library")));
    assert!(has_user_path(concat!("`/", "home/", "emil/.cargo`")));
    for ok in [
        concat!("C:", "\\Users\\", "<Name>\\.cargo"),
        concat!("C:", "\\Users\\", "%USERNAME%"),
        concat!("/", "home/", "runner/work"),
        concat!("C:", "\\Users\\", "Public"),
        concat!("https://api.github.com/", "users/", "octo"),
        "keine Pfade hier",
    ] {
        assert!(!has_user_path(ok), "{ok}");
    }
}

#[test]
fn secrets_are_found() {
    assert!(has_secret(concat!(
        "-----BEGIN ",
        "OPENSSH PRI",
        "VATE KEY-----"
    )));
    assert!(has_secret(concat!(
        "token = gh",
        "p_",
        "aaaaBBBBccccDDDDeeee1234"
    )));
    assert!(has_secret(concat!("AK", "IA", "ABCDEFGHIJKLMNOP")));
    assert!(has_secret(concat!(
        "untrusted comment: minisign encrypted ",
        "secret key"
    )));
    assert!(!has_secret(
        "ghp_ ist nur ein Präfix, -----BEGIN PUBLIC KEY-----"
    ));
    assert!(!has_secret(concat!("AK", "IA", "kurz")));
}

#[test]
fn private_networks_are_found_documentation_ranges_are_not() {
    for bad in [
        concat!("10.", "0.0.1"),
        concat!("192.", "168.178.1"),
        concat!("172.", "20.1.2"),
    ] {
        assert!(has_private_ip(bad), "{bad}");
    }
    for ok in [
        "192.0.2.10",
        "198.51.100.1",
        "203.0.113.5",
        "127.0.0.1",
        "10.0.26300",
        "1.2.3",
        "172.32.0.1",
    ] {
        assert!(!has_private_ip(ok), "{ok}");
    }
}

#[test]
fn short_listed_words_count_only_as_whole_words() {
    // Erfundenes Kürzel: kurze Wörter (bis vier Zeichen) treffen sonst in Base64 oder längeren Wörtern.
    let words = vec!["mvx".to_string()];
    assert!(has_listed_word("beim MVX e.V.", &words));
    assert!(has_listed_word("Kreis-MVX/Ortsgruppe", &words));
    assert!(!has_listed_word("MVXS und AMVX sind andere Wörter", &words));
    assert!(!has_listed_word("aGVsbG9tdnhxd2U=", &words));
}

#[test]
fn long_listed_words_count_inside_longer_words() {
    let words = vec!["musterwacht".to_string()];
    assert!(has_listed_word("Kreis-Musterwachtgruppe", &words));
}

#[test]
fn listed_words_ignore_case() {
    let words = vec!["musterstadt".to_string()];
    assert!(has_listed_word("Schule in MUSTERSTADT", &words));
    assert!(!has_listed_word("Schule", &words));
    assert!(!has_listed_word("irgendwas", &[]));
}

#[test]
fn role_addresses_and_obvious_placeholder_names_are_not_personal() {
    assert!(!has_real_mail(concat!(
        "Co-Authored-By: Bot <noreply",
        "@",
        "firma.example.de>"
    )));
    assert!(!has_real_mail(concat!("no-reply", "@", "dienst.de")));
    assert!(!has_user_path(concat!(
        "C:",
        "\\\\Users\\\\",
        "x\\\\AppData"
    )));
    assert!(!has_user_path(concat!(
        "C:",
        "\\Users\\",
        "Beispiel\\AppData"
    )));
    assert!(has_user_path(concat!("C:", "\\Users\\", "xy\\AppData")));
}

#[test]
fn image_names_and_test_folders_are_no_personal_data() {
    // Bildnamen für hohe Auflösung („@2x“) sehen aus wie Adressen, sind aber keine.
    assert!(!has_real_mail("\"icons/128x128@2x.png\","));
    assert!(!has_real_mail("logo@3x.webp und bild@2x.jpg"));
    assert!(
        has_real_mail(concat!("anna", "@", "beispiel.de")),
        "echte Domains bleiben ein Fund"
    );
    // `test` ist ein Platzhalter wie `user`.
    assert!(!has_user_path(
        "save.mockResolvedValue(\"C:/Users/test/Protokoll.pdf\")"
    ));
    assert!(
        has_user_path(concat!("C:/Users/", "anna/x")),
        "echte Namen bleiben ein Fund"
    );
}
