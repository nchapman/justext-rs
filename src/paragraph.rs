use std::collections::HashSet;

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
    /// Context-free classification before neighbor-based revision.
    pub initial_class: ClassType,
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
            initial_class: ClassType::Short,
            heading: false,
        }
    }

    /// Returns `true` if this paragraph is classified as boilerplate.
    pub fn is_boilerplate(&self) -> bool {
        self.class_type != ClassType::Good
    }

    /// Returns `true` if the dom_path contains a heading tag (h0-h9).
    ///
    /// dom_path is dot-separated ASCII tags (e.g. "html.body.div.h1").
    /// A segment matches if it is exactly two bytes: `h` followed by an ASCII digit.
    /// This mirrors Python's `\bh\d\b` regex where dots act as word boundaries.
    pub fn is_heading(&self) -> bool {
        self.dom_path.split('.').any(|seg| {
            let b = seg.as_bytes();
            b.len() == 2 && b[0] == b'h' && b[1].is_ascii_digit()
        })
    }

    /// Link density: chars_count_in_links / text char count. Returns 0.0 if text is empty.
    ///
    /// Uses Unicode codepoint count (not byte length) to match Python's `len()` semantics.
    pub fn links_density(&self) -> f64 {
        let char_count = self.text.chars().count();
        if char_count == 0 {
            0.0
        } else {
            self.chars_count_in_links as f64 / char_count as f64
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
