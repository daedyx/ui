# Abhängigkeiten

Jede direkte Abhängigkeit steht hier mit Zweck, Lizenz und Grund. Seit Tasks T1b kommen die Bibliotheken des
Hintergrundbetriebs dazu (aus Desk, gleiche Fassungen, nichts heruntergeladen); `daedyx-sys` und `daedyx-build` haben
keine Abhängigkeiten. Versionen sind in `Cargo.toml`
festgenagelt (`=x.y.z`) und waren beim Eintragen mindestens 14 Tage alt.

| Crate | Version | Lizenz | Wofür | Warum diese |
|---|---|---|---|---|
| eframe / egui | 0.37.0 | MIT OR Apache-2.0 | Fenster, Oberfläche, AccessKit | gleiche Version wie der Launcher; Renderer **glow** (Spike S1: wgpu hält im Leerlauf einen Kern auf 100 %) |
| egui_kittest | 0.37.0 | MIT OR Apache-2.0 | Oberflächen-Tests ohne Fenster (nur Tests) | passt zu egui 0.37 |
| serde / serde_json | 1.0.229 / 1.0.154 | MIT OR Apache-2.0 | Tokens- und Sprachdateien lesen | Standard, schon im Launcher |
| tokio | 1.54.1 | MIT | Backend-Brücke (Kern-Aufrufe im Hintergrund) | nur `rt` und `sync`; 1.52.3 hatte RUSTSEC-2026-0386 |
| png | 0.18.1 | MIT OR Apache-2.0 | Bildschirmfoto-Hilfe (nur App-Fenster) | kleiner als `image` |
| daedyx-tasks-core | Tag `tasks-core-v0.2.3` | MIT | Aufgaben-Kern für `daedyx-tasks-ui` | eigenes Repo `daedyx/core`; dieselbe Fassung wie in Desk und Tasks |
| chrono / chrono-tz | 0.4.45 / 0.10.4 | MIT OR Apache-2.0 / MIT OR Apache-2.0 | Fälligkeiten, Zeitzonen in `daedyx-tasks-ui` | schon über den Kern im Baum, gleiche Fassung wie Desk |
| rusqlite | 0.37.0 | MIT | Datenbank-Verbindung des Kerns in `daedyx-tasks-ui` | schon über den Kern im Baum; Backend (`bundled`) nur in Tests, die App wählt SQLCipher |
| tray-icon | 0.25.1 | MIT OR Apache-2.0 | Symbol im Infobereich mit Menü (`daedyx-background`, aus Desk M7b) | Fassung wie in Desk; ohne Standard-Merkmale |
| tauri-winrt-notification | 0.8.1 | MIT OR Apache-2.0 | Windows-Benachrichtigungen (`daedyx-background`, nur Windows) | Fassung wie in Desk (Spike S5) |
| winreg | 0.56.0 | MIT | Autostart und App-Kennung unter HKCU (`daedyx-background`, nur Windows) | Fassung wie in Desk |
| windows | 0.62.2 | MIT OR Apache-2.0 | Info-Center zählen und leeren (`daedyx-background`, nur Windows) | Fassung wie in Desk; nur die drei nötigen Merkmale |
| tokio (weitere Merkmale) | 1.54.1 | MIT | Pipe der Einzelinstanz in `daedyx-background`: `rt-multi-thread`, `time`, `net`, `io-util` | gleiche Fassung, nur Merkmale dazu (wie Desk) |
| tokio (in `daedyx-sys`) | 1.54.1 | MIT | Gehärtete Pipe der Einzelinstanz (Entscheid T4-7): `net`, in den Tests dazu `rt`, `macros`, `time`, `io-util`; nur unter Windows | gleiche Fassung, keine neue Abhängigkeit im Baum |

Nicht genommen: eframe-Merkmal `system_fonts` (egui sucht dann selbst Ersatzschriften im System) –
bringt eine weitere Abhängigkeit; für Deutsch, Englisch und Französisch reichen Systemschrift und
die eingebaute egui-Schrift. Wieder prüfen, falls weitere Schriftsysteme Pflicht werden.
