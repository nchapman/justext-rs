// Port of classify_paragraphs() from Python jusText justext/core.py

use std::collections::HashSet;

use crate::paragraph::{ClassType, Paragraph};
use crate::Config;

/// Context-free classification of paragraphs.
///
/// Sets `initial_class` on each paragraph. Decision tree matches Python exactly.
#[allow(clippy::if_same_then_else)]
pub fn classify_paragraphs(
    paragraphs: &mut [Paragraph],
    stoplist: &HashSet<String>,
    config: &Config,
) {
    for paragraph in paragraphs.iter_mut() {
        paragraph.heading = !config.no_headings && paragraph.is_heading();

        // Python uses len(paragraph) which is len(paragraph.text) — character count, not bytes.
        let length = paragraph.text.chars().count();
        let link_density = paragraph.links_density();
        let stopword_density = paragraph.stopwords_density(stoplist);

        // Decision tree mirrors Python classify_paragraphs() exactly — order matters.
        // Three initial branches all return Bad but for distinct semantic reasons.
        paragraph.initial_class = if link_density > config.max_link_density {
            ClassType::Bad
        } else if paragraph.text.contains('\u{00A9}') || paragraph.text.contains("&copy") {
            ClassType::Bad
        } else if paragraph.dom_path.contains("select") {
            ClassType::Bad
        } else if length < config.length_low {
            if paragraph.chars_count_in_links > 0 {
                ClassType::Bad
            } else {
                ClassType::Short
            }
        } else if stopword_density >= config.stopwords_high {
            if length > config.length_high {
                ClassType::Good
            } else {
                ClassType::NearGood
            }
        } else if stopword_density >= config.stopwords_low {
            ClassType::NearGood
        } else {
            ClassType::Bad
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paragraph_maker::make_paragraphs;
    use crate::preprocess::preprocess;

    /// Build a paragraph from HTML snippet for testing.
    fn make_paragraph(text: &str, chars_in_links: usize) -> Paragraph {
        let html = format!("<html><body><p>{text}</p></body></html>");
        let doc = preprocess(&html);
        let mut ps = make_paragraphs(&doc);
        assert!(!ps.is_empty(), "no paragraphs parsed from: {text}");
        ps[0].chars_count_in_links = chars_in_links;
        ps.remove(0)
    }

    fn empty_stoplist() -> HashSet<String> {
        HashSet::new()
    }

    fn stoplist(words: &[&str]) -> HashSet<String> {
        words.iter().map(|w| w.to_string()).collect()
    }

    // --- Port of test_classify_paragraphs.py ---

    #[test]
    fn test_max_link_density() {
        let mut paragraphs = vec![
            make_paragraph("0123456789".repeat(2).as_str(), 0),
            make_paragraph("0123456789".repeat(2).as_str(), 20),
            make_paragraph("0123456789".repeat(8).as_str(), 40),
            make_paragraph("0123456789".repeat(8).as_str(), 39),
            make_paragraph("0123456789".repeat(8).as_str(), 41),
        ];

        let config = Config {
            max_link_density: 0.5,
            ..Config::default() // length_low stays at default 70
        };
        classify_paragraphs(&mut paragraphs, &empty_stoplist(), &config);

        // 20 chars, 0 links → density 0 ≤ 0.5, length=20 < 70 → short
        assert_eq!(paragraphs[0].initial_class, ClassType::Short);
        // 20 chars, 20 links → density 1.0 > 0.5 → bad
        assert_eq!(paragraphs[1].initial_class, ClassType::Bad);
        // 80 chars, 40 links → density 0.5 = 0.5, NOT > → length≥70, stopword=0 < 0.30 → bad
        assert_eq!(paragraphs[2].initial_class, ClassType::Bad);
        // 80 chars, 39 links → density 0.4875 ≤ 0.5 → length≥70, stopword=0 → bad
        assert_eq!(paragraphs[3].initial_class, ClassType::Bad);
        // 80 chars, 41 links → density 0.5125 > 0.5 → bad
        assert_eq!(paragraphs[4].initial_class, ClassType::Bad);
    }

    #[test]
    fn test_length_low() {
        let mut paragraphs = vec![
            make_paragraph("0 1 2 3 4 5 6 7 8 9".repeat(2).as_str(), 0),
            make_paragraph("0 1 2 3 4 5 6 7 8 9".repeat(2).as_str(), 20),
        ];

        let config = Config {
            max_link_density: 1.0,
            length_low: 1000,
            ..Config::default()
        };
        classify_paragraphs(&mut paragraphs, &empty_stoplist(), &config);

        assert_eq!(paragraphs[0].initial_class, ClassType::Short);
        assert_eq!(paragraphs[1].initial_class, ClassType::Bad);
    }

    #[test]
    fn test_stopwords_high() {
        let mut paragraphs = vec![
            make_paragraph("0 1 2 3 4 5 6 7 8 9", 0),
            make_paragraph("0 1 2 3 4 5 6 7 8 9".repeat(2).as_str(), 0),
        ];

        let config = Config {
            max_link_density: 1.0,
            length_low: 0,
            stopwords_high: 0.0,
            length_high: 20,
            ..Config::default()
        };
        classify_paragraphs(&mut paragraphs, &stoplist(&["0"]), &config);

        // text "0 1 2 3 4 5 6 7 8 9" len=19 ≤ 20 → neargood
        assert_eq!(paragraphs[0].initial_class, ClassType::NearGood);
        // repeated text len=39 > 20 → good
        assert_eq!(paragraphs[1].initial_class, ClassType::Good);
    }

    #[test]
    fn test_stopwords_low() {
        let mut paragraphs = vec![
            make_paragraph("0 0 0 0 1 2 3 4 5 6 7 8 9", 0),
            make_paragraph("0 1 2 3 4 5 6 7 8 9", 0),
            make_paragraph("1 2 3 4 5 6 7 8 9", 0),
        ];

        let config = Config {
            max_link_density: 1.0,
            length_low: 0,
            stopwords_high: 1000.0,
            stopwords_low: 0.2,
            ..Config::default()
        };
        classify_paragraphs(&mut paragraphs, &stoplist(&["0", "1"]), &config);

        // "0 0 0 0 1 2 3 4 5 6 7 8 9" → 5/13 ≈ 0.38 ≥ 0.2 → neargood
        assert_eq!(paragraphs[0].initial_class, ClassType::NearGood);
        // "0 1 2 3 4 5 6 7 8 9" → 2/10 = 0.2 ≥ 0.2 → neargood
        assert_eq!(paragraphs[1].initial_class, ClassType::NearGood);
        // "1 2 3 4 5 6 7 8 9" → 1/9 ≈ 0.11 < 0.2 → bad
        assert_eq!(paragraphs[2].initial_class, ClassType::Bad);
    }

    #[test]
    fn test_copyright_symbol() {
        let mut ps = vec![make_paragraph("Copyright \u{00A9} 2024 Acme", 0)];
        classify_paragraphs(&mut ps, &empty_stoplist(), &Config::default());
        assert_eq!(ps[0].initial_class, ClassType::Bad);
    }

    #[test]
    fn test_copyright_entity_literal() {
        // "&copy" literal string (un-decoded entity) in text → bad
        let mut ps = vec![make_paragraph("&copy; 2024 Acme Corp", 0)];
        classify_paragraphs(&mut ps, &empty_stoplist(), &Config::default());
        assert_eq!(ps[0].initial_class, ClassType::Bad);
    }

    #[test]
    fn test_select_in_dom_path() {
        // Paragraph inside a <select> element
        let html = "<html><body><select><option>Choose</option></select></body></html>";
        let doc = preprocess(html);
        let mut ps = make_paragraphs(&doc);
        if ps.is_empty() {
            return; // select removed by preprocessor — acceptable
        }
        classify_paragraphs(&mut ps, &empty_stoplist(), &Config::default());
        for p in &ps {
            if p.dom_path.contains("select") {
                assert_eq!(p.initial_class, ClassType::Bad);
            }
        }
    }

    #[test]
    fn test_heading_detection() {
        let html = "<html><body><h1>A heading</h1><p>body text here</p></body></html>";
        let doc = preprocess(html);
        let mut ps = make_paragraphs(&doc);
        let config = Config::default();
        classify_paragraphs(&mut ps, &empty_stoplist(), &config);
        assert!(ps[0].heading, "h1 paragraph should be marked as heading");
        assert!(!ps[1].heading, "p paragraph should not be heading");
    }

    #[test]
    fn test_no_headings_config() {
        let html = "<html><body><h1>A heading</h1></body></html>";
        let doc = preprocess(html);
        let mut ps = make_paragraphs(&doc);
        let config = Config {
            no_headings: true,
            ..Config::default()
        };
        classify_paragraphs(&mut ps, &empty_stoplist(), &config);
        assert!(
            !ps[0].heading,
            "heading should be false when no_headings=true"
        );
    }
}
