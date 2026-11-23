//! Texte der Oberfläche. Alle sichtbaren Texte stehen in `lang/<sprache>.json` (flache Schlüssel,
//! Platzhalter `{name}`), nie im Code. Deutsch ist die Hauptsprache; Englisch und Französisch
//! müssen genau dieselben Schlüssel haben (Test).
use std::collections::BTreeMap;
use std::fmt;

/// Sprachen der Oberfläche.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum Language {
    #[default]
    De,
    En,
    Fr,
}

impl Language {
    pub const ALL: [Language; 3] = [Language::De, Language::En, Language::Fr];

    /// Sprache aus einer Systemkennung wie `de-DE` oder `fr_CA`. Andere Sprachen fallen auf
    /// Englisch zurück; ist nichts bekannt, gilt Deutsch.
    pub fn from_locale(locale: Option<&str>) -> Language {
        let Some(locale) = locale else {
            return Language::De;
        };
        let lower = locale.to_ascii_lowercase();
        let base = lower.split(['-', '_']).next().unwrap_or_default();
        match base {
            "de" => Language::De,
            "fr" => Language::Fr,
            _ => Language::En,
        }
    }

    /// Name der Sprache in ihr selbst (aus ihrer eigenen Datei).
    pub fn native_name(self) -> String {
        Texts::for_language(self).t("sprache.name")
    }

    fn source(self) -> &'static str {
        match self {
            Language::De => include_str!("../../lang/de.json"),
            Language::En => include_str!("../../lang/en.json"),
            Language::Fr => include_str!("../../lang/fr.json"),
        }
    }
}

/// Sprachdatei ist kein flaches JSON-Objekt aus Texten.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct I18nError;

impl fmt::Display for I18nError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("i18n-invalid-json")
    }
}

impl std::error::Error for I18nError {}

/// Alle Texte einer Sprache.
#[derive(Debug, Clone, Default)]
pub struct Texts {
    map: BTreeMap<String, String>,
}

impl Texts {
    pub fn from_json(json: &str) -> Result<Texts, I18nError> {
        let map = serde_json::from_str(json).map_err(|_| I18nError)?;
        Ok(Texts { map })
    }

    /// Eingebaute Texte. Eine kaputte Datei (verhindert ein Test) ergäbe leere Texte, die als
    /// `[[schlüssel]]` sichtbar werden, statt dass das Programm abstürzt.
    pub fn for_language(language: Language) -> Texts {
        Texts::from_json(language.source()).unwrap_or_default()
    }

    pub fn has(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn entries(&self) -> impl Iterator<Item = (&String, &String)> {
        self.map.iter()
    }

    /// Text zum Schlüssel; fehlt er, steht `[[schlüssel]]` sichtbar da.
    pub fn t(&self, key: &str) -> String {
        self.map
            .get(key)
            .cloned()
            .unwrap_or_else(|| format!("[[{key}]]"))
    }

    /// Text mit ersetzten Platzhaltern `{name}`.
    pub fn tf(&self, key: &str, args: &[(&str, &str)]) -> String {
        args.iter().fold(self.t(key), |text, (name, value)| {
            text.replace(&format!("{{{name}}}"), value)
        })
    }
}

#[cfg(test)]
mod tests;
