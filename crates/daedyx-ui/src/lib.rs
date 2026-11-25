//! Gemeinsames Oberflächen-Gerüst der Daedyx-Apps (egui), Stil „Daedyx Stil 1“.
//! Nur Gerüst: Tokens, Thema, Schriften, Texte, Tastenkürzel, Grundbausteine, Backend-Brücke, Screenreader-Hilfen, Startfehler-Fenster.
pub mod a11y;
pub mod bridge;
pub mod fonts;
pub mod i18n;
pub mod keys;
#[cfg(any(test, feature = "lint"))]
pub mod lint;
pub mod screenshot;
pub mod startup;
pub mod theme;
pub mod tokens;
pub mod widgets;
