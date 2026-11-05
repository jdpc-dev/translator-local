use crate::data::{Lang, synonyms};
use crate::tokenize::{Token, apply_case, tokenize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordAlternatives {
    pub word: String,
    pub alternatives: Vec<String>,
}

/// Return deterministic alternatives (up to three) for every word in the
/// translated output. Words without synonyms get an empty list.
pub fn alternatives(text: &str, lang: Lang) -> Vec<WordAlternatives> {
    let synonyms = synonyms();

    tokenize(text)
        .into_iter()
        .filter_map(|token| match token {
            Token::Word(word) => {
                let key = word.to_lowercase();
                let mut seen = HashSet::from([key.clone()]);
                let mut matches = Vec::new();

                for candidate in synonyms.get(lang, &key) {
                    if seen.insert(candidate.to_lowercase()) {
                        matches.push(apply_case(candidate, &word));
                        if matches.len() == 3 {
                            break;
                        }
                    }
                }

                Some(WordAlternatives {
                    word,
                    alternatives: matches,
                })
            }
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_alternatives_for_the_demo_sentence() {
        let result = alternatives("The fast cat jumps", Lang::En);

        assert_eq!(result.len(), 4);
        assert_eq!(result[0].word, "The");
        assert_eq!(result[0].alternatives, vec!["A", "That"]);
        assert_eq!(result[1].alternatives, vec!["quick", "rapid", "swift"]);
        assert_eq!(result[2].alternatives, vec!["feline", "kitty"]);
        assert_eq!(result[3].alternatives, vec!["leaps", "hops", "bounds"]);
    }

    #[test]
    fn returns_an_empty_list_without_synonyms() {
        let result = alternatives("zorblax", Lang::En);
        assert_eq!(
            result,
            vec![WordAlternatives {
                word: "zorblax".into(),
                alternatives: vec![],
            }]
        );
    }

    #[test]
    fn ignores_punctuation_tokens() {
        let result = alternatives("Hello, world!", Lang::En);
        assert_eq!(result.len(), 2);
    }
}
