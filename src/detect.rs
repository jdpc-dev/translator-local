use crate::data::{Lang, dictionary, normalize_key};

/// Characters that strongly signal Spanish.
pub const ES_SPECIAL_CHARS: [char; 9] = ['ñ', '¿', '¡', 'á', 'é', 'í', 'ó', 'ú', 'ü'];

/// Common Spanish function words. The first sixteen are the required core
/// list; the remainder improve confidence on ordinary prose.
pub const ES_STOPWORDS: &[&str] = &[
    "el", "la", "de", "que", "y", "en", "un", "ser", "se", "no", "haber", "por", "con", "su",
    "para", "como", "los", "las", "una", "unos", "unas", "del", "al", "es", "son", "está", "están",
    "este", "esta", "estos", "estas", "pero", "si", "o", "más", "muy", "ya", "hay", "nos", "me",
    "te", "le", "les", "lo", "sus", "nuestro", "nuestra", "porque", "cuando", "donde", "tiene",
    "tienen", "hace", "hacen",
];

/// Common English function words. The first sixteen are the required core
/// list; the remainder improve confidence on ordinary prose.
pub const EN_STOPWORDS: &[&str] = &[
    "the", "be", "to", "of", "and", "a", "in", "that", "have", "i", "it", "for", "not", "on",
    "with", "he", "she", "they", "we", "you", "his", "her", "their", "our", "this", "these",
    "those", "is", "are", "was", "were", "been", "do", "does", "did", "has", "had", "will",
    "would", "can", "could", "may", "might", "must", "should", "there", "what", "which", "who",
    "whom", "whose", "when", "where", "why", "how", "us", "them", "him", "my", "your", "its",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detection {
    pub language: Lang,
    pub ambiguous: bool,
    pub es_score: u32,
    pub en_score: u32,
}

/// Deterministic heuristic: Spanish-specific characters score 10, then each
/// occurrence of a stopword adds 1 to its language. Ties and empty signals
/// default to English and mark the result as ambiguous.
pub fn detect(text: &str) -> Detection {
    let lower = text.to_lowercase();

    let mut es_score = 0;
    if lower.chars().any(|c| ES_SPECIAL_CHARS.contains(&c)) {
        es_score += 10;
    }

    let mut en_score = 0;
    let mut en_dictionary_hits = 0;
    let mut infinitive_words: Vec<&str> = Vec::new();
    let mut ar_ir_count = 0;
    let dictionary = dictionary();

    for word in lower.split(|c: char| !c.is_alphanumeric()) {
        if word.is_empty() {
            continue;
        }

        let is_stopword = ES_STOPWORDS.contains(&word) || EN_STOPWORDS.contains(&word);
        if ES_STOPWORDS.contains(&word) {
            es_score += 1;
        }
        if EN_STOPWORDS.contains(&word) {
            en_score += 1;
        }

        if is_stopword || word.len() < 3 {
            continue;
        }

        let key = normalize_key(word);
        let in_es = dictionary.contains(Lang::Es, &key);
        let in_en = dictionary.contains(Lang::En, &key);

        match (in_es, in_en) {
            (true, false) => es_score += 1,
            (false, true) => {
                en_score += 1;
                en_dictionary_hits += 1;
            }
            _ => {}
        }

        // Spanish infinitives are a useful signal when several distinct forms
        // appear with no English dictionary word present. Requiring at least
        // two -ar/-ir endings keeps lists of English -er agent nouns from
        // being mistaken for Spanish.
        if !in_en
            && !in_es
            && word.len() >= 4
            && (word.ends_with("ar") || word.ends_with("er") || word.ends_with("ir"))
            && !infinitive_words.contains(&word)
        {
            if word.ends_with("ar") || word.ends_with("ir") {
                ar_ir_count += 1;
            }
            infinitive_words.push(word);
        }
    }

    if infinitive_words.len() >= 3 && ar_ir_count >= 2 && en_dictionary_hits == 0 {
        es_score += 2;
    }

    // A full-input phrase match is a stronger signal than isolated words,
    // especially for short phrases such as "a veces".
    let normalized_text =
        normalize_key(text.trim_matches(|c: char| !c.is_alphanumeric() && !c.is_whitespace()));
    if normalized_text.contains(' ') {
        if dictionary.lookup(Lang::Es, &normalized_text).is_some() {
            es_score += 3;
        }
        if dictionary.lookup(Lang::En, &normalized_text).is_some() {
            en_score += 3;
        }
    }

    let language = if es_score > en_score {
        Lang::Es
    } else {
        Lang::En
    };

    Detection {
        language,
        ambiguous: es_score == en_score,
        es_score,
        en_score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_spanish_with_accents_and_stopwords() {
        let result = detect("El gato rápido salta.");
        assert_eq!(result.language, Lang::Es);
        assert!(!result.ambiguous);
        assert!(result.es_score >= 11);
    }

    #[test]
    fn detects_english_with_stopwords() {
        let result = detect("The cat is in the house");
        assert_eq!(result.language, Lang::En);
        assert!(!result.ambiguous);
    }

    #[test]
    fn defaults_to_english_when_there_are_no_signals() {
        let result = detect("zorblax blorp");
        assert_eq!(result.language, Lang::En);
        assert!(result.ambiguous);
        assert_eq!(result.es_score, 0);
        assert_eq!(result.en_score, 0);
    }

    #[test]
    fn defaults_to_english_on_a_tie() {
        let result = detect("el the");
        assert_eq!(result.language, Lang::En);
        assert!(result.ambiguous);
        assert_eq!(result.es_score, result.en_score);
    }

    #[test]
    fn spanish_punctuation_alone_is_enough() {
        let result = detect("¿Dónde?");
        assert_eq!(result.language, Lang::Es);
    }

    #[test]
    fn detects_spanish_prose_without_accents() {
        let result = detect("el gobierno nos miente y el dinero nos manipula");
        assert_eq!(result.language, Lang::Es);
        assert!(!result.ambiguous);
    }

    #[test]
    fn detects_english_prose_without_accents() {
        let result = detect("the government lies to us and the money manipulates us");
        assert_eq!(result.language, Lang::En);
        assert!(!result.ambiguous);
    }

    #[test]
    fn detects_spanish_infinitive_lists() {
        let result = detect("alimentar, insertar, cortar, apuntar, disparar, repetir");
        assert_eq!(result.language, Lang::Es);
        assert!(!result.ambiguous);
    }

    #[test]
    fn does_not_mistake_english_er_words_for_spanish() {
        let result = detect("summer winter computer teacher");
        assert_eq!(result.language, Lang::En);
        assert!(!result.ambiguous);
    }

    #[test]
    fn does_not_mistake_oov_english_agent_nouns_for_infinitives() {
        for text in [
            "renderer compiler linker",
            "docker container builder",
            "buffer handler logger",
            "bicker bicker bicker",
        ] {
            let result = detect(text);
            assert_eq!(result.language, Lang::En, "text: {text}");
        }
    }

    #[test]
    fn detects_short_spanish_phrase_from_the_dictionary() {
        let result = detect("a veces");
        assert_eq!(result.language, Lang::Es);
        assert!(!result.ambiguous);
    }
}
