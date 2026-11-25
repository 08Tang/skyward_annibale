//! Chinese input: the spelling bar above the keyboard, which turns letters
//! into pinyin and the pinyin into characters.
//!
//! Only a page with a field open has the bar: search, rename, the WLAN
//! password, adding a network, a download's address, and the lock screen
//! whose password is letters. Latin letters still come from the keyboard as
//! they always did.

use super::*;

impl App {
    /// This page has a field waiting for what is typed.
    pub(super) fn text_context(&self) -> bool {
        match self.screen() {
            Screen::Search | Screen::Rename | Screen::WlanPassword | Screen::Fetch => true,
            Screen::WlanAdd => self.state.adding.is_some(),
            Screen::Editor => self.state.editor.is_some(),
            Screen::Terminal => true,
            // A report is written in either language: the bar is what types the
            // Chinese, and the page says so.
            Screen::Feedback => matches!(self.state.feedback.report, Report::Writing),
            Screen::Decrypt => matches!(self.state.data, Data::Locked { keys: Keys::Letters, checking: false, .. }),
            _ => false,
        }
    }

    /// The bar is drawn, and takes keys, on this page.
    pub(super) fn ime_context(&self) -> bool {
        cfg!(feature = "pinyin-input") && self.text_context()
    }

    /// Write the candidate at `index` into the field; with no candidate,
    /// write the spelling itself.
    pub(super) fn ime_commit(&mut self, index: usize) -> Vec<Effect> {
        if !self.ime_context() || self.state.ime.spelling.is_empty() {
            return Vec::new();
        }
        let text = self.state.ime.choose(index).unwrap_or_else(|| {
            let text = self.state.ime.spelling.clone();
            self.state.ime.clear();
            text
        });
        if self.screen() == Screen::Terminal {
            self.edits += 1;
            self.refresh();
            return match self.term.as_ref().filter(|term| term.running) {
                Some(_) => vec![Effect::TermInput(text.into_bytes())],
                None => Vec::new(),
            };
        }
        if self.screen() == Screen::Editor {
            if let Some(editor) = self.state.editor.as_mut() {
                for ch in text.chars() {
                    editor.insert(ch);
                }
            }
            self.edits += 1;
            self.refresh();
            return Vec::new();
        }
        self.typing().push_str(&text);
        self.edits += 1;
        self.refresh();
        Vec::new()
    }

    /// Which cell of the bar was pressed.
    pub(super) fn ime_action(&mut self, action: Action) {
        if !self.ime_context() {
            return;
        }
        match action {
            Action::ImeToggle => {
                self.ime_commit(0);
                self.state.ime.enabled = !self.state.ime.enabled;
                if !self.state.ime.enabled {
                    self.state.ime.clear();
                }
                self.refresh();
            }
            Action::ImePick(index) if index < self.state.ime.candidates.len() => {
                self.ime_commit(index);
            }
            Action::ImePage(next) => {
                let ime = &mut self.state.ime;
                let pages = ime.candidates.len().div_ceil(crate::ime::PER_PAGE).max(1);
                ime.page = if next { (ime.page + 1) % pages } else { (ime.page + pages - 1) % pages };
                self.refresh();
            }
            _ => {}
        }
    }

    /// A key goes to the bar first: a letter joins the spelling, backspace
    /// takes one off, space or return writes the first candidate. True means
    /// the bar took the key, and the field does not see it.
    pub(super) fn ime_key(&mut self, cap: Cap) -> bool {
        if !self.ime_context() || !self.state.ime.enabled {
            return false;
        }
        match cap {
            Cap::Char(ch) if ch.is_ascii_alphabetic() || ch == '\'' => {
                if self.state.ime.spelling.len() < 32 {
                    self.state.ime.spelling.push(ch.to_ascii_lowercase());
                    self.state.ime.update();
                }
                if self.state.layer == Layer::Shifted {
                    self.state.layer = Layer::Letters;
                }
                self.refresh();
                true
            }
            Cap::Backspace if !self.state.ime.spelling.is_empty() => {
                self.state.ime.spelling.pop();
                self.state.ime.update();
                self.refresh();
                true
            }
            Cap::Space | Cap::Enter if !self.state.ime.spelling.is_empty() => {
                self.ime_commit(0);
                true
            }
            Cap::Char(_) if !self.state.ime.spelling.is_empty() => {
                // A key that is not a letter writes down what has been
                // spelled so far, and then goes on as it always did.
                self.ime_commit(0);
                false
            }
            _ => false,
        }
    }
}
