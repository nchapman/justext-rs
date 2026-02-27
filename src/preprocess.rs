// Port of Python jusText preprocessor() from justext/core.py

use scraper::Html;

/// Tags to completely remove (including all children).
const REMOVE_TAGS: &[&str] = &[
    // scripts, style, head (Python kill_tags)
    "script", "style", "head",
    // forms=True
    "form", "input", "button", "select", "textarea",
    // embedded=True (embed, object, applet, layer, param)
    "embed", "object", "applet", "layer", "param",
];

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

            out.push('<');
            out.push_str(tag);
            for (attr, val) in el.attrs() {
                out.push(' ');
                out.push_str(attr);
                out.push_str("=\"");
                out.push_str(&val.replace('"', "&quot;"));
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
            out.push_str(&text.text);
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

/// HTML void elements that must not have a closing tag.
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
        assert!(!has_tag(&doc, "form"));
        assert!(!has_tag(&doc, "input"));
        assert!(!has_tag(&doc, "button"));
        assert!(has_tag(&doc, "p"));
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
}
