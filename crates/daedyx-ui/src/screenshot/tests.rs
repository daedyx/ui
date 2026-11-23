use super::{Shot, encode_png, valid_target};
use eframe::egui::{Color32, ColorImage};
use std::path::Path;

#[test]
fn png_hat_die_richtige_groesse_und_farbe() {
    let pixel = Color32::from_rgb(10, 20, 30);
    let image = ColorImage::new([3, 2], vec![pixel; 6]);
    let bytes = encode_png(&image).unwrap();
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buf).unwrap();
    assert_eq!((info.width, info.height), (3, 2));
    assert_eq!(&buf[..4], &[10, 20, 30, 255]);
}

#[test]
fn nur_png_in_vorhandenem_ordner() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(valid_target(&here.join("probe.png")));
    assert!(valid_target(&here.join("probe.PNG")));
    assert!(!valid_target(&here.join("probe.exe")));
    assert!(!valid_target(&here.join("gibt-es-nicht").join("probe.png")));
    assert!(Shot::new(here.join("probe.txt")).is_none());
}

#[test]
fn ohne_umgebungsvariable_kein_foto() {
    if std::env::var_os(super::ENV).is_none() {
        assert!(Shot::from_env().is_none());
    }
}
