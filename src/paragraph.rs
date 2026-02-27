use std::collections::HashSet;
use std::sync::LazyLock;

use regex::Regex;

/// Regex matching heading tags in a dom_path (h0-h9).
static HEADING_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bh\d\b").unwrap());

/// Classification label for a paragraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassType {
    Good,
    Bad,
    Short,
    NearGood,
}

/// A classified text paragraph extracted from HTML.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Paragraph {
    /// Dot-separated DOM path without ordinals (e.g., "body.div.p").
    pub dom_path: String,
    /// XPath with ordinals (e.g., "/html[1]/body[1]/div[2]/p[1]").
    pub xpath: String,
    /// Normalized text content.
    pub text: String,
    /// Word count (whitespace-split).
    pub words_count: usize,
    /// Character count inside `<a>` tags.
    pub chars_count_in_links: usize,
    /// Count of inline (non-block-level) tags within this paragraph.
    pub tags_count: usize,
    /// Final classification (set by revision stage).
    pub class_type: ClassType,
    /// Context-free classification (set by classification stage).
    pub cf_class: ClassType,
    /// Whether this paragraph is a heading.
    pub heading: bool,
}

impl Paragraph {
    /// Create a new paragraph with the given path and text.
    pub(crate) fn new(
        dom_path: String,
        xpath: String,
        text: String,
        chars_count_in_links: usize,
        tags_count: usize,
    ) -> Self {
        let words_count = text.split_whitespace().count();
        Self {
            dom_path,
            xpath,
            text,
            words_count,
            chars_count_in_links,
            tags_count,
            class_type: ClassType::Short,
            cf_class: ClassType::Short,
            heading: false,
        }
    }

    /// Returns `true` if this paragraph is classified as boilerplate.
    pub fn is_boilerplate(&self) -> bool {
        self.class_type != ClassType::Good
    }

    /// Returns `true` if the dom_path contains a heading tag (h0-h9).
    pub fn is_heading(&self) -> bool {
        HEADING_RE.is_match(&self.dom_path)
    }

    /// Link density: chars_count_in_links / text.len(). Returns 0.0 if text is empty.
    pub fn links_density(&self) -> f64 {
        let len = self.text.len();
        if len == 0 {
            0.0
        } else {
            self.chars_count_in_links as f64 / len as f64
        }
    }

    /// Count of words present in the stoplist (case-insensitive).
    pub fn stopwords_count(&self, stoplist: &HashSet<String>) -> usize {
        self.text
            .split_whitespace()
            .filter(|word| stoplist.contains(&word.to_lowercase()))
            .count()
    }

    /// Stopword density: stopwords_count / words_count. Returns 0.0 if no words.
    pub fn stopwords_density(&self, stoplist: &HashSet<String>) -> f64 {
        if self.words_count == 0 {
            0.0
        } else {
            self.stopwords_count(stoplist) as f64 / self.words_count as f64
        }
    }
}
