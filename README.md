# translator-local

[![CI](https://github.com/jdpc-dev/translator-local/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/jdpc-dev/translator-local/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/jdpc-dev/translator-local)](https://github.com/jdpc-dev/translator-local/releases)

An offline, deterministic English ↔ Spanish command-line translator. It uses a
bundled bilingual dictionary, n-gram matching, heuristic language detection,
and a local synonym table. There is no machine learning, model runtime, or
network access.

## Constraint checklist

| Requirement | Implementation |
| :--- | :--- |
| No AI/ML dependencies | Pure Rust with `regex`, `serde`, and `clap` only |
| Footprint under 15 MB | 2.5 MB stripped release binary with embedded JSON data; verify with `make size` |
| Under 50 ms | About 9 ms per invocation including process startup on the development machine; verify with `make bench` |
| Single standalone binary | `data/*.json` is compiled in with `include_str!` |
| Fully offline | No network calls at runtime; all data is bundled |

## Build and install

```bash
cargo build --release

# Optional: install to /usr/local/bin
sudo make install

# Or install into your home directory
make install PREFIX="$HOME/.local"
```

The binary is `target/release/translate-cli`.

Prebuilt archives are available from the
[Releases](https://github.com/jdpc-dev/translator-local/releases) page for Linux
(x86_64 and aarch64), macOS (universal), and Windows (x86_64). Each archive
contains the binary, `LICENSE`, `README.md`, `CHANGELOG.md`, and a SHA-256
checksum file.

## Usage

```bash
# Auto-detect Spanish, translate to English, and show alternatives
translate-cli "El gato rápido salta"

# Explicit source and JSON output
translate-cli "The fast cat jumps" --source en --format json

# Read from a file; the target defaults to the opposite of the source
translate-cli --file input.txt --target es

# Read from stdin
echo "good morning" | translate-cli
```

Input precedence is: positional `TEXT`, then `--file`, then stdin. If stdin is
a terminal and no text or file was supplied, the command exits with a usage
error instead of waiting for input.

### Options

| Option | Description |
| :--- | :--- |
| `TEXT` | Text to translate |
| `-f, --file PATH` | Read input from a file |
| `-s, --source en\|es` | Source language; auto-detected when omitted |
| `-t, --target en\|es` | Target language; defaults to the opposite of the source |
| `--format text\|json` | Human-readable output (default) or JSON |
| `--oov keep\|bracket` | Pass unknown words through (default) or wrap them in `[...]` |

The target must differ from the source. When detection is ambiguous or finds no
signals, the tool warns on **stderr** and defaults to English; JSON on stdout
stays valid.

### Text output

```text
Source (Detected: ES): El gato rápido salta.
Target (EN): The fast cat jumps.

Word Alternatives:
- The → (A, That)
- fast → (quick, rapid, swift)
- cat → (feline, kitty)
- jumps → (leaps, hops, bounds)
```

### JSON output

```json
{
  "source_text": "El gato rápido salta",
  "detected_language": "es",
  "target_language": "en",
  "translation": "The fast cat jumps",
  "word_alternatives": [
    {
      "word": "The",
      "alternatives": ["A", "That"]
    }
  ]
}
```

`detected_language` reports the **effective source language**, including when
`--source` was supplied explicitly.

## How it works

1. **Input** is read from an argument, file, or stdin.
2. **Detection** adds 10 points to Spanish for any of `ñ ¿ ¡ á é í ó ú ü`, then
   adds 1 point per stopword match for each language. Dictionary membership adds
   another point for content words, and a list of three or more Spanish
   infinitives adds 2 points to Spanish. Ties and zero scores default to English
   with a warning.
3. **Translation** tokenizes the text while keeping whitespace and punctuation.
   It tries dictionary phrases longest-first (up to the longest bundled phrase),
   then falls back to single-word lookup. Plural forms retry their singular
   entry. Code-like tokens (snake_case, camelCase, paths, dotted names, file
   extensions) are preserved unchanged. Matched spans preserve the source
   capitalization. Unknown words are passed through or bracketed.
4. **Alternatives** tokenize the translated output and look up up to three
   deterministic synonyms per word in the target language.
5. **Output** is rendered as text or JSON.

## Data

The bundled dataset contains **2,252 bilingual pairs** (1,614 words and 638
phrases), **668 Spanish→English reverse overrides**, and **1,147 synonym
entries** (583 English and 564 Spanish), all embedded in the binary. Synonyms
are curated in bilingual concept groups so the English and Spanish alternatives
stay aligned. Words without a suitable synonym return an empty list.

Data lives in:

- `data/dictionary.json` — one canonical list of `{"en": ..., "es": ...}` pairs.
  Lookup tables are built in both directions at startup. If several English
  words share a Spanish value (for example `fast` and `quick` → `rápido`), the
  first pair in the file wins for the reverse lookup.
- `data/reverse.json` — explicit Spanish→English overrides for inflected verb
  forms, object pronouns, and common phrases. These take precedence over reverse
  lookups derived from the forward dictionary.
- `data/synonyms.json` — `en` and `es` maps of word → ordered synonym list.
  The first three entries are returned, excluding the headword itself.

See `data/README.md` for the full schema and data-maintenance notes.

All three files are embedded at compile time. To extend the dictionary, add
pairs using lowercase keys and values, keep English keys unique, then run
`cargo test`. The data tests enforce lowercase keys, unique English entries
(including after whitespace normalization), non-empty values, and clean synonym
lists.

## Known limitations

1. **Literal translations.** Idioms translate correctly only when they exist as
   phrase entries in the dictionary (`I am cold` is bundled as `tengo frío`).
2. **No grammatical agreement.** Gender, number, and verb conjugation are not
   adjusted. `The red cars` may produce an ungrammatical word order.
3. **Out-of-vocabulary words.** Proper nouns, slang, and technical terms are
   passed through unchanged by default. Use `--oov bracket` to mark them.

These limitations are also printed by `translate-cli --help`.

## Development

```bash
make test    # unit, integration, and data-integrity tests
make check   # rustfmt check and clippy with warnings denied
make build   # release build
make size    # verify the release binary is under 15 MB
make bench   # average runtime over 50 invocations
```

CI runs tests on Linux, macOS, and Windows, enforces formatting and clippy, and
checks the 15 MB size budget. Releases are built when a `v*` tag is pushed:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The release workflow creates a GitHub Release and uploads Linux x86_64/aarch64,
universal macOS, and Windows x86_64 archives.

Project layout:

```text
src/
  main.rs          CLI entry point and input handling
  cli.rs           Argument definitions and help text
  detect.rs        Heuristic language detection
  tokenize.rs      Punctuation-preserving tokenizer and case helper
  translate.rs     Longest-phrase-first translation
  alternatives.rs  Synonym lookup for translated words
  output.rs        Text and JSON rendering
  data.rs          Embedded data loading and lookup tables
data/
  dictionary.json
  synonyms.json
tests/
  cli.rs           End-to-end CLI tests
  data.rs          Dataset validation
.github/workflows/
  ci.yml           Test, lint, and size checks
  release.yml      Tag-driven release archives
CHANGELOG.md
```

## License

GPL-3.0. See `LICENSE`.
