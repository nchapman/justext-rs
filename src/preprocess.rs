// Port of Python jusText preprocessor() from justext/core.py

use scraper::Html;

/// Tags to completely remove (including all children).
const REMOVE_TAGS: &[&str] = &[
    // scripts, style, head (Python kill_tags); noscript contains raw text in HTML5 parsing
    "script", "style", "head", "noscript",
    // forms=True: form controls are dropped entirely
    "input", "button", "select", "textarea",
    // embedded=True (embed, object, applet, iframe, layer, param)
    "embed", "object", "applet", "iframe", "layer", "param",
];

/// Tags whose element is dropped but whose children are preserved.
///
/// Python's lxml Cleaner(forms=True) removes the <form> wrapper but keeps
/// child content (paragraphs, divs, text) floating up to the parent level.
/// Form controls (input, button, select, textarea) are dropped entirely above.
const REMOVE_TAG_KEEP_CHILDREN: &[&str] = &["form"];

/// Remove unwanted tags from HTML and return a cleaned document.
///
/// Mirrors the Python `preprocessor()` which uses lxml's Cleaner with:
/// - scripts=True, comments=True, style=True, embedded=True, forms=True
/// - kill_tags=("head",)
pub fn preprocess(html: &str) -> Html {
    // Scraper parses into an owned Html; we must rebuild without unwanted nodes.
    // Strategy: serialize to string after stripping unwanted tags, then reparse.
    let cleaned = remove_tags_and_comments(html);
    Html::parse_document(&cleaned)
}

/// Remove unwanted tags and HTML comments via string manipulation before parsing.
///
/// This is simpler and more reliable than trying to mutate scraper's arena.
fn remove_tags_and_comments(html: &str) -> String {
    // We do a two-pass approach:
    // 1. Parse with scraper to get a proper DOM
    // 2. Walk the tree, skipping unwanted nodes, and rebuild the text
    let doc = Html::parse_document(html);
    let mut out = String::with_capacity(html.len());
    serialize_node(&doc.tree.root(), &mut out);
    out
}

/// Recursively serialize the node tree, skipping unwanted tags and comments.
fn serialize_node(node: &ego_tree::NodeRef<scraper::node::Node>, out: &mut String) {
    use scraper::node::Node;

    match node.value() {
        Node::Document => {
            for child in node.children() {
                serialize_node(&child, out);
            }
        }
        Node::Element(el) => {
            let tag = el.name();
            if REMOVE_TAGS.contains(&tag) {
                return; // skip element and all its children
            }
            if REMOVE_TAG_KEEP_CHILDREN.contains(&tag) {
                // Drop the element tag but recurse into children (content floats up).
                for child in node.children() {
                    serialize_node(&child, out);
                }
                return;
            }

            out.push('<');
            out.push_str(tag);
            for (attr, val) in el.attrs() {
                out.push(' ');
                out.push_str(attr);
                out.push_str("=\"");
                escape_attr(val, out);
                out.push('"');
            }
            if is_void_element(tag) {
                out.push_str(" />");
            } else {
                out.push('>');
                for child in node.children() {
                    serialize_node(&child, out);
                }
                out.push_str("</");
                out.push_str(tag);
                out.push('>');
            }
        }
        Node::Text(text) => {
            // HTML-escape so that decoded entities (e.g. &lt;year&gt; decoded to <year>
            // by the first parse) are not re-interpreted as markup in the second parse.
            for ch in text.text.chars() {
                match ch {
                    '&' => out.push_str("&amp;"),
                    '<' => out.push_str("&lt;"),
                    '>' => out.push_str("&gt;"),
                    _ => out.push(ch),
                }
            }
        }
        // Skip comments and doctypes.
        // Note: Python's Cleaner has processing_instructions=False (preserves PIs), but PIs
        // are vanishingly rare in real-world HTML so we strip them here for simplicity.
        Node::Comment(_) | Node::ProcessingInstruction(_) | Node::Doctype(_) => {}
        Node::Fragment => {
            for child in node.children() {
                serialize_node(&child, out);
            }
        }
    }
}

/// Write an HTML-escaped attribute value into `out`.
///
/// Escapes `&`, `<`, `>`, and `"` so that the serialized attribute string is valid HTML
/// and round-trips correctly through the second parse. Bare `&` is common in URL query
/// strings (e.g. `href="/?a=1&b=2"`) and must be re-encoded as `&amp;`.
fn escape_attr(val: &str, out: &mut String) {
    for ch in val.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
}

/// HTML void elements that must not have a closing tag.
///
/// Note: `embed` and `param` also appear in `REMOVE_TAGS` and will be skipped
/// before this function is ever reached — they are listed here for completeness
/// against the HTML5 spec void-element list.
fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_content(doc: &Html) -> String {
        doc.tree
            .nodes()
            .filter_map(|n| {
                if let scraper::node::Node::Text(t) = n.value() {
                    Some(t.text.as_ref())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn has_tag(doc: &Html, tag: &str) -> bool {
        let sel = scraper::Selector::parse(tag).unwrap();
        doc.select(&sel).next().is_some()
    }

    #[test]
    fn test_remove_head_tag() {
        let html = "<html><head><title>Title</title></head><body><p>text</p></body></html>";
        let doc = preprocess(html);
        // scraper's HTML5 parser always adds an implicit <head>, but its contents must be gone
        assert!(
            !has_tag(&doc, "title"),
            "<title> should be removed with <head>"
        );
        assert!(has_tag(&doc, "p"));
    }

    #[test]
    fn test_remove_script() {
        let html = "<html><body><script>alert('x')</script><p>text</p></body></html>";
        let doc = preprocess(html);
        assert!(!has_tag(&doc, "script"));
        assert!(has_tag(&doc, "p"));
    }

    #[test]
    fn test_remove_style() {
        let html = "<html><body><style>body{color:red}</style><p>text</p></body></html>";
        let doc = preprocess(html);
        assert!(!has_tag(&doc, "style"));
        assert!(has_tag(&doc, "p"));
    }

    #[test]
    fn test_remove_form_family() {
        let html = "<html><body><form><input type=\"text\" /><button>Go</button></form><p>text</p></body></html>";
        let doc = preprocess(html);
        // Form tag is removed (children kept), controls are dropped entirely
        assert!(!has_tag(&doc, "form"));
        assert!(!has_tag(&doc, "input"));
        assert!(!has_tag(&doc, "button"));
        assert!(has_tag(&doc, "p"));
    }

    #[test]
    fn test_form_wrapper_text_preserved() {
        // Python lxml Cleaner(forms=True) removes the <form> wrapper but keeps
        // child content (article paragraphs, divs, etc.) floating up to parent.
        let html = "<html><body><form id=\"main\"><p>Article content</p></form></body></html>";
        let doc = preprocess(html);
        assert!(!has_tag(&doc, "form"), "form tag should be removed");
        assert!(has_tag(&doc, "p"), "child <p> should survive");
        let content = text_content(&doc);
        assert!(
            content.contains("Article content"),
            "text inside form should be preserved"
        );
    }

    #[test]
    fn test_remove_comments() {
        let html = "<html><body><!-- a comment --><p>text</p></body></html>";
        let doc = preprocess(html);
        // No comment nodes should remain
        let has_comment = doc
            .tree
            .nodes()
            .any(|n| matches!(n.value(), scraper::node::Node::Comment(_)));
        assert!(!has_comment);
        assert!(has_tag(&doc, "p"));
    }

    #[test]
    fn test_remove_embedded_layer() {
        // <layer> is a legacy Netscape tag removed by Python's embedded=True
        let html = "<html><body><layer>plugin</layer><p>text</p></body></html>";
        let doc = preprocess(html);
        assert!(!has_tag(&doc, "layer"));
        assert!(has_tag(&doc, "p"));
    }

    #[test]
    fn test_remove_embedded_param() {
        // <param> is removed by Python's embedded=True
        let html = "<html><body><object><param name=\"src\" value=\"x\" /></object><p>text</p></body></html>";
        let doc = preprocess(html);
        assert!(!has_tag(&doc, "param"));
        assert!(!has_tag(&doc, "object"));
    }

    #[test]
    fn test_preserve_content() {
        let html = "<html><body><p>Hello <em>world</em></p></body></html>";
        let doc = preprocess(html);
        assert!(has_tag(&doc, "p"));
        assert!(has_tag(&doc, "em"));
        let content = text_content(&doc);
        assert!(content.contains("Hello"));
        assert!(content.contains("world"));
    }

    #[test]
    fn test_attribute_ampersand_survives_double_parse() {
        // Bare & in URL query strings must be re-encoded as &amp; in the serialized
        // intermediate so it round-trips correctly through the second parse.
        let html = r#"<html><body><a href="/?a=1&amp;b=2">link</a></body></html>"#;
        let doc = preprocess(html);
        let sel = scraper::Selector::parse("a").unwrap();
        let href = doc
            .select(&sel)
            .next()
            .unwrap()
            .value()
            .attr("href")
            .unwrap();
        assert_eq!(href, "/?a=1&b=2", "decoded & must survive the double parse");
    }

    #[test]
    fn test_text_entities_not_reparsed_as_tags() {
        // &lt;year&gt; must survive as literal text, not become a real <year> element
        // after the double-parse in remove_tags_and_comments().
        let html = "<html><body><p>Use &lt;year&gt; as placeholder</p></body></html>";
        let doc = preprocess(html);
        let content = text_content(&doc);
        assert!(
            content.contains("<year>"),
            "decoded entity text should be preserved as text"
        );
        assert!(
            !has_tag(&doc, "year"),
            "<year> must not become a DOM element"
        );
    }
}
