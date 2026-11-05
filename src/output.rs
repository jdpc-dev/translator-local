use crate::alternatives::WordAlternatives;
use crate::data::Lang;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct JsonReport<'a> {
    source_text: &'a str,
    detected_language: &'a str,
    target_language: &'a str,
    translation: &'a str,
    word_alternatives: Vec<JsonAlternative<'a>>,
}

#[derive(Debug, Serialize)]
struct JsonAlternative<'a> {
    word: &'a str,
    alternatives: Vec<&'a str>,
}

pub fn render_text(
    source_text: &str,
    source: Lang,
    target: Lang,
    detected: bool,
    translation: &str,
    alternatives: &[WordAlternatives],
) -> String {
    let source_label = if detected {
        format!("Detected: {}", source.code().to_uppercase())
    } else {
        source.code().to_uppercase()
    };

    let mut output = format!(
        "Source ({source_label}): {source_text}\nTarget ({}): {translation}\n\nWord Alternatives:",
        target.code().to_uppercase()
    );

    if alternatives.is_empty() {
        output.push_str("\n(none)");
    }

    for entry in alternatives {
        if entry.alternatives.is_empty() {
            output.push_str(&format!("\n- {} → []", entry.word));
        } else {
            output.push_str(&format!(
                "\n- {} → ({})",
                entry.word,
                entry.alternatives.join(", ")
            ));
        }
    }

    output
}

pub fn render_json(
    source_text: &str,
    source: Lang,
    target: Lang,
    translation: &str,
    alternatives: &[WordAlternatives],
) -> Result<String, serde_json::Error> {
    let report = JsonReport {
        source_text,
        detected_language: source.code(),
        target_language: target.code(),
        translation,
        word_alternatives: alternatives
            .iter()
            .map(|entry| JsonAlternative {
                word: &entry.word,
                alternatives: entry.alternatives.iter().map(String::as_str).collect(),
            })
            .collect(),
    };

    serde_json::to_string_pretty(&report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_alternatives() -> Vec<WordAlternatives> {
        vec![
            WordAlternatives {
                word: "The".into(),
                alternatives: vec!["A".into(), "That".into()],
            },
            WordAlternatives {
                word: "cat".into(),
                alternatives: vec!["feline".into(), "kitty".into()],
            },
        ]
    }

    #[test]
    fn text_output_marks_detected_language() {
        let output = render_text(
            "El gato salta",
            Lang::Es,
            Lang::En,
            true,
            "The cat jumps",
            &sample_alternatives(),
        );
        assert!(output.starts_with("Source (Detected: ES): El gato salta"));
        assert!(output.contains("Target (EN): The cat jumps"));
        assert!(output.contains("- The → (A, That)"));
    }

    #[test]
    fn explicit_source_omits_the_detected_label() {
        let output = render_text("Hola", Lang::Es, Lang::En, false, "Hello", &[]);
        assert!(output.starts_with("Source (ES): Hola"));
        assert!(output.contains("Word Alternatives:\n(none)"));
    }

    #[test]
    fn json_uses_the_required_schema() {
        let json = render_json(
            "El gato salta",
            Lang::Es,
            Lang::En,
            "The cat jumps",
            &sample_alternatives(),
        )
        .expect("serialization succeeds");

        let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(value["source_text"], "El gato salta");
        assert_eq!(value["detected_language"], "es");
        assert_eq!(value["target_language"], "en");
        assert_eq!(value["translation"], "The cat jumps");
        assert_eq!(value["word_alternatives"][0]["word"], "The");
        assert_eq!(value["word_alternatives"][0]["alternatives"][0], "A");
    }
}
