//! Schriften: Systemschrift (Segoe UI unter Windows, SF Pro auf dem Mac), dahinter die in egui
//! eingebaute Open-Source-Schrift als Ersatz für fehlende Zeichen. Zwei Gewichte: normal (400)
//! in `Proportional`, halbfett (600) in der eigenen Familie [`semibold`]. Emoji sind keine Pflicht.
use eframe::egui::{Context, FontData, FontDefinitions, FontFamily};
use std::path::PathBuf;
use std::sync::Arc;

/// Name der Familie für Gewicht 600 (Knöpfe, Überschriften).
pub const SEMIBOLD: &str = "daedyx-semibold";
const REGULAR_FACE: &str = "system-regular";
const SEMIBOLD_FACE: &str = "system-semibold";
/// Größer ist keine Schriftdatei, die wir laden wollen (Grenze für Eingaben von außen).
const MAX_FONT_BYTES: u64 = 40 * 1024 * 1024;

/// Dateinamen je Gewicht, in Reihenfolge der Vorliebe (Windows, dann macOS).
const REGULAR_FILES: &[&str] = &["segoeui.ttf", "SFNS.ttf"];
const SEMIBOLD_FILES: &[&str] = &["seguisb.ttf"];

/// Familie für Gewicht 600.
pub fn semibold() -> FontFamily {
    FontFamily::Name(SEMIBOLD.into())
}

/// Halbfett, wenn diese Schriften im `ctx` geladen sind, sonst normal. So stürzt ein Baustein
/// nicht ab, wenn eine App `theme::install` vergessen hat (egui bricht bei fehlender Familie ab).
pub fn semibold_in(ctx: &Context) -> FontFamily {
    let bold = semibold();
    if ctx.fonts(|f| f.definitions().families.contains_key(&bold)) {
        bold
    } else {
        FontFamily::Proportional
    }
}

/// Schriftordner des Systems. Unter Windows aus `WINDIR`, nicht fest verdrahtet.
pub fn system_font_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(win) = std::env::var_os("WINDIR") {
        // Windows unterscheidet Groß- und Kleinschreibung in Pfaden nicht.
        dirs.push(PathBuf::from(win).join("fonts"));
    }
    if cfg!(target_os = "macos") {
        dirs.push(PathBuf::from("/System/Library/Fonts"));
        dirs.push(PathBuf::from("/Library/Fonts"));
    }
    dirs
}

/// Liest eine Schriftdatei aus den Systemordnern, höchstens [`MAX_FONT_BYTES`] groß.
pub fn read_system_font(file: &str) -> Option<Vec<u8>> {
    system_font_dirs().into_iter().find_map(|dir| {
        let path = dir.join(file);
        let len = std::fs::metadata(&path).ok()?.len();
        if len > MAX_FONT_BYTES {
            return None;
        }
        std::fs::read(path).ok()
    })
}

fn first(files: &[&str], read: &impl Fn(&str) -> Option<Vec<u8>>) -> Option<Vec<u8>> {
    files.iter().find_map(|f| read(f))
}

/// Baut die Schriftliste. `read` liefert eine Datei nach Namen (im Test erfunden).
pub fn definitions(read: impl Fn(&str) -> Option<Vec<u8>>) -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let fallback = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let regular = first(REGULAR_FILES, &read);
    let bold_bytes = first(SEMIBOLD_FILES, &read);
    let mut prop = Vec::new();
    let mut bold = Vec::new();
    if let Some(bytes) = bold_bytes {
        fonts
            .font_data
            .insert(SEMIBOLD_FACE.into(), Arc::new(FontData::from_owned(bytes)));
        bold.push(SEMIBOLD_FACE.to_string());
    }
    if let Some(bytes) = regular {
        fonts
            .font_data
            .insert(REGULAR_FACE.into(), Arc::new(FontData::from_owned(bytes)));
        prop.push(REGULAR_FACE.to_string());
        bold.push(REGULAR_FACE.to_string());
    }
    prop.extend(fallback.iter().cloned());
    bold.extend(fallback);
    fonts.families.insert(FontFamily::Proportional, prop);
    fonts.families.insert(semibold(), bold);
    fonts
}

/// Lädt die Schriften des Systems in egui.
pub fn install(ctx: &Context) {
    ctx.set_fonts(definitions(read_system_font));
}

#[cfg(test)]
mod tests;
