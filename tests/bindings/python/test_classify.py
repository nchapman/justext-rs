"""Tests for classify_paragraphs() and paragraph struct fields."""

import pytest

from justext_uniffi import (
    ClassType,
    JustextError,
    classify_paragraphs,
    classify_paragraphs_with,
    default_config,
)

GOOD_PARAGRAPH = (
    "This is a sentence that contains many common English stopwords and it "
    "should be classified as good content by the algorithm because the text is "
    "long enough that it exceeds the length_high threshold of two hundred characters."
)


def test_classify_good_paragraph():
    html = f"<html><body><p>{GOOD_PARAGRAPH}</p></body></html>"
    paragraphs = classify_paragraphs(html, "English")
    assert len(paragraphs) > 0
    assert paragraphs[0].class_type == ClassType.GOOD


def test_classify_boilerplate():
    html = '<html><body><p><a>Home</a> | <a>About</a> | <a>Contact</a> | <a>Privacy</a> | <a>Terms</a></p></body></html>'
    paragraphs = classify_paragraphs(html, "English")
    assert len(paragraphs) > 0
    assert paragraphs[0].class_type == ClassType.BAD


def test_classify_empty_html():
    paragraphs = classify_paragraphs("<html><body></body></html>", "English")
    assert len(paragraphs) == 0


def test_classify_unknown_language():
    with pytest.raises(JustextError.UnknownLanguage):
        classify_paragraphs("<p>hello</p>", "Klingon")


def test_paragraph_fields():
    html = "<html><body><h2>My Heading</h2></body></html>"
    paragraphs = classify_paragraphs(html, "English")
    assert len(paragraphs) > 0
    h = paragraphs[0]
    assert h.text == "My Heading"
    assert "h2" in h.dom_path
    assert "h2" in h.xpath
    assert h.heading is True
    assert h.words_count == 2
    assert isinstance(h.chars_count_in_links, int)
    assert isinstance(h.tags_count, int)


def test_classify_with_custom_config():
    """With lowered length thresholds, a short paragraph should not be classified as SHORT."""
    config = default_config()
    config.length_low = 10
    config.length_high = 20
    html = "<html><body><p>Short text here.</p></body></html>"
    paragraphs = classify_paragraphs_with(html, "English", config)
    assert len(paragraphs) > 0
    assert paragraphs[0].class_type != ClassType.SHORT
