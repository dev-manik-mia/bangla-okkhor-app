//! Avro Phonetic: Latin text → Bangla, e.g. `ami bangla` → `আমি বাংলা`.
//!
//! A faithful port of the jsAvroPhonetic parser. The grammar itself lives in
//! `data/avro-phonetic.json` (OmicronLab, MPL-1.1).

use std::sync::OnceLock;

use serde::Deserialize;

#[derive(Deserialize)]
struct Data {
    patterns: Vec<Pattern>,
    vowel: String,
    consonant: String,
    casesensitive: String,
}

#[derive(Deserialize)]
struct Pattern {
    find: String,
    replace: String,
    #[serde(default)]
    rules: Vec<Rule>,
}

#[derive(Deserialize)]
struct Rule {
    matches: Vec<Match>,
    replace: String,
}

#[derive(Deserialize)]
struct Match {
    #[serde(rename = "type")]
    kind: String,
    scope: String,
    #[serde(default)]
    value: String,
}

enum Scope {
    Punctuation,
    Vowel,
    Consonant,
    Exact(Vec<char>),
}

struct CMatch {
    suffix: bool,
    negative: bool,
    scope: Scope,
}

struct CRule {
    matches: Vec<CMatch>,
    replace: String,
}

struct CPattern {
    find: Vec<char>,
    replace: String,
    rules: Vec<CRule>,
}

struct Grammar {
    patterns: Vec<CPattern>,
    vowel: Vec<char>,
    consonant: Vec<char>,
    case_sensitive: Vec<char>,
}

fn grammar() -> &'static Grammar {
    static G: OnceLock<Grammar> = OnceLock::new();
    G.get_or_init(|| {
        let data: Data = serde_json::from_str(include_str!("../data/avro-phonetic.json")).expect("valid Avro grammar");
        let patterns = data
            .patterns
            .into_iter()
            .map(|p| CPattern {
                find: p.find.chars().collect(),
                replace: p.replace,
                rules: p
                    .rules
                    .into_iter()
                    .map(|r| CRule {
                        replace: r.replace,
                        matches: r
                            .matches
                            .into_iter()
                            .map(|m| {
                                let (negative, scope) = match m.scope.strip_prefix('!') {
                                    Some(s) => (true, s.to_string()),
                                    None => (false, m.scope),
                                };
                                let scope = match scope.as_str() {
                                    "punctuation" => Scope::Punctuation,
                                    "vowel" => Scope::Vowel,
                                    "consonant" => Scope::Consonant,
                                    _ => Scope::Exact(m.value.chars().collect()),
                                };
                                CMatch { suffix: m.kind == "suffix", negative, scope }
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect();
        Grammar {
            patterns,
            vowel: data.vowel.chars().collect(),
            consonant: data.consonant.chars().collect(),
            case_sensitive: data.casesensitive.chars().collect(),
        }
    })
}

impl Grammar {
    fn is_vowel(&self, c: char) -> bool {
        self.vowel.contains(&c.to_ascii_lowercase())
    }
    fn is_consonant(&self, c: char) -> bool {
        self.consonant.contains(&c.to_ascii_lowercase())
    }
    fn is_punctuation(&self, c: char) -> bool {
        !(self.is_vowel(c) || self.is_consonant(c))
    }

    /// Lower-case everything except the letters whose case is meaningful (O, I, U, D, G, J, N, R, S, T, Y, Z).
    fn fix(&self, input: &str) -> Vec<char> {
        input
            .chars()
            .map(|c| if self.case_sensitive.contains(&c.to_ascii_lowercase()) { c } else { c.to_ascii_lowercase() })
            .collect()
    }

    fn rule_applies(&self, rule: &CRule, s: &[char], start: usize, end: usize) -> bool {
        let len = s.len() as isize;
        rule.matches.iter().all(|m| {
            let chk: isize = if m.suffix { end as isize } else { start as isize - 1 };
            let at = |i: isize| if i >= 0 && i < len { Some(s[i as usize]) } else { None };
            let hit = match &m.scope {
                Scope::Punctuation => {
                    (chk < 0 && !m.suffix) || (chk >= len && m.suffix) || at(chk).is_some_and(|c| self.is_punctuation(c))
                }
                Scope::Vowel => at(chk).is_some_and(|c| self.is_vowel(c)),
                Scope::Consonant => at(chk).is_some_and(|c| self.is_consonant(c)),
                Scope::Exact(value) => {
                    let (a, b) = if m.suffix {
                        (end as isize, (end + value.len()) as isize)
                    } else {
                        (start as isize - value.len() as isize, start as isize)
                    };
                    // Same bounds as the reference implementation (note `b < len`).
                    a >= 0 && b < len && s[a as usize..b as usize] == value[..]
                }
            };
            hit != m.negative
        })
    }
}

/// Transliterate Latin text with the Avro Phonetic rules.
pub fn parse(input: &str) -> String {
    let g = grammar();
    let s = g.fix(input);
    let mut out = String::with_capacity(input.len() * 3);
    let mut cur = 0;
    while cur < s.len() {
        let found = g.patterns.iter().find(|p| s[cur..].starts_with(&p.find));
        match found {
            Some(p) => {
                let end = cur + p.find.len();
                let replace = p.rules.iter().find(|r| g.rule_applies(r, &s, cur, end)).map(|r| &r.replace).unwrap_or(&p.replace);
                out.push_str(replace);
                cur = end;
            }
            None => {
                out.push(s[cur]);
                cur += 1;
            }
        }
    }
    crate::converter::normalize(&out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Expected outputs generated with the reference jsAvroPhonetic 1.1.5.
    #[test]
    fn matches_reference_implementation() {
        let cases = [
            ("ami", "আমি"),
            ("amar", "আমার"),
            ("bangla", "বাংলা"),
            ("banglades", "বাংলাদেস"),
            ("amar sOnar bangla", "আমার সোনার বাংলা"),
            ("ami tOmay bhalobashi", "আমি তোমায় ভালবাশি"),
            ("kemon acho", "কেমন আছ"),
            ("bhalo", "ভাল"),
            ("kOthay", "কোথায়"),
            ("shadhinota", "শাধিনতা"),
            ("rajdhani", "রাজধানি"),
            ("dhaka", "ধাকা"),
            ("prithibi", "প্রিথিবি"),
            ("kRiShok", "কড়িষক"),
            ("bijNan", "বিজণান"),
            ("shonkho", "শঙ্খ"),
            ("oo", "উ"),
            ("ou", "অউ"),
            ("k", "ক"),
            ("kh", "খ"),
            ("ekTi", "এক্টি"),
            ("Onek", "ওনেক"),
            ("2024", "২০২৪"),
            ("ami.", "আমি।"),
            ("hridoy", "হ্রিদয়"),
            ("rr", "রর"),
            ("krri", "কৃ"),
            ("ZZ", "্য্য"),
            ("bidyaloy", "বিদ্যালয়"),
            ("kotha", "কথা"),
            ("shikkha", "শিক্ষা"),
            ("kSh", "ক্ষ"),
            ("ng", "ং"),
            ("Ng", "ঙ"),
            ("otto", "অত্ত"),
            ("attiyo", "আত্তিয়"),
            ("bondhu", "বন্ধু"),
            ("gaan", "গাআন"),
            ("sOkal", "সোকাল"),
            ("sondhya", "সন্ধ্যা"),
        ];
        for (latin, bangla) in cases {
            assert_eq!(parse(latin), crate::converter::normalize(bangla), "parse({latin})");
        }
    }
}
