# Bedrohungsmodell daedyx-ui (Gerüst)

Kurzfassung für das Gerüst. Fachbausteine (Mail-Anzeige, Editor) ergänzen eigene Abschnitte vor dem Bau.

| Fläche | Gefahr | Gegenmaßnahme im Gerüst | Test |
|---|---|---|---|
| Sprachdateien | kaputte Datei bringt Absturz | `Texts::for_language` fällt auf leere Texte zurück (`[[schlüssel]]` sichtbar) | `i18n::tests` |
| Tokens | falsche Farbe, zu schwacher Kontrast | Abschrift per Test an JSON gebunden; Kontrast WCAG AA | `tokens::tests`, `contrast_tests` |
| Eingabefeld | riesige Eingabe (Speicher) | `MAX_INPUT_CHARS` = 10 000 Zeichen | `eingabe_begrenzt_die_laenge` |
| Backend-Brücke | Anfrage-Schwall, hängende Oberfläche | begrenzte Warteschlangen (1024), höchstens 256 Ereignisse je Bild, Oberfläche wartet nie | `bridge::tests` |
| Schriften | übergroße oder fremde Datei im Systemordner | nur feste Dateinamen aus dem Systemordner, höchstens 40 MB | `fonts::tests` (ohne echte Dateien) |
| Bildschirmfoto | Foto zeigt fremde Fenster oder echte Daten | nur Bildpuffer des eigenen Fensters, nur `.png` in vorhandenem Ordner, nur Galerie/Demo | `screenshot::tests` |
| Fehlende Einrichtung | App vergisst `theme::install`, egui bricht bei fehlender Schriftfamilie ab | `fonts::semibold_in` fällt auf normal zurück | `oberflaeche_fragt_den_kern_und_zeigt_die_antwort` |
| Lieferkette | Lücke oder falsche Lizenz in Abhängigkeit | `deny.toml`, Versionen festgenagelt | `cargo deny check` |
| Repo | echte Daten oder Geheimnisse im Commit | `.githooks/pre-commit` + `commit-msg` | `scripts/scan-selbsttest.sh` |

Nicht abgedeckt (bewusst, kommt mit den Fachbausteinen): Netz, Dateien schreiben außer Foto, HTML, Parser.
