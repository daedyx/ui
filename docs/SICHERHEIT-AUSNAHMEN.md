# Ausnahmen vom Sicherheitsrahmen

Jede Ausnahme (`#[allow(...)]`, `ignore` in `deny.toml`, `#[ignore]`, `unsafe`) braucht hier eine
Zeile und das Ja des Betreuers. Stand: keine Ausnahme im Code; drei `ignore` in `deny.toml` (Tasks T1a).

| Datei | Regel | Grund | Entscheid von | Datum | Ablauf |
|---|---|---|---|---|---|
| `crates/daedyx-ui/src/tokens/generated.rs` | Dateigröße unter 200 Zeilen, `rustfmt` | erzeugte Abschrift von `tokens.json` (Test prüft Gleichheit), `#[rustfmt::skip]` | offen (Björn) | 2026-11-23 | – |
| `deny.toml` | `RUSTSEC-2026-0459` (libsqlite3-sys, SQLite) | kommt mit `daedyx-tasks-ui` über den Aufgaben-Kern; Lücke nur mit präparierter Datenbankdatei, hier nur Speicher-Datenbanken in Tests; gleiche Ausnahme wie Kern A-004, Desk und Tasks | Ja des Betreuers ausstehend | 2026-11-25 | wie Kern A-004 |
| `deny.toml` | `RUSTSEC-2026-0543` (chrono nicht gepflegt) | über den Aufgaben-Kern; keine bekannte Lücke, Ersatz ist ein eigener Auftrag | Ja des Betreuers ausstehend | 2026-11-25 | wie Kern |
| `deny.toml` | `RUSTSEC-2026-0555` (chrono-tz nicht gepflegt) | über den Aufgaben-Kern; Ersatz zusammen mit chrono | Ja des Betreuers ausstehend | 2026-11-25 | wie Kern |
