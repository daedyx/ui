# Ausnahmen vom Sicherheitsrahmen

Jede Ausnahme (`#[allow(...)]`, `ignore` in `deny.toml`, `#[ignore]`, `unsafe`) braucht hier eine
Zeile und das Ja des Betreuers. Stand (Tasks T1a): eine Ausnahme im Code (`Mutex::lock` in `daedyx-tasks-ui`), drei `ignore` in `deny.toml`.

| Datei | Regel | Grund | Entscheid von | Datum | Ablauf |
|---|---|---|---|---|---|
| `crates/daedyx-ui/src/tokens/generated.rs` | Dateigröße unter 200 Zeilen, `rustfmt` | erzeugte Abschrift von `tokens.json` (Test prüft Gleichheit), `#[rustfmt::skip]` | offen (Björn) | 2026-11-23 | – |
| `deny.toml` | `RUSTSEC-2026-0459` (libsqlite3-sys, SQLite) | kommt mit `daedyx-tasks-ui` über den Aufgaben-Kern; Lücke nur mit präparierter Datenbankdatei, hier nur Speicher-Datenbanken in Tests; gleiche Ausnahme wie Kern A-004, Desk und Tasks | Ja des Betreuers ausstehend | 2026-11-25 | wie Kern A-004 |
| `deny.toml` | `RUSTSEC-2026-0543` (chrono nicht gepflegt) | über den Aufgaben-Kern; keine bekannte Lücke, Ersatz ist ein eigener Auftrag | Ja des Betreuers ausstehend | 2026-11-25 | wie Kern |
| `deny.toml` | `RUSTSEC-2026-0555` (chrono-tz nicht gepflegt) | über den Aufgaben-Kern; Ersatz zusammen mit chrono | Ja des Betreuers ausstehend | 2026-11-25 | wie Kern |
| `crates/daedyx-tasks-ui/src/store_db.rs` (`impl TaskDb for Mutex<Connection>`) | `clippy::disallowed_methods` (`Mutex::lock`, Regel 2.2) | Die Aufgaben-Oberfläche greift wie bisher in Desk auf eine Datenbank-Verbindung zu (`rusqlite::Connection` ist nicht `Sync`, der Hintergrund-Takt nutzt dieselbe Datenbank). Gesperrt wird nur für einen Kern-Aufruf, nie über ein Bild hinweg; eine vergiftete Sperre wird übernommen. Umzug aus Desk, kein neues Verhalten | Ja des Betreuers ausstehend | 2026-11-25 | mit dem Umbau auf die Backend-Brücke (Kern-Aufrufe im Hintergrund) |
