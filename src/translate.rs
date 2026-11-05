use crate::data::{Lang, dictionary};
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
                lower: word.to_lowercase(),
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

            let mut translation = dictionary.lookup(source, &key);
            if translation.is_none() && length == 1 {
                for candidate in singular_candidates(&key) {
                    if let Some(found) = dictionary.lookup(source, &candidate) {
                        translation = Some(found);
                        break;
                    }
                }
            }

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
            None => output.push_str(token.text()),
        }
    }
    output
}

fn is_code_like(tokens: &[Token], token_index: usize, word: &str) -> bool {
    if word.contains('_') || word.chars().any(|c| c.is_ascii_digit()) {
        return true;
    }

    let mut previous_lower = false;
    for c in word.chars() {
        if previous_lower && c.is_uppercase() {
            return true;
        }
        previous_lower = c.is_lowercase();
    }

    let prev = token_index.checked_sub(1).and_then(|i| tokens.get(i));
    let next = tokens.get(token_index + 1);
    let path_punct = |token: &Token| matches!(token, Token::Punct(p) if matches!(p.as_str(), "/" | "\\" | "@" | "$"));
    if prev.is_some_and(path_punct) || next.is_some_and(path_punct) {
        return true;
    }

    let word_at = |index: usize| match tokens.get(index) {
        Some(Token::Word(text)) => Some(text.as_str()),
        _ => None,
    };

    if let Some(Token::Punct(p)) = prev
        && p == "."
    {
        if let Some(before) = token_index.checked_sub(2).and_then(word_at)
            && (is_code_word(before) || is_file_extension(word))
        {
            return true;
        }
        if is_file_extension(word) {
            return true;
        }
    }

    if let Some(Token::Punct(p)) = next
        && p == "."
    {
        if let Some(after) = word_at(token_index + 2)
            && (is_code_word(after) || is_file_extension(after))
        {
            return true;
        }
        if is_file_extension(word) {
            return true;
        }
    }

    false
}

fn is_code_word(word: &str) -> bool {
    if word.contains('_') || word.chars().any(|c| c.is_ascii_digit()) {
        return true;
    }
    let mut previous_lower = false;
    for c in word.chars() {
        if previous_lower && c.is_uppercase() {
            return true;
        }
        previous_lower = c.is_lowercase();
    }
    false
}

fn is_file_extension(word: &str) -> bool {
    matches!(
        word,
        "php"
            | "vue"
            | "js"
            | "ts"
            | "json"
            | "md"
            | "xlsx"
            | "pdf"
            | "csv"
            | "txt"
            | "html"
            | "css"
            | "yml"
            | "yaml"
            | "xml"
            | "sql"
            | "env"
            | "lock"
    )
}

fn singular_candidates(word: &str) -> Vec<String> {
    if word.chars().count() <= 3 {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    let mut push = |candidate: &str| {
        if !candidate.is_empty()
            && candidate != word
            && !candidates.iter().any(|existing| existing == candidate)
        {
            candidates.push(candidate.to_string());
        }
    };

    if let Some(stem) = word.strip_suffix("ies") {
        push(&format!("{stem}y"));
    }
    if let Some(stem) = word.strip_suffix("ces") {
        push(&format!("{stem}z"));
    }
    if let Some(stem) = word.strip_suffix("es") {
        push(stem);
    }
    if !word.ends_with("ss")
        && let Some(stem) = word.strip_suffix('s')
    {
        push(stem);
    }

    candidates
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
    matches!(text, "I" | "I'm" | "I've" | "I'll" | "I'd")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_spanish_to_english() {
        assert_eq!(
            translate("El gato rápido salta.", Lang::Es, OovMode::Keep),
            "The cat fast jumps."
        );
    }

    #[test]
    fn translates_english_to_spanish() {
        assert_eq!(
            translate("The fast cat jumps", Lang::En, OovMode::Keep),
            "El rápido gato salta"
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
}
