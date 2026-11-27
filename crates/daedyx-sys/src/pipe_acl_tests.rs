//! Vergleich über SIDs statt SDDL-Text (Release-Lauf 1): Das eingebaute Administratorkonto (`…-500`) erscheint in
//! SDDL als „LA“. Die Prüfung der eigenen Pipe erkennt es trotzdem als eigene SID und bleibt so streng wie vorher:
//! Besitzer, geschützte Zugriffsliste und genau eine Regel „voller Zugriff für die eigene SID“.
use super::probe::descriptor_text;
use super::{acl, ffi, owner_only, user_sid};

/// Eingebautes Administratorkonto (RID 500) der Konten dieses Rechners. SDDL kürzt nur SIDs der eigenen Domäne ab,
/// deshalb aus der Domäne des eigenen Benutzers gebildet.
fn admin_500() -> String {
    let me = user_sid().unwrap();
    let (domain, _) = me.rsplit_once('-').unwrap();
    format!("{domain}-500")
}

/// Erfundenes anderes Konto.
const OTHER: &str = "S-1-5-21-1000000001-1000000002-1000000003-1001";

fn read(sddl: &str) -> acl::Security {
    acl::of_descriptor(&ffi::descriptor(sddl).unwrap()).unwrap()
}

/// Die Rechte, mit denen eine Pipe nach `owner_only_sddl` dasteht (`GA` wird an der Pipe zu `FA`).
fn exact(sid: &str) -> String {
    format!("O:{sid}G:{sid}D:P(A;;FA;;;{sid})")
}

#[test]
fn sddl_kuerzt_das_administratorkonto_die_sid_pruefung_nicht() {
    let admin = admin_500();
    let sd = ffi::descriptor(&exact(&admin)).unwrap();
    // Die Ursache: Der frühere Textvergleich erwartete die SID, SDDL schreibt „LA“.
    assert_eq!(
        descriptor_text(&sd, ffi::OWNER_AND_DACL).unwrap(),
        "O:LAD:P(A;;FA;;;LA)"
    );
    let found = acl::of_descriptor(&sd).unwrap();
    assert_eq!(found.owner, admin);
    assert!(owner_only(&found, &admin));
}

#[test]
fn genau_die_eigenen_rechte_gelten_sonst_nichts() {
    let a = admin_500();
    assert!(owner_only(&read(&exact(OTHER)), OTHER));
    assert!(!owner_only(&read(&exact(OTHER)), &a), "fremde SID");
    let bad = [
        format!("O:{OTHER}G:{a}D:P(A;;FA;;;{a})"),
        format!("O:{a}G:{a}D:(A;;FA;;;{a})"),
        format!("O:{a}G:{a}D:PAI(A;;FA;;;{a})"),
        format!("O:{a}G:{a}D:PAR(A;;FA;;;{a})"),
        format!("O:{a}G:{a}D:P(A;;FA;;;{a})(A;;FA;;;WD)"),
        format!("O:{a}G:{a}D:P(D;;FA;;;{OTHER})(A;;FA;;;{a})"),
        format!("O:{a}G:{a}D:P(A;;FR;;;{a})"),
        format!("O:{a}G:{a}D:P(A;OICI;FA;;;{a})"),
        format!("O:{a}G:{a}D:P(A;;FA;;;{OTHER})"),
        format!("O:{a}G:{a}D:P"),
        format!("O:{a}G:{a}"),
    ];
    for sddl in bad {
        assert!(!owner_only(&read(&sddl), &a), "{sddl}");
    }
}

#[test]
fn ohne_zugriffsliste_gibt_es_keine_regeln() {
    let a = admin_500();
    assert_eq!(read(&format!("O:{a}G:{a}")).rules, None);
    assert_eq!(read(&format!("O:{a}G:{a}D:P")).rules, Some(Vec::new()));
}
