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
        let mut max_phrase_words = 1;

        for pair in file.pairs {
            max_phrase_words = max_phrase_words
                .max(pair.en.split_whitespace().count())
                .max(pair.es.split_whitespace().count());

            // Several English words can share a Spanish translation (fast/quick ->
            // rápido). Keep the first curated mapping so reverse lookups stay
            // deterministic.
            en_to_es
                .entry(normalize_key(&pair.en))
                .or_insert_with(|| pair.es.clone());
            es_to_en.entry(normalize_key(&pair.es)).or_insert(pair.en);
        }

        // Explicit reverse pairs cover Spanish inflections and object pronouns
        // that cannot be represented by the forward EN-ES pair list. They take
        // precedence over reverse lookups derived from the forward pairs.
        for pair in reverse.pairs {
            max_phrase_words = max_phrase_words.max(pair.es.split_whitespace().count());
            es_to_en.insert(normalize_key(&pair.es), pair.en);
        }

        Self {
            en_to_es,
            es_to_en,
            max_phrase_words,
        }
    }

    pub fn lookup(&self, source: Lang, key: &str) -> Option<&str> {
        let table = match source {
            Lang::En => &self.en_to_es,
            Lang::Es => &self.es_to_en,
        };
        table.get(key).map(String::as_str)
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
            .get(&word.to_lowercase())
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

fn lowercase_keys(map: HashMap<String, Vec<String>>) -> HashMap<String, Vec<String>> {
    map.into_iter()
        .map(|(key, value)| (key.to_lowercase(), value))
        .collect()
}

pub fn normalize_key(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

static DICTIONARY: OnceLock<Dictionary> = OnceLock::new();
static SYNONYMS: OnceLock<Synonyms> = OnceLock::new();

pub fn dictionary() -> &'static Dictionary {
    DICTIONARY.get_or_init(Dictionary::load)
}

pub fn synonyms() -> &'static Synonyms {
    SYNONYMS.get_or_init(Synonyms::load)
}
