use std::collections::HashMap;
use std::path::Path;

use okkhor_core::{Encoding, Key, Layout, Options};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Shortcut {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
    /// Web `KeyboardEvent.code` of the key, e.g. "KeyB".
    pub code: String,
}

impl Default for Shortcut {
    fn default() -> Self {
        Shortcut { ctrl: true, alt: true, shift: false, meta: false, code: String::new() }
    }
}

impl Shortcut {
    fn ctrl_alt(code: &str) -> Self {
        Shortcut { code: code.into(), ..Default::default() }
    }

    pub fn matches(&self, key: Option<Key>, m: &okkhor_input::Modifiers) -> bool {
        !self.code.is_empty()
            && key.is_some_and(|k| k.code() == self.code)
            && self.ctrl == m.ctrl
            && self.alt == m.alt
            && self.shift == m.shift
            && self.meta == m.meta
    }

    /// Accelerator text for menus, e.g. "Ctrl+Alt+B".
    pub fn accelerator(&self) -> Option<String> {
        let key = Key::from_code(&self.code)?;
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.alt {
            parts.push("Alt".to_string());
        }
        if self.shift {
            parts.push("Shift".to_string());
        }
        if self.meta {
            parts.push("Super".to_string());
        }
        let k = key.latin(false).to_ascii_uppercase();
        parts.push(if k == ' ' { "Space".into() } else { k.to_string() });
        Some(parts.join("+"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Shortcuts {
    pub toggle: Shortcut,
    /// Switch between Bijoy71 and Avro Phonetic.
    pub layout: Shortcut,
    pub keyboard: Shortcut,
    pub converter: Shortcut,
    pub widget: Shortcut,
    pub taka: Shortcut,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Shortcuts {
            toggle: Shortcut::ctrl_alt("KeyB"),
            layout: Shortcut::ctrl_alt("KeyL"),
            keyboard: Shortcut::ctrl_alt("KeyK"),
            converter: Shortcut::ctrl_alt("KeyC"),
            widget: Shortcut::ctrl_alt("KeyW"),
            taka: Shortcut::ctrl_alt("Digit4"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub launch: bool,
    pub indicator: bool,
    /// Show the floating widget (suggestions, switch, convert).
    pub widget: bool,
    pub sugg: bool,
    pub sound: bool,
    pub bn_digits: bool,
    pub reorder: bool,
    pub auto_vowel: bool,
    pub reph: bool,
    /// The Bangla keyboard: Bijoy71 or Avro Phonetic.
    pub layout: Layout,
    /// "bn" or "en"
    pub start_in: String,
    /// "system", "light" or "dark"
    pub theme: String,
    /// Interface language: "en" or "bn"
    pub lang: String,
    pub font: String,
    pub font_size: u32,
    pub shortcuts: Shortcuts,
    /// Output encoding per app identifier.
    pub encodings: HashMap<String, Encoding>,
    pub onboarded: bool,
    /// Widget position per monitor name.
    pub widget_pos: HashMap<String, (f64, f64)>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            launch: false,
            indicator: true,
            widget: false,
            sugg: true,
            sound: false,
            bn_digits: true,
            reorder: true,
            auto_vowel: true,
            reph: true,
            layout: Layout::Bijoy,
            start_in: "bn".into(),
            theme: "system".into(),
            lang: "en".into(),
            font: "Noto Sans Bengali".into(),
            font_size: 20,
            shortcuts: Shortcuts::default(),
            encodings: HashMap::new(),
            onboarded: false,
            widget_pos: HashMap::new(),
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }

    pub fn save(&self, path: &Path) {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        match serde_json::to_string_pretty(self) {
            Ok(s) => {
                if let Err(e) = std::fs::write(path, s) {
                    log::warn!("could not save settings: {e}");
                }
            }
            Err(e) => log::warn!("could not encode settings: {e}"),
        }
    }

    pub fn engine_options(&self) -> Options {
        Options { reorder: self.reorder, auto_vowel: self.auto_vowel, reph: self.reph, bn_digits: self.bn_digits }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_json_fills_defaults() {
        let s: Settings = serde_json::from_str(r#"{"sound":true,"shortcuts":{"toggle":{"code":"KeyL"}}}"#).unwrap();
        assert!(s.sound);
        assert!(s.reorder);
        assert_eq!(s.shortcuts.toggle.code, "KeyL");
        assert!(s.shortcuts.toggle.ctrl);
        assert_eq!(s.shortcuts.keyboard.code, "KeyK");
    }

    #[test]
    fn accelerator_text() {
        assert_eq!(Shortcuts::default().toggle.accelerator().as_deref(), Some("Ctrl+Alt+B"));
        assert_eq!(Shortcuts::default().taka.accelerator().as_deref(), Some("Ctrl+Alt+4"));
    }
}
