mod alternatives;
mod cli;
mod data;
mod detect;
mod error;
mod output;
mod tokenize;
mod translate;

use clap::Parser;
use cli::{Cli, OutputFormat};
use data::Lang;
use error::AppError;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), AppError> {
    let text = read_input(&cli)?;
    if text.trim().is_empty() {
        return Err(AppError::EmptyInput);
    }

    let source_override = match cli.source.as_deref() {
        Some(code) => Some(
            Lang::from_code(code).ok_or_else(|| AppError::UnsupportedLanguage {
                role: "source",
                code: code.to_string(),
            })?,
        ),
        None => None,
    };

    let detection = if source_override.is_none() {
        Some(detect::detect(&text))
    } else {
        None
    };

    if let Some(detection) = &detection
        && detection.ambiguous
    {
        if detection.es_score == 0 && detection.en_score == 0 {
            eprintln!("warning: no language signals found; defaulting to English");
        } else {
            eprintln!(
                "warning: language detection was ambiguous (en={}, es={}); defaulting to English",
                detection.en_score, detection.es_score
            );
        }
    }

    let source = source_override.unwrap_or_else(|| {
        detection
            .as_ref()
            .expect("detection is present when no source override was supplied")
            .language
    });

    let target = match cli.target.as_deref() {
        Some(code) => {
            let target = Lang::from_code(code).ok_or_else(|| AppError::UnsupportedLanguage {
                role: "target",
                code: code.to_string(),
            })?;
            if target == source {
                return Err(AppError::SameSourceAndTarget(source));
            }
            target
        }
        None => source.opposite(),
    };

    let translation = translate::translate(&text, source, cli.oov);
    let alternatives = alternatives::alternatives(&translation, target);
    let detected = detection.is_some();

    let rendered = match cli.format {
        OutputFormat::Text => {
            output::render_text(&text, source, target, detected, &translation, &alternatives)
        }
        OutputFormat::Json => {
            output::render_json(&text, source, target, &translation, &alternatives)
                .map_err(AppError::SerializeOutput)?
        }
    };

    write_output(&rendered)
}

fn write_output(text: &str) -> Result<(), AppError> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    match writeln!(stdout, "{text}") {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(error) => Err(AppError::WriteOutput(error)),
    }
}

fn read_input(cli: &Cli) -> Result<String, AppError> {
    if let Some(text) = &cli.text {
        return Ok(strip_trailing_newlines(text));
    }

    if let Some(path) = &cli.file {
        let content = fs::read_to_string(path).map_err(|source| AppError::ReadFile {
            path: path.clone(),
            source,
        })?;
        return Ok(strip_trailing_newlines(&content));
    }

    if io::stdin().is_terminal() {
        return Err(AppError::NoInput);
    }

    let mut content = String::new();
    io::stdin()
        .read_to_string(&mut content)
        .map_err(AppError::ReadStdin)?;
    Ok(strip_trailing_newlines(&content))
}

fn strip_trailing_newlines(text: &str) -> String {
    text.trim_end_matches(['\r', '\n']).to_string()
}
