//! Design-Tokens „Daedyx Stil 1“: eine Quelle (`assets/tokens.json`) für Hell und Dunkel.
//! Farben und Maße kommen in allen Bausteinen nur von hier, nie als Wert im Code.
pub mod contrast;
#[rustfmt::skip]
mod generated;
mod metrics;
mod schema;

pub use metrics::{
    Font, Line, Motion, Radius, ShadowPair, ShadowSpec, Shadows, Sizes, Spacing, TypeStep,
    TypeSteps,
};
pub use schema::{
    Accent, Accents, Kanban, Lane, Palette, Priority, States, StrongTone, Themes, TokenFile, Tone,
};

use eframe::egui::Color32;

/// Alle Werte aus `assets/tokens.json`, zur Bauzeit fest im Programm.
pub const TOKENS: TokenFile<Color32> = generated::TOKENS;

/// Welche App gerade zeichnet; bestimmt die Akzentfarbe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum AppAccent {
    #[default]
    Desk,
    Tasks,
    Board,
}

impl AppAccent {
    pub const ALL: [AppAccent; 3] = [AppAccent::Desk, AppAccent::Tasks, AppAccent::Board];
}

/// Farben für hell (`dark = false`) oder dunkel.
pub fn palette(dark: bool) -> &'static Palette<Color32> {
    if dark {
        &TOKENS.color.dark
    } else {
        &TOKENS.color.light
    }
}

impl Palette<Color32> {
    /// Akzentfarben der gewählten App.
    pub fn accent(&self, app: AppAccent) -> &Accent<Color32> {
        match app {
            AppAccent::Desk => &self.accent.desk,
            AppAccent::Tasks => &self.accent.tasks,
            AppAccent::Board => &self.accent.board,
        }
    }
}

impl ShadowPair<Color32> {
    /// Schatten für hell oder dunkel.
    pub fn of(&self, dark: bool) -> &ShadowSpec<Color32> {
        if dark { &self.dark } else { &self.light }
    }
}

#[cfg(test)]
mod contrast_tests;
#[cfg(test)]
mod tests;
