//! Bijoy-style fixed layout, keyed by *physical* key position so it works on
//! top of any Latin layout (QWERTY, AZERTY, Dvorak…).

use serde::{Deserialize, Serialize};

/// A physical key on the main block of an ANSI keyboard. Names match the
/// web `KeyboardEvent.code` values so the frontend can share them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    Backquote, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9, Digit0, Minus, Equal,
    KeyQ, KeyW, KeyE, KeyR, KeyT, KeyY, KeyU, KeyI, KeyO, KeyP, BracketLeft, BracketRight, Backslash,
    KeyA, KeyS, KeyD, KeyF, KeyG, KeyH, KeyJ, KeyK, KeyL, Semicolon, Quote,
    KeyZ, KeyX, KeyC, KeyV, KeyB, KeyN, KeyM, Comma, Period, Slash,
    Space,
}

use Key::*;

/// Every mapped key with its `[normal, shift]` Bangla output.
pub const LAYOUT: &[(Key, &str, &str)] = &[
    (Backquote, "`", "~"), (Digit1, "১", "!"), (Digit2, "২", "@"), (Digit3, "৩", "#"),
    (Digit4, "৪", "৳"), (Digit5, "৫", "%"), (Digit6, "৬", "^"), (Digit7, "৭", "ঁ"),
    (Digit8, "৮", "*"), (Digit9, "৯", "("), (Digit0, "০", ")"), (Minus, "-", "_"), (Equal, "=", "+"),
    (KeyQ, "ঙ", "ং"), (KeyW, "য", "\u{09DF}"), (KeyE, "ড", "ঢ"), (KeyR, "প", "ফ"), (KeyT, "ট", "ঠ"),
    (KeyY, "চ", "ছ"), (KeyU, "জ", "ঝ"), (KeyI, "হ", "ঞ"), (KeyO, "গ", "ঘ"), (KeyP, "\u{09DC}", "\u{09DD}"),
    (BracketLeft, "[", "{"), (BracketRight, "]", "}"), (Backslash, "ৎ", "ঃ"),
    (KeyA, "ৃ", "র্"), (KeyS, "ু", "ূ"), (KeyD, "ি", "ী"), (KeyF, "া", "অ"), (KeyG, "্", "।"),
    (KeyH, "ব", "ভ"), (KeyJ, "ক", "খ"), (KeyK, "ত", "থ"), (KeyL, "দ", "ধ"), (Semicolon, ";", ":"), (Quote, "'", "\""),
    (KeyZ, "্র", "্য"), (KeyX, "ও", "ৗ"), (KeyC, "ে", "ৈ"), (KeyV, "র", "ল"), (KeyB, "ন", "ণ"),
    (KeyN, "স", "ষ"), (KeyM, "ম", "শ"), (Comma, ",", "<"), (Period, ".", ">"), (Slash, "/", "?"),
    (Space, " ", " "),
];

/// US-QWERTY Latin characters for each key: `[normal, shift]`.
const LATIN: &[(Key, char, char)] = &[
    (Backquote, '`', '~'), (Digit1, '1', '!'), (Digit2, '2', '@'), (Digit3, '3', '#'), (Digit4, '4', '$'),
    (Digit5, '5', '%'), (Digit6, '6', '^'), (Digit7, '7', '&'), (Digit8, '8', '*'), (Digit9, '9', '('),
    (Digit0, '0', ')'), (Minus, '-', '_'), (Equal, '=', '+'),
    (KeyQ, 'q', 'Q'), (KeyW, 'w', 'W'), (KeyE, 'e', 'E'), (KeyR, 'r', 'R'), (KeyT, 't', 'T'), (KeyY, 'y', 'Y'),
    (KeyU, 'u', 'U'), (KeyI, 'i', 'I'), (KeyO, 'o', 'O'), (KeyP, 'p', 'P'), (BracketLeft, '[', '{'),
    (BracketRight, ']', '}'), (Backslash, '\\', '|'),
    (KeyA, 'a', 'A'), (KeyS, 's', 'S'), (KeyD, 'd', 'D'), (KeyF, 'f', 'F'), (KeyG, 'g', 'G'), (KeyH, 'h', 'H'),
    (KeyJ, 'j', 'J'), (KeyK, 'k', 'K'), (KeyL, 'l', 'L'), (Semicolon, ';', ':'), (Quote, '\'', '"'),
    (KeyZ, 'z', 'Z'), (KeyX, 'x', 'X'), (KeyC, 'c', 'C'), (KeyV, 'v', 'V'), (KeyB, 'b', 'B'), (KeyN, 'n', 'N'),
    (KeyM, 'm', 'M'), (Comma, ',', '<'), (Period, '.', '>'), (Slash, '/', '?'), (Space, ' ', ' '),
];

impl Key {
    /// Bangla output for this key.
    pub fn bangla(self, shift: bool) -> &'static str {
        let (_, n, s) = LAYOUT.iter().find(|(k, _, _)| *k == self).expect("every key is mapped");
        if shift { s } else { n }
    }

    /// Latin (US QWERTY) output for this key.
    pub fn latin(self, shift: bool) -> char {
        let (_, n, s) = LATIN.iter().find(|(k, _, _)| *k == self).expect("every key is mapped");
        if shift { *s } else { *n }
    }

    pub fn is_digit(self) -> bool {
        matches!(self, Digit0 | Digit1 | Digit2 | Digit3 | Digit4 | Digit5 | Digit6 | Digit7 | Digit8 | Digit9)
    }

    /// `1..=9, 0` for digit keys.
    pub fn digit_value(self) -> Option<u8> {
        Some(match self {
            Digit0 => 0, Digit1 => 1, Digit2 => 2, Digit3 => 3, Digit4 => 4,
            Digit5 => 5, Digit6 => 6, Digit7 => 7, Digit8 => 8, Digit9 => 9,
            _ => return None,
        })
    }

    /// Parse a web `KeyboardEvent.code` (e.g. `"KeyQ"`).
    pub fn from_code(code: &str) -> Option<Key> {
        LAYOUT.iter().map(|(k, _, _)| *k).find(|k| k.code() == code)
    }

    pub fn code(self) -> &'static str {
        match self {
            Backquote => "Backquote", Digit1 => "Digit1", Digit2 => "Digit2", Digit3 => "Digit3",
            Digit4 => "Digit4", Digit5 => "Digit5", Digit6 => "Digit6", Digit7 => "Digit7",
            Digit8 => "Digit8", Digit9 => "Digit9", Digit0 => "Digit0", Minus => "Minus", Equal => "Equal",
            KeyQ => "KeyQ", KeyW => "KeyW", KeyE => "KeyE", KeyR => "KeyR", KeyT => "KeyT", KeyY => "KeyY",
            KeyU => "KeyU", KeyI => "KeyI", KeyO => "KeyO", KeyP => "KeyP", BracketLeft => "BracketLeft",
            BracketRight => "BracketRight", Backslash => "Backslash",
            KeyA => "KeyA", KeyS => "KeyS", KeyD => "KeyD", KeyF => "KeyF", KeyG => "KeyG", KeyH => "KeyH",
            KeyJ => "KeyJ", KeyK => "KeyK", KeyL => "KeyL", Semicolon => "Semicolon", Quote => "Quote",
            KeyZ => "KeyZ", KeyX => "KeyX", KeyC => "KeyC", KeyV => "KeyV", KeyB => "KeyB", KeyN => "KeyN",
            KeyM => "KeyM", Comma => "Comma", Period => "Period", Slash => "Slash", Space => "Space",
        }
    }

    /// PC set-1 scan code / Linux evdev key code (identical for the main block).
    pub fn from_scancode(sc: u32) -> Option<Key> {
        Some(match sc {
            0x29 => Backquote, 0x02 => Digit1, 0x03 => Digit2, 0x04 => Digit3, 0x05 => Digit4, 0x06 => Digit5,
            0x07 => Digit6, 0x08 => Digit7, 0x09 => Digit8, 0x0A => Digit9, 0x0B => Digit0, 0x0C => Minus,
            0x0D => Equal,
            0x10 => KeyQ, 0x11 => KeyW, 0x12 => KeyE, 0x13 => KeyR, 0x14 => KeyT, 0x15 => KeyY, 0x16 => KeyU,
            0x17 => KeyI, 0x18 => KeyO, 0x19 => KeyP, 0x1A => BracketLeft, 0x1B => BracketRight, 0x2B => Backslash,
            0x1E => KeyA, 0x1F => KeyS, 0x20 => KeyD, 0x21 => KeyF, 0x22 => KeyG, 0x23 => KeyH, 0x24 => KeyJ,
            0x25 => KeyK, 0x26 => KeyL, 0x27 => Semicolon, 0x28 => Quote,
            0x2C => KeyZ, 0x2D => KeyX, 0x2E => KeyC, 0x2F => KeyV, 0x30 => KeyB, 0x31 => KeyN, 0x32 => KeyM,
            0x33 => Comma, 0x34 => Period, 0x35 => Slash, 0x39 => Space,
            _ => return None,
        })
    }

    /// macOS virtual key code (`kVK_ANSI_*`).
    pub fn from_mac_keycode(kc: u16) -> Option<Key> {
        Some(match kc {
            0 => KeyA, 1 => KeyS, 2 => KeyD, 3 => KeyF, 4 => KeyH, 5 => KeyG, 6 => KeyZ, 7 => KeyX, 8 => KeyC,
            9 => KeyV, 11 => KeyB, 12 => KeyQ, 13 => KeyW, 14 => KeyE, 15 => KeyR, 16 => KeyY, 17 => KeyT,
            18 => Digit1, 19 => Digit2, 20 => Digit3, 21 => Digit4, 22 => Digit6, 23 => Digit5, 24 => Equal,
            25 => Digit9, 26 => Digit7, 27 => Minus, 28 => Digit8, 29 => Digit0, 30 => BracketRight, 31 => KeyO,
            32 => KeyU, 33 => BracketLeft, 34 => KeyI, 35 => KeyP, 37 => KeyL, 38 => KeyJ, 39 => Quote,
            40 => KeyK, 41 => Semicolon, 42 => Backslash, 43 => Comma, 44 => Slash, 45 => KeyN, 46 => KeyM,
            47 => Period, 49 => Space, 50 => Backquote,
            _ => return None,
        })
    }
}

pub(crate) fn is_consonant(c: char) -> bool {
    matches!(c, '\u{0995}'..='\u{09B9}' | '\u{09DC}' | '\u{09DD}' | '\u{09DF}' | '\u{09CE}')
}

/// Combining marks (signs, kars, hasanta) — shown on a dotted circle in the UI.
pub fn is_mark(c: char) -> bool {
    matches!(c, '\u{0981}'..='\u{0983}' | '\u{09BC}'..='\u{09D7}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_round_trips_its_code() {
        for (k, _, _) in LAYOUT {
            assert_eq!(Key::from_code(k.code()), Some(*k));
            k.latin(false);
        }
    }

    #[test]
    fn scancodes_and_mac_keycodes_cover_the_layout() {
        let sc = (0..0x60).filter_map(Key::from_scancode).count();
        let mac = (0..128).filter_map(Key::from_mac_keycode).count();
        assert_eq!(sc, LAYOUT.len());
        assert_eq!(mac, LAYOUT.len());
    }
}
