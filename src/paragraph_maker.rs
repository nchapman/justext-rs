// Port of ParagraphMaker + PathInfo from Python jusText justext/core.py

use std::collections::HashMap;

use ego_tree::NodeRef;
use scraper::node::Node;
use scraper::Html;

use crate::paragraph::Paragraph;

/// Tags that create paragraph boundaries when entered or exited.
const PARAGRAPH_TAGS: &[&str] = &[
    "body",
    "blockquote",
    "caption",
    "center",
    "col",
    "colgroup",
    "dd",
    "div",
    "dl",
    "dt",
    "fieldset",
    "form",
    "legend",
    "optgroup",
    "option",
    "p",
    "pre",
    "table",
    "td",
    "textarea",
    "tfoot",
    "th",
    "thead",
    "tr",
    "ul",
    "li",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
];

/// Returns true if `tag` is a paragraph-boundary tag.
fn is_paragraph_tag(tag: &str) -> bool {
    PARAGRAPH_TAGS.contains(&tag)
}

/// Tracks the current DOM path during the tree walk.
///
/// Maintains both a dot-separated `dom` path (no ordinals) and a
/// slash-separated `xpath` with per-level sibling ordinals, exactly
/// matching Python's `PathInfo` class.
#[derive(Default)]
pub(crate) struct PathInfo {
    /// Stack of (tag_name, ordinal, children_counts).
    elements: Vec<(String, usize, HashMap<String, usize>)>,
}

impl PathInfo {
    pub fn new() -> Self {
        Self::default()
    }

    /// Dot-separated path without ordinals, e.g. "html.body.div.p".
    pub fn dom(&self) -> String {
        self.elements
            .iter()
            .map(|(name, _, _)| name.as_str())
            .collect::<Vec<_>>()
            .join(".")
    }

    /// Slash-separated XPath with ordinals, e.g. "/html[1]/body[1]/div[2]/p[1]".
    pub fn xpath(&self) -> String {
        let parts: Vec<String> = self
            .elements
            .iter()
            .map(|(name, ord, _)| format!("{}[{}]", name, ord))
            .collect();
        format!("/{}", parts.join("/"))
    }

    /// Push a new element onto the path.
    pub fn push(&mut self, tag: &str) {
        // Extract ordinal before pushing (drops borrow on elements before the push).
        let order = if let Some((_, _, children)) = self.elements.last_mut() {
            let count = children.entry(tag.to_string()).or_insert(0);
            *count += 1;
            *count
        } else {
            1
        };
        self.elements.push((tag.to_string(), order, HashMap::new()));
    }

    /// Pop the top element from the path.
    pub fn pop(&mut self) {
        self.elements.pop();
    }
}

/// Normalizes whitespace in a text node, matching Python's `normalize_whitespace()`:
/// - Runs containing `\n` or `\r` collapse to `\n`
/// - Other whitespace runs (including no-break spaces) collapse to ` `
pub(crate) fn normalize_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_ws = false;
    let mut ws_has_newline = false;

    for ch in text.chars() {
        if ch.is_whitespace() {
            if !in_ws {
                in_ws = true;
                ws_has_newline = false;
            }
            if ch == '\n' || ch == '\r' {
                ws_has_newline = true;
            }
        } else {
            if in_ws {
                out.push(if ws_has_newline { '\n' } else { ' ' });
                in_ws = false;
            }
            out.push(ch);
        }
    }
    if in_ws {
        out.push(if ws_has_newline { '\n' } else { ' ' });
    }
    out
}

/// Returns true if the string is empty or all whitespace.
fn is_blank(s: &str) -> bool {
    s.chars().all(|c| c.is_whitespace())
}

/// Accumulates text nodes into a paragraph during the DOM walk.
struct ParagraphAccumulator {
    dom_path: String,
    xpath: String,
    text_nodes: Vec<String>,
    chars_count_in_links: usize,
    tags_count: usize,
}

impl ParagraphAccumulator {
    fn new(path: &PathInfo) -> Self {
        Self {
            dom_path: path.dom(),
            xpath: path.xpath(),
            text_nodes: Vec::new(),
            chars_count_in_links: 0,
            tags_count: 0,
        }
    }

    fn append_text(&mut self, text: &str) -> String {
        let normalized = normalize_whitespace(text);
        self.text_nodes.push(normalized.clone());
        normalized
    }

    fn contains_text(&self) -> bool {
        !self.text_nodes.is_empty()
    }

    fn build(self) -> Paragraph {
        let raw = self.text_nodes.join("");
        // Final strip after joining, matching Python's `text_nodes.join("").strip()`
        let text = normalize_whitespace(raw.trim());
        Paragraph::new(
            self.dom_path,
            self.xpath,
            text,
            self.chars_count_in_links,
            self.tags_count,
        )
    }
}

/// Walk state threaded through the recursive DOM walk.
struct Walker {
    path: PathInfo,
    paragraphs: Vec<Paragraph>,
    current: ParagraphAccumulator,
    link: bool,
    br: bool,
}

impl Walker {
    fn new() -> Self {
        let path = PathInfo::new();
        let current = ParagraphAccumulator::new(&path);
        Self {
            path,
            paragraphs: Vec::new(),
            current,
            link: false,
            br: false,
        }
    }

    /// Flush the current paragraph accumulator and start a new one.
    fn start_new_paragraph(&mut self) {
        let finished = std::mem::replace(&mut self.current, ParagraphAccumulator::new(&self.path));
        if finished.contains_text() {
            self.paragraphs.push(finished.build());
        }
        self.br = false;
    }

    fn visit_node(&mut self, node: NodeRef<Node>) {
        match node.value() {
            Node::Element(el) => {
                let tag = el.name();

                self.path.push(tag);

                if is_paragraph_tag(tag) {
                    self.start_new_paragraph();
                    // Recurse into children
                    for child in node.children() {
                        self.visit_node(child);
                    }
                    self.path.pop();
                    self.start_new_paragraph();
                } else if tag == "br" {
                    if self.br {
                        // Second consecutive <br>: paragraph boundary.
                        // Undo the tag_count increment from the first <br>.
                        self.current.tags_count = self.current.tags_count.saturating_sub(1);
                        self.path.pop();
                        self.start_new_paragraph();
                    } else {
                        // First <br>: insert a space, set br flag.
                        self.br = true;
                        let _ = self.current.append_text(" ");
                        self.current.tags_count += 1;
                        self.path.pop();
                    }
                } else {
                    // Inline tag
                    if tag == "a" {
                        self.link = true;
                    }
                    self.current.tags_count += 1;
                    self.br = false;

                    for child in node.children() {
                        self.visit_node(child);
                    }
                    self.path.pop();

                    if tag == "a" {
                        self.link = false;
                    }
                }
            }
            Node::Text(text) => {
                let content = text.text.as_ref();
                if is_blank(content) {
                    return;
                }
                let normalized = self.current.append_text(content);
                if self.link {
                    // Count Unicode codepoints, not bytes — matches Python's len() on str.
                    self.current.chars_count_in_links += normalized.chars().count();
                }
                self.br = false;
            }
            // Document / fragment: recurse into children
            Node::Document | Node::Fragment => {
                for child in node.children() {
                    self.visit_node(child);
                }
            }
            // Skip comments, doctypes, processing instructions
            _ => {}
        }
    }
}

/// Convert a preprocessed HTML document into a list of paragraphs.
///
/// Port of `ParagraphMaker.make_paragraphs()` from Python jusText.
pub fn make_paragraphs(doc: &Html) -> Vec<Paragraph> {
    let mut walker = Walker::new();
    walker.visit_node(doc.tree.root());
    // Flush any remaining paragraph (mirrors Python's endDocument handler)
    walker.start_new_paragraph();
    walker.paragraphs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preprocess::preprocess;

    fn parse(html: &str) -> Vec<Paragraph> {
        let doc = preprocess(html);
        make_paragraphs(&doc)
    }

    // --- Port of test_sax.py ---

    #[test]
    fn test_no_paragraphs() {
        let ps = parse("<html><body></body></html>");
        assert_eq!(ps.len(), 0);
    }

    #[test]
    fn test_basic() {
        let html = concat!(
            "<html><body>",
            "<h1>Header</h1>",
            "<p>text and some <em>other</em> words <span>that I</span> have in my head now</p>",
            "<p>footer</p>",
            "</body></html>"
        );
        let ps = parse(html);
        assert_eq!(ps.len(), 3);

        assert_eq!(ps[0].text, "Header");
        assert_eq!(ps[0].words_count, 1);
        assert_eq!(ps[0].tags_count, 0);

        assert_eq!(
            ps[1].text,
            "text and some other words that I have in my head now"
        );
        assert_eq!(ps[1].words_count, 12);
        assert_eq!(ps[1].tags_count, 2);

        assert_eq!(ps[2].text, "footer");
        assert_eq!(ps[2].words_count, 1);
        assert_eq!(ps[2].tags_count, 0);
    }

    #[test]
    fn test_whitespace_handling() {
        let html = concat!(
            "<html><body>",
            "<p>pre<em>in</em>post \t pre  <span class=\"class\"> in </span>  post</p>",
            "<div>pre<em> in </em>post</div>",
            "<pre>pre<em>in </em>post</pre>",
            "<blockquote>pre<em> in</em>post</blockquote>",
            "</body></html>"
        );
        let ps = parse(html);
        assert_eq!(ps.len(), 4);

        assert_eq!(ps[0].text, "preinpost pre in post");
        assert_eq!(ps[0].words_count, 4);
        assert_eq!(ps[0].tags_count, 2);

        assert_eq!(ps[1].text, "pre in post");
        assert_eq!(ps[1].words_count, 3);
        assert_eq!(ps[1].tags_count, 1);

        assert_eq!(ps[2].text, "prein post");
        assert_eq!(ps[2].words_count, 2);
        assert_eq!(ps[2].tags_count, 1);

        assert_eq!(ps[3].text, "pre inpost");
        assert_eq!(ps[3].words_count, 2);
        assert_eq!(ps[3].tags_count, 1);
    }

    #[test]
    fn test_multiple_line_break() {
        let html = concat!(
            "<html><body>",
            "  normal text   <br><br> another   text  ",
            "</body></html>"
        );
        let ps = parse(html);
        assert_eq!(ps.len(), 2);
        assert_eq!(ps[0].text, "normal text");
        assert_eq!(ps[0].words_count, 2);
        assert_eq!(ps[0].tags_count, 0);
        assert_eq!(ps[1].text, "another text");
        assert_eq!(ps[1].words_count, 2);
        assert_eq!(ps[1].tags_count, 0);
    }

    #[test]
    fn test_inline_text_in_body() {
        let html = concat!(
            "<html><body>",
            "<sup>I am <strong>top</strong>-inline\n\n\n\n and I am happy \n</sup>",
            "<p>normal text</p>",
            "<code>\nvar i = -INFINITY;\n</code>",
            "<div>after text with variable <var>N</var> </div>",
            "   I am inline\n\n\n\n and I am happy \n",
            "</body></html>"
        );
        let ps = parse(html);
        assert_eq!(ps.len(), 5);

        assert_eq!(ps[0].text, "I am top-inline\nand I am happy");
        assert_eq!(ps[0].words_count, 7);
        assert_eq!(ps[0].tags_count, 2);

        assert_eq!(ps[1].text, "normal text");
        assert_eq!(ps[1].words_count, 2);
        assert_eq!(ps[1].tags_count, 0);

        assert_eq!(ps[2].text, "var i = -INFINITY;");
        assert_eq!(ps[2].words_count, 4);
        assert_eq!(ps[2].tags_count, 1);

        assert_eq!(ps[3].text, "after text with variable N");
        assert_eq!(ps[3].words_count, 5);
        assert_eq!(ps[3].tags_count, 1);

        assert_eq!(ps[4].text, "I am inline\nand I am happy");
        assert_eq!(ps[4].words_count, 7);
        assert_eq!(ps[4].tags_count, 0);
    }

    #[test]
    fn test_links() {
        let html = concat!(
            "<html><body>",
            "<a>I am <strong>top</strong>-inline\n\n\n\n and I am happy \n</a>",
            "<p>normal text</p>",
            "<code>\nvar i = -INFINITY;\n</code>",
            "<div>after <a>text</a> with variable <var>N</var> </div>",
            "   I am inline\n\n\n\n and I am happy \n",
            "</body></html>"
        );
        let ps = parse(html);
        assert_eq!(ps.len(), 5);

        assert_eq!(ps[0].text, "I am top-inline\nand I am happy");
        assert_eq!(ps[0].words_count, 7);
        assert_eq!(ps[0].tags_count, 2);
        assert_eq!(ps[0].chars_count_in_links, 31);

        assert_eq!(ps[1].text, "normal text");
        assert_eq!(ps[1].words_count, 2);
        assert_eq!(ps[1].tags_count, 0);

        assert_eq!(ps[2].text, "var i = -INFINITY;");
        assert_eq!(ps[2].words_count, 4);
        assert_eq!(ps[2].tags_count, 1);

        assert_eq!(ps[3].text, "after text with variable N");
        assert_eq!(ps[3].words_count, 5);
        assert_eq!(ps[3].tags_count, 2);
        assert_eq!(ps[3].chars_count_in_links, 4);

        assert_eq!(ps[4].text, "I am inline\nand I am happy");
        assert_eq!(ps[4].words_count, 7);
        assert_eq!(ps[4].tags_count, 0);
    }

    // --- Port of test_core.py ---

    #[test]
    fn test_words_split_by_br_tag() {
        // Single <br> inserts a space between words, NOT a paragraph boundary
        let ps = parse("abc<br/>def becoming abcdef");
        assert_eq!(ps.len(), 1);
        assert_eq!(ps[0].text, "abc def becoming abcdef");
    }

    // --- Port of test_paths.py ---

    #[test]
    fn test_path_empty() {
        let p = PathInfo::new();
        assert_eq!(p.dom(), "");
        assert_eq!(p.xpath(), "/");
    }

    #[test]
    fn test_path_single_element() {
        let mut p = PathInfo::new();
        p.push("html");
        assert_eq!(p.dom(), "html");
        assert_eq!(p.xpath(), "/html[1]");
    }

    #[test]
    fn test_path_nested() {
        let mut p = PathInfo::new();
        p.push("html");
        p.push("body");
        p.push("div");
        assert_eq!(p.dom(), "html.body.div");
        assert_eq!(p.xpath(), "/html[1]/body[1]/div[1]");
    }

    #[test]
    fn test_path_sibling_ordinals() {
        let mut p = PathInfo::new();
        p.push("html");
        p.push("body");
        p.push("div"); // first div → div[1]
        p.pop();
        p.push("div"); // second div → div[2]
        assert_eq!(p.xpath(), "/html[1]/body[1]/div[2]");
    }

    #[test]
    fn test_path_mixed_siblings() {
        let mut p = PathInfo::new();
        p.push("html");
        p.push("body");
        p.push("div"); // div[1]
        p.pop();
        p.push("p"); // p[1]
        p.pop();
        p.push("div"); // div[2]
        assert_eq!(p.xpath(), "/html[1]/body[1]/div[2]");
    }

    #[test]
    fn test_path_pop() {
        let mut p = PathInfo::new();
        p.push("html");
        p.push("body");
        p.pop();
        assert_eq!(p.dom(), "html");
        assert_eq!(p.xpath(), "/html[1]");
    }

    // --- Port of test_utils.py TestStringUtils ---

    #[test]
    fn test_is_blank_empty() {
        assert!(is_blank(""));
    }

    #[test]
    fn test_is_blank_space() {
        assert!(is_blank(" "));
    }

    #[test]
    fn test_is_blank_nobreak_space() {
        assert!(is_blank("\u{00A0}\t "));
    }

    #[test]
    fn test_is_blank_narrow_nobreak_space() {
        assert!(is_blank("\u{202F} \t"));
    }

    #[test]
    fn test_is_blank_spaces() {
        assert!(is_blank("    "));
    }

    #[test]
    fn test_is_blank_newline() {
        assert!(is_blank("\n"));
    }

    #[test]
    fn test_is_blank_tab() {
        assert!(is_blank("\t"));
    }

    #[test]
    fn test_is_blank_mixed_whitespace() {
        assert!(is_blank("\t\n "));
    }

    #[test]
    fn test_is_blank_with_chars() {
        assert!(!is_blank("  #  "));
    }

    #[test]
    fn test_normalize_no_change() {
        let s = "a b c d e f g h i j k l m n o p q r s ...";
        assert_eq!(normalize_whitespace(s), s);
    }

    #[test]
    fn test_normalize_dont_trim() {
        // Leading/trailing whitespace collapses to a single space — not stripped.
        let input = "  a b c d e f g h i j k l m n o p q r s ...  ";
        let expected = " a b c d e f g h i j k l m n o p q r s ... ";
        assert_eq!(normalize_whitespace(input), expected);
    }

    #[test]
    fn test_normalize_newline_and_tab() {
        // Whitespace runs containing \n collapse to \n; trailing \t\n → \n.
        let input = "123 \n456\t\n";
        let expected = "123\n456\n";
        assert_eq!(normalize_whitespace(input), expected);
    }

    #[test]
    fn test_normalize_non_break_spaces() {
        // \u{00A0}\t and \u{202F} \t are whitespace runs without \n → collapse to space.
        let input = "\u{00A0}\t €\u{202F} \t";
        let expected = " € ";
        assert_eq!(normalize_whitespace(input), expected);
    }
}
