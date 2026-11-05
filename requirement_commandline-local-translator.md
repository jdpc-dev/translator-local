## 1. Functional Requirements

### 1.1 Input Handling
*   Accept text via standard input (stdin), command-line arguments, or file reading (`--file`).
*   Allow explicit source language override via flag (e.g., `--source en` or `--source es`).
*   Automatically infer the target language as the opposite of the source (EN ↔ ES).

### 1.2 Language Detection (Heuristic & Deterministic)
*   **Character Scoring:** Assign high probability to Spanish if the text contains: `ñ`, `¿`, `¡`, `á`, `é`, `í`, `ó`, `ú`, `ü`.
*   **Stopword Frequency Scoring:** Maintain a hardcoded list of the top 30 most common words for each language.
    *   *Spanish:* `el`, `la`, `de`, `que`, `y`, `en`, `un`, `ser`, `se`, `no`, `haber`, `por`, `con`, `su`, `para`, `como`.
    *   *English:* `the`, `be`, `to`, `of`, `and`, `a`, `in`, `that`, `have`, `i`, `it`, `for`, `not`, `on`, `with`, `he`.
*   **Logic:** Count matches. The language with the highest score wins. If tied or score is 0, default to English (or raise a warning).

### 1.3 Translation Engine (Dictionary & Rule-Based)
*   **Data Source:** A local, embedded SQLite database or compressed JSON file containing bidirectional word/phrase mappings (EN ↔ ES).
*   **N-gram Priority Matching:** The engine must scan the input text for the longest possible matching phrases first (e.g., match "good morning" → "buenos días" *before* matching "good" → "bueno" and "morning" → "mañana").
*   **Fallback Handling:** If a word is not found in the dictionary (Out-Of-Vocabulary), it must be passed through unchanged, optionally wrapped in brackets (e.g., `[unknown_word]`) to alert the user.
*   **Basic Tokenization:** Split input text into words and punctuation using standard Regular Expressions (Regex), preserving original capitalization and punctuation placement.

### 1.4 Word-Level Alternatives
*   Post-translation, tokenize the *translated* output into individual words.
*   Query a local synonym database (e.g., a pre-compiled SQLite table derived from open-source WordNet / Open Multilingual WordNet).
*   Return 2–3 deterministic synonyms for each translated word. If no synonyms exist, return an empty array `[]`.

### 1.5 Output Formatting
*   Support human-readable terminal output and structured `--format json`.
*   **Example Output:**
  ```text
  Source (Detected: ES): El gato rápido salta.
  Target (EN): The cat fast jumps.  <-- Note: Literal translation
  
  Word Alternatives:
  - The → (A, That)
  - cat → (feline, kitty)
  - fast → (quick, rapid, swift)
  - jumps → (leaps, hops, bounds)
  ```

---

## 2. Non-Functional Requirements

*   **Zero AI/ML Dependencies:** No PyTorch, TensorFlow, Hugging Face, FastText, or LLMs. The codebase must be purely algorithmic.
*   **Ultra-Lightweight Footprint:** Total installation size (including CLI tool, dictionary JSON/SQLite, and synonym DB) must be **< 15 MB**.
*   **Performance:** Execution time must be **< 50 milliseconds** on any modern CPU, as it only involves Regex matching and database lookups.
*   **Portability:** Distributable as a single standalone binary (via PyInstaller, Go build, or Rust Cargo) with the dictionary embedded as an asset.
*   **100% Offline:** No network calls whatsoever, not even for initial setup. All data is bundled or manually placed in a local directory.

---

## 3. Recommended Technical Stack

| Component | Recommended Technology | Rationale |
| :--- | :--- | :--- |
| **Language** | Python, Go, or Rust | Python for rapid regex/dict prototyping; Go/Rust for compiling to a single, dependency-free binary. |
| **CLI Framework** | `argparse` (Python stdlib) or `clap` (Rust) | Zero external dependencies, fast, standard. |
| **Text Processing** | Standard Library `re` (Regex) | For deterministic tokenization, punctuation preservation, and n-gram extraction. |
| **Data Storage** | Embedded `sqlite3` or `json` | SQLite allows fast `SELECT synonym FROM table WHERE word = ?` lookups. JSON is simpler for < 5MB datasets. |
| **Dictionary Data** | Custom curated JSON/SQLite | Built from open-source bilingual dictionaries (e.g., Wiktionary dumps, Apertium bilingual dicts). |

---

## 4. CLI Interface Design

```bash
# Auto-detect language, translate, and show alternatives
$ translate-cli "El gato rápido salta"

# Explicitly set source language, output as JSON
$ translate-cli "The fast cat jumps" --source en --format json

# Read from a file
$ translate-cli --file input.txt --target es
```

**Expected JSON Output Structure:**
```json
{
  "source_text": "El gato rápido salta",
  "detected_language": "es",
  "target_language": "en",
  "translation": "The cat fast jumps",
  "word_alternatives": [
    {"word": "The", "alternatives": ["A", "That"]},
    {"word": "cat", "alternatives": ["feline", "kitty"]},
    {"word": "fast", "alternatives": ["quick", "rapid"]},
    {"word": "jumps", "alternatives": ["leaps", "hops"]}
  ]
}
```

---

## 5. Implementation Workflow (Deterministic)

1. **Parse Input:** Read text and CLI flags.
2. **Detect Language:** 
   - Run Regex for Spanish-specific characters. If found, score +10 for ES.
   - Lowercase text, split into words, and count matches against the hardcoded ES/EN stopword lists.
   - Declare the language with the highest score.
3. **Translate (N-gram First):**
   - Load the bilingual dictionary into memory (or query SQLite).
   - Sort dictionary keys by length (descending) to prioritize multi-word phrases.
   - Iterate through the sorted keys. If a key matches a segment of the input text, replace it with the target language value and mark that segment as "translated".
   - For remaining un-translated single words, perform a direct dictionary lookup.
   - Reconstruct the sentence, preserving original punctuation and capitalization.
4. **Generate Alternatives:**
   - Tokenize the final translated string.
   - For each word, query the local synonym SQLite database.
5. **Output:** Format and print the result.

---

## 6. Known Limitations (Must be documented in `--help`)

1. **Literal Translations:** "I am cold" will translate to "Estoy frío" (literal) instead of "Tengo frío" (idiomatic), unless "I am cold" is explicitly added as a key in the phrase dictionary.
2. **No Grammatical Agreement:** The tool will not automatically adjust adjectives for gender or number (e.g., "The red cars" might translate to "El rojo coches" instead of "Los coches rojos") unless specific POS (Part-of-Speech) rules are manually coded.
3. **Out-of-Vocabulary (OOV):** Proper nouns, slang, or highly technical terms not in the bundled dictionary will be passed through untranslated.

---

## 7. Next Steps for Development

1. **Assemble the Data:** Create a base `dictionary.json` (EN↔ES) and `synonyms.json` (EN synonyms, ES synonyms). Start small (e.g., top 1,000 most common words and 100 common phrases) to prove the concept.
2. **Build the Heuristic Detector:** Write the 30-line stopword/character scoring function.
3. **Build the N-gram Translator:** Write the logic that replaces longest matching phrases first, then falls back to single words.
4. **Package:** Bundle the JSON files into the executable using your chosen language's asset-embedding features.

Would you like a **Python starter script** demonstrating the heuristic language detection and the N-gram dictionary translation logic?
