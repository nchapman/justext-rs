"""Tests for extract_text() and extract_text_with() API."""

import pytest

from justext_uniffi import (
    JustextError,
    extract_text,
    extract_text_with,
    default_config,
)

GOOD_PARAGRAPH = (
    "This is a sentence that contains many common English stopwords and it "
    "should be classified as good content by the algorithm because the text is "
    "long enough that it exceeds the length_high threshold of two hundred characters."
)


def test_extract_text_basic():
    html = f"<html><body><p>{GOOD_PARAGRAPH}</p></body></html>"
    result = extract_text(html, "English")
    assert "good content" in result


def test_extract_text_empty_html():
    result = extract_text("<html><body></body></html>", "English")
    assert result == ""


def test_extract_text_with_default_config():
    html = f"<html><body><p>{GOOD_PARAGRAPH}</p></body></html>"
    result = extract_text_with(html, "English", default_config())
    assert result == extract_text(html, "English")


def test_extract_text_unknown_language():
    with pytest.raises(JustextError.UnknownLanguage):
        extract_text("<p>hello</p>", "Klingon")


def test_extract_text_filters_boilerplate():
    html = f"""<html><body>
        <p><a>Home</a> | <a>About</a> | <a>Contact</a></p>
        <p>{GOOD_PARAGRAPH}</p>
    </body></html>"""
    result = extract_text(html, "English")
    assert "good content" in result
    assert "Home" not in result
