#!/usr/bin/env python3
"""Run the trafilatura-rs comparison dataset through Python jusText and print a diff-ready report.

Usage:
    python3 scripts/compare_python.py [--html-dir PATH] [--limit N]

Reads HTML files from the trafilatura-rs test-files/comparison directory,
runs Python jusText (English stoplist, default config), and prints per-file
extracted text to stdout as JSONL. Errors and a summary go to stderr.
"""

import json
import os
import sys
import argparse

import justext

TRAFILATURA_TEST_FILES = os.path.join(
    os.path.dirname(__file__),
    "../../trafilatura-rs/test-files/comparison",
)

DEFAULT_HTML_DIR = os.path.abspath(TRAFILATURA_TEST_FILES)


def load_html(path):
    with open(path, "rb") as f:
        raw = f.read()
    try:
        return raw.decode("utf-8")
    except UnicodeDecodeError:
        return raw.decode("latin-1")


def extract(html):
    paragraphs = justext.justext(html, justext.get_stoplist("English"))
    return "\n".join(p.text for p in paragraphs if not p.is_boilerplate)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--html-dir", default=DEFAULT_HTML_DIR,
                        help="directory of .html files")
    parser.add_argument("--limit", type=int, default=None,
                        help="process at most N files")
    args = parser.parse_args()

    files = sorted(f for f in os.listdir(args.html_dir) if f.endswith(".html"))
    if args.limit:
        files = files[:args.limit]

    ok = errors = 0
    for filename in files:
        path = os.path.join(args.html_dir, filename)
        try:
            html = load_html(path)
            text = extract(html)
            print(json.dumps({"file": filename, "text": text}))
            ok += 1
        except Exception as e:
            print(json.dumps({"file": filename, "error": str(e)}), file=sys.stderr)
            errors += 1

    print(f"Done: {ok} ok, {errors} errors  (total {ok+errors})", file=sys.stderr)


if __name__ == "__main__":
    main()
