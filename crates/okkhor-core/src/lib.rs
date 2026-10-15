//! Platform-independent core of BanglaOkkhor: the Bijoy-style layout, the
//! typing engine, the Unicode ↔ SutonnyMJ ANSI converter and suggestions.

pub mod avro;
pub mod converter;
pub mod dictionary;
pub mod engine;
pub mod layout;

pub use converter::{to_ansi, to_unicode};
pub use dictionary::Dictionary;
pub use engine::{Edit, Encoding, Engine, Layout, Options};
pub use layout::Key;
