# Sicherheits-Ausnahmen (daedyx-ui)

Jede Ausnahme von einer Regel aus `SICHERHEIT.md` steht hier: `#[allow(...)]` im Code, `ignore` in `deny.toml`,
`#[ignore]` an Tests, `unsafe`, Lockerungen in `deny.toml`, `clippy.toml`, `.cargo/audit.toml` und im Manifest. Ohne
Eintrag ist die Ausnahme nicht erlaubt. Seit Tasks T1a (Review N-3) zählt `cargo xtask exceptions` die Stellen und
vergleicht **Datei und Regel** mit dieser Tabelle (Format und Regeln wie im Kern `daedyx/core`); eine Stelle ohne
passende Zeile, eine Zeile ohne Stelle oder ein abgelaufener Eintrag sind rot.

Alle Zeilen sind Vorschläge des Bau-Chats T1a (2026-11-25), **Ja des Betreuers 2026-11-25**. A-005 bis A-011
bestanden schon vorher in `Cargo.toml` und `deny.toml` und werden erst jetzt gezählt. A-009 (kein Bann für
`openssl`) und A-012 (Lizenz `OpenSSL`) sind nach Review T1a R-2 entfallen: `openssl` ist jetzt gebannt, die Lizenz
nicht mehr erlaubt (kein Paket im Baum braucht sie). Die Nummern bleiben frei.

| Nr. | Datei | Regel | Grund | Entscheid von | Datum | Ablauf |
|---|---|---|---|---|---|---|
| A-001 | crates/daedyx-tasks-ui/src/store_db.rs | clippy::disallowed_methods (`Mutex::lock`, Regel 2.2) | Die Aufgaben-Oberfläche greift wie bisher in Desk auf eine Datenbank-Verbindung zu (`rusqlite::Connection` ist nicht `Sync`, der Hintergrund-Takt nutzt dieselbe Datenbank). Gesperrt wird nur für einen Kern-Aufruf, nie über ein Bild hinweg; eine vergiftete Sperre wird übernommen. Umzug aus Desk, kein neues Verhalten; Desk nutzt eigene `Db` und ruft diese Stelle nicht auf. Ablösung mit der Backend-Brücke. | Bau-Chat T1a, Ja des Betreuers 2026-11-25 | 2026-11-25 | 2027-05-31 |
| A-002 | deny.toml | RUSTSEC-2026-0459 (libsqlite3-sys) | Kommt mit `daedyx-tasks-ui` über den Aufgaben-Kern. Lücke nur mit präparierter Datenbankdatei; hier nur Speicher-Datenbanken in Tests, die Apps bauen mit SQLCipher. Wie Kern A-004, Desk und Tasks. Auch in `.cargo/audit.toml`. | Bau-Chat T1a, Ja des Betreuers 2026-11-25 | 2026-11-25 | 2027-02-28 |
| A-003 | deny.toml | RUSTSEC-2026-0543 (chrono nicht gepflegt) | Über den Aufgaben-Kern; keine bekannte Lücke, Ersatz ist ein eigener Auftrag (wie Kern A-005). | Bau-Chat T1a, Ja des Betreuers 2026-11-25 | 2026-11-25 | 2027-02-28 |
| A-004 | deny.toml | RUSTSEC-2026-0555 (chrono-tz nicht gepflegt) | wie A-003 (wie Kern A-006). | Bau-Chat T1a, Ja des Betreuers 2026-11-25 | 2026-11-25 | 2027-02-28 |
| A-005 | Cargo.toml | workspace.lints.rust.missing_debug_implementations=allow | Bestand schon vor T1a (Gerüst): egui-Bausteine mit Verweisen auf `Ui`/`Context` haben kein sinnvolles `Debug`. Ohne Sicherheitswirkung; erst mit dem Prüfwerkzeug (Review T1a N-3) gezählt. | Bau-Chat T1a, Ja des Betreuers 2026-11-25 | 2026-11-25 | 2027-05-31 |
| A-006 | deny.toml | graph.targets=["x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc", "x86_64-apple-darwin", "aarch64-apple-darwin"] | Bestand schon vor T1a: Daedyx läuft nur auf Windows und macOS (Entscheid 7, 10); Linux-Abhängigkeiten zählen nicht (wie Tasks A-026). Erst jetzt gezählt. | Bau-Chat T1a, Ja des Betreuers 2026-11-25 | 2026-11-25 | 2027-05-31 |
| A-007 | deny.toml | graph.all-features=false | Bestand schon vor T1a: Prüfung mit den Standard-Merkmalen; das Merkmal `lint` bringt keine Abhängigkeit mit. Erst jetzt gezählt. | Bau-Chat T1a, Ja des Betreuers 2026-11-25 | 2026-11-25 | 2027-05-31 |
| A-008 | deny.toml | bans.allow-wildcard-paths=true | Bestand schon vor T1a: Pfad-Abhängigkeiten im eigenen Workspace (nicht veröffentlicht) sind keine Platzhalter. Erst jetzt gezählt. | Bau-Chat T1a, Ja des Betreuers 2026-11-25 | 2026-11-25 | 2027-05-31 |
| A-010 | deny.toml | licenses.allow=OFL-1.1 | Bestand schon vor T1a: Schriften (egui-Standardschriften) stehen unter der SIL Open Font License. Erst jetzt gezählt. | Bau-Chat T1a, Ja des Betreuers 2026-11-25 | 2026-11-25 | 2027-05-31 |
| A-011 | deny.toml | licenses.allow=Ubuntu-font-1.0 | Bestand schon vor T1a: Ubuntu-Schrift aus den egui-Standardschriften. Erst jetzt gezählt. | Bau-Chat T1a, Ja des Betreuers 2026-11-25 | 2026-11-25 | 2027-05-31 |
| A-013 | xtask/src/files.rs | clippy::disallowed_methods | Das Prüfwerkzeug (aus dem Kern übernommen) startet `git` mit festen Argumenten, um die versionierten bzw. vorgemerkten Dateien zu lesen. Keine Fremddaten in der Befehlszeile außer Dateinamen aus Git selbst. Läuft nie in einer App. | Bau-Chat T1a, Ja des Betreuers 2026-11-25 (im Kern als A-002 angenommen) | 2026-11-25 | 2027-11-23 |
| A-014 | xtask/src/main.rs | clippy::print_stdout, clippy::print_stderr | Das Prüfwerkzeug meldet Ergebnisse auf der Konsole; nur Datei, Zeile und Regel, nie der gefundene Text. Läuft nie in einer App. | Bau-Chat T1a, Ja des Betreuers 2026-11-25 (im Kern als A-007 angenommen) | 2026-11-25 | 2027-11-23 |
| A-015 | deny.toml | RUSTSEC-2026-0370 (zvariant, nicht gepflegt) | Nur Linux-D-Bus über accesskit/egui, unter Windows und macOS nicht im Bau (`cargo deny` mit `graph.targets` sieht es nicht, `cargo audit` schon). Wie Desk A-016. Auch in `.cargo/audit.toml`. Neu prüfen mit dem nächsten egui. | Bau-Chat T1a, Ja des Betreuers 2026-11-25 | 2026-11-25 | 2027-03-31 |
| A-016 | deny.toml | RUSTSEC-2026-0371 (zbus_names, nicht gepflegt) | wie A-015 (Desk A-017). | Bau-Chat T1a, Ja des Betreuers 2026-11-25 | 2026-11-25 | 2027-03-31 |
| A-017 | deny.toml | RUSTSEC-2026-0372 (zvariant_derive, nicht gepflegt) | wie A-015 (Desk A-018). | Bau-Chat T1a, Ja des Betreuers 2026-11-25 | 2026-11-25 | 2027-03-31 |
| A-018 | .github/workflows/ci.yml | scan:secret-im-workflow | Die CI braucht Lesezugriff auf den privaten Kern (`CORE_READ_TOKEN`, nur in den Abruf-Schritten, nur Lesen), wie Desk A-043. Entfällt, sobald `daedyx/core` öffentlich ist. Das Secret ist im Repo daedyx/ui noch nicht angelegt; bis dahin ist die CI rot. | Bau-Chat T1a, Ja des Betreuers 2026-11-25 | 2026-11-25 | 2027-05-31 |

Nicht vom Zähler erfasst (keine Code-Stelle im Sinn von `cargo xtask exceptions`):
- `crates/daedyx-ui/src/tokens/generated.rs` ist länger als 200 Zeilen und trägt `#[rustfmt::skip]`. Die Datei ist
  eine erzeugte Abschrift von `tokens.json` (ein Test prüft die Gleichheit). Entscheid offen (der Betreuer), seit
  2026-11-23.
- `rust-toolchain.toml` ist seit T1a entfernt (wie in Kern, Desk und Tasks: die Version nagelt die CI fest); sie
  pinnte 1.99.1, genau die installierte Fassung.
