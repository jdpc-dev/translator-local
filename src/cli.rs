use crate::translate::OovMode;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

pub const KNOWN_LIMITATIONS: &str = "\
Known limitations:
  1. Translations are literal. Idioms only translate correctly when they
     exist as phrase entries in the bundled dictionary.
  2. There is no grammatical agreement for gender, number, or verb
     conjugation.
  3. Out-of-vocabulary words pass through unchanged by default; use
     --oov bracket to mark them.
  4. Language detection is heuristic; very short or mixed-language input
     may default to English.";

#[derive(Debug, Parser)]
#[command(
    name = "translate-cli",
    version,
    about = "Offline, rule-based English-Spanish translator",
    after_help = KNOWN_LIMITATIONS
)]
pub struct Cli {
    /// Text to translate. If omitted, reads --file or standard input.
    #[arg(value_name = "TEXT")]
    pub text: Option<String>,

    /// Read input text from a file.
    #[arg(short, long, value_name = "PATH", conflicts_with = "text")]
    pub file: Option<PathBuf>,

    /// Source language (auto-detected when omitted).
    #[arg(short, long, value_parser = ["en", "es"])]
    pub source: Option<String>,

    /// Target language (defaults to the opposite of the source).
    #[arg(short, long, value_parser = ["en", "es"])]
    pub target: Option<String>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,

    /// How to render out-of-vocabulary words.
    #[arg(long, value_enum, default_value_t = OovMode::Keep)]
    pub oov: OovMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}
