use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

const DICTIONARY_JSON: &str = include_str!("../data/dictionary.json");
const REVERSE_JSON: &str = include_str!("../data/reverse.json");
const SYNONYMS_JSON: &str = include_str!("../data/synonyms.json");

/// The two languages supported by the tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Es,
}

impl Lang {
    pub const fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Es => "es",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Es => "Spanish",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_ascii_lowercase().as_str() {
            "en" => Some(Self::En),
            "es" => Some(Self::Es),
            _ => None,
        }
    }

    pub const fn opposite(self) -> Self {
        match self {
            Self::En => Self::Es,
            Self::Es => Self::En,
        }
    }
}

#[derive(Debug, Deserialize)]
struct DictionaryFile {
    #[allow(dead_code)]
    version: u32,
    pairs: Vec<Pair>,
}

#[derive(Debug, Deserialize)]
struct Pair {
    en: String,
    es: String,
}

#[derive(Debug, Deserialize)]
struct ReverseFile {
    #[allow(dead_code)]
    version: u32,
    pairs: Vec<ReversePair>,
}

#[derive(Debug, Deserialize)]
struct ReversePair {
    es: String,
    en: String,
}

#[derive(Debug, Deserialize)]
struct SynonymsFile {
    #[allow(dead_code)]
    version: u32,
    en: HashMap<String, Vec<String>>,
    es: HashMap<String, Vec<String>>,
}

/// Bidirectional lookup tables loaded from the embedded JSON dataset.
pub struct Dictionary {
    en_to_es: HashMap<String, String>,
    es_to_en: HashMap<String, String>,
    en_to_es_folded: HashMap<String, String>,
    es_to_en_folded: HashMap<String, String>,
    max_phrase_words: usize,
}

impl Dictionary {
    fn load() -> Self {
        let file: DictionaryFile =
            serde_json::from_str(DICTIONARY_JSON).expect("embedded dictionary.json is valid");
        let reverse: ReverseFile =
            serde_json::from_str(REVERSE_JSON).expect("embedded reverse.json is valid");

        let mut en_to_es = HashMap::new();
        let mut es_to_en = HashMap::new();
        let mut en_keys = Vec::new();
        let mut es_keys = Vec::new();
        let mut max_phrase_words = 1;

        for pair in file.pairs {
            max_phrase_words = max_phrase_words
                .max(pair.en.split_whitespace().count())
                .max(pair.es.split_whitespace().count());

            let en_key = normalize_key(&pair.en);
            let es_key = normalize_key(&pair.es);
            en_keys.push(en_key.clone());
            es_keys.push(es_key.clone());

            // Several English words can share a Spanish translation (fast/quick ->
            // rápido). Keep the first curated mapping so reverse lookups stay
            // deterministic.
            en_to_es.entry(en_key).or_insert(pair.es);
            es_to_en.entry(es_key).or_insert(pair.en);
        }

        // Explicit reverse pairs cover Spanish inflections and object pronouns
        // that cannot be represented by the forward EN-ES pair list. They take
        // precedence over reverse lookups derived from the forward pairs.
        for pair in reverse.pairs {
            max_phrase_words = max_phrase_words.max(pair.es.split_whitespace().count());
            let es_key = normalize_key(&pair.es);
            es_keys.push(es_key.clone());
            es_to_en.insert(es_key, pair.en);
        }

        let en_to_es_folded = folded_aliases(&en_keys, &en_to_es);
        let es_to_en_folded = folded_aliases(&es_keys, &es_to_en);

        Self {
            en_to_es,
            es_to_en,
            en_to_es_folded,
            es_to_en_folded,
            max_phrase_words,
        }
    }

    /// Look up an exact key without consulting accent-folded aliases.
    pub fn lookup_exact(&self, source: Lang, key: &str) -> Option<&str> {
        let table = match source {
            Lang::En => &self.en_to_es,
            Lang::Es => &self.es_to_en,
        };
        table.get(key).map(String::as_str)
    }

    pub fn lookup(&self, source: Lang, key: &str) -> Option<&str> {
        if let Some(value) = self.lookup_exact(source, key) {
            return Some(value);
        }

        let folded = match source {
            Lang::En => &self.en_to_es_folded,
            Lang::Es => &self.es_to_en_folded,
        };

        // Folded aliases are ASCII, so try the key directly first.
        if let Some(value) = folded.get(key) {
            return Some(value);
        }

        let folded_key = fold_accents(key);
        if folded_key.as_str() == key {
            return None;
        }

        folded.get(&folded_key).map(String::as_str)
    }

    /// Return whether `key` resolves in the given source language.
    pub fn contains(&self, source: Lang, key: &str) -> bool {
        self.lookup(source, key).is_some()
    }

    pub fn max_phrase_words(&self) -> usize {
        self.max_phrase_words
    }
}

/// Synonyms grouped by target language.
pub struct Synonyms {
    en: HashMap<String, Vec<String>>,
    es: HashMap<String, Vec<String>>,
}

impl Synonyms {
    fn load() -> Self {
        let file: SynonymsFile =
            serde_json::from_str(SYNONYMS_JSON).expect("embedded synonyms.json is valid");
        Self {
            en: lowercase_keys(file.en),
            es: lowercase_keys(file.es),
        }
    }

    pub fn get(&self, lang: Lang, word: &str) -> &[String] {
        let table = match lang {
            Lang::En => &self.en,
            Lang::Es => &self.es,
        };
        table
            .get(&normalize_key(word))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

fn lowercase_keys(map: HashMap<String, Vec<String>>) -> HashMap<String, Vec<String>> {
    map.into_iter()
        .map(|(key, value)| (normalize_key(&key), value))
        .collect()
}

pub fn normalize_key(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
        .replace(['’', '‘'], "'")
}

/// Fold Spanish accents so unaccented input can fall back to accented entries.
fn fold_accents(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' | 'ã' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            other => other,
        })
        .collect()
}

/// Build unaccented aliases in dataset order. Exact keys always win, and the
/// first alias for a folded form is kept so lookups stay deterministic.
fn folded_aliases(keys: &[String], table: &HashMap<String, String>) -> HashMap<String, String> {
    let mut aliases = HashMap::new();

    for key in keys {
        // Short accented words (`año`, `sí`, `más`) collide too easily with
        // unrelated unaccented words (`ano`, `si`, `mas`).
        if key.chars().count() < 4 {
            continue;
        }

        let folded = fold_accents(key);
        if folded.as_str() == key.as_str() || table.contains_key(&folded) {
            continue;
        }

        if let Some(value) = table.get(key) {
            aliases.entry(folded).or_insert_with(|| value.clone());
        }
    }

    aliases
}

static DICTIONARY: OnceLock<Dictionary> = OnceLock::new();
static SYNONYMS: OnceLock<Synonyms> = OnceLock::new();

pub fn dictionary() -> &'static Dictionary {
    DICTIONARY.get_or_init(Dictionary::load)
}

pub fn synonyms() -> &'static Synonyms {
    SYNONYMS.get_or_init(Synonyms::load)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_typographic_apostrophes() {
        assert_eq!(normalize_key("Don’t  worry"), "don't worry");
        assert_eq!(normalize_key("I‘m sorry"), "i'm sorry");
    }

    #[test]
    fn folds_spanish_accents() {
        assert_eq!(
            fold_accents("rápido, mañana, pingüino"),
            "rapido, manana, pinguino"
        );
    }

    #[test]
    fn lookup_accepts_unaccented_spanish() {
        let dictionary = dictionary();
        assert_eq!(dictionary.lookup(Lang::Es, "rápido"), Some("fast"));
        assert_eq!(dictionary.lookup(Lang::Es, "rapido"), Some("fast"));
        assert_eq!(
            dictionary.lookup(Lang::Es, "la manana"),
            Some("the morning")
        );
    }

    #[test]
    fn does_not_fold_short_words_into_aliases() {
        let mut table = HashMap::new();
        table.insert("año".to_string(), "year".to_string());
        let keys = vec!["año".to_string()];

        assert!(folded_aliases(&keys, &table).is_empty());
    }

    #[test]
    fn exact_lookup_ignores_accent_folded_aliases() {
        let dictionary = dictionary();
        assert_eq!(dictionary.lookup_exact(Lang::Es, "rapido"), None);
        assert_eq!(dictionary.lookup(Lang::Es, "rapido"), Some("fast"));
    }
}
