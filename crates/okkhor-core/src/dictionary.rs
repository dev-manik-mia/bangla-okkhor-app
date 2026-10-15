//! Word suggestions: a built-in list of common words plus words the user
//! has typed, ranked by how often they were used.

use std::collections::HashMap;

pub const WORDS: &[&str] = &[
    "অক্ষর", "অনেক", "অন্য", "অবশ্য", "অবস্থা", "অভিযোগ", "অর্থ", "অর্থনীতি", "অধিকার", "অনুষ্ঠান", "অনুযায়ী",
    "অফিস", "অপেক্ষা", "অসুখ", "আকাশ", "আগে", "আগামী", "আজ", "আজকে", "আদালত", "আনন্দ", "আপনি", "আপনার",
    "আবার", "আমি", "আমার", "আমরা", "আমাদের", "আম", "আলো", "আলোচনা", "আসা", "আছে", "আর", "আরও", "আইন",
    "ইতিহাস", "ইচ্ছা", "ইংরেজি", "উত্তর", "উন্নয়ন", "উপর", "উপায়", "উদ্যোগ", "ঋতু", "এই", "এক", "একটি",
    "একটু", "এখন", "এখানে", "এবং", "এবার", "এমন", "ঐতিহ্য", "ও", "ওই", "ওদের", "ঔষধ",
    "কবিতা", "কথা", "কাজ", "কলম", "কখন", "কখনো", "কত", "কম", "করা", "করে", "করতে", "কর্মী", "কারণ",
    "কার্যক্রম", "কিছু", "কিন্তু", "কী", "কে", "কেন", "কোথায়", "কোন", "কোনো", "ক্ষমতা", "ক্লাস", "কৃষক",
    "কৃষি", "খবর", "খাবার", "খুব", "খেলা", "খুশি", "গান", "গাছ", "গ্রাম", "গল্প", "গত", "গুরুত্বপূর্ণ",
    "গবেষণা", "ঘর", "ঘটনা", "ঘুম", "চলে", "চাই", "চিঠি", "চোখ", "চেষ্টা", "চিন্তা", "ছবি", "ছাত্র", "ছাত্রী",
    "ছেলে", "ছোট", "জন্য", "জল", "জানা", "জানি", "জাতীয়", "জীবন", "জিনিস", "জায়গা", "জ্ঞান", "ঝড়",
    "টাকা", "ঠিক", "ডাক্তার", "ঢাকা", "তবে", "তখন", "তাই", "তার", "তারা", "তাদের", "তুমি", "তোমার",
    "তোমায়", "তোমরা", "থাকা", "থেকে", "দেশ", "দিন", "দিকে", "দুই", "দেখা", "দেওয়া", "দরকার", "দল", "দাম",
    "দায়িত্ব", "দ্রুত", "ধন্যবাদ", "ধরে", "ধর্ম", "নদী", "নতুন", "নাম", "নিয়ে", "নিজের", "নির্বাচন",
    "নেতা", "নারী", "পাখি", "পানি", "পরে", "পর্যন্ত", "পরিবার", "পরিবেশ", "পড়া", "পড়াশোনা", "পাওয়া",
    "পুলিশ", "পৃথিবী", "প্রতি", "প্রথম", "প্রশ্ন", "প্রয়োজন", "প্রধান", "প্রকাশ", "প্রতিষ্ঠান", "প্রিয়",
    "ফল", "ফুল", "ফিরে", "বই", "বছর", "বন্ধু", "বলা", "বলে", "বাংলা", "বাংলাদেশ", "বাড়ি", "বাবা",
    "বাজার", "বিষয়", "বিভিন্ন", "বিশ্ব", "বিশ্ববিদ্যালয়", "বিজ্ঞান", "বৃষ্টি", "বেশি", "ব্যবহার", "ব্যবসা",
    "ব্যাংক", "ভাই", "ভাষা", "ভালো", "ভালোবাসা", "ভালোবাসি", "ভাবে", "ভোট", "মা", "মানুষ", "মনে", "মধ্যে",
    "মন্ত্রী", "মাঠ", "মাধ্যমে", "মাস", "মেয়ে", "মোট", "যখন", "যদি", "যা", "যাওয়া", "যে", "যেমন",
    "রাস্তা", "রাজনীতি", "রাত", "রাষ্ট্র", "রয়েছে", "লেখা", "লিখি", "লোক", "শব্দ", "শহর", "শিক্ষক",
    "শিক্ষা", "শিশু", "শুধু", "শুরু", "শেষ", "সকাল", "সঙ্গে", "সব", "সবাই", "সময়", "সমস্যা", "সরকার",
    "সাথে", "সাধারণ", "সুন্দর", "সূর্য", "সেই", "সোনার", "স্কুল", "স্বাধীনতা", "স্বাস্থ্য", "সংবাদ",
    "সংস্কৃতি", "হয়", "হয়েছে", "হাত", "হাজার", "হিসেবে", "হৃদয়", "হবে",
];

#[derive(Debug, Clone, Default)]
pub struct Dictionary {
    /// Use counts for words, including user-learned ones.
    counts: HashMap<String, u32>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Restore learned counts (e.g. from a saved file).
    pub fn with_learned(counts: HashMap<String, u32>) -> Self {
        Dictionary { counts }
    }

    pub fn learned(&self) -> &HashMap<String, u32> {
        &self.counts
    }

    /// Record a completed word. Only Bangla words of 2+ letters are kept.
    pub fn learn(&mut self, word: &str) {
        let n = word.chars().count();
        if n < 2 || !word.chars().all(|c| ('\u{0980}'..='\u{09FF}').contains(&c)) {
            return;
        }
        *self.counts.entry(word.to_string()).or_default() += 1;
    }

    /// Up to `limit` words starting with `prefix`, most used first.
    pub fn suggest(&self, prefix: &str, limit: usize) -> Vec<String> {
        if prefix.is_empty() {
            return Vec::new();
        }
        let mut seen = std::collections::HashSet::new();
        let mut hits: Vec<(u32, usize, &str)> = WORDS
            .iter()
            .enumerate()
            .map(|(i, w)| (*w, i))
            .chain(self.counts.keys().map(|w| (w.as_str(), usize::MAX)))
            .filter(|(w, _)| w.starts_with(prefix) && *w != prefix && seen.insert(*w))
            .map(|(w, i)| (self.counts.get(w).copied().unwrap_or(0), i, w))
            .collect();
        hits.sort_by(|a, b| b.0.cmp(&a.0).then(a.2.chars().count().cmp(&b.2.chars().count())).then(a.1.cmp(&b.1)));
        hits.into_iter().take(limit).map(|(_, _, w)| w.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_matches() {
        let d = Dictionary::new();
        let s = d.suggest("বাং", 4);
        assert_eq!(s, vec!["বাংলা", "বাংলাদেশ"]);
        assert!(d.suggest("", 4).is_empty());
    }

    #[test]
    fn learned_words_rank_first() {
        let mut d = Dictionary::new();
        d.learn("আমাদের");
        d.learn("আমাদের");
        d.learn("আমড়া");
        assert_eq!(d.suggest("আম", 4)[0], "আমাদের");
        assert!(d.suggest("আম", 10).contains(&"আমড়া".to_string()));
        d.learn("x");
        assert!(!d.learned().contains_key("x"));
    }
}
