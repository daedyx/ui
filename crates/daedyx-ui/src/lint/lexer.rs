//! Sehr kleiner Rust-Zerleger für die Prüftests: trennt Code, Kommentare und Zeichenketten.
//! Genau genug für unseren eigenen Code, kein vollständiger Rust-Parser.

/// Ein Quelltext ohne Kommentare, und alle Zeichenketten-Literale mit Zeilennummer.
pub struct Lexed {
    pub code: String,
    pub strings: Vec<(usize, String)>,
}

struct Cursor {
    chars: Vec<char>,
    pos: usize,
    line: usize,
}

impl Cursor {
    fn peek(&self, ahead: usize) -> Option<char> {
        self.chars.get(self.pos.saturating_add(ahead)).copied()
    }

    /// Ein Zeichen weiter; zählt Zeilen mit.
    fn bump(&mut self) -> Option<char> {
        let c = self.peek(0)?;
        if c == '\n' {
            self.line = self.line.saturating_add(1);
        }
        self.pos = self.pos.saturating_add(1);
        Some(c)
    }

    fn skip(&mut self, n: usize) {
        for _ in 0..n {
            self.bump();
        }
    }

    fn line_comment(&mut self) {
        while self.peek(0).is_some_and(|c| c != '\n') {
            self.bump();
        }
    }

    fn block_comment(&mut self) {
        self.skip(2);
        while self.peek(0).is_some() && !(self.peek(0) == Some('*') && self.peek(1) == Some('/')) {
            self.bump();
        }
        self.skip(2);
    }

    /// Rohzeichenkette ab `r`; `None`, wenn es doch keine ist.
    fn raw_string(&mut self) -> Option<String> {
        let mut hashes = 0usize;
        while self.peek(hashes.saturating_add(1)) == Some('#') {
            hashes = hashes.saturating_add(1);
        }
        if self.peek(hashes.saturating_add(1)) != Some('"') {
            return None;
        }
        self.skip(hashes.saturating_add(2));
        let mut text = String::new();
        while let Some(c) = self.peek(0) {
            let closes = (1..=hashes).all(|k| self.peek(k) == Some('#'));
            if c == '"' && closes {
                self.skip(hashes.saturating_add(1));
                break;
            }
            text.push(c);
            self.bump();
        }
        Some(text)
    }

    fn string(&mut self) -> String {
        self.bump();
        let mut text = String::new();
        while let Some(c) = self.bump() {
            if c == '"' {
                break;
            }
            text.push(c);
            if c == '\\'
                && let Some(escaped) = self.bump()
            {
                text.push(escaped);
            }
        }
        text
    }

    /// Zeichen-Literal `'x'` oder `'\n'`; Lebensdauern wie `'a` sind keine.
    fn char_literal(&mut self) -> bool {
        if self.peek(1) == Some('\\') {
            self.skip(2);
            while self.peek(0).is_some_and(|c| c != '\'') {
                self.bump();
            }
            self.bump();
            return true;
        }
        if self.peek(2) == Some('\'') {
            self.skip(3);
            return true;
        }
        false
    }
}

fn is_ident(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_alphanumeric() || c == '_')
}

pub fn lex(src: &str) -> Lexed {
    let mut cur = Cursor {
        chars: src.chars().collect(),
        pos: 0,
        line: 1,
    };
    let mut code = String::new();
    let mut strings = Vec::new();
    let mut prev = None;
    while let Some(c) = cur.peek(0) {
        let next = cur.peek(1);
        let start = cur.line;
        if c == '/' && next == Some('/') {
            cur.line_comment();
        } else if c == '/' && next == Some('*') {
            cur.block_comment();
        } else if c == 'r' && matches!(next, Some('"' | '#')) && !is_ident(prev) {
            if let Some(text) = cur.raw_string() {
                strings.push((start, text));
                code.push_str("\"\"");
            } else {
                code.push(c);
                cur.bump();
            }
        } else if c == '"' {
            strings.push((start, cur.string()));
            code.push_str("\"\"");
        } else if c == '\'' && cur.char_literal() {
            code.push_str("' '");
        } else {
            code.push(c);
            cur.bump();
        }
        prev = Some(c);
    }
    Lexed { code, strings }
}
