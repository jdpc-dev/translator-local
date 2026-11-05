use regex::Regex;
use std::sync::LazyLock;

static TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?P<word>[\p{L}\p{M}]+(?:['’\-][\p{L}\p{M}]+)*)|(?P<space>\s+)|(?P<punct>[^\p{L}\p{M}\s]+)",
    )
    .expect("tokenizer regex is valid")
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Word(String),
    Space(String),
    Punct(String),
}

impl Token {
    pub fn text(&self) -> &str {
        match self {
            Self::Word(text) | Self::Space(text) | Self::Punct(text) => text,
        }
    }
}

/// Split text into word, whitespace, and punctuation tokens. Whitespace and
/// punctuation are kept verbatim so the original layout can be reconstructed.
pub fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    for captures in TOKEN_RE.captures_iter(text) {
        if let Some(word) = captures.name("word") {
            tokens.push(Token::Word(word.as_str().to_string()));
        } else if let Some(space) = captures.name("space") {
            tokens.push(Token::Space(space.as_str().to_string()));
        } else if let Some(punct) = captures.name("punct") {
            tokens.push(Token::Punct(punct.as_str().to_string()));
        }
    }

    tokens
}

/// Apply the capitalization pattern of `source` to `translation`.
pub fn apply_case(translation: &str, source: &str) -> String {
    let letters: Vec<char> = source.chars().filter(|c| c.is_alphabetic()).collect();

    if letters.len() > 1 && letters.iter().all(|c| c.is_uppercase()) {
        return translation.to_uppercase();
    }

    let starts_uppercase = letters.first().is_some_and(|c| c.is_uppercase());
    if starts_uppercase {
        let mut chars = translation.chars();
        if let Some(first) = chars.next() {
            let mut result = first.to_uppercase().collect::<String>();
            result.extend(chars);
            return result;
        }
    }

    translation.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_words_space_and_punctuation() {
        let tokens = tokenize("Hello, world!");
        assert_eq!(
            tokens,
            vec![
                Token::Word("Hello".into()),
                Token::Punct(",".into()),
                Token::Space(" ".into()),
                Token::Word("world".into()),
                Token::Punct("!".into()),
            ]
        );
    }

    #[test]
    fn keeps_apostrophes_inside_words() {
        let tokens = tokenize("don't stop");
        assert_eq!(tokens[0], Token::Word("don't".into()));
    }

    #[test]
    fn keeps_hyphens_inside_words() {
        let tokens = tokenize("low-ass wage and one-shot");
        assert_eq!(tokens[0], Token::Word("low-ass".into()));
        assert_eq!(tokens[6], Token::Word("one-shot".into()));
    }

    #[test]
    fn handles_accented_letters() {
        let tokens = tokenize("rápido, ¿dónde?");
        assert_eq!(tokens[0], Token::Word("rápido".into()));
        assert_eq!(tokens[1], Token::Punct(",".into()));
        assert_eq!(tokens[3], Token::Punct("¿".into()));
        assert_eq!(tokens[4], Token::Word("dónde".into()));
    }

    #[test]
    fn applies_sentence_case() {
        assert_eq!(apply_case("the", "The"), "The");
        assert_eq!(apply_case("the", "the"), "the");
        assert_eq!(apply_case("buenos días", "GOOD"), "BUENOS DÍAS");
    }
}
