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

## Einzelinstanz-Pipe (`daedyx-sys::pipe`, Entscheid T4-7)

Über die Pipe fließt nur `SHOW` oder `QUIT` (höchstens 16 Byte, 2 s Zeitgrenze, in `daedyx-background`). Ein Prozess
desselben Benutzers kann beides schicken; dagegen helfen Rechte nicht, er ist derselbe Benutzer.

| Fläche | Gefahr | Gegenmaßnahme | Test |
|---|---|---|---|
| Pipe-Rechte | anderes Konto auf demselben PC verbindet sich, belegt Instanzen oder liest mit | Sicherheitsbeschreibung: Besitzer und einzige Regel sind die SID des eigenen Benutzers (kein „Jeder“, anonym, Netzwerk, Administratoren, SYSTEM) | `die_pipe_erlaubt_nur_dem_eigenen_benutzer_zugriff`, `ein_fremdes_konto_kann_nicht_verbinden` (je mit Gegenprobe) |
| Fernzugriff | anderer Rechner verbindet sich über das Netz | `PIPE_REJECT_REMOTE_CLIENTS` bei jeder Instanz | `ein_entfernter_client_wird_abgewiesen` (über `\\127.0.0.1`, mit Gegenprobe) |
| Name belegt | anderes Konto legt den (vorhersagbaren) Namen vorher an; der echte Start hält sich für den zweiten und schickt sein Wort dorthin | Der zweite Start prüft, dass der Server-Prozess demselben Benutzer gehört, sonst sendet er nichts und endet mit Fehlercode (`SECOND_INSTANCE_FAILED`). Die App startet dann nicht, verrät aber nichts | `einem_fremden_server_wird_nichts_gesendet` |
| Pipe-Name | Name hängt an einer Umgebungsvariablen, ein Start mit anderer Variable läuft als zweite Instanz | Name aus der SID des Prozess-Tokens; nur Buchstaben, Ziffern, Bindestriche | `der_pipe_name_enthaelt_die_sid_des_benutzers`, `nur_echte_sid_formen_gelten` |

Nicht abgedeckt (bewusst, kommt mit den Fachbausteinen): Netz, Dateien schreiben außer Foto, HTML, Parser.
