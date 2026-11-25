//! Erkennungsmuster für den Scan (SICHERHEIT 3.2): Geheimnisse und echte persönliche Daten.
//! Nur Standardbibliothek. Gefundener Text wird nie ausgegeben, nur Regel und Zeile.

/// Mail-Domains, die für Tests reserviert sind (RFC 2606/6761) oder keine Person bezeichnen.
const ALLOWED_MAIL_DOMAINS: [&str; 5] = [
    "example.com",
    "example.org",
    "example.net",
    "users.noreply.github.com",
    "noreply.github.com",
];
const ALLOWED_MAIL_ENDINGS: [&str; 4] = [".example", ".test", ".invalid", ".localhost"];
/// Platzhalter nach `Users/` oder `home/`, die keinen echten Benutzer nennen.
/// Dazu gilt jeder einzelne Buchstabe als Platzhalter (`C:\Users\x`).
const PATH_PLACEHOLDERS: [&str; 11] = [
    "public", "default", "name", "user", "username", "runner", "benutzer", "me", "beispiel",
    "example", "test",
];
/// Bildnamen für hohe Auflösung (`icon@2x.png`) haben die Form einer Adresse; diese Endungen sind keine Domains.
const IMAGE_ENDINGS: [&str; 7] = [".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp", ".ico"];
/// Rollen-Adressen ohne Person dahinter.
const ROLE_LOCAL_PARTS: [&[u8]; 2] = [b"noreply", b"no-reply"];
const TOKEN_PREFIXES: [&str; 10] = [
    "ghp_",
    "gho_",
    "ghu_",
    "ghs_",
    "ghr_",
    "github_pat_",
    "glpat-",
    "xoxb-",
    "xoxp-",
    "sk-ant-",
];

fn is_local_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || b"._%+-".contains(&c)
}

fn is_domain_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'.' || c == b'-'
}

fn mail_domain_allowed(domain: &str) -> bool {
    let d = domain.to_ascii_lowercase();
    ALLOWED_MAIL_DOMAINS
        .iter()
        .any(|a| d == *a || d.ends_with(&format!(".{a}")))
        || ALLOWED_MAIL_ENDINGS.iter().any(|e| d.ends_with(e))
}

/// Mailadressen außerhalb der erlaubten Test-Domains (`git@github.com` ist keine Person).
pub fn has_real_mail(line: &str) -> bool {
    let b = line.as_bytes();
    b.iter()
        .enumerate()
        .filter(|(_, c)| **c == b'@')
        .any(|(i, _)| {
            let left = b.get(..i).unwrap_or_default();
            let local_len = left.iter().rev().take_while(|c| is_local_char(**c)).count();
            let right = b.get(i.saturating_add(1)..).unwrap_or_default();
            let dom_len = right.iter().take_while(|c| is_domain_char(**c)).count();
            let local = left
                .get(left.len().saturating_sub(local_len)..)
                .unwrap_or_default();
            let Some(domain) = right
                .get(..dom_len)
                .and_then(|d| std::str::from_utf8(d).ok())
            else {
                return false;
            };
            let domain = domain.trim_end_matches(['.', '-']);
            let tld_ok = domain
                .rsplit_once('.')
                .is_some_and(|(_, t)| t.len() >= 2 && t.bytes().all(|c| c.is_ascii_alphabetic()));
            if local.is_empty() || !tld_ok {
                return false;
            }
            let role = ROLE_LOCAL_PARTS
                .iter()
                .any(|r| local.eq_ignore_ascii_case(r));
            let image = IMAGE_ENDINGS
                .iter()
                .any(|e| domain.to_ascii_lowercase().ends_with(e));
            !role
                && !image
                && !(local == b"git" && domain == "github.com")
                && !mail_domain_allowed(domain)
        })
}

/// Benutzerordner unter Windows (`X:\Users\name`), macOS (`/Users/name`) oder Linux (`/home/name`).
pub fn has_user_path(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let b = lower.as_bytes();
    let drive = [":\\users\\", ":/users/", ":\\\\users\\\\"];
    let root = ["/users/", "/home/"];
    let hit_at = |start: usize, len: usize, need_drive: bool| -> bool {
        let before = start.checked_sub(1).and_then(|p| b.get(p)).copied();
        let ok_before = if need_drive {
            before.is_some_and(|c| c.is_ascii_alphabetic())
        } else {
            before.is_none_or(|c| c.is_ascii_whitespace() || b"\"'`(=:~".contains(&c))
        };
        let rest = b.get(start.saturating_add(len)..).unwrap_or_default();
        let name: Vec<u8> = rest
            .iter()
            .take_while(|c| !b"\\/ \"'`)<>,;".contains(c))
            .copied()
            .collect();
        ok_before
            && name.len() > 1
            && !b"<%${*.~".contains(name.first().unwrap_or(&b'<'))
            && !PATH_PLACEHOLDERS
                .iter()
                .any(|p| p.as_bytes() == name.as_slice())
    };
    let find_all = |pat: &str, need_drive: bool| {
        lower
            .match_indices(pat)
            .any(|(i, _)| hit_at(i, pat.len(), need_drive))
    };
    drive.iter().any(|p| find_all(p, true)) || root.iter().any(|p| find_all(p, false))
}

/// Private Schlüssel und bekannte Token-Formate.
pub fn has_secret(line: &str) -> bool {
    if line.contains(concat!("-----BEGIN", " ")) && line.contains("PRIVATE KEY") {
        return true;
    }
    if line.contains(concat!("minisign encrypted ", "secret key")) {
        return true;
    }
    let token_body = |rest: &str| {
        rest.bytes()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == b'_')
            .count()
            >= 16
    };
    TOKEN_PREFIXES.iter().any(|p| {
        line.match_indices(p).any(|(i, _)| {
            line.get(i.saturating_add(p.len())..)
                .is_some_and(token_body)
        })
    }) || line.match_indices(concat!("AK", "IA")).any(|(i, _)| {
        line.get(i.saturating_add(4)..).is_some_and(|r| {
            r.bytes()
                .take(16)
                .filter(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                .count()
                == 16
        })
    })
}

/// IPv4-Adressen aus privaten Netzen (10/8, 172.16/12, 192.168/16); Dokumentationsbereiche sind erlaubt.
pub fn has_private_ip(line: &str) -> bool {
    line.split(|c: char| !(c.is_ascii_digit() || c == '.'))
        .any(|w| {
            let parts: Vec<u8> = w.split('.').filter_map(|p| p.parse::<u8>().ok()).collect();
            if w.split('.').count() != 4 || parts.len() != 4 {
                return false;
            }
            matches!(parts.as_slice(), [10, ..] | [192, 168, ..])
                || matches!(parts.as_slice(), [172, b, ..] if (16..=31).contains(b))
        })
}

/// Namen echter Organisationen, die in kein Repository gehören.
pub fn has_org_name(line: &str) -> bool {
    let words = line.split(|c: char| !c.is_alphanumeric());
    let org = concat!("M", "VX");
    words.clone().any(|w| w == org) || line.to_lowercase().contains(concat!("muster", "wacht"))
}

/// Wörter aus der lokalen Wortliste (nie im Repository), ohne Groß-/Kleinschreibung.
pub fn has_listed_word(line: &str, words: &[String]) -> bool {
    let lower = line.to_lowercase();
    words.iter().any(|w| lower.contains(w.as_str()))
}

#[cfg(test)]
#[path = "patterns_tests.rs"]
mod tests;
