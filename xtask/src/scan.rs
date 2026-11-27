//! Scan eines Dateisatzes (SICHERHEIT 3.2): Dateinamen, Größe und jede Textzeile.
use crate::patterns;

pub const MAX_FILE_BYTES: usize = 1024 * 1024;
const FORBIDDEN_EXT: [&str; 13] = [
    "db", "sqlite", "sqlite3", "eml", "msg", "mbox", "pst", "ics", "vcf", "p12", "pfx", "pem",
    "key",
];
/// Nur diese Endungen gelten als Binärdatei ohne Textzeilen (Bilder, Symbole, Schriften). Jede andere Datei wird
/// als Text gelesen; enthält sie trotzdem NUL-Bytes, wird sie gemeldet statt still übersprungen.
/// Je Endung die Dateikennung am Anfang (`None` = Byte beliebig). Passt sie nicht, ist es kein Bild und keine Schrift,
/// und die Datei wird als Text gelesen (Review Prüflücken N-1).
const BINARY_EXT: [(&str, &[Option<u8>]); 12] = [
    ("png", &[Some(0x89), Some(b'P'), Some(b'N'), Some(b'G')]),
    ("jpg", &[Some(0xFF), Some(0xD8), Some(0xFF)]),
    ("jpeg", &[Some(0xFF), Some(0xD8), Some(0xFF)]),
    ("gif", &[Some(b'G'), Some(b'I'), Some(b'F'), Some(b'8')]),
    ("bmp", &[Some(b'B'), Some(b'M')]),
    (
        "webp",
        &[
            Some(b'R'),
            Some(b'I'),
            Some(b'F'),
            Some(b'F'),
            None,
            None,
            None,
            None,
            Some(b'W'),
            Some(b'E'),
            Some(b'B'),
            Some(b'P'),
        ],
    ),
    ("ico", &[Some(0), Some(0), Some(1), Some(0)]),
    ("icns", &[Some(b'i'), Some(b'c'), Some(b'n'), Some(b's')]),
    ("ttf", &[Some(0), Some(1), Some(0), Some(0)]),
    ("otf", &[Some(b'O'), Some(b'T'), Some(b'T'), Some(b'O')]),
    ("woff", &[Some(b'w'), Some(b'O'), Some(b'F'), Some(b'F')]),
    ("woff2", &[Some(b'w'), Some(b'O'), Some(b'F'), Some(b'2')]),
];

/// Bild oder Schrift mit passender Dateikennung?
fn is_known_binary(ext: Option<&str>, content: &[u8]) -> bool {
    BINARY_EXT.iter().any(|(e, magic)| {
        ext == Some(*e)
            && magic.len() <= content.len()
            && magic
                .iter()
                .zip(content)
                .all(|(m, c)| m.is_none_or(|m| m == *c))
    })
}
/// Erfundene Beispieldateien dürfen diese Endungen haben.
const FIXTURE_DIR: &str = "testdata/erfunden/";

/// Ein Fund; `line` 0 = betrifft die ganze Datei. Der gefundene Text selbst wird nie mitgegeben.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub path: String,
    pub line: usize,
    pub rule: &'static str,
}

/// Geheimnis-Verweise in Workflows: nur `secrets.GITHUB_TOKEN` ist erlaubt (plan-ohne-secrets).
fn has_workflow_secret(line: &str) -> bool {
    line.match_indices("secrets.").any(|(i, p)| {
        let rest = line.get(i.saturating_add(p.len())..).unwrap_or_default();
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        name != "GITHUB_TOKEN"
    })
}

fn line_rules(line: &str, words: &[String], workflow: bool) -> Vec<&'static str> {
    let checks = [
        (patterns::has_secret(line), "geheimnis"),
        (patterns::has_real_mail(line), "mailadresse"),
        (patterns::has_user_path(line), "benutzerpfad"),
        (patterns::has_private_ip(line), "private-ip"),
        (patterns::has_listed_word(line, words), "lokale-wortliste"),
        (workflow && has_workflow_secret(line), "secret-im-workflow"),
    ];
    checks
        .into_iter()
        .filter(|(hit, _)| *hit)
        .map(|(_, rule)| rule)
        .collect()
}

/// UTF-16 in der gewählten Byte-Reihenfolge; kaputte Einheiten werden zu U+FFFD, damit der Rest lesbar bleibt.
fn decode_utf16(bytes: &[u8], big_endian: bool) -> String {
    let units = bytes.chunks(2).map(|p| {
        let pair = [
            p.first().copied().unwrap_or(0),
            p.get(1).copied().unwrap_or(0),
        ];
        if big_endian {
            u16::from_be_bytes(pair)
        } else {
            u16::from_le_bytes(pair)
        }
    });
    char::decode_utf16(units)
        .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect()
}

/// Text einer Datei in jeder Kodierung, die im Repo vorkommen kann: UTF-16 mit BOM (so schreibt Windows PowerShell 5.1),
/// sonst UTF-8 mit Ersatzzeichen für ungültige Bytes (z. B. Windows-1252). NUL-Bytes werden entfernt, damit auch
/// UTF-16 ohne BOM noch gelesen wird.
pub fn decode_text(content: &[u8]) -> String {
    let text = match content {
        [0xFF, 0xFE, rest @ ..] => decode_utf16(rest, false),
        [0xFE, 0xFF, rest @ ..] => decode_utf16(rest, true),
        [0xEF, 0xBB, 0xBF, rest @ ..] => String::from_utf8_lossy(rest).into_owned(),
        _ => String::from_utf8_lossy(content).into_owned(),
    };
    if text.contains('\0') {
        text.replace('\0', "")
    } else {
        text
    }
}

/// Prüft eine Datei. `path` mit `/` als Trenner, relativ zum Repository.
pub fn scan_file(path: &str, content: &[u8], words: &[String]) -> Vec<Finding> {
    let mut rules: Vec<(usize, &'static str)> = Vec::new();
    let name = path.rsplit('/').next().unwrap_or(path);
    let ext = name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase());
    if name == "CLAUDE.md" {
        rules.push((0, "claude-md-im-git"));
    }
    if name == ".env" || name.starts_with(".env.") {
        rules.push((0, "env-datei"));
    }
    if ext.as_deref().is_some_and(|e| FORBIDDEN_EXT.contains(&e)) && !path.contains(FIXTURE_DIR) {
        rules.push((0, "verbotener-dateityp"));
    }
    if content.len() > MAX_FILE_BYTES {
        rules.push((0, "datei-ueber-1-mb"));
    }
    // Bekannte Binärdateien (Bilder, Symbole, Schriften) haben keine Textzeilen; alles andere wird gelesen.
    if is_known_binary(ext.as_deref(), content) {
        return to_findings(path, rules);
    }
    let utf16 = content.starts_with(&[0xFF, 0xFE]) || content.starts_with(&[0xFE, 0xFF]);
    let disguised = ext
        .as_deref()
        .is_some_and(|e| BINARY_EXT.iter().any(|(b, _)| *b == e));
    if disguised || (!utf16 && content.contains(&0)) {
        rules.push((0, "unbekannte-binaerdatei"));
    }
    let text = decode_text(content);
    let workflow = path.starts_with(".github/workflows/");
    for (i, line) in text.lines().enumerate() {
        for rule in line_rules(line, words, workflow) {
            rules.push((i.saturating_add(1), rule));
        }
    }
    to_findings(path, rules)
}

fn to_findings(path: &str, rules: Vec<(usize, &'static str)>) -> Vec<Finding> {
    rules
        .into_iter()
        .map(|(line, rule)| Finding {
            path: path.to_string(),
            line,
            rule,
        })
        .collect()
}

#[cfg(test)]
#[path = "scan_tests.rs"]
mod tests;
