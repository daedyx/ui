# Ausnahmen vom Sicherheitsrahmen

Jede Ausnahme (`#[allow(...)]`, `ignore` in `deny.toml`, `#[ignore]`, `unsafe`) braucht hier eine
Zeile und Björns Ja. Stand: keine Ausnahme im Code.

| Datei | Regel | Grund | Entscheid von | Datum | Ablauf |
|---|---|---|---|---|---|
| `crates/daedyx-ui/src/tokens/generated.rs` | Dateigröße unter 200 Zeilen, `rustfmt` | erzeugte Abschrift von `tokens.json` (Test prüft Gleichheit), `#[rustfmt::skip]` | offen (Björn) | 2026-11-23 | – |
