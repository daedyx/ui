# daedyx-ui

> **Hinweis zur Entstehung:** Daedyx wird mit Claude Code programmiert, einem KI-Assistenten von Anthropic. Der Projektinhaber legt fest, was gebaut wird, prüft die Ergebnisse und nimmt sie ab. Der größte Teil des Quellcodes, der Tests und der Dokumentation wird von der KI geschrieben; im Git-Verlauf ist das an den Zeilen „Co-Authored-By: Claude“ zu erkennen. KI kann Fehler machen. Bitte prüfe die Software, bevor du ihr wichtige Daten anvertraust. Sie wird ohne Gewähr bereitgestellt (siehe MIT-Lizenz).

Gemeinsames Oberflächen-Gerüst der Daedyx-Apps (Desk, Tasks, Launcher, später Board) in reinem
Rust mit egui 0.37, Stil „Daedyx Stil 1“. **Stand: erster Entwurf, nur Gerüst, keine Fachfunktionen.**

## Inhalt

| Teil | Datei | Was es tut |
|---|---|---|
| Tokens | `crates/daedyx-ui/assets/tokens.json` → `src/tokens/` | Farben hell/dunkel, App-Akzente Desk/Tasks/Board, Maße. Rust-Abschrift wird per Test gegen die JSON geprüft; Kontrasttest WCAG AA |
| Thema | `src/theme/` | füllt `egui::Visuals` und `Style` aus den Tokens, merkt den App-Akzent |
| Schriften | `src/fonts/` | Systemschrift (Segoe UI, SF Pro), eingebaute egui-Schrift als Ersatz; normal und halbfett |
| Texte | `lang/de.json`, `en.json`, `fr.json`, `src/i18n/` | alle sichtbaren Texte; Tests: gleiche Schlüssel, jeder benutzte Schlüssel existiert |
| Tastenkürzel | `src/keys/` | eine Tabelle für Programm und Hilfe; Strg/Cmd über `Modifiers::COMMAND` |
| Bausteine | `src/widgets/` | Knopf (4 Arten), Eingabe, Liste, Karte, Marke, Dialog, Menü, Leer-/Fehlerzustand, Fokusring; je Rolle + Beschriftung für Screenreader |
| Brücke | `src/bridge/` | Kern-Aufrufe in einem Tokio-Thread, Antworten als Ereignisse, `request_repaint` |
| Bildschirmfoto | `src/screenshot/` | `DAEDYX_SCREENSHOT=datei.png` schreibt nur das eigene Fenster |
| Galerie | `crates/daedyx-galerie` | Beispiel-App mit allen Bausteinen |
| Screenreader-Hilfen | `crates/daedyx-ui/src/a11y` | Bereich mit Rolle und Name, Auswahl, Baum-Ebene, Überschrift |
| Aufgaben-Oberfläche | `crates/daedyx-tasks-ui` | Leiste, Liste, Detail, Board auf dem Aufgaben-Kern (aus Desk M6a); eigene Texte `aufgaben.*` in `lang/`, Testrahmen mit Speicher-Datenbank und fester Uhr; die App reicht ihre Datenbank über `TaskDb` herein |
| Prüftests | `src/lint/` | keine festen Texte im Code, keine Farbwerte außerhalb der Tokens |

## Starten und prüfen

```powershell
cargo run -p daedyx-galerie
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
cargo deny --locked check
sh scripts/scan-selbsttest.sh
```

Galerie-Schalter: `DAEDYX_THEME=dark|light`, `DAEDYX_APP=desk|tasks|board`, `DAEDYX_LANG=de|en|fr`,
`DAEDYX_DIALOG=1`, `DAEDYX_SCREENSHOT=datei.png`. Tokens nach Änderung der JSON neu schreiben:
`DAEDYX_TOKENS_SCHREIBEN=1 cargo test -p daedyx-ui tokens`.

Einmal je Klon: `git config core.hooksPath .githooks` (Pre-Commit gegen echte Daten und Geheimnisse).

## Sicherheit

Workspace-Lints: `unsafe_code = "forbid"`, kein `unwrap`/`expect`/`panic`/Index im Normalpfad, keine
Ausgaben auf die Konsole. `deny.toml` prüft Lücken, Lizenzen und Quellen. Ausnahmen stehen in
`docs/SICHERHEIT-AUSNAHMEN.md`, Abhängigkeiten in `docs/ABHAENGIGKEITEN.md`.

## Lizenz

MIT, „Daedyx contributors“ (siehe `LICENSE`). Name und Logo „Daedyx“ sind nicht Teil der MIT-Lizenz.
