# justext

[![Crates.io](https://img.shields.io/crates/v/justext.svg)](https://crates.io/crates/justext)
[![License: BSD-2-Clause](https://img.shields.io/badge/license-BSD--2--Clause-blue.svg)](LICENSE)
[![Rust: 1.80+](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org)

Paragraph-level boilerplate removal for HTML.

A Rust port of [JusText](https://github.com/miso-belica/jusText) (v3.0.2) — classifies every
paragraph in an HTML page as content or boilerplate using stopword density, link density, and
text length, then refines classifications using neighbor context.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
justext = "0.1"
```

```rust
use justext::{extract_text_lang, Config};

let html = r#"<html><body>
  <nav>Menu | About | Contact</nav>
  <article>
    <p>This is the main article body with enough text to be classified
    as content by the stopword density algorithm.</p>
  </article>
  <footer>Copyright 2024</footer>
</body></html>"#;

let text = extract_text_lang(html, "English", &Config::default()).unwrap();
println!("{text}");
```

For access to the full paragraph classification:

```rust
use justext::{justext_lang, Config};

let paragraphs = justext_lang(html, "English", &Config::default()).unwrap();

for p in &paragraphs {
    println!("{:?}  {}", p.class_type, p.text);
}
```

The `_lang` variants look up the stoplist internally. If you already have a stoplist,
use `justext()` / `extract_text()` directly:

```rust
use justext::{extract_text, get_stoplist, Config};

let stoplist = get_stoplist("English").unwrap();
let text = extract_text(html, &stoplist, &Config::default());
```

## How it works

Each paragraph goes through two stages:

1. **Context-free classification** — each paragraph is classified independently using link
   density, stopword density, and character length: `Good`, `Bad`, `NearGood`, or `Short`.

2. **Context-sensitive revision** — four passes use neighboring paragraph classes to promote
   or demote ambiguous paragraphs, resolving `Short` and `NearGood` into `Good` or `Bad`.

Paragraphs classified `Good` are content; everything else is boilerplate.

## What the paragraph struct contains

| Field | Description |
|-------|-------------|
| `text` | Normalized text content |
| `class_type` | Final classification (`Good`, `Bad`, `NearGood`, `Short`) |
| `initial_class` | Context-free classification (before revision) |
| `dom_path` | Dot-separated DOM path, e.g. `"body.div.p"` |
| `xpath` | XPath with ordinals, e.g. `"/html[1]/body[1]/div[2]/p[1]"` |
| `words_count` | Whitespace-split word count |
| `chars_count_in_links` | Character count inside `<a>` tags |
| `tags_count` | Count of inline tags within the paragraph |
| `heading` | Whether the paragraph is a heading (`h0`–`h9` in dom_path) |

## Stoplists

100 languages are bundled and embedded at compile time. Retrieve one by name
(case-insensitive):

```rust
let stoplist = get_stoplist("English").unwrap();   // HashSet<String>
let stoplist = get_stoplist("french").unwrap();    // case-insensitive
```

For language-independent extraction, pass an empty stoplist and zero thresholds:

```rust
use std::collections::HashSet;
use justext::{extract_text, Config};

let config = Config::default()
    .with_stopwords_low(0.0)
    .with_stopwords_high(0.0);

let text = extract_text(html, &HashSet::new(), &config);
```

To get the merged set of all stopwords across every language:

```rust
let all = get_all_stoplists(); // &'static HashSet<String>
```

Available languages:

Afrikaans, Albanian, Arabic, Aragonese, Armenian, Aromanian, Asturian, Azerbaijani,
Basque, Belarusian, Belarusian_Taraskievica, Bengali, Bishnupriya_Manipuri, Bosnian,
Breton, Bulgarian, Catalan, Cebuano, Chuvash, Croatian, Czech, Danish, Dutch, English,
Esperanto, Estonian, Finnish, French, Galician, Georgian, German, Greek, Gujarati,
Haitian, Hebrew, Hindi, Hungarian, Icelandic, Ido, Igbo, Indonesian, Irish, Italian,
Javanese, Kannada, Kazakh, Korean, Kurdish, Kyrgyz, Latin, Latvian, Lithuanian,
Lombard, Low_Saxon, Luxembourgish, Macedonian, Malay, Malayalam, Maltese, Marathi,
Neapolitan, Nepali, Newar, Norwegian_Bokmal, Norwegian_Nynorsk, Occitan, Persian,
Piedmontese, Polish, Portuguese, Quechua, Romanian, Russian, Samogitian, Serbian,
Serbo_Croatian, Sicilian, Simple_English, Slovak, Slovenian, Spanish, Sundanese,
Swahili, Swedish, Tagalog, Tamil, Telugu, Turkish, Turkmen, Ukrainian, Urdu, Uzbek,
Vietnamese, Volapuk, Walloon, Waray_Waray, Welsh, West_Frisian, Western_Panjabi, Yoruba

## Configuration

All parameters default to the Python JusText 3.0.2 values.

```rust
let config = Config::default()
    .with_length_low(70)           // min chars for a non-short paragraph
    .with_length_high(200)         // min chars for stopword-dense paragraph to be Good
    .with_stopwords_low(0.30)      // min stopword density for NearGood
    .with_stopwords_high(0.32)     // min stopword density for Good/NearGood branch
    .with_max_link_density(0.2)    // max link-char ratio before Bad
    .with_max_heading_distance(200)// max chars to scan ahead when promoting headings
    .with_no_headings(false);      // set true to disable heading detection
```

## Optional features

| Feature | Description |
|---------|-------------|
| `tracing` | Enable debug/trace logging (zero-cost when disabled) |

```toml
justext = { version = "0.1", features = ["tracing"] }
```

## Comparison to readability

| | justext | libreadability |
|---|---|---|
| Unit of extraction | Paragraphs | DOM subtree |
| Output | `Vec<Paragraph>` (plain text) | Cleaned HTML |
| Approach | Stopword/link density heuristics | DOM scoring |
| Works best on | Pages without clear `<article>` structure | Standard news/blog articles |

The two are complementary. [trafilatura](https://github.com/adbar/trafilatura) uses readability
first and falls back to JusText — this crate enables the same pattern in Rust.

## Benchmarks

Rust vs Python ([jusText](https://github.com/miso-belica/jusText)) — full pipeline (parse + classify + revise):

| Input | Rust | Python | Speedup |
|-------|-----:|-------:|--------:|
| small (2 paragraphs, 733 B) | 21 µs | 202 µs | **10x** |
| medium (20 paragraphs, 5 KB) | 98 µs | 1.4 ms | **14x** |
| large (100 paragraphs, 34 KB) | 604 µs | 11.1 ms | **18x** |

### Output comparison

On a 925-file dataset (the [trafilatura comparison corpus](https://github.com/adbar/trafilatura)),
Rust and Python produce identical extracted text on **99.4%** of files (919/925).

Measured on Apple M4 Max, Rust 1.93, macOS 15.7.

Reproduce:

```sh
cargo bench                                        # speed benchmarks
cargo run --bin compare -- <html-dir>              # Rust output (JSONL)
python3 scripts/compare_python.py --html-dir ...   # Python output (JSONL)
```

## License

BSD-2-Clause
