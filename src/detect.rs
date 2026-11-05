use crate::data::Lang;

/// Characters that strongly signal Spanish.
pub const ES_SPECIAL_CHARS: [char; 9] = ['ñ', '¿', '¡', 'á', 'é', 'í', 'ó', 'ú', 'ü'];

/// Top common Spanish words, as required by the specification.
pub const ES_STOPWORDS: [&str; 16] = [
    "el", "la", "de", "que", "y", "en", "un", "ser", "se", "no", "haber", "por", "con", "su",
    "para", "como",
];

/// Top common English words, as required by the specification.
pub const EN_STOPWORDS: [&str; 16] = [
    "the", "be", "to", "of", "and", "a", "in", "that", "have", "i", "it", "for", "not", "on",
    "with", "he",
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
    for word in lower.split(|c: char| !c.is_alphanumeric()) {
        if word.is_empty() {
            continue;
        }
        if ES_STOPWORDS.contains(&word) {
            es_score += 1;
        }
        if EN_STOPWORDS.contains(&word) {
            en_score += 1;
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
        let result = detect("hello world");
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
}
