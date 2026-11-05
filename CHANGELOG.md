# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.5] - 2026-11-05

### Added

- Dictionary-membership scoring for language detection, plus a heuristic that
  recognizes lists of Spanish infinitives (`-ar`, `-er`, `-ir`).
- Verb entries for `feed`/`alimentar`, `insert`/`insertar`, `aim`/`apuntar`,
  and the four seasons.
- Synonym entries for the new verbs.
- Regression tests for Spanish infinitive lists and English `-er` words.

### Fixed

- Spanish infinitive lists such as
  `alimentar, insertar, cortar, apuntar, disparar, repetir` are now detected as
  Spanish and translated instead of being treated as English and passed through.
- Dataset now covers 1,836 pairs and 1,044 synonym entries.

## [0.2.4] - 2026-11-05

### Added

- Reverse overrides and phrase entries for `la perra` / `the bitch`,
  `esta amarrada` / `is tied up`, `dejenla que llore` / `let her cry`,
  `todo el dia` / `all day`, and
  `no la deben de soltar` / `you should not let her go`.
- Singular/plural vocabulary for `bitch`, `tied`, `release`, `cries`, and
  related terms, plus alternatives.
- Data tests now reject non-Latin characters in synonym entries.

### Fixed

- Spanish articles `la`, `las`, and `los` now translate to `the` instead of
  being treated as object pronouns. Object-pronoun uses are handled by explicit
  phrase entries.
- Unaccented Spanish variants such as `dia` and `esta amarrada` are recognized.
- Dataset now covers 1,829 pairs (1,403 words and 426 phrases), 267 reverse
  overrides, and 1,034 synonym entries.

## [0.2.3] - 2026-11-05

### Added

- Plural noun forms for common people/body terms (`boys`, `girls`, `dicks`,
  `pussies`, and related terms).
- Spanish plural-subject phrases such as `boys has` → `los niños tienen`,
  including the forms needed for `boys has dick and girls has pussy.`
- Expanded anatomy/slang vocabulary and alternatives, bringing the dataset to
  1,819 pairs (1,399 words and 420 phrases) and 1,026 synonym entries.

### Fixed

- Plural subjects now produce `tienen` and articles in the covered phrase
  patterns instead of the singular `tiene`.

## [0.2.2] - 2026-11-05

### Fixed

- English verb + object-pronoun phrases now produce Spanish clitic order:
  `lies to us` → `nos miente`, `manipulates us` → `nos manipula`.
- Added 52 EN→ES phrase pairs for common object-pronoun constructions, bringing
  the dictionary to 1,780 pairs (1,374 words and 406 phrases).
- Added an end-to-end regression test for the English→Spanish direction.

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
