//! Schutzregeln der Installationsprobe (Review T1b N-4), ohne sie auszuführen: Vor dem ersten Start des Setups
//! bricht sie ab, wenn die Kennungen keine Vorschau sind oder schon etwas da ist, das Deinstallation und Probe
//! löschen würden. Ausgeführt wurde die Probe von Hand (Bericht T1b, Nachtrag 1).
use crate::installer::PROBE;

/// Stelle des ersten Vorkommens; fehlt der Text, schlägt der Test fehl.
fn at(needle: &str) -> usize {
    PROBE
        .find(needle)
        .unwrap_or_else(|| panic!("fehlt: {needle}"))
}

#[test]
fn probe_nimmt_nur_vorschau_kennungen_an() {
    let rule = at(r#"-not $Product.EndsWith(" Vorschau") -or -not $Aumid.EndsWith(".vorschau")"#);
    assert!(
        rule < at("Run $Setup"),
        "Prüfung vor dem ersten Setup-Start"
    );
}

#[test]
fn probe_bricht_ab_wenn_kennung_oder_verknuepfung_schon_da_sind() {
    let guard = at(
        "(Test-Path $uninstKey) -or (Test-Path $dirKey) -or ($null -ne (RunValue)) -or (Test-Path $aumidKey) -or (Test-Path $lnk)",
    );
    let after = PROBE.get(guard..).unwrap();
    let exit = guard.saturating_add(after.find("exit 2").unwrap());
    assert!(
        exit < at("Run $Setup"),
        "Abbruch vor dem ersten Setup-Start"
    );
    // Genau das löscht die Probe am Ende wieder: Uninstall-Eintrag, Merkschlüssel, App-Kennung, Verknüpfung, Autostart.
    for gone in ["$uninstKey", "$dirKey", "$aumidKey", "$lnk", "RunValue"] {
        assert!(PROBE.get(guard..exit).unwrap().contains(gone), "{gone}");
    }
}
