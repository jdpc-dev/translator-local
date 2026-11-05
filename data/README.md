# Bundled data

These files are compiled into the `translate-cli` binary with `include_str!`.
They are the complete offline dataset; the tool makes no network requests.

## `dictionary.json`

```json
{
  "version": 1,
  "pairs": [
    {"en": "good morning", "es": "buenos días"},
    {"en": "cat", "es": "gato"}
  ]
}
```

- All English keys are unique and lowercase.
- Spanish values are lowercase.
- A pair may be a single word or a phrase. Phrase entries are matched
  longest-first.
- Lookups are bidirectional. When multiple English keys share a Spanish value,
  the first pair in the file wins for the reverse direction.

Current coverage: 1,719 pairs — 1,367 single words and 352 phrases.

## `synonyms.json`

```json
{
  "version": 1,
  "en": {"cat": ["feline", "kitty"]},
  "es": {"gato": ["felino", "minino"]}
}
```

- Keys are lowercase headwords in the target language.
- Lists are ordered; the first three alternatives are shown.
- Headwords and duplicate alternatives are not allowed.
- A word may legitimately have no list; the tool returns `[]` for it.

Current coverage: 1,004 entries — 511 English and 493 Spanish.

## Provenance

The word/phrase pairs and synonym groups were curated for this project. No
third-party dictionary text is bundled. The JSON files are the source of truth;
no data-generation step is required at build time.

## Extending the data

Add lowercase pairs or synonym lists, keep English dictionary keys unique
(including after collapsing whitespace), then run:

```bash
cargo test
```

The data tests validate the JSON shape, capitalization, uniqueness, non-empty
values, and synonym lists.
