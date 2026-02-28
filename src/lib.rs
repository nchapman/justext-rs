// Port of Python jusText v3.0.2

//! Paragraph-level boilerplate removal for HTML.
//!
//! `justext` classifies HTML paragraphs as content or boilerplate using
//! stopword density, link density, and text length — then refines
//! classifications using neighbor context.
//!
//! # Quick start
//!
//! ```rust
//! use justext::{extract_text_lang, Config};
//!
//! let html = "<html><body><p>This is the main content.</p></body></html>";
//! let text = extract_text_lang(html, "English", &Config::default()).unwrap();
//! println!("{text}");
//! ```
//!
//! # Related crates
//!
//! - [`trafilatura`](https://crates.io/crates/trafilatura) — full-featured web
//!   content extraction with metadata, comments, and fallback strategies.
//! - [`libreadability`](https://crates.io/crates/libreadability) — Mozilla Readability
//!   port for extracting a clean article DOM subtree.
//! - [`html2markdown`](https://crates.io/crates/html2markdown) — converts HTML to
//!   Markdown via an intermediate AST.

mod classify;
mod error;
mod paragraph;
mod paragraph_maker;
mod preprocess;
mod revise;
pub mod stoplists;

pub use error::JustextError;
pub use paragraph::{ClassType, Paragraph};
pub use stoplists::{available_languages, get_all_stoplists, get_stoplist};

use std::collections::HashSet;

/// Configuration for the JusText algorithm.
///
/// Defaults match Python JusText 3.0.2.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Config {
    pub length_low: usize,
    pub length_high: usize,
    pub stopwords_low: f64,
    pub stopwords_high: f64,
    pub max_link_density: f64,
    pub max_heading_distance: usize,
    pub no_headings: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            length_low: 70,
            length_high: 200,
            stopwords_low: 0.30,
            stopwords_high: 0.32,
            max_link_density: 0.2,
            max_heading_distance: 200,
            no_headings: false,
        }
    }
}

impl Config {
    pub fn with_length_low(mut self, n: usize) -> Self {
        self.length_low = n;
        self
    }
    pub fn with_length_high(mut self, n: usize) -> Self {
        self.length_high = n;
        self
    }
    pub fn with_stopwords_low(mut self, v: f64) -> Self {
        self.stopwords_low = v;
        self
    }
    pub fn with_stopwords_high(mut self, v: f64) -> Self {
        self.stopwords_high = v;
        self
    }
    pub fn with_max_link_density(mut self, v: f64) -> Self {
        self.max_link_density = v;
        self
    }
    pub fn with_max_heading_distance(mut self, n: usize) -> Self {
        self.max_heading_distance = n;
        self
    }
    pub fn with_no_headings(mut self, v: bool) -> Self {
        self.no_headings = v;
        self
    }
}

/// Classify paragraphs in HTML as content or boilerplate.
pub fn justext(html: &str, stoplist: &HashSet<String>, config: &Config) -> Vec<Paragraph> {
    let doc = preprocess::preprocess(html);
    let mut paragraphs = paragraph_maker::make_paragraphs(&doc);
    classify::classify_paragraphs(&mut paragraphs, stoplist, config);
    revise::revise_paragraph_classification(&mut paragraphs, config.max_heading_distance);
    paragraphs
}

/// Convenience: extract only the good paragraph text.
pub fn extract_text(html: &str, stoplist: &HashSet<String>, config: &Config) -> String {
    justext(html, stoplist, config)
        .into_iter()
        .filter(|p| !p.is_boilerplate())
        .map(|p| p.text)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Classify paragraphs using a language name instead of a pre-loaded stoplist.
///
/// Equivalent to `get_stoplist(language)` followed by `justext()`.
///
/// # Example
///
/// ```rust
/// let paragraphs = justext::justext_lang("<html><body><p>Hello world</p></body></html>", "English", &justext::Config::default()).unwrap();
/// ```
pub fn justext_lang(
    html: &str,
    language: &str,
    config: &Config,
) -> Result<Vec<Paragraph>, JustextError> {
    let stoplist = get_stoplist(language)?;
    Ok(justext(html, &stoplist, config))
}

/// Extract only the good paragraph text using a language name.
///
/// Equivalent to `get_stoplist(language)` followed by `extract_text()`.
///
/// # Example
///
/// ```rust
/// let text = justext::extract_text_lang("<html><body><p>Hello world</p></body></html>", "English", &justext::Config::default()).unwrap();
/// ```
pub fn extract_text_lang(
    html: &str,
    language: &str,
    config: &Config,
) -> Result<String, JustextError> {
    let stoplist = get_stoplist(language)?;
    Ok(extract_text(html, &stoplist, config))
}
