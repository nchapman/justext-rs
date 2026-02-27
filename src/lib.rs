// Port of Python jusText v3.0.2

//! Paragraph-level boilerplate removal for HTML.
//!
//! `justext` classifies HTML paragraphs as content or boilerplate using
//! stopword density, link density, and text length — then refines
//! classifications using neighbor context.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use justext::{justext, get_stoplist, Config};
//!
//! let html = "<html><body><p>This is the main content.</p></body></html>";
//! let stoplist = get_stoplist("English").unwrap();
//! let config = Config::default();
//! let paragraphs = justext(html, &stoplist, &config);
//!
//! for p in &paragraphs {
//!     if !p.is_boilerplate() {
//!         println!("{}", p.text);
//!     }
//! }
//! ```

mod error;
mod paragraph;
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

/// Classify paragraphs in HTML as content or boilerplate.
pub fn justext(_html: &str, _stoplist: &HashSet<String>, _config: &Config) -> Vec<Paragraph> {
    todo!("Phase 3-6: preprocess → make_paragraphs → classify → revise")
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
