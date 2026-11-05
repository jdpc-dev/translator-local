use crate::data::{Dictionary, Lang, dictionary, normalize_key};
use crate::tokenize::{Token, apply_case, tokenize};
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OovMode {
    /// Pass unknown words through unchanged.
    Keep,
    /// Wrap unknown words in square brackets.
    Bracket,
}

#[derive(Debug)]
struct WordRef {
    token_index: usize,
    text: String,
    lower: String,
    code_like: bool,
}

/// Translate `text` from `source` to the opposite language.
///
/// Matching is greedy longest-first over dictionary phrases, falling back to
/// single words. Original whitespace, punctuation, and capitalization are
/// preserved. Unknown words are either kept or bracketed.
pub fn translate(text: &str, source: Lang, oov: OovMode) -> String {
    let tokens = tokenize(text);
    let words: Vec<WordRef> = tokens
        .iter()
        .enumerate()
        .filter_map(|(token_index, token)| match token {
            Token::Word(word) => Some(WordRef {
                token_index,
                text: word.clone(),
                lower: normalize_key(word),
                code_like: is_code_like(&tokens, token_index, word),
            }),
            _ => None,
        })
        .collect();

    let dictionary = dictionary();
    let max_phrase_words = dictionary.max_phrase_words();
    let mut replaced: Vec<Option<String>> = vec![None; tokens.len()];
    let mut index = 0;

    while index < words.len() {
        if words[index].code_like {
            replaced[words[index].token_index] = Some(words[index].text.clone());
            index += 1;
            continue;
        }

        let max_length = max_phrase_words.min(words.len() - index);
        let mut chosen: Option<(usize, &str)> = None;

        for length in (1..=max_length).rev() {
            if words[index..index + length]
                .iter()
                .any(|word| word.code_like)
            {
                continue;
            }

            if length > 1 {
                let start = words[index].token_index;
                let end = words[index + length - 1].token_index;
                // A phrase may only span words and whitespace; punctuation is a
                // boundary.
                if tokens[start + 1..end]
                    .iter()
                    .any(|token| matches!(token, Token::Punct(_)))
                {
                    continue;
                }
            }

            let key = words[index..index + length]
                .iter()
                .map(|word| word.lower.as_str())
                .collect::<Vec<_>>()
                .join(" ");

            let translation = lookup_word(dictionary, source, &key, length == 1);
            if let Some(translation) = translation {
                chosen = Some((length, translation));
                break;
            }
        }

        match chosen {
            Some((length, translation)) => {
                let source_word = &words[index].text;
                let sentence_start = is_sentence_start(&tokens, words[index].token_index);
                let replacement = if is_english_pronoun_i(source_word) && !sentence_start {
                    translation.to_string()
                } else {
                    apply_case(translation, source_word)
                };
                let start = words[index].token_index;
                let end = words[index + length - 1].token_index;

                // Consume the whole span, including whitespace between its
                // words, then place the translation at the first token.
                for slot in &mut replaced[start..=end] {
                    *slot = Some(String::new());
                }
                replaced[start] = Some(replacement);
                index += length;
            }
            None => {
                let original = &words[index].text;
                let replacement = match oov {
                    OovMode::Keep => original.clone(),
                    OovMode::Bracket => format!("[{original}]"),
                };
                replaced[words[index].token_index] = Some(replacement);
                index += 1;
            }
        }
    }

    let mut output = String::with_capacity(text.len());
    for (index, token) in tokens.iter().enumerate() {
        match &replaced[index] {
            Some(piece) => output.push_str(piece),
            None => match token {
                // Inverted Spanish punctuation has no place in English output;
                // the closing mark is preserved.
                Token::Punct(punct) if source == Lang::Es => {
                    output.extend(punct.chars().filter(|c| !matches!(c, '¿' | '¡')));
                }
                _ => output.push_str(token.text()),
            },
        }
    }
    output
}

fn is_code_like(tokens: &[Token], token_index: usize, word: &str) -> bool {
    if word.contains('_') || word.chars().any(|c| c.is_ascii_digit()) {
        return true;
    }
    if has_internal_uppercase(word) {
        return true;
    }

    let previous = token_index
        .checked_sub(1)
        .and_then(|index| tokens.get(index));
    let next = tokens.get(token_index + 1);
    if is_code_punctuation(previous) || is_code_punctuation(next) {
        return true;
    }

    // Keep dotted identifiers such as `user.name` and `README.md` together.
    // A dot followed by whitespace or the end of the input is sentence
    // punctuation, so ordinary prose still translates.
    is_dotted_identifier(tokens, token_index)
}

fn has_internal_uppercase(word: &str) -> bool {
    let mut previous_lower = false;
    for c in word.chars() {
        if previous_lower && c.is_uppercase() {
            return true;
        }
        previous_lower = c.is_lowercase();
    }
    false
}

fn is_code_punctuation(token: Option<&Token>) -> bool {
    matches!(token, Some(Token::Punct(text))
        if text.chars().any(|c| matches!(c, '/' | '\\' | '@' | '$' | '_' | '-'))
            || text.chars().any(|c| c.is_ascii_digit()))
}

fn is_dotted_identifier(tokens: &[Token], token_index: usize) -> bool {
    let after_dot = token_index
        .checked_sub(1)
        .and_then(|index| tokens.get(index))
        .is_some_and(|token| matches!(token, Token::Punct(text) if text == "."));

    let before_dot = matches!(tokens.get(token_index + 1), Some(Token::Punct(text)) if text == ".")
        && matches!(tokens.get(token_index + 2), Some(Token::Word(_)));

    after_dot || before_dot
}

/// Look up a word, retrying inflected forms when the exact key is absent.
///
/// English plurals use the existing `-ies`, `-ces`, `-es`, and `-s` rules.
/// Spanish present-tense forms are tried before accent-folded aliases, so
/// `hablo` (I speak) is not confused with the preterite `habló` (he spoke).
/// Spanish plural reduction runs after the folded lookup so accent-folded
/// plural reverse entries such as `paises` → `countries` still win.
fn lookup_word<'a>(
    dictionary: &'a Dictionary,
    source: Lang,
    word: &str,
    allow_inflection: bool,
) -> Option<&'a str> {
    if !allow_inflection {
        return dictionary.lookup(source, word);
    }

    // Exact entries always win.
    if let Some(found) = dictionary.lookup_exact(source, word) {
        return Some(found);
    }

    if source == Lang::Es {
        let mut verb_candidates = Vec::new();
        spanish_verb_candidates(word, &mut verb_candidates);
        for candidate in verb_candidates {
            if let Some(found) = dictionary.lookup(source, &candidate) {
                return Some(found);
            }
        }
    }

    // Accent-folded aliases for the original word.
    if let Some(found) = dictionary.lookup(source, word) {
        return Some(found);
    }

    // Remaining plural/singular fallbacks.
    let mut candidates = Vec::new();
    match source {
        Lang::En => english_singular_candidates(word, &mut candidates),
        Lang::Es => spanish_plural_candidates(word, &mut candidates),
    }
    candidates
        .iter()
        .find_map(|candidate| dictionary.lookup(source, candidate))
}

fn push_candidate(candidates: &mut Vec<String>, word: &str, candidate: impl Into<String>) {
    let candidate = candidate.into();
    if !candidate.is_empty() && candidate != word && !candidates.contains(&candidate) {
        candidates.push(candidate);
    }
}

fn english_singular_candidates(word: &str, candidates: &mut Vec<String>) {
    if let Some(stem) = word.strip_suffix("ies") {
        push_candidate(candidates, word, format!("{stem}y"));
    }
    if let Some(stem) = word.strip_suffix("ces") {
        push_candidate(candidates, word, format!("{stem}z"));
    }
    if let Some(stem) = word.strip_suffix("es") {
        push_candidate(candidates, word, stem);
    }
    if !word.ends_with("ss")
        && let Some(stem) = word.strip_suffix('s')
    {
        push_candidate(candidates, word, stem);
    }
}

fn spanish_verb_candidates(word: &str, candidates: &mut Vec<String>) {
    // Present-tense endings mapped to the infinitive endings they imply.
    let endings: &[(&str, &[&str])] = &[
        ("amos", &["ar"]),
        ("emos", &["er"]),
        ("imos", &["ir"]),
        ("an", &["ar"]),
        ("en", &["er", "ir"]),
        ("as", &["ar"]),
        ("es", &["er", "ir"]),
        ("a", &["ar"]),
        ("e", &["er", "ir"]),
        ("o", &["ar", "er", "ir"]),
    ];

    for &(ending, infinitives) in endings {
        let Some(stem) = word.strip_suffix(ending) else {
            continue;
        };
        if stem.chars().count() < 2 {
            continue;
        }
        for &infinitive in infinitives {
            push_candidate(candidates, word, format!("{stem}{infinitive}"));
        }
    }
}

fn spanish_plural_candidates(word: &str, candidates: &mut Vec<String>) {
    if let Some(stem) = word.strip_suffix("ces") {
        push_candidate(candidates, word, format!("{stem}z"));
    }

    if let Some(stem) = word.strip_suffix("es")
        && stem
            .chars()
            .next_back()
            .is_some_and(|c| !is_spanish_vowel(c))
    {
        push_candidate(candidates, word, stem);
    }

    if !word.ends_with("ss")
        && let Some(stem) = word.strip_suffix('s')
        && stem.chars().next_back().is_some_and(is_spanish_vowel)
    {
        push_candidate(candidates, word, stem);
    }
}

fn is_spanish_vowel(c: char) -> bool {
    matches!(
        c,
        'a' | 'e' | 'i' | 'o' | 'u' | 'á' | 'é' | 'í' | 'ó' | 'ú' | 'ü'
    )
}

fn is_sentence_start(tokens: &[Token], token_index: usize) -> bool {
    for token in tokens[..token_index].iter().rev() {
        match token {
            Token::Space(_) => continue,
            Token::Punct(text) => {
                return text.ends_with(['.', '!', '?', '…']);
            }
            Token::Word(_) => return false,
        }
    }
    true
}

/// English `I` is always capitalized, but its Spanish translation is not
/// capitalized mid-sentence.
fn is_english_pronoun_i(text: &str) -> bool {
    let normalized = text.replace('’', "'");
    matches!(normalized.as_str(), "I" | "I'm" | "I've" | "I'll" | "I'd")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_spanish_to_english() {
        assert_eq!(
            translate("El gato rápido salta.", Lang::Es, OovMode::Keep),
            "The fast cat jumps."
        );
        assert_eq!(
            translate("el gato rápido salta", Lang::Es, OovMode::Keep),
            "the fast cat jumps"
        );
    }

    #[test]
    fn translates_english_to_spanish() {
        assert_eq!(
            translate("The fast cat jumps", Lang::En, OovMode::Keep),
            "El gato rápido salta"
        );
    }

    #[test]
    fn prefers_the_longest_phrase() {
        assert_eq!(
            translate("good morning", Lang::En, OovMode::Keep),
            "buenos días"
        );
    }

    #[test]
    fn does_not_match_a_phrase_across_punctuation() {
        assert_eq!(
            translate("good, morning", Lang::En, OovMode::Keep),
            "bueno, mañana"
        );
    }

    #[test]
    fn handles_idiomatic_phrase_entries() {
        assert_eq!(
            translate("I am cold", Lang::En, OovMode::Keep),
            "Tengo frío"
        );
    }

    #[test]
    fn preserves_case() {
        assert_eq!(
            translate("Good morning", Lang::En, OovMode::Keep),
            "Buenos días"
        );
        assert_eq!(
            translate("GOOD MORNING", Lang::En, OovMode::Keep),
            "BUENOS DÍAS"
        );
    }

    #[test]
    fn preserves_punctuation_and_adjacent_spacing() {
        assert_eq!(
            translate("Hello, world!", Lang::En, OovMode::Keep),
            "Hola, mundo!"
        );
    }

    #[test]
    fn preserves_code_identifiers_and_paths() {
        assert_eq!(
            translate("tbl_mantenimiento_ups", Lang::En, OovMode::Keep),
            "tbl_mantenimiento_ups"
        );
        assert_eq!(
            translate("product_warehouse", Lang::En, OovMode::Keep),
            "product_warehouse"
        );
        assert_eq!(
            translate("getNextDetailId", Lang::En, OovMode::Keep),
            "getNextDetailId"
        );
        assert_eq!(
            translate(
                "@/resources/src/views/products/index_products.vue",
                Lang::En,
                OovMode::Keep
            ),
            "@/resources/src/views/products/index_products.vue"
        );
    }

    #[test]
    fn resolves_plural_forms_to_singular_entries() {
        assert_eq!(
            translate("mantenimientos", Lang::Es, OovMode::Keep),
            "maintenance"
        );
        assert_eq!(translate("campos", Lang::Es, OovMode::Keep), "fields");
        assert_eq!(translate("fields", Lang::En, OovMode::Keep), "campo");
        assert_eq!(translate("clients", Lang::En, OovMode::Keep), "cliente");
    }

    #[test]
    fn resolves_spanish_plural_forms() {
        for (input, expected) in [
            ("gatos", "cat"),
            ("casas", "house"),
            ("árboles", "tree"),
            ("arboles", "tree"),
            ("voces", "voice"),
            ("flores", "flower"),
            ("nubes", "cloud"),
            ("ventanas", "window"),
            ("computadoras", "computer"),
            ("servidores", "server"),
            ("camiones", "truck"),
            ("lápices", "pencil"),
            ("lapices", "pencil"),
            ("paises", "countries"),
        ] {
            assert_eq!(
                translate(input, Lang::Es, OovMode::Keep),
                expected,
                "input: {input}"
            );
        }
    }

    #[test]
    fn resolves_spanish_present_tense_verbs() {
        for (input, expected) in [
            ("hablas", "talk"),
            ("hablamos", "talk"),
            ("hablan", "talk"),
            ("comes", "eat"),
            ("comemos", "eat"),
            ("vives", "live"),
            ("vivimos", "live"),
            ("trabajas", "work"),
            ("trabajan", "work"),
            ("podemos", "can"),
            ("aprendemos", "learn"),
            ("escribimos", "write"),
            ("bebemos", "drink"),
            ("caminamos", "walk"),
            ("estudiamos", "study"),
            ("hablo", "talk"),
            ("llego", "arrive"),
            ("compro", "buy"),
            ("miro", "look"),
            ("tomo", "take"),
            ("llamo", "call"),
            ("dejo", "let"),
            ("uso", "use"),
        ] {
            assert_eq!(
                translate(input, Lang::Es, OovMode::Keep),
                expected,
                "input: {input}"
            );
        }
    }

    #[test]
    fn distinguishes_present_tense_from_accented_preterite() {
        assert_eq!(translate("hablo", Lang::Es, OovMode::Keep), "talk");
        assert_eq!(translate("habló", Lang::Es, OovMode::Keep), "spoke");
    }

    #[test]
    fn does_not_capitalize_english_i_mid_sentence() {
        assert_eq!(
            translate("I am here", Lang::En, OovMode::Keep),
            "Estoy aquí"
        );
        assert_eq!(
            translate("Hello, I am here", Lang::En, OovMode::Keep),
            "Hola, estoy aquí"
        );
    }

    #[test]
    fn keeps_or_brackets_unknown_words() {
        assert_eq!(
            translate("hello zorblax", Lang::En, OovMode::Keep),
            "hola zorblax"
        );
        assert_eq!(
            translate("hello zorblax", Lang::En, OovMode::Bracket),
            "hola [zorblax]"
        );
    }

    #[test]
    fn preserves_dotted_and_underscored_identifiers() {
        for identifier in [
            "user.name",
            "hello.world",
            "_private",
            "CAT_2",
            "hello123",
            "api/v2/users",
            "README.md",
        ] {
            assert_eq!(
                translate(identifier, Lang::En, OovMode::Keep),
                identifier,
                "identifier was translated: {identifier}"
            );
        }
    }

    #[test]
    fn still_translates_words_before_sentence_punctuation() {
        assert_eq!(translate("hello.", Lang::En, OovMode::Keep), "hola.");
        assert_eq!(
            translate("hello. world", Lang::En, OovMode::Keep),
            "hola. mundo"
        );
    }

    #[test]
    fn accepts_unaccented_spanish_and_typographic_apostrophes() {
        assert_eq!(
            translate("el gato rapido salta.", Lang::Es, OovMode::Keep),
            "the fast cat jumps."
        );
        assert_eq!(
            translate("don’t worry", Lang::En, OovMode::Keep),
            "no te preocupes"
        );
        assert_eq!(translate("I’m sorry", Lang::En, OovMode::Keep), "Lo siento");
    }

    #[test]
    fn removes_inverted_spanish_punctuation_from_english_output() {
        assert_eq!(
            translate("¿Dónde está la farmacia?", Lang::Es, OovMode::Keep),
            "Where is the pharmacy?"
        );
        assert_eq!(translate("¡Hola!", Lang::Es, OovMode::Keep), "Hello!");
    }
}
