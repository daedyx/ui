//! Wie Benachrichtigungen aussehen: Sprache und ob Titel gezeigt werden. Titel stehen sonst unverschlüsselt im
//! Info-Center und auf dem Sperrbildschirm (Review M-2); deshalb sind sie standardmäßig aus.
use daedyx_ui::i18n::Language;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoticeStyle {
    pub language: Language,
    pub show_titles: bool,
}

impl NoticeStyle {
    pub fn new(language: Language) -> NoticeStyle {
        NoticeStyle {
            language,
            show_titles: false,
        }
    }
}

/// Der Stil, geteilt zwischen Oberfläche und Takt-Thread. Ohne Sperre (Regel 2.2): zwei unabhängige Werte, der Takt
/// liest sie zu Beginn jedes Durchgangs. In Desk war das bis Tasks T1b ein `Mutex<NoticeStyle>`.
#[derive(Debug)]
pub struct SharedStyle {
    /// Stelle in [`Language::ALL`].
    language: AtomicUsize,
    show_titles: AtomicBool,
}

fn index(language: Language) -> usize {
    Language::ALL
        .iter()
        .position(|l| *l == language)
        .unwrap_or_default()
}

impl SharedStyle {
    pub fn new(style: NoticeStyle) -> SharedStyle {
        SharedStyle {
            language: AtomicUsize::new(index(style.language)),
            show_titles: AtomicBool::new(style.show_titles),
        }
    }

    pub fn get(&self) -> NoticeStyle {
        let i = self.language.load(Ordering::Relaxed);
        NoticeStyle {
            language: Language::ALL.get(i).copied().unwrap_or_default(),
            show_titles: self.show_titles.load(Ordering::Relaxed),
        }
    }

    pub fn set_language(&self, language: Language) {
        self.language.store(index(language), Ordering::Relaxed);
    }

    pub fn set_show_titles(&self, on: bool) {
        self.show_titles.store(on, Ordering::Relaxed);
    }
}
