//! The typing engine. It keeps the text it inserted during the current
//! composition run and, for each key, tells the host how to edit the
//! document: delete N characters before the caret, then insert a string.
//! It never touches text it did not insert itself.

use serde::{Deserialize, Serialize};

use crate::avro;
use crate::converter::to_ansi;
use crate::layout::{is_consonant, Key};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    /// Pre-base kar (ি ে ৈ) is typed first and moved after the next consonant.
    pub reorder: bool,
    /// Hasanta + kar becomes the full vowel (্ + া → আ).
    pub auto_vowel: bool,
    /// র্ typed after a consonant moves before it.
    pub reph: bool,
    /// Digit keys type ০–৯ instead of 0–9.
    pub bn_digits: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options { reorder: true, auto_vowel: true, reph: true, bn_digits: true }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Encoding {
    #[default]
    Unicode,
    /// Legacy SutonnyMJ / Bijoy ANSI, for old documents.
    Ansi,
}

/// Which Bangla keyboard is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    /// Fixed Bijoy71-style layout.
    #[default]
    Bijoy,
    /// Avro Phonetic: type how it sounds (`ami` → আমি).
    Avro,
}

/// What the host should do to the document.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Edit {
    /// Characters (Unicode scalar values) to delete before the caret.
    pub delete: usize,
    pub insert: String,
}

impl Edit {
    pub fn is_empty(&self) -> bool {
        self.delete == 0 && self.insert.is_empty()
    }
}

const PRE_KARS: [&str; 3] = ["ি", "ে", "ৈ"];

fn full_vowel(kar: &str) -> Option<&'static str> {
    Some(match kar {
        "া" => "আ", "ি" => "ই", "ী" => "ঈ", "ু" => "উ", "ূ" => "ঊ",
        "ৃ" => "ঋ", "ে" => "এ", "ৈ" => "ঐ", "ৗ" => "ঔ",
        _ => return None,
    })
}

/// Longest run we keep before starting a fresh one. Keeps diffing cheap.
const MAX_RUN: usize = 48;

#[derive(Debug, Clone, Default)]
pub struct Engine {
    pub options: Options,
    pub encoding: Encoding,
    pub layout: Layout,
    /// Unicode text inserted during the current run.
    run: String,
    /// Avro: the Latin letters typed in the current run.
    latin: String,
    /// 0 none, 1 pre-kar waiting for consonant, 2 consonant placed after pre-kar.
    pend: u8,
}

impl Engine {
    pub fn new(options: Options) -> Self {
        Engine { options, ..Default::default() }
    }

    /// Forget the current run (backspace, enter, arrows, click, app switch…).
    pub fn reset(&mut self) {
        self.run.clear();
        self.latin.clear();
        self.pend = 0;
    }

    /// The Unicode text of the current run.
    pub fn run(&self) -> &str {
        &self.run
    }

    /// Bangla letters at the end of the run — the word being typed.
    pub fn current_word(&self) -> &str {
        let start = self
            .run
            .char_indices()
            .rev()
            .take_while(|(_, c)| ('\u{0980}'..='\u{09FF}').contains(c) && !matches!(c, '।' | '০'..='৯'))
            .last()
            .map(|(i, _)| i)
            .unwrap_or(self.run.len());
        &self.run[start..]
    }

    /// The text a Bangla-mode key produces, honouring `bn_digits`.
    pub fn key_text(&self, key: Key, shift: bool) -> String {
        if !self.options.bn_digits && !shift && key.is_digit() {
            return key.latin(false).to_string();
        }
        key.bangla(shift).to_string()
    }

    /// Type a key in Bangla mode.
    pub fn key(&mut self, key: Key, shift: bool) -> Edit {
        match self.layout {
            Layout::Bijoy => {
                let text = self.key_text(key, shift);
                self.type_str(&text)
            }
            Layout::Avro => self.avro_type(key.latin(shift)),
        }
    }

    /// Avro: add a Latin letter and re-spell the whole word.
    pub fn avro_type(&mut self, c: char) -> Edit {
        let before = self.render();
        self.latin.push(c);
        self.run = self.avro_render();
        let edit = diff(&before, &self.render());
        if self.latin.chars().count() > MAX_RUN {
            self.reset();
        }
        edit
    }

    /// Backspace inside the current word. Avro removes the last Latin letter
    /// and re-spells the word; `None` means "let the app handle it".
    pub fn backspace(&mut self) -> Option<Edit> {
        if self.layout != Layout::Avro || self.latin.is_empty() {
            return None;
        }
        let before = self.render();
        self.latin.pop();
        self.run = self.avro_render();
        Some(diff(&before, &self.render()))
    }

    fn avro_render(&self) -> String {
        let out = avro::parse(&self.latin);
        if self.options.bn_digits {
            out
        } else {
            out.chars()
                .map(|c| match c {
                    '০'..='৯' => char::from(b'0' + (c as u32 - '০' as u32) as u8),
                    _ => c,
                })
                .collect()
        }
    }

    /// Type one layout unit (a letter, kar, `র্`, `্র`, …).
    pub fn type_str(&mut self, ch: &str) -> Edit {
        let before = self.render();
        self.apply(ch);
        let edit = diff(&before, &self.render());
        if self.run.chars().count() > MAX_RUN && self.pend == 0 {
            self.run.clear();
        }
        edit
    }

    /// Insert text verbatim (no composition rules), e.g. English mode on the
    /// on-screen keyboard. Ends the composition.
    pub fn insert_plain(&mut self, s: &str) -> Edit {
        self.reset();
        Edit { delete: 0, insert: s.to_string() }
    }

    /// Replace the word being typed with `word` followed by a space.
    pub fn accept_suggestion(&mut self, word: &str) -> Edit {
        let before = self.render();
        let cut = self.run.len() - self.current_word().len();
        self.run.truncate(cut);
        self.run.push_str(word);
        self.run.push(' ');
        self.pend = 0;
        let edit = diff(&before, &self.render());
        self.reset();
        edit
    }

    fn render(&self) -> String {
        match self.encoding {
            Encoding::Unicode => self.run.clone(),
            Encoding::Ansi => to_ansi(&self.run),
        }
    }

    fn apply(&mut self, ch: &str) {
        let o = self.options;
        let last = self.run.chars().last();
        let last_s = last.map(String::from).unwrap_or_default();
        let p = self.pend;

        if o.auto_vowel && last == Some('্') && full_vowel(ch).is_some() {
            self.replace_last(full_vowel(ch).unwrap());
            self.pend = 0;
        } else if ch == "া" && last == Some('অ') {
            self.replace_last("আ");
            self.pend = 0;
        } else if ch == "া" && last == Some('ে') && p != 1 {
            self.replace_last("ো");
            self.pend = 0;
        } else if ch == "ৗ" && last == Some('ে') && p != 1 {
            self.replace_last("ৌ");
            self.pend = 0;
        } else if o.reorder && PRE_KARS.contains(&ch) {
            self.run.push_str(ch);
            self.pend = 1;
        } else if p == 1 && single_consonant(ch) {
            self.replace_last(&format!("{ch}{last_s}"));
            self.pend = 2;
        } else if p == 2 && ch == "্" {
            self.replace_last(&format!("{ch}{last_s}"));
            self.pend = 1;
        } else if p == 2 && (ch == "্র" || ch == "্য") {
            self.replace_last(&format!("{ch}{last_s}"));
        } else if o.reph && ch == "র্" && last.is_some_and(is_consonant) {
            self.replace_last(&format!("র্{last_s}"));
            self.pend = 0;
        } else {
            self.run.push_str(ch);
            self.pend = 0;
        }
    }

    fn replace_last(&mut self, with: &str) {
        self.run.pop();
        self.run.push_str(with);
    }
}

fn single_consonant(s: &str) -> bool {
    let mut it = s.chars();
    matches!((it.next(), it.next()), (Some(c), None) if is_consonant(c))
}

/// Minimal edit turning `old` into `new`, in Unicode scalar values.
pub fn diff(old: &str, new: &str) -> Edit {
    let prefix: usize = old
        .chars()
        .zip(new.chars())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a.len_utf8())
        .sum();
    Edit { delete: old[prefix..].chars().count(), insert: new[prefix..].to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Type a Latin key sequence (US QWERTY) and return the resulting document.
    fn type_keys(engine: &mut Engine, keys: &str) -> String {
        let mut doc = String::new();
        for c in keys.chars() {
            let (key, shift) = latin_key(c);
            let e = engine.key(key, shift);
            for _ in 0..e.delete {
                doc.pop();
            }
            doc.push_str(&e.insert);
        }
        doc
    }

    fn latin_key(c: char) -> (Key, bool) {
        for (k, _, _) in crate::layout::LAYOUT {
            if k.latin(false) == c {
                return (*k, false);
            }
            if k.latin(true) == c {
                return (*k, true);
            }
        }
        panic!("no key for {c}");
    }

    fn t(keys: &str) -> String {
        type_keys(&mut Engine::new(Options::default()), keys)
    }

    #[test]
    fn acceptance() {
        assert_eq!(t("hfQVf"), "বাংলা");
        assert_eq!(t("dj"), "কি");
        assert_eq!(t("cjf"), "কো");
        assert_eq!(t("cjX"), "কৌ");
        assert_eq!(t("gf"), "আ");
        assert_eq!(t("gd"), "ই");
        assert_eq!(t("cjzf"), "ক্রো");
        assert_eq!(t("cngj"), "স্কে");
        assert_eq!(t("jA"), "র্ক");
    }

    #[test]
    fn digits_follow_setting() {
        assert_eq!(t("2024"), "২০২৪");
        let mut e = Engine::new(Options { bn_digits: false, ..Options::default() });
        assert_eq!(type_keys(&mut e, "2024"), "2024");
    }

    #[test]
    fn options_can_be_disabled() {
        let mut e = Engine::new(Options { reorder: false, ..Options::default() });
        assert_eq!(type_keys(&mut e, "dj"), "িক");
        let mut e = Engine::new(Options { auto_vowel: false, ..Options::default() });
        assert_eq!(type_keys(&mut e, "gd"), "্ি");
        let mut e = Engine::new(Options { reph: false, ..Options::default() });
        assert_eq!(type_keys(&mut e, "jA"), "কর্");
    }

    #[test]
    fn sentence() {
        // আমার সোনার বাংলা
        assert_eq!(t("Ffmfv"), "আমার");
        assert_eq!(t("cnfbfv"), "সোনার");
    }

    #[test]
    fn edits_are_minimal() {
        let mut e = Engine::new(Options::default());
        assert_eq!(e.key(Key::KeyD, false), Edit { delete: 0, insert: "ি".into() });
        assert_eq!(e.key(Key::KeyJ, false), Edit { delete: 1, insert: "কি".into() });
        assert_eq!(e.key(Key::KeyF, false), Edit { delete: 0, insert: "া".into() });
    }

    #[test]
    fn ansi_mode_types_in_visual_order() {
        let mut e = Engine::new(Options::default());
        e.encoding = Encoding::Ansi;
        assert_eq!(type_keys(&mut e, "dj"), "wK");
        e.reset();
        assert_eq!(type_keys(&mut e, "hfQVf"), "evsjv");
    }

    #[test]
    fn suggestions_replace_the_current_word() {
        let mut e = Engine::new(Options::default());
        let mut doc = type_keys(&mut e, "hfQ");
        assert_eq!(e.current_word(), "বাং");
        let edit = e.accept_suggestion("বাংলাদেশ");
        for _ in 0..edit.delete {
            doc.pop();
        }
        doc.push_str(&edit.insert);
        assert_eq!(doc, "বাংলাদেশ ");
        assert_eq!(e.run(), "");
    }

    fn avro(engine: &mut Engine, latin: &str) -> String {
        let mut doc = String::new();
        for c in latin.chars() {
            let e = engine.avro_type(c);
            for _ in 0..e.delete {
                doc.pop();
            }
            doc.push_str(&e.insert);
        }
        doc
    }

    #[test]
    fn avro_respells_as_you_type() {
        let mut e = Engine::new(Options::default());
        e.layout = Layout::Avro;
        assert_eq!(avro(&mut e, "bangla"), "বাংলা");
        e.reset();
        // k → ক, then kh → খ replaces it
        assert_eq!(e.avro_type('k'), Edit { delete: 0, insert: "ক".into() });
        assert_eq!(e.avro_type('h'), Edit { delete: 1, insert: "খ".into() });
    }

    #[test]
    fn avro_backspace_removes_a_latin_letter() {
        let mut e = Engine::new(Options::default());
        e.layout = Layout::Avro;
        let mut doc = avro(&mut e, "kh");
        let edit = e.backspace().unwrap();
        for _ in 0..edit.delete {
            doc.pop();
        }
        doc.push_str(&edit.insert);
        assert_eq!(doc, "ক");
        e.backspace();
        assert_eq!(e.backspace(), None);
    }

    #[test]
    fn avro_keys_use_latin_letters() {
        let mut e = Engine::new(Options::default());
        e.layout = Layout::Avro;
        let mut doc = String::new();
        for (k, shift) in [(Key::KeyA, false), (Key::KeyM, false), (Key::KeyI, false)] {
            let ed = e.key(k, shift);
            for _ in 0..ed.delete {
                doc.pop();
            }
            doc.push_str(&ed.insert);
        }
        assert_eq!(doc, "আমি");
        assert_eq!(e.current_word(), "আমি");
    }

    #[test]
    fn bijoy_has_no_engine_backspace() {
        let mut e = Engine::new(Options::default());
        e.key(Key::KeyJ, false);
        assert_eq!(e.backspace(), None);
    }

    #[test]
    fn diff_counts_scalars() {
        assert_eq!(diff("কে", "কো"), Edit { delete: 1, insert: "ো".into() });
        assert_eq!(diff("", "ক"), Edit { delete: 0, insert: "ক".into() });
    }
}
