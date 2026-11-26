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

/// Update U2: Die App startet ihr Setup selbst und beendet sich danach. Das Setup wartet, bis das Programm frei ist,
/// und bricht sonst ab, statt halb zu überschreiben (vorher fest 500 ms).
#[test]
fn setup_wartet_auf_das_programmende_und_bricht_sonst_ab() {
    use crate::installer::NSI;
    let find = |needle: &str| {
        NSI.find(needle)
            .unwrap_or_else(|| panic!("fehlt: {needle}"))
    };
    let from = |start: usize| NSI.get(start..).unwrap();
    let after = |start: usize, needle: &str| start + from(start).find(needle).unwrap();
    assert!(!NSI.contains("Sleep 500"), "kein festes Warten mehr");
    let section = find(r#"Section "${PRODUCT}" SecMain"#);
    let quit = after(section, "--quit-running");
    let wait = after(section, "!insertmacro WaitForExeFree 1");
    let copy = after(section, r#"File "${SRCDIR}\${EXE}""#);
    assert!(
        quit < wait && wait < copy,
        "erst beenden, dann warten, dann ersetzen"
    );
    let mac = NSI
        .get(find("!macro WaitForExeFree")..find("!macroend"))
        .unwrap();
    for want in [
        "IntOp $R9 $R9 + 1",
        "${If} $R9 >= 50",
        "Sleep 200",
        "SetErrorLevel 5",
        "Quit",
        "/SD IDOK",
    ] {
        assert!(mac.contains(want), "{want}");
    }
    let uninst = find(r#"Section "Uninstall""#);
    assert!(from(uninst).contains("!insertmacro WaitForExeFree 0"));
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
