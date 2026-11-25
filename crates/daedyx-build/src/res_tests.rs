//! Der Ressourcen-Schreiber (Programmsymbol): Aufbau der `.res`-Datei und Verhalten bei kaputten Eingaben. Mit
//! Desk umgezogen (Tasks T1b); statt des Desk-Symbols eine erfundene `.ico` mit drei Bildern. Ob das Symbol im
//! fertigen Programm steckt, prüft die Startprobe der App.
use crate::res;

/// Eine `.ico` mit `n` Bildern (Inhalt beliebig; der Schreiber prüft nur den Aufbau).
fn ico(n: u16) -> Vec<u8> {
    let mut out = vec![0, 0, 1, 0];
    out.extend_from_slice(&n.to_le_bytes());
    let mut data = Vec::new();
    let mut offset = u32::from(n).saturating_mul(16).saturating_add(6);
    for i in 0..n {
        let image = vec![u8::try_from(i).unwrap(); usize::from(i).saturating_add(5)];
        let len = u32::try_from(image.len()).unwrap();
        out.extend_from_slice(&[16, 16, 0, 0, 1, 0, 32, 0]);
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset = offset.saturating_add(len);
        data.extend_from_slice(&image);
    }
    out.extend_from_slice(&data);
    out
}

fn u16_le(b: &[u8], at: usize) -> u16 {
    let s = &b[at..];
    u16::from_le_bytes([s[0], s[1]])
}

fn u32_le(b: &[u8], at: usize) -> u32 {
    let s = &b[at..];
    u32::from_le_bytes([s[0], s[1], s[2], s[3]])
}

#[test]
fn res_hat_leeren_anfang_alle_bilder_und_eine_gruppe() {
    let ico = ico(3);
    let images = usize::from(u16_le(&ico, 4));
    assert_eq!(images, 3);
    let res = res::icon_res(&ico).unwrap();
    assert_eq!(res.len() & 3, 0, "auf 4 Bytes aufgefüllt");
    // Einträge ablaufen: (Datengröße, Kopfgröße, Typ)
    let mut at = 0;
    let mut kinds = Vec::new();
    while at < res.len() {
        let size = usize::try_from(u32_le(&res, at)).unwrap();
        let header = usize::try_from(u32_le(&res, at.saturating_add(4))).unwrap();
        assert_eq!(header, 32);
        kinds.push(u16_le(&res, at.saturating_add(10)));
        at = at.saturating_add(header).saturating_add(size);
        at = at.saturating_add(3) & !3;
    }
    assert_eq!(at, res.len());
    assert_eq!(kinds.first(), Some(&0), "leerer Anfangseintrag");
    assert_eq!(
        kinds.iter().filter(|k| **k == 3).count(),
        images,
        "je Bild ein RT_ICON"
    );
    assert_eq!(kinds.last(), Some(&14), "zum Schluss die Gruppe");
}

#[test]
fn kaputte_oder_leere_ico_ergibt_keine_res() {
    let ico = ico(2);
    assert!(res::icon_res(&[]).is_none());
    assert!(res::icon_res(&[0, 0, 1, 0, 0, 0]).is_none(), "null Bilder");
    assert!(res::icon_res(&[0, 0, 2, 0, 1, 0]).is_none(), "falscher Typ");
    let mut cut = ico.clone();
    cut.truncate(30);
    assert!(res::icon_res(&cut).is_none(), "abgeschnitten");
    let mut bad = ico.clone();
    bad[18..22].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(res::icon_res(&bad).is_none(), "Offset zeigt ins Leere");
}
