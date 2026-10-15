//! Unicode ↔ ANSI (SutonnyMJ / Bijoy) converter.
//!
//! ANSI Bangla stores glyphs in *visual* order: pre-base kars come before
//! the consonant cluster, reph (©) after it, and many conjuncts have their
//! own code points. The conjunct tables follow the de-facto SutonnyMJ font
//! layout used by the MIT-licensed `@codesigntheory/bnunicode2ansi` and
//! `@codesigntheory/bnbijoy2unicode` converters.

use crate::layout::is_consonant;

/// Unicode cluster → ANSI. Matched greedily, longest first.
const U2A_CONJUNCTS: &[(&str, &str)] = &[
    ("ক্ষ্ম", "²"), ("স্ক্র", "¯Œ"), ("ষ্ক্র", "®Œ"), ("ঙ্ক্ষ", "•¶"), ("চ্ছ্ব", "”Q¡"), ("জ্জ্ব", "¾¡"),
    ("ত্ত্ব", "Ë¡"), ("ন্ত্ব", "š—¡"), ("ন্ত্র", "š¿"), ("ন্দ্ব", "›Ø"), ("ম্প্র", "¤cª"), ("ম্ভ্র", "¤£"),
    ("স্ত্র", "¯¿"), ("্র্য", "ª¨"),
    ("ক্ক", "°"), ("ক্ট", "±"), ("ক্ত", "³"), ("ক্ব", "K¡"), ("ক্ম", "´"), ("ক্র", "µ"), ("ক্ল", "K¬"),
    ("ক্ষ", "¶"), ("ক্স", "·"), ("গু", "¸"), ("গ্দ", "º"), ("গ্ধ", "»"), ("গ্ন", "Mœ"), ("গ্ম", "M¥"), ("গ্ল", "M­"),
    ("গ্রু", "Mªy"), ("ঙ্ক", "¼"), ("ঙ্খ", "•L"), ("ঙ্গ", "½"), ("ঙ্ঘ", "•N"), ("চ্চ", "”P"), ("চ্ছ", "”Q"),
    ("চ্ঞ", "”T"), ("জ্জ", "¾"), ("জ্ঝ", "À"), ("জ্ঞ", "Á"), ("জ্ব", "R¡"), ("ঞ্চ", "Â"), ("ঞ্ছ", "Ã"),
    ("ঞ্জ", "Ä"), ("ঞ্ঝ", "Å"), ("ট্ট", "Æ"), ("ট্ব", "U¡"), ("ট্ম", "U¥"), ("ড্ড", "Ç"), ("ণ্ট", "È"),
    ("ণ্ঠ", "É"), ("ণ্ড", "Ê"), ("ণ্ব", "Y^"), ("ত্ত", "Ë"), ("ত্থ", "Ì"), ("ত্ন", "Zœ"), ("ত্ম", "Z¥"),
    ("ত্ব", "Z¡"), ("ত্র", "Î"), ("থ্ব", "_¡"), ("দ্গ", "˜M"), ("দ্ঘ", "˜N"), ("দ্দ", "Ï"), ("দ্ধ", "×"),
    ("দ্ব", "Ø"), ("দ্ভ", "™¢"), ("দ্ম", "Ù"), ("দ্রু", "`ª“"), ("ধ্ব", "aŸ"), ("ধ্ম", "a¥"), ("ন্ট", "›U"),
    ("ন্ঠ", "Ú"), ("ন্ড", "Û"), ("ন্তু", "š‘"), ("ন্ত", "š—"), ("ন্থ", "š’"), ("ন্দ", "›`"), ("ন্ধ", "Ü"),
    ("ন্ন", "bœ"), ("ন্ব", "š^"), ("ন্ম", "b¥"), ("ন্স", "Ý"), ("প্ট", "Þ"), ("প্ত", "ß"), ("প্ন", "cœ"),
    ("প্প", "à"), ("প্ল", "c­"), ("প্স", "á"), ("ফ্ল", "d¬"), ("ব্জ", "â"), ("ব্দ", "ã"), ("ব্ধ", "ä"),
    ("ব্ব", "eŸ"), ("ব্ল", "e­"), ("ভ্র", "å"), ("ম্ন", "gœ"), ("ম্প", "¤ú"), ("ম্ফ", "ç"), ("ম্ব", "¤^"),
    ("ম্ভ", "¤¢"), ("ম্ম", "¤§"), ("ম্ল", "¤­"), ("রু", "i“"), ("রূ", "iƒ"), ("ল্ক", "é"), ("ল্গ", "ê"),
    ("ল্ট", "ë"), ("ল্ড", "ì"), ("ল্প", "í"), ("ল্ফ", "î"), ("ল্ব", "j¦"), ("ল্ম", "j¥"), ("ল্ল", "j­"),
    ("শু", "ï"), ("শ্চ", "ð"), ("শ্ছ", "ñ"), ("শ্ন", "kœ"), ("শ্ব", "k¦"), ("শ্ম", "k¥"), ("শ্ল", "k­"),
    ("ষ্ক", "®‹"), ("ষ্ট", "ó"), ("ষ্ঠ", "ô"), ("ষ্ণ", "ò"), ("ষ্প", "®ú"), ("ষ্ফ", "õ"), ("ষ্ম", "®§"),
    ("স্ক", "¯‹"), ("স্ট", "÷"), ("স্খ", "ö"), ("স্তু", "¯‘"), ("স্ত", "¯Í"), ("স্থ", "¯’"), ("স্ন", "mœ"),
    ("স্প", "¯ú"), ("স্ফ", "ù"), ("স্ব", "¯^"), ("স্ম", "¯§"), ("স্ল", "¯­"), ("হু", "û"), ("হৃ", "ü"),
    ("হ্ণ", "nè"), ("হ্ন", "ý"), ("হ্ম", "þ"), ("হ্ব", "nŸ"), ("হ্ল", "n¬"),
    // Generic phala forms after any consonant.
    ("্র", "ª"), ("্য", "¨"), ("্ব", "¡"), ("্ম", "¥"), ("্ন", "œ"), ("্ল", "­"),
];

fn u2a_char(c: char) -> Option<&'static str> {
    Some(match c {
        'অ' => "A", 'আ' => "Av", 'ই' => "B", 'ঈ' => "C", 'উ' => "D", 'ঊ' => "E", 'ঋ' => "F", 'এ' => "G",
        'ঐ' => "H", 'ও' => "I", 'ঔ' => "J",
        'ক' => "K", 'খ' => "L", 'গ' => "M", 'ঘ' => "N", 'ঙ' => "O", 'চ' => "P", 'ছ' => "Q", 'জ' => "R",
        'ঝ' => "S", 'ঞ' => "T", 'ট' => "U", 'ঠ' => "V", 'ড' => "W", 'ঢ' => "X", 'ণ' => "Y", 'ত' => "Z",
        'থ' => "_", 'দ' => "`", 'ধ' => "a", 'ন' => "b", 'প' => "c", 'ফ' => "d", 'ব' => "e", 'ভ' => "f",
        'ম' => "g", 'য' => "h", 'র' => "i", 'ল' => "j", 'শ' => "k", 'ষ' => "l", 'স' => "m", 'হ' => "n",
        '\u{09DC}' => "o", '\u{09DD}' => "p", '\u{09DF}' => "q", 'ৎ' => "r", 'ং' => "s", 'ঃ' => "t", 'ঁ' => "u",
        'া' => "v", 'ি' => "w", 'ী' => "x", 'ু' => "y", 'ূ' => "~", 'ৃ' => "…", 'ে' => "‡", 'ৈ' => "‰",
        'ো' => "‡v", 'ৌ' => "‡Š", 'ৗ' => "Š", '্' => "&", '।' => "|",
        '০' => "0", '১' => "1", '২' => "2", '৩' => "3", '৪' => "4", '৫' => "5", '৬' => "6", '৭' => "7",
        '৮' => "8", '৯' => "9",
        '‘' => "Ô", '’' => "Õ", '“' => "Ò", '”' => "Ó",
        _ => return None,
    })
}

fn is_kar(c: char) -> bool {
    matches!(c, 'া' | 'ি' | 'ী' | 'ু' | 'ূ' | 'ৃ' | 'ে' | 'ৈ' | 'ো' | 'ৌ' | 'ৗ')
}

/// Precompose the forms some apps produce decomposed.
fn normalize(s: &str) -> String {
    s.replace("\u{09C7}\u{09BE}", "\u{09CB}")
        .replace("\u{09C7}\u{09D7}", "\u{09CC}")
        .replace("\u{09A1}\u{09BC}", "\u{09DC}")
        .replace("\u{09A2}\u{09BC}", "\u{09DD}")
        .replace("\u{09AF}\u{09BC}", "\u{09DF}")
}

/// Greedy longest-match of a cluster string against the conjunct table.
fn greedy(s: &[char], out: &mut String) {
    let mut i = 0;
    while i < s.len() {
        let rest: String = s[i..].iter().collect();
        let best = U2A_CONJUNCTS
            .iter()
            .filter(|(u, _)| rest.starts_with(u))
            .max_by_key(|(u, _)| u.chars().count());
        if let Some((u, a)) = best {
            out.push_str(a);
            i += u.chars().count();
        } else {
            match u2a_char(s[i]) {
                Some(a) => out.push_str(a),
                None => out.push(s[i]),
            }
            i += 1;
        }
    }
}

/// Unicode Bangla → SutonnyMJ ANSI.
pub fn to_ansi(input: &str) -> String {
    let s: Vec<char> = normalize(input).chars().collect();
    let n = s.len();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < n {
        // র + ZWJ + ্য: ra with ya-phala, not reph.
        if s[i] == 'র' && s.get(i + 1) == Some(&'\u{200D}') && s.get(i + 2) == Some(&'্') && s.get(i + 3) == Some(&'য') {
            out.push_str("i¨");
            i += 4;
            continue;
        }
        let reph = s[i] == 'র' && s.get(i + 1) == Some(&'্') && s.get(i + 2).is_some_and(|c| is_consonant(*c));
        let start = if reph { i + 2 } else { i };
        if !is_consonant(s[start]) {
            match u2a_char(s[i]) {
                Some(a) => out.push_str(a),
                None => out.push(s[i]),
            }
            i += 1;
            continue;
        }
        let mut j = start + 1;
        while j + 1 < n && s[j] == '্' && is_consonant(s[j + 1]) {
            j += 2;
        }
        let kar = s.get(j).copied().filter(|c| is_kar(*c));
        let end = j + kar.is_some() as usize;
        let mut cluster = String::new();
        match kar {
            Some(k @ ('ি' | 'ে' | 'ৈ' | 'ো' | 'ৌ')) => {
                greedy(&s[start..j], &mut cluster);
                if reph {
                    cluster.push('©');
                }
                let (pre, post) = match k {
                    'ি' => ("w", ""),
                    'ে' => ("‡", ""),
                    'ৈ' => ("‰", ""),
                    'ো' => ("‡", "v"),
                    _ => ("‡", "Š"),
                };
                out.push_str(pre);
                out.push_str(&cluster);
                out.push_str(post);
            }
            Some(k) if reph => {
                greedy(&s[start..j], &mut cluster);
                out.push_str(&cluster);
                out.push('©');
                out.push_str(u2a_char(k).unwrap());
            }
            _ => {
                greedy(&s[start..end], &mut cluster);
                out.push_str(&cluster);
                if reph {
                    out.push('©');
                }
            }
        }
        i = end;
    }
    out
}

/// ANSI glyphs that are a full consonant (or conjunct) unit.
fn a2u_cons(c: char) -> Option<&'static str> {
    Some(match c {
        'K' => "ক", 'L' => "খ", 'M' => "গ", 'N' => "ঘ", 'O' => "ঙ", 'P' => "চ", 'Q' => "ছ", 'R' => "জ",
        'S' => "ঝ", 'T' => "ঞ", 'U' => "ট", 'V' => "ঠ", 'W' => "ড", 'X' => "ঢ", 'Y' => "ণ", 'Z' => "ত",
        '_' => "থ", '`' => "দ", 'a' => "ধ", 'b' => "ন", 'c' => "প", 'd' => "ফ", 'e' => "ব", 'f' => "ভ",
        'g' => "ম", 'h' => "য", 'i' => "র", 'j' => "ল", 'k' => "শ", 'l' => "ষ", 'm' => "স", 'n' => "হ",
        'o' => "\u{09DC}", 'p' => "\u{09DD}", 'q' => "\u{09DF}",
        '°' => "ক্ক", '±' => "ক্ট", '²' => "ক্ষ্ম", '³' => "ক্ত", '´' => "ক্ম", 'µ' => "ক্র", '¶' => "ক্ষ",
        '·' => "ক্স", '¸' => "গু", '¹' => "জ্ঞ", 'º' => "গ্দ", '»' => "গ্ধ", '¼' => "ঙ্ক", '½' => "ঙ্গ",
        '¾' => "জ্জ", 'À' => "জ্ঝ", 'Á' => "জ্ঞ", 'Â' => "ঞ্চ", 'Ã' => "ঞ্ছ", 'Ä' => "ঞ্জ", 'Å' => "ঞ্ঝ",
        'Æ' => "ট্ট", 'Ç' => "ড্ড", 'È' => "ণ্ট", 'É' => "ণ্ঠ", 'Ê' => "ণ্ড", 'Ë' => "ত্ত", 'Ì' => "ত্থ",
        'Î' => "ত্র", 'Ï' => "দ্দ", 'Ð' => "ণ্ড", '×' => "দ্ধ", 'Ø' => "দ্ব", 'Ù' => "দ্ম", 'Ú' => "ন্ঠ",
        'Û' => "ন্ড", 'Ü' => "ন্ধ", 'Ý' => "ন্স", 'Þ' => "প্ট", 'ß' => "প্ত", 'à' => "প্প", 'á' => "প্স",
        'â' => "ব্জ", 'ã' => "ব্দ", 'ä' => "ব্ধ", 'å' => "ভ্র", 'ç' => "ম্ফ", 'é' => "ল্ক", 'ê' => "ল্গ",
        'ë' => "ল্ট", 'ì' => "ল্ড", 'í' => "ল্প", 'î' => "ল্ফ", 'ï' => "শু", 'ð' => "শ্চ", 'ñ' => "শ্ছ",
        'ò' => "ষ্ণ", 'ó' => "ষ্ট", 'ô' => "ষ্ঠ", 'õ' => "ষ্ফ", 'ö' => "স্খ", '÷' => "স্ট", 'ø' => "স্ন",
        'ù' => "স্ফ", 'û' => "হু", 'ü' => "হৃ", 'ý' => "হ্ন", 'þ' => "হ্ম", 'ÿ' => "ক্ষ",
        _ => return None,
    })
}

/// Half forms that join the following consonant.
fn a2u_pre(c: char) -> Option<&'static str> {
    Some(match c {
        '®' => "ষ্", '¯' => "স্", '”' => "চ্", '˜' | '™' => "দ্", 'š' | '›' => "ন্", '¤' => "ম্", '•' => "ঙ্",
        _ => return None,
    })
}

/// Subscript / phala forms that attach to the preceding consonant.
fn a2u_post(c: char) -> Option<&'static str> {
    Some(match c {
        'ú' => "্প", 'è' | 'œ' => "্ন", '^' | 'Ÿ' | '¡' | '¦' => "্ব", '‘' => "্তু", '’' => "্থ", '‹' => "্ক",
        'Œ' => "্ক্র", '—' | 'Í' => "্ত", '¢' => "্ভ", '£' => "্ভ্র", '¥' | '§' => "্ম", '¨' => "্য",
        'ª' | '«' | 'Ö' => "্র", '¬' | '­' => "্ল", '¿' => "্ত্র",
        _ => return None,
    })
}

fn a2u_kar(c: char) -> Option<&'static str> {
    Some(match c {
        'v' => "া", 'w' => "ি", 'x' => "ী", 'y' | 'z' | 'æ' | '“' | '–' => "ু", '~' | 'ƒ' | '‚' => "ূ",
        '„' | '…' => "ৃ", '†' | '‡' => "ে", 'ˆ' | '‰' => "ৈ", 'Š' => "ৗ",
        _ => return None,
    })
}

fn a2u_other(c: char) -> Option<&'static str> {
    Some(match c {
        'A' => "অ", 'B' => "ই", 'C' => "ঈ", 'D' => "উ", 'E' => "ঊ", 'F' => "ঋ", 'G' => "এ", 'H' => "ঐ",
        'I' => "ও", 'J' => "ঔ", 'r' => "ৎ", 's' => "ং", 't' => "ঃ", 'u' => "ঁ", '&' => "্", '|' => "।",
        '©' => "র্", 'ª' => "্র", '¨' => "্য",
        '0' => "০", '1' => "১", '2' => "২", '3' => "৩", '4' => "৪", '5' => "৫", '6' => "৬", '7' => "৭",
        '8' => "৮", '9' => "৯",
        'Ô' => "‘", 'Õ' => "’", 'Ò' => "“", 'Ó' => "”", 'Ñ' => "–",
        _ => return None,
    })
}

fn starts_cluster(s: &[char], j: usize) -> bool {
    match s.get(j) {
        Some(&c) if a2u_cons(c).is_some() => true,
        Some(&c) if a2u_pre(c).is_some() => s.get(j + 1).is_some_and(|&d| a2u_cons(d).is_some() || a2u_post(d).is_some()),
        _ => false,
    }
}

/// Parse a consonant cluster starting at `j`. Returns the Unicode text and the next index.
fn parse_cluster(s: &[char], mut j: usize) -> (String, usize) {
    let mut out = String::new();
    while starts_cluster(s, j) {
        if let Some(pre) = a2u_pre(s[j]) {
            out.push_str(pre);
            j += 1;
            match a2u_post(s[j]) {
                Some(post) => out.push_str(post.trim_start_matches('্')),
                None => out.push_str(a2u_cons(s[j]).unwrap()),
            }
        } else {
            out.push_str(a2u_cons(s[j]).unwrap());
        }
        j += 1;
        while let Some(post) = s.get(j).and_then(|&c| a2u_post(c)) {
            out.push_str(post);
            j += 1;
        }
        if s.get(j) == Some(&'&') && starts_cluster(s, j + 1) {
            out.push('্');
            j += 1;
            continue;
        }
        break;
    }
    (out, j)
}

/// SutonnyMJ ANSI → Unicode Bangla.
pub fn to_unicode(input: &str) -> String {
    let s: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len() * 2);
    let mut i = 0;
    while i < s.len() {
        let c = s[i];
        let pre_kar = matches!(c, 'w' | '‡' | '†' | '‰' | 'ˆ');
        let cluster_at = if pre_kar { i + 1 } else { i };
        if (pre_kar || a2u_cons(c).is_some() || a2u_pre(c).is_some()) && starts_cluster(&s, cluster_at) {
            let (unit, mut j) = parse_cluster(&s, cluster_at);
            let reph = s.get(j) == Some(&'©');
            if reph {
                j += 1;
            }
            if reph {
                out.push_str("র্");
            }
            out.push_str(&unit);
            if pre_kar {
                let mut kar = a2u_kar(c).unwrap();
                if kar == "ে" {
                    match s.get(j) {
                        Some('v') => {
                            kar = "ো";
                            j += 1;
                        }
                        Some('Š') => {
                            kar = "ৌ";
                            j += 1;
                        }
                        _ => {}
                    }
                }
                out.push_str(kar);
            }
            i = j;
            continue;
        }
        match a2u_kar(c).or_else(|| a2u_other(c)) {
            Some(u) => out.push_str(u),
            None => out.push(c),
        }
        i += 1;
    }
    out.replace("অা", "আ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "আমার সোনার বাংলা, আমি তোমায় ভালোবাসি।";

    #[test]
    fn sample_to_ansi() {
        assert_eq!(to_ansi(SAMPLE), "Avgvi ‡mvbvi evsjv, Avwg ‡Zvgvq fv‡jvevwm|");
    }

    #[test]
    fn sample_round_trips() {
        assert_eq!(to_unicode(&to_ansi(SAMPLE)), SAMPLE);
    }

    #[test]
    fn conjuncts_and_reph() {
        let cases = [
            ("ক্ষমা", "¶gv"), ("স্কুল", "¯‹yj"), ("রাষ্ট্র", "ivóª"), ("কার্যক্রম", "Kvh©µg"),
            ("বিজ্ঞান", "weÁvb"), ("উজ্জ্বল", "D¾¡j"), ("সন্ধ্যা", "mÜ¨v"), ("শিক্ষক", "wk¶K"),
            ("চট্টগ্রাম", "PÆMªvg"), ("লক্ষ্মী", "j²x"), ("হৃদয়", "ü`q"), ("ক্রিকেট", "wµ‡KU"),
            ("প্রশ্ন", "cªkœ"), ("কৌশল", "‡KŠkj"), ("সূর্য", "m~h©"), ("গুরু", "¸i“"), ("র্কি", "wK©"),
        ];
        for (u, a) in cases {
            assert_eq!(to_ansi(u), a, "to_ansi({u})");
            assert_eq!(to_unicode(a), u, "to_unicode({a})");
        }
    }

    #[test]
    fn round_trips_a_vocabulary() {
        for w in crate::dictionary::WORDS {
            assert_eq!(to_unicode(&to_ansi(w)), *w, "round trip of {w} via {}", to_ansi(w));
        }
    }

    #[test]
    fn decodes_glyph_variants() {
        assert_eq!(to_unicode("¯Í"), "স্ত");
        assert_eq!(to_unicode("†K"), "কে");
        assert_eq!(to_unicode("K&l"), "ক্ষ");
        assert_eq!(to_unicode("Av"), "আ");
    }

    #[test]
    fn normalizes_decomposed_input() {
        assert_eq!(to_ansi("ক\u{09C7}\u{09BE}"), "‡Kv");
        assert_eq!(to_ansi("\u{09AF}\u{09BC}"), "q");
    }
}
