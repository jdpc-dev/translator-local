use crate::data::Lang;
use serde_json::Error as JsonError;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// Errors that can terminate a CLI run.
///
/// Each variant carries enough context to render an actionable message and to
/// preserve the underlying I/O or serialization error as its source.
#[derive(Debug)]
pub enum AppError {
    EmptyInput,
    NoInput,
    UnsupportedLanguage { role: &'static str, code: String },
    SameSourceAndTarget(Lang),
    ReadFile { path: PathBuf, source: io::Error },
    ReadStdin(io::Error),
    WriteOutput(io::Error),
    SerializeOutput(JsonError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => f.write_str("input text is empty"),
            Self::NoInput => {
                f.write_str("no input provided; pass TEXT, use --file PATH, or pipe text via stdin")
            }
            Self::UnsupportedLanguage { role, code } => {
                write!(f, "unsupported {role} language: {code}")
            }
            Self::SameSourceAndTarget(source) => write!(
                f,
                "target language must differ from source language ({})",
                source.name()
            ),
            Self::ReadFile { path, source } => {
                write!(f, "cannot read '{}': {source}", path.display())
            }
            Self::ReadStdin(source) => write!(f, "cannot read stdin: {source}"),
            Self::WriteOutput(source) => write!(f, "cannot write output: {source}"),
            Self::SerializeOutput(source) => write!(f, "cannot serialize output: {source}"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadFile { source, .. } => Some(source),
            Self::ReadStdin(source) | Self::WriteOutput(source) => Some(source),
            Self::SerializeOutput(source) => Some(source),
            Self::EmptyInput
            | Self::NoInput
            | Self::UnsupportedLanguage { .. }
            | Self::SameSourceAndTarget(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_actionable_messages() {
        assert_eq!(AppError::EmptyInput.to_string(), "input text is empty");
        assert_eq!(
            AppError::UnsupportedLanguage {
                role: "source",
                code: "fr".into(),
            }
            .to_string(),
            "unsupported source language: fr"
        );
        assert_eq!(
            AppError::SameSourceAndTarget(Lang::En).to_string(),
            "target language must differ from source language (English)"
        );
    }
}
