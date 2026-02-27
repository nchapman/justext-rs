# justext-rs — Porting Guide

## Port Source
Python JusText v3.0.2 at `/Users/nchapman/Drive/Code/lessisbetter/refs/jusText`.
The Go port (`refs/justext-go`) is reference-only — no tests, one stoplist.

## Porting Philosophy
1. **Faithful to Python first** — same algorithm, same thresholds, same edge cases
2. **Idiomatic Rust second** — use Rust patterns where they improve clarity
3. **Performance third** — avoid gratuitous allocations, but correctness > speed

## File Mapping
| Rust file | Python source |
|-----------|---------------|
| `src/paragraph.rs` | `justext/paragraph.py` |
| `src/paragraph_maker.rs` | `ParagraphMaker` + `PathInfo` in `justext/core.py` |
| `src/classify.rs` | `classify_paragraphs()` in `justext/core.py` |
| `src/revise.rs` | `revise_paragraph_classification()` in `justext/core.py` |
| `src/preprocess.rs` | `preprocessor()` in `justext/core.py` |
| `src/stoplists/mod.rs` | `justext/utils.py` stoplist functions |
| `src/lib.rs` | `justext()` in `justext/core.py` |

## Conventions
- **DOM walk instead of SAX**: Python uses SAX streaming for paragraph extraction. We use a recursive DOM tree walk (scraper's ego-tree) — matches readability-rs patterns.
- **No `unwrap()` in library code** — only in tests and `LazyLock` regex compilation.
- **Use `std::sync::LazyLock`** (stable in 1.80) for cached statics, not `once_cell`.
- **Stoplists embedded** via `include_str!` at compile time — all 100 languages.
- **No encoding detection** — our API accepts `&str` (already-decoded UTF-8).

## Build & Test
```bash
cargo test          # all tests
cargo clippy        # no warnings
cargo fmt --check   # formatted
cargo bench         # benchmarks (after Phase 8)
```
