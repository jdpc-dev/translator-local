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
            }),
            _ => None,
        })
        .collect();

    let dictionary = dictionary();
    let max_phrase_words = dictionary.max_phrase_words();
    let mut replaced: Vec<Option<String>> = vec![None; tokens.len()];
    let mut index = 0;

    while index < words.len() {
        let max_length = max_phrase_words.min(words.len() - index);
        let mut chosen: Option<(usize, &str)> = None;

        for length in (1..=max_length).rev() {
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

            if let Some(translation) = dictionary.lookup(source, &key) {
                chosen = Some((length, translation));
                break;
            }
        }

        match chosen {
            Some((length, translation)) => {
                let replacement = apply_case(translation, &words[index].text);
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
