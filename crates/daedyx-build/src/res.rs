//! Baut aus einer `.ico`-Datei eine Windows-Ressourcendatei (`.res`) mit dem Programmsymbol: je Bild eine
//! `RT_ICON`-Ressource und eine `RT_GROUP_ICON`-Gruppe, die sie aufzählt. Den Rest erledigt der MSVC-Linker, der
//! `.res`-Dateien direkt annimmt. Kein externes Werkzeug (kein `rc.exe`), kein Prozessstart.
//! Format: Microsoft „Resource File Formats“ (`RESOURCEHEADER`) und „Icons“ (`ICONDIR`, `GRPICONDIR`).

const RT_ICON: u16 = 3;
const RT_GROUP_ICON: u16 = 14;
/// Windows nimmt die Symbolgruppe mit der kleinsten Kennung als Programmsymbol.
const GROUP_ID: u16 = 1;
/// `MOVEABLE | DISCARDABLE` (Bilder) bzw. zusätzlich `PURE` (Gruppe), wie `rc.exe` sie setzt.
const FLAGS_ICON: u16 = 0x1010;
const FLAGS_GROUP: u16 = 0x1030;

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    let bytes: [u8; 2] = b.get(at..at.checked_add(2)?)?.try_into().ok()?;
    Some(u16::from_le_bytes(bytes))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    let bytes: [u8; 4] = b.get(at..at.checked_add(4)?)?.try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

/// Ein Bild der `.ico`: die ersten 12 Bytes seines Verzeichniseintrags (Größe, Farben, Ebenen, Bits, Länge)
/// und seine Bilddaten.
struct Image<'a> {
    entry: &'a [u8],
    data: &'a [u8],
}

fn images(ico: &[u8]) -> Option<Vec<Image<'_>>> {
    if u16_at(ico, 0)? != 0 || u16_at(ico, 2)? != 1 {
        return None;
    }
    let count = usize::from(u16_at(ico, 4)?);
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let at = i.checked_mul(16)?.checked_add(6)?;
        let entry = ico.get(at..at.checked_add(12)?)?;
        let size = usize::try_from(u32_at(ico, at.checked_add(8)?)?).ok()?;
        let offset = usize::try_from(u32_at(ico, at.checked_add(12)?)?).ok()?;
        let data = ico.get(offset..offset.checked_add(size)?)?;
        out.push(Image { entry, data });
    }
    (!out.is_empty()).then_some(out)
}

/// Ein `RESOURCEHEADER` mit Zahl-Typ und Zahl-Name (32 Bytes), dann die Daten, aufgefüllt auf 4 Bytes.
fn push_resource(out: &mut Vec<u8>, kind: u16, name: u16, flags: u16, data: &[u8]) -> Option<()> {
    let size = u32::try_from(data.len()).ok()?;
    out.extend_from_slice(&size.to_le_bytes());
    out.extend_from_slice(&32u32.to_le_bytes());
    out.extend_from_slice(&0xFFFFu16.to_le_bytes());
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(&0xFFFFu16.to_le_bytes());
    out.extend_from_slice(&name.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // DataVersion
    out.extend_from_slice(&flags.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // Sprache: neutral
    out.extend_from_slice(&0u32.to_le_bytes()); // Version
    out.extend_from_slice(&0u32.to_le_bytes()); // Characteristics
    out.extend_from_slice(data);
    while out.len() & 3 != 0 {
        out.push(0);
    }
    Some(())
}

/// Die `.res`-Datei für ein Programmsymbol; `None`, wenn die `.ico` kaputt oder leer ist.
pub fn icon_res(ico: &[u8]) -> Option<Vec<u8>> {
    let imgs = images(ico)?;
    let mut out = Vec::new();
    // Jede `.res`-Datei beginnt mit einem leeren Eintrag.
    push_resource(&mut out, 0, 0, 0, &[])?;
    let mut group = Vec::new();
    group.extend_from_slice(&0u16.to_le_bytes());
    group.extend_from_slice(&1u16.to_le_bytes());
    group.extend_from_slice(&u16::try_from(imgs.len()).ok()?.to_le_bytes());
    for (i, img) in imgs.iter().enumerate() {
        let id = u16::try_from(i.checked_add(1)?).ok()?;
        push_resource(&mut out, RT_ICON, id, FLAGS_ICON, img.data)?;
        group.extend_from_slice(img.entry);
        group.extend_from_slice(&id.to_le_bytes());
    }
    push_resource(&mut out, RT_GROUP_ICON, GROUP_ID, FLAGS_GROUP, &group)?;
    Some(out)
}
