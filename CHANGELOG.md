# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Reverse overrides for `papel` → `paper`, `la mañana` → `the morning`, and
  `el vino` → `the wine`.
- Regression tests for short Spanish phrases, code identifiers, and
  unaccented/typographic input.

### Changed

- Language detection is more conservative about English `-er` terms, adds a
  full-phrase bonus, and accepts unaccented Spanish dictionary entries.
- Translation now preserves dotted names, leading/trailing underscores, and
  digit-containing identifiers.
- Lookups normalize typographic apostrophes and fall back to accent-folded
  keys.
- `Cargo.toml` declares a minimum Rust version and an SPDX license identifier.
- Install and size targets are portable, and CI builds use the lockfile.

### Fixed

- Closed stdout pipes no longer panic with exit code 101.
- Inverted Spanish punctuation is removed from English output.

## [0.2.11] - 2026-11-05

### Added

- `shine`/`brillar` and `shines`/`brilla`.
- Reverse overrides for `por que` → `why` and `en la noche` → `at night`.
- Phrase `why does the moon shine` / `por que la luna brilla`.
- Alternatives for `shine`, `shines`, `brillar`, and `brilla`.
- Regression tests for both `porque` and `por que` moon questions.

### Fixed

- `brilla` now translates to `shines`.
- `por que` (unaccented) now translates as `why` instead of `by that`.
- `en la noche` now renders as `at night`.

## [0.2.10] - 2026-11-05

### Changed

- Added the phrase `fast cat` / `gato rápido` so Spanish noun-adjective order is
  rendered correctly: `el gato rápido salta` → `the fast cat jumps`.
- Updated the demo, JSON example, and regression tests for the new word order.

## [0.2.9] - 2026-11-05

### Added

- `hide`/`esconder`, plus reverse overrides for `ocultar` → `hide`.
- `medkit`/`botiquín`, `medkits`/`botiquines`, and identity handling for the
  Spanish gaming loanword `medkits`.
- Alternatives for `hide`, `ocultar`, `esconder`, and `medkits`.
- Regression test for single Spanish infinitive input and `ocultar medkits`.

### Fixed

- Single Spanish infinitive words such as `ocultar` now detect as Spanish and
  translate instead of defaulting to English and passing through.

## [0.2.8] - 2026-11-05

### Added

- Technical and business vocabulary derived from the prompt corpus: maintenance,
  clients, equipment, fields, locations, modules, validation, records, reports,
  inventory, warehouses, purchases, sales, balances, migrations, controllers,
  routes, views, components, and related terms.
- Common prompt phrases such as `create table`, `add fields`, `product
  selector`, `creation form`, `sales module`, `to add`, `to allow`, `to
  modify`, `to export`, and `to validate`.
- Unaccented and plural reverse overrides for common Spanish forms
  (`mantenimientos`, `campos`, `validaciones`, `opciones`, `ubicaciones`,
  `exportar`, and others).
- Regression tests for representative lines from `prompts.md`.

### Changed

- Plural forms now fall back to their singular dictionary entry.
- Code-like tokens are preserved unchanged: snake_case, camelCase, file paths,
  dotted identifiers, and common file extensions.
- Dataset now covers 2,245 pairs, 662 reverse overrides, and 1,139 synonym
  entries.

## [0.2.7] - 2026-11-05

### Added

- `hurts` → `duele` and the phrase `my ass hurts` → `me duele el culo`.
- Alternatives for `hurts`/`duele`.
- Regression test for the phrase translation.

### Changed

- Racial slurs and comparable derogatory terms are intentionally not curated
  in the bundled dictionary; they remain out-of-vocabulary and can be marked
  with `--oov bracket`.
- Dataset now covers 2,072 pairs and 1,139 synonym entries.

## [0.2.6] - 2026-11-05

### Added

- Technical vocabulary and phrases for daemons, memory leaks, kernel modules,
  microservices, deployment, Kubernetes, git/merge conflicts, API rate limits,
  status codes, databases, restoring backups, and CI/CD.
- Gaming vocabulary for ping/lag, matchmaking, boss mechanics, hitboxes, XP,
  grinding, leveling, skill trees, ranked play, junglers, feeding, hotfixes,
  nerfs, crafting, legendary armor, raids, aim, and lobbies.
- English and Spanish slang/profanity coverage for the supplied corpus,
  including `no mames`, `wey`, `trolear`, `curro`, `pesao`, `plata`,
  `tacaño`, `me vale verga`, `me la pela`, and common insults.
- Hyphenated tokens such as `low-ass` and `one-shot` are now preserved as single
  tokens.
- Regression tests for technical, gaming, slang, and profanity sentences.

### Fixed

- English pronoun `I` is no longer capitalized mid-sentence after translation.
- Generic `for such a` no longer overrides reverse translations of `por un`.
- Dataset now covers 2,068 pairs, 303 reverse overrides, and 1,137 synonym
  entries.

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
