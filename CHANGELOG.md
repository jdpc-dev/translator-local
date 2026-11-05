# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-11-05

### Added

- Offline English-Spanish CLI translator with stdin, positional text, and
  `--file` input.
- Heuristic language detection based on Spanish characters and stopword
  frequency, with explicit `--source` overrides.
- Longest-phrase-first dictionary translation preserving capitalization and
  punctuation.
- Optional bracketed output for out-of-vocabulary words (`--oov bracket`).
- Deterministic word alternatives from a bundled EN/ES synonym table.
- Human-readable and JSON output (`--format text|json`).
- Embedded seed dataset with roughly 970 bilingual pairs and 270 synonym
  entries.
- Unit, integration, and data-integrity tests.
- Size-budget and benchmark Makefile targets.
- GitHub Actions CI and tag-driven cross-platform release archives.
