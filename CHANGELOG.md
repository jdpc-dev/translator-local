# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.1] - 2026-11-05

### Added

- Spanish-to-English reverse overrides (`data/reverse.json`) with 253 entries
  covering common inflections, object pronouns, and pronoun/verb phrases.
- Regression tests for Spanish prose without accents and the sentence
  `el gobierno nos miente y el dinero nos manipula`.

### Changed

- Expanded language-detection stopword lists to improve confidence on ordinary
  prose.
- Updated dataset counts to 1,728 pairs and 1,008 synonym entries.

### Fixed

- Spanish inflected verbs and object pronouns now translate correctly:
  `nos miente` → `lies to us`, `nos manipula` → `manipulates us`.
- Human-readable output now omits words that have no alternatives instead of
  printing empty `[]` entries; JSON still includes empty arrays as specified.

## [0.2.0] - 2026-11-05

### Changed

- Expanded the embedded dictionary from 967 to 1,719 pairs: 1,367 words and 352
  phrases, including common function words, inflections, everyday vocabulary,
  travel/health phrases, and conversation phrases.
- Expanded synonym tables from 273 to 1,004 entries (511 English and 493
  Spanish), adding curated bilingual concept groups for adjectives, nouns, and
  verbs.
- Updated data-integrity tests to enforce the larger seed size and normalized
  English-key uniqueness.

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
