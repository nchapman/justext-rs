"""Tests for default_config() and available_languages()."""

import pytest

from justext_uniffi import available_languages, default_config


def test_default_config():
    config = default_config()
    assert config.length_low == 70
    assert config.length_high == 200
    assert config.stopwords_low == pytest.approx(0.30)
    assert config.stopwords_high == pytest.approx(0.32)
    assert config.max_link_density == pytest.approx(0.2)
    assert config.max_heading_distance == 200
    assert config.no_headings is False


def test_available_languages():
    langs = available_languages()
    assert isinstance(langs, list)
    assert len(langs) > 50
    assert "English" in langs
    assert "French" in langs
    assert "German" in langs
