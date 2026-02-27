// Integration tests: end-to-end HTML → classified paragraphs
// Ports test_core.py and provides additional real-world coverage.

use justext::{extract_text, get_stoplist, justext, ClassType, Config};

fn english() -> std::collections::HashSet<String> {
    get_stoplist("English").unwrap()
}

// --- Port of test_core.py ---

#[test]
fn test_words_split_by_br_tag() {
    // Single <br> inserts a space; does NOT create a paragraph boundary.
    let paragraphs = justext(
        "abc<br/>def becoming abcdef",
        &english(),
        &Config::default(),
    );
    let texts: Vec<&str> = paragraphs.iter().map(|p| p.text.as_str()).collect();
    assert_eq!(texts, vec!["abc def becoming abcdef"]);
}

// --- Basic pipeline tests ---

#[test]
fn test_empty_html() {
    let ps = justext("<html><body></body></html>", &english(), &Config::default());
    assert!(ps.is_empty());
}

#[test]
fn test_single_good_paragraph() {
    // Paragraph must be >200 chars with high stopword density to be classified
    // directly as Good (not just NearGood) without needing neighbor context.
    let text = "This is a sentence that contains many common English stopwords and it \
                should be classified as good content by the algorithm because the text is \
                long enough that it exceeds the length_high threshold of two hundred characters.";
    assert!(text.len() > 200, "test text must exceed length_high=200");
    let html = format!("<html><body><p>{text}</p></body></html>");
    let ps = justext(&html, &english(), &Config::default());
    assert!(!ps.is_empty());
    assert_eq!(ps[0].class_type, ClassType::Good);
}

#[test]
fn test_boilerplate_nav_links() {
    // Dense link text → Bad
    let html = concat!(
        "<html><body>",
        "<p><a>Home</a> | <a>About</a> | <a>Contact</a> | <a>Privacy</a> | <a>Terms</a></p>",
        "</body></html>"
    );
    let ps = justext(html, &english(), &Config::default());
    assert!(!ps.is_empty());
    for p in &ps {
        assert_eq!(
            p.class_type,
            ClassType::Bad,
            "nav link paragraph should be Bad: {:?}",
            p.text
        );
    }
}

#[test]
fn test_extract_text_returns_only_good() {
    // Text must be >200 chars to be classified Good without neighbor context.
    let good = "This is a content paragraph with many common stopwords and it is long enough \
                to exceed the length_high threshold so that it will be classified as good \
                content worth extracting by the justext algorithm when applied here.";
    assert!(good.len() > 200);
    let html = format!(
        "<html><body>\
         <p><a>nav link boilerplate here</a></p>\
         <p>{good}</p>\
         </body></html>"
    );
    let text = extract_text(&html, &english(), &Config::default());
    assert!(
        text.contains("content paragraph"),
        "extracted text should contain good content"
    );
}

#[test]
fn test_language_independent_mode() {
    // Empty stoplist + thresholds at 0 → classify by length alone.
    // Long paragraph must be >200 chars to get Good directly (not just NearGood).
    let config = Config::default()
        .with_stopwords_low(0.0)
        .with_stopwords_high(0.0);
    let stoplist = std::collections::HashSet::new();
    let long = "This paragraph is long enough to exceed both the length_low and the \
                length_high thresholds so it will be classified as good content by the \
                algorithm even when using an empty stoplist in language independent mode.";
    assert!(long.len() > 200);
    let html = format!("<html><body><p>Short.</p><p>{long}</p></body></html>");
    let ps = justext(&html, &stoplist, &config);
    assert_eq!(ps.len(), 2);
    // "Short." → length < 70 → Short → revised to Bad (neighbor edge defaults)
    assert_eq!(ps[0].class_type, ClassType::Bad);
    // Long paragraph → stopword_density=0 >= stopwords_high=0, length>200 → Good
    assert_eq!(ps[1].class_type, ClassType::Good);
}

#[test]
fn test_copyright_paragraph_is_bad() {
    let html = "<html><body><p>\u{00A9} 2024 Example Corp. All rights reserved.</p></body></html>";
    let ps = justext(html, &english(), &Config::default());
    assert!(!ps.is_empty());
    assert_eq!(ps[0].class_type, ClassType::Bad);
}

#[test]
fn test_heading_near_content_is_promoted() {
    // Content paragraph must be >200 chars with high stopwords to be Good without neighbors.
    let content = "This paragraph contains many common English stopwords and it is long \
                   enough to be classified as good content with the English stoplist applied \
                   correctly by the justext algorithm when processing this article text here.";
    assert!(content.len() > 200);
    let html = format!(
        "<html><body>\
         <h1>Article Title</h1>\
         <p>{content}</p>\
         </body></html>"
    );
    let ps = justext(&html, &english(), &Config::default());
    assert!(ps.len() >= 2);
    // Content paragraph should be Good (long + high stopword density)
    assert_eq!(ps[1].class_type, ClassType::Good);
    // Heading near a Good paragraph should be promoted (not Bad)
    assert_ne!(
        ps[0].class_type,
        ClassType::Bad,
        "heading near good content should not be Bad"
    );
}

#[test]
fn test_paragraph_struct_fields() {
    let html = "<html><body><h2>My Heading</h2></body></html>";
    let ps = justext(html, &english(), &Config::default());
    assert!(!ps.is_empty());
    let h = &ps[0];
    assert_eq!(h.text, "My Heading");
    assert!(h.dom_path.contains("h2"), "dom_path should contain h2");
    assert!(h.xpath.contains("h2"), "xpath should contain h2");
    assert!(h.heading, "h2 paragraph should have heading=true");
}
