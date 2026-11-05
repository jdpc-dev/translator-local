use serde_json::Value;
use std::collections::HashSet;

const DICTIONARY_JSON: &str = include_str!("../data/dictionary.json");
const SYNONYMS_JSON: &str = include_str!("../data/synonyms.json");

fn dictionary_pairs() -> Vec<(String, String)> {
    let value: Value = serde_json::from_str(DICTIONARY_JSON).expect("dictionary.json is valid");
    value["pairs"]
        .as_array()
        .expect("pairs is an array")
        .iter()
        .map(|pair| {
            (
                pair["en"].as_str().expect("en is a string").to_string(),
                pair["es"].as_str().expect("es is a string").to_string(),
            )
        })
        .collect()
}

#[test]
fn dictionary_seed_is_substantial_and_well_formed() {
    let pairs = dictionary_pairs();
    assert!(
        pairs.len() >= 900,
        "expected at least 900 pairs, found {}",
        pairs.len()
    );

    let mut seen = HashSet::new();
    let mut phrases = 0;
    let mut max_phrase_words = 1;

    for (en, es) in &pairs {
        assert!(!en.is_empty(), "empty English key");
        assert!(!es.is_empty(), "empty Spanish value for {en:?}");
        assert_eq!(
            *en,
            en.to_lowercase(),
            "English key is not lowercase: {en:?}"
        );
        assert_eq!(
            *es,
            es.to_lowercase(),
            "Spanish value is not lowercase: {es:?}"
        );
        assert!(seen.insert(en.clone()), "duplicate English key: {en:?}");

        if en.contains(' ') {
            phrases += 1;
        }
        max_phrase_words = max_phrase_words
            .max(en.split_whitespace().count())
            .max(es.split_whitespace().count());
    }

    assert!(phrases >= 100, "expected at least 100 phrases");
    assert!(max_phrase_words >= 3, "expected multi-word phrase coverage");
}

#[test]
fn required_demo_entries_are_present() {
    let pairs = dictionary_pairs();
    for (en, es) in [
        ("the", "el"),
        ("cat", "gato"),
        ("fast", "rápido"),
        ("jumps", "salta"),
        ("good morning", "buenos días"),
        ("i am cold", "tengo frío"),
        ("you're welcome", "de nada"),
    ] {
        assert!(
            pairs
                .iter()
                .any(|(pair_en, pair_es)| pair_en == en && pair_es == es),
            "missing pair: {en:?} -> {es:?}"
        );
    }
}

#[test]
fn synonyms_are_clean_and_do_not_repeat_the_headword() {
    let value: Value = serde_json::from_str(SYNONYMS_JSON).expect("synonyms.json is valid");

    for lang in ["en", "es"] {
        let entries = value[lang].as_object().expect("language map is an object");
        assert!(
            entries.len() >= 100,
            "{lang} should have at least 100 synonym entries"
        );

        for (word, alternatives) in entries {
            assert_eq!(*word, word.to_lowercase(), "synonym key is not lowercase");
            let alternatives = alternatives
                .as_array()
                .expect("alternative list is an array");
            assert!(!alternatives.is_empty(), "empty alternatives for {word:?}");

            let mut seen = HashSet::new();
            for alternative in alternatives {
                let alternative = alternative.as_str().expect("alternative is a string");
                assert!(!alternative.is_empty());
                assert_ne!(
                    alternative.to_lowercase(),
                    word.to_lowercase(),
                    "synonym repeats its headword: {word:?}"
                );
                assert!(
                    seen.insert(alternative.to_lowercase()),
                    "duplicate synonym for {word:?}: {alternative:?}"
                );
            }
        }
    }
}
