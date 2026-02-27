# Plan: justext-rs — Rust Port of JusText Boilerplate Classifier

## Context

[JusText](https://github.com/miso-belica/jusText) is a paragraph-level boilerplate removal algorithm. Unlike readability (which scores DOM subtrees), JusText classifies individual text paragraphs using stopword density, link density, and text length — then refines classifications using neighbor context. This makes it a strong complementary fallback to readability, especially on pages without clear `<article>` structure.

**Why we need it:** trafilatura-rs currently only has readability-rs as a fallback extractor. Python trafilatura uses JusText as a second fallback after readability, and this gap affects extraction quality.

**Port source:** Python original (v3.0.2) at `/Users/nchapman/Drive/Code/lessisbetter/refs/jusText`. The Go port (`refs/justext-go`) has no tests and only ships one stoplist — useful for reference but not as the primary source.

## Architecture

### Crate: `justext` at `/Users/nchapman/Drive/Code/lessisbetter/justext-rs`

Standalone library, same family as readability-rs and trafilatura-rs.

### File Layout

```
justext-rs/
├── Cargo.toml
├── CLAUDE.md
├── README.md
├── .rustfmt.toml            # edition = "2021"
├── clippy.toml               # msrv = "1.80"
├── src/
│   ├── lib.rs                # Public API, re-exports, crate docs
│   ├── error.rs              # Error enum
│   ├── paragraph.rs          # Paragraph struct + ClassType enum
│   ├── paragraph_maker.rs    # HTML → paragraph segmentation (DOM walk)
│   ├── classify.rs           # Context-free classification
│   ├── revise.rs             # Context-sensitive revision
│   ├── preprocess.rs         # HTML cleaning before segmentation
│   └── stoplists/
│       ├── mod.rs            # Stoplist loading API (include_str! embedded)
│       ├── English.txt       # \
│       ├── French.txt        #  > all 100 .txt files from Python source
│       └── ...               # /
├── tests/
│   ├── classify.rs           # Port of test_classify_paragraphs.py
│   ├── paragraph_maker.rs    # Port of test_sax.py + test_paths.py
│   ├── preprocess.rs         # Port of test_dom_utils.py
│   ├── utils.rs              # Port of test_utils.py (whitespace, stoplists)
│   └── integration.rs        # End-to-end tests + test_core.py
└── benches/
    └── extraction.rs
```

### Public API

```rust
/// Classify paragraphs in HTML as content or boilerplate.
pub fn justext(html: &str, stoplist: &HashSet<String>, config: &Config) -> Vec<Paragraph>

/// Convenience: extract only the good paragraph text.
pub fn extract_text(html: &str, stoplist: &HashSet<String>, config: &Config) -> String

/// Configuration with defaults matching Python JusText 3.0.2.
#[non_exhaustive]
pub struct Config { /* see Phase 6 */ }

/// A classified text paragraph.
#[non_exhaustive]
pub struct Paragraph {
    pub dom_path: String,
    pub xpath: String,              // XPath with ordinals, e.g. "/html[1]/body[1]/div[2]/p[1]"
    pub text: String,
    pub words_count: usize,
    pub chars_count_in_links: usize,
    pub tags_count: usize,
    pub class_type: ClassType,
    pub cf_class: ClassType,
    pub heading: bool,
}

#[derive(PartialEq, Eq)]
pub enum ClassType { Good, Bad, Short, NearGood }

impl Paragraph {
    pub fn is_boilerplate(&self) -> bool { self.class_type != ClassType::Good }
    pub fn is_heading(&self) -> bool { /* \bh\d\b in dom_path (matches h0-h9) */ }
}

/// Stoplist helpers
pub fn get_stoplist(language: &str) -> Option<HashSet<String>>
pub fn get_all_stoplists() -> HashSet<String>
pub fn available_languages() -> Vec<&'static str>
```

### Dependencies

Minimal — JusText is a simpler algorithm than readability:
- `scraper` + `ego-tree` — HTML parsing + DOM traversal (same as readability-rs)
- `thiserror` — error types
- `tracing` (optional) — debug logging
- `pretty_assertions` (dev) — test diffs
- `criterion` (dev) — benchmarks

- `regex` — heading detection (`\bh\d\b` pattern in dom_path)
- `once_cell` or `std::sync::LazyLock` — cached stoplists

**Not needed:** `url`, `serde_json`, `dateparser`, `chrono` (no URL resolution, no JSON-LD, no dates)

**Encoding:** Python has extensive `decode_html` with charset meta-tag detection and encoding fallback cascades. We deliberately omit this — our API accepts `&str` (already-decoded UTF-8). The consumer (trafilatura-rs) handles encoding detection before calling us.

## Implementation Phases

### Phase 1: Project Scaffold + Stoplists

Create the project and get stoplists working first — they're the foundation everything depends on.

**Scaffold:**
- `cargo init --lib` in `justext-rs/`
- Cargo.toml matching readability-rs conventions (edition 2021, MSRV 1.80, Apache-2.0)
- `.rustfmt.toml`, `clippy.toml`
- `CLAUDE.md` with porting philosophy referencing Python source
- `error.rs` with `JustextError` enum
- Stub `lib.rs`

**Stoplists (`src/stoplists/mod.rs`):**
- Copy all 100 `.txt` files from `/Users/nchapman/Drive/Code/lessisbetter/refs/jusText/justext/stoplists/`
- `include_str!` each file at compile time
- `get_stoplist(language: &str) -> Option<HashSet<String>>` — match by name (case-insensitive), parse one-word-per-line, lowercase
- `get_all_stoplists() -> HashSet<String>` — merge all stoplists, cache via `LazyLock`
- `available_languages() -> Vec<&'static str>` — return list of supported language names

**Tests for Phase 1** (port of `test_utils.py` stoplist tests):
- `test_get_stopwords_list` — verify all 100 languages are available
- `test_get_real_stoplist` — load a real stoplist and verify contents
- `test_get_missing_stoplist` — returns `None` for unknown language

### Phase 2: Paragraph Struct

**`paragraph.rs`** — Port of `justext/paragraph.py`:

```rust
pub enum ClassType { Good, Bad, Short, NearGood }

pub struct Paragraph {
    pub dom_path: String,           // "body.div.p" format (dot-separated, no ordinals)
    pub xpath: String,              // "/html[1]/body[1]/div[2]/p[1]" (slash-separated with ordinals)
    pub text: String,               // Normalized combined text (computed during paragraph construction)
    pub words_count: usize,         // Whitespace-split word count (cached, not recomputed)
    pub chars_count_in_links: usize,
    pub tags_count: usize,          // Non-block-level tags within paragraph
    pub class_type: ClassType,      // Final classification (set by revise)
    pub cf_class: ClassType,        // Context-free classification (set by classify)
    pub heading: bool,              // True if dom_path matches \bh\d\b
}
```

**PathInfo** helper (lives in `paragraph_maker.rs`):
- Tracks current DOM position during tree walk
- `dom` property → dot-separated path without ordinals (e.g., `"html.body.div.p"`)
- `xpath` property → slash-separated path with ordinals (e.g., `"/html[1]/body[1]/div[2]/p[1]"`)
- `append(tag_name)` → push element, tracking sibling ordinals
- `pop()` → remove last element

Methods:
- `is_boilerplate()` → `class_type != Good`
- `is_heading()` → checks dom_path with `\bh\d\b` regex (matches h0-h9, not just h1-h6)
- `stopwords_density(stoplist)` → stopword_count / words_count (returns 0.0 if no words)
- `links_density()` → chars_count_in_links / text.len() (returns 0.0 if text is empty)
- `stopwords_count(stoplist)` → count words present in stoplist (each word lowercased before lookup)

### Phase 3: Preprocessing + Paragraph Segmentation

**`preprocess.rs`** — Port of Python's `preprocessor()`:
- Parse HTML with scraper
- Remove these elements and all their children:
  - `<script>` — scripts
  - `<style>` — stylesheets
  - `<head>` — document head
  - `<form>`, `<input>`, `<button>`, `<select>`, `<textarea>` — form elements (Python uses lxml Cleaner `forms=True` which removes the full form family)
  - `<embed>`, `<object>`, `<applet>` — embedded content
- Remove HTML comments
- Return cleaned scraper `Html` document

**Tests for Phase 3** (port of `test_dom_utils.py`):
- `test_remove_comments` — HTML comments are stripped
- `test_remove_head_tag` — `<head>` and contents removed
- `test_preprocess_unicode` — basic Unicode HTML preprocessing

**`paragraph_maker.rs`** — Port of `ParagraphMaker` from `justext/core.py`:

Python uses SAX streaming; we'll use a **recursive DOM tree walk** instead (matches readability-rs patterns, simpler in Rust).

Block-level tags that create paragraph boundaries (PARAGRAPH_TAGS):
```
body, blockquote, caption, center, col, colgroup, dd, div, dl, dt,
fieldset, form, legend, optgroup, option, p, pre, table, td, textarea,
tfoot, th, thead, tr, ul, li, h1, h2, h3, h4, h5, h6
```

Walk algorithm:
- Maintain a "current paragraph" accumulator
- When entering a block-level tag: flush current paragraph, start new one, push tag to dom_path
- When exiting a block-level tag: flush current paragraph, pop from dom_path
- Text nodes: append to current paragraph, track chars (and chars-in-links if inside `<a>`)
- Inline tags (not in PARAGRAPH_TAGS): increment `tags_count`
- `<br>` handling:
  - Single `<br>`: appends a space to current paragraph, sets `br` flag, increments `tags_count`
  - Second consecutive `<br>`: triggers paragraph boundary, **decrements** `tags_count` by 1 (undoes the first `<br>`'s increment)
  - The `br` flag is tracked on the maker instance (not per-paragraph) and resets on any non-blank text or paragraph boundary
- `<a>` tag handling: set `link` flag on open, clear on close; while set, `chars_count_in_links` accumulates normalized text length
- Whitespace normalization (`normalize_whitespace`):
  - Runs of whitespace containing `\n` or `\r` → collapse to single `\n`
  - Other whitespace runs (including `\u{00A0}` no-break space, `\u{202F}` narrow no-break space) → collapse to single space
  - Blank text (all-whitespace or empty) is skipped entirely — not appended
- Final paragraph text is joined from text_nodes, normalized, and stripped
- Skip empty paragraphs (no text nodes appended = `contains_text()` is false)

**Tests for Phase 3** (port of `test_sax.py` + `test_paths.py` + `test_core.py`):
- `test_no_paragraphs` — empty body yields 0 paragraphs
- `test_basic` — h1 + p + p yields 3 paragraphs with correct text, word count, tag count
- `test_whitespace_handling` — inline elements, whitespace normalization across tags
- `test_multiple_line_break` — `<br><br>` creates paragraph boundary
- `test_inline_text_in_body` — inline text and newline normalization
- `test_links` — `<a>` tags correctly count `chars_count_in_links`
- `test_words_split_by_br` — single `<br>` inserts space between words (from `test_core.py`)
- `test_path_construction` — PathInfo dom/xpath with sibling ordinals
- `test_path_pop` — PathInfo pop behavior

### Phase 4: Context-Free Classification

**`classify.rs`** — Port of `classify_paragraphs()` from `justext/core.py`:

```rust
pub fn classify_paragraphs(paragraphs: &mut [Paragraph], stoplist: &HashSet<String>, config: &Config)
```

Decision tree per paragraph (applied in order — first match wins):
1. `links_density() > config.max_link_density` → **Bad**
2. Text contains `'\u{00A9}'` (©) OR literal string `"&copy"` (un-decoded HTML entity) → **Bad**
3. `dom_path` contains `"select"` → **Bad**
4. `text.len() < config.length_low`:
   - If `chars_count_in_links > 0` → **Bad**
   - Else → **Short**
5. `stopwords_density(stoplist) >= config.stopwords_high`:
   - If `text.len() > config.length_high` → **Good**
   - Else → **NearGood**
6. `stopwords_density(stoplist) >= config.stopwords_low` → **NearGood**
7. Else → **Bad**

Also: if `!config.no_headings` and `is_heading()` returns true (`\bh\d\b` in dom_path), set `heading = true`.

Sets `cf_class` on each paragraph (the pre-revision classification).

**Tests for Phase 4** (port of `test_classify_paragraphs.py`):
- `test_max_link_density` — high link density → bad; short with no links → short
- `test_length_low` — below length_low: no links → short, with links → bad
- `test_stopwords_high` — high stopwords: short text → neargood, long text → good
- `test_stopwords_low` — between low/high → neargood; below low → bad
- Test copyright symbol and `&copy` literal detection
- Test `select` in dom_path detection

### Phase 5: Context-Sensitive Revision

**`revise.rs`** — Port of `revise_paragraph_classification()` from `justext/core.py`:

```rust
pub fn revise_paragraph_classification(paragraphs: &mut [Paragraph], max_heading_distance: usize)
```

Four stages (order matters). First, copy `cf_class` to `class_type` for all paragraphs, then mutate `class_type`:

**Stage 1 — Promote short headings near good blocks:**
For each paragraph where `heading == true` and `class_type == Short`:
- Scan forward from `i+1`, accumulating `distance` as `len(paragraph.text)` of each intervening paragraph
- If a `Good` paragraph is found within `max_heading_distance` chars → set `class_type = NearGood`
- Stop scanning if distance exceeds `max_heading_distance`

**Stage 2 — Classify short blocks by neighbors (BATCHED):**
**Critical:** Collect new classifications in a HashMap, apply **after** the loop completes. This prevents one reclassification from affecting neighbor lookups for subsequent short paragraphs in the same pass.

For each paragraph where `class_type == Short`:
- Find prev/next non-short neighbor with `ignore_neargood=true` (skips both Short and NearGood)
- Both Good → **Good**
- Both Bad → **Bad**
- Mixed (one Good, one Bad) — neargood proximity exception:
  - Re-check the bad side with `ignore_neargood=false` (now NearGood is visible)
  - If the bad side is actually NearGood → **Good**
  - Otherwise → **Bad**

After the loop, apply all collected classifications at once.

**Stage 3 — Classify NearGood blocks (NOT batched — immediate):**
For each paragraph where `class_type == NearGood`:
- Find prev/next non-short neighbor with `ignore_neargood=true`
- Both Bad → **Bad**
- Otherwise (at least one Good) → **Good**

Changes apply immediately during iteration. Since neighbor lookups skip NearGood (`ignore_neargood=true`), already-processed NearGood paragraphs won't cause cascading effects.

**Stage 4 — Promote non-bad headings near good blocks:**
For each paragraph where `heading == true`, `class_type == Bad`, and `cf_class != Bad`:
- Scan forward, accumulating distance (same as Stage 1)
- If a `Good` paragraph is found within `max_heading_distance` → set `class_type = Good` (note: **Good**, not NearGood — differs from Stage 1)

Helper functions:
- `get_prev_neighbour(i, paragraphs, ignore_neargood) -> ClassType` — walk backward, skip Short always, skip NearGood if `ignore_neargood`. **Returns `Bad` if no qualifying neighbor found** (not Option — Python defaults to `'bad'` at document edges).
- `get_next_neighbour(i, paragraphs, ignore_neargood) -> ClassType` — same logic walking forward.

### Phase 6: Top-Level API

**`lib.rs`** — Wire the pipeline:

```rust
pub fn justext(html: &str, stoplist: &HashSet<String>, config: &Config) -> Vec<Paragraph> {
    let doc = preprocess(html);
    let mut paragraphs = make_paragraphs(&doc);
    classify_paragraphs(&mut paragraphs, stoplist, config);
    revise_paragraph_classification(&mut paragraphs, config.max_heading_distance);
    paragraphs
}

pub fn extract_text(html: &str, stoplist: &HashSet<String>, config: &Config) -> String {
    justext(html, stoplist, config)
        .into_iter()
        .filter(|p| !p.is_boilerplate())
        .map(|p| p.text)
        .collect::<Vec<_>>()
        .join("\n")
}
```

**Config struct with builder methods:**
```rust
#[non_exhaustive]
pub struct Config {
    pub length_low: usize,           // default: 70
    pub length_high: usize,          // default: 200
    pub stopwords_low: f64,          // default: 0.30
    pub stopwords_high: f64,         // default: 0.32
    pub max_link_density: f64,       // default: 0.2
    pub max_heading_distance: usize, // default: 200
    pub no_headings: bool,           // default: false
}

impl Config {
    pub fn new() -> Self { /* defaults */ }
    pub fn with_length_low(mut self, n: usize) -> Self { ... }
    // ... etc
}
```

### Phase 7: Integration Tests + Revision Tests

Unit tests are written alongside each phase (see "Tests for Phase N" sections above). This phase adds integration tests and revision-specific tests.

**`tests/revise.rs`** — Revision algorithm tests:
- Test Stage 1: short heading near good → neargood
- Test Stage 2: short between two good → good, two bad → bad, mixed with neargood proximity
- Test Stage 2 batching: verify reclassifications don't cascade within the pass
- Test Stage 3: neargood with both bad neighbors → bad, with one good → good
- Test Stage 4: non-bad heading revised to bad, near good → promoted to good
- Test neighbor helpers: document edges default to bad

**`tests/integration.rs`**:
- End-to-end: HTML → classified paragraphs
- Test with English stoplist
- Test language-independent mode (empty stoplist with `stopwords_low=0, stopwords_high=0`)
- Real-world HTML snippets

### Phase 8: Documentation + Polish

- Crate-level `//!` doc example in `lib.rs`
- README with usage examples, language list, comparison to readability
- Benchmark with criterion (classify various HTML sizes)
- Code review
- `cargo publish --dry-run`

## Key Design Decisions

**DOM walk vs SAX streaming:** Python uses SAX events for paragraph extraction. We use a recursive DOM tree walk (scraper's ego-tree) instead — matches readability-rs patterns, simpler in Rust, no stateful handler needed.

**Stoplist embedding:** All 100 stoplists (~2.5MB) embedded via `include_str!` at compile time. WASM-compatible, zero runtime I/O. Can be feature-gated later if binary size becomes a concern.

**No DOM output:** JusText returns classified paragraphs as plain text, not cleaned HTML. The consumer (trafilatura-rs) wraps them in `<p>` tags. This matches Python behavior.

**Python-faithful port:** Mirror the Python algorithm exactly — same thresholds, same decision tree, same revision stages, same edge cases (batched Stage 2, `&copy` literal check, `\bh\d\b` heading pattern). The Go port is reference-only.

**`is_heading` matches `h0-h9`:** Python uses `\bh\d\b` which matches any single-digit heading tag in the dom_path. We preserve this behavior rather than restricting to h1-h6, since it's what the Python tests expect.

**Whitespace normalization preserves newlines:** This is a subtle but important detail — runs of whitespace containing `\n`/`\r` collapse to `\n`, not a space. This matches Python's `normalize_whitespace()` behavior.

## Verification

After each phase:
1. `cargo test` — all tests pass
2. `cargo clippy` — no warnings
3. `cargo fmt --check` — formatted

Final:
4. `cargo test --doc` — doc examples compile
5. `cargo bench` — benchmarks run
