use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use justext::{get_all_stoplists, get_stoplist, justext, Config};

// ---------------------------------------------------------------------------
// HTML fixtures
// ---------------------------------------------------------------------------

/// Small page: a handful of paragraphs + minimal nav/footer.
const SMALL_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body>
<nav><a href="/">Home</a> | <a href="/about">About</a></nav>
<h1>Article Title</h1>
<p>This is the first paragraph of the main article content. It contains enough
words to pass the length threshold and has a good stopword density because it
uses common English words like the and is and of and in.</p>
<p>The second paragraph continues the article with more substantive content.
Reading this text should convince the classifier that it is good content and
not boilerplate navigation or footer material at all.</p>
<p>Copyright &copy; 2024 Example Corp. All rights reserved.</p>
<footer><a href="/privacy">Privacy</a> | <a href="/terms">Terms</a></footer>
</body>
</html>"#;

/// Medium page: ~20 content paragraphs with nav/aside/footer boilerplate.
fn medium_html() -> String {
    let mut s = String::from(
        "<!DOCTYPE html><html><head><title>Article</title></head><body>\n\
         <nav><a href=\"/\">Home</a><a href=\"/news\">News</a><a href=\"/sports\">Sports</a></nav>\n\
         <h1>Main Article Heading</h1>\n",
    );
    for i in 1..=20 {
        s.push_str(&format!(
            "<p>Paragraph {} of the article body. It contains a reasonable amount of \
             text with common English stopwords like the, is, of, and, in, to, a so \
             that the classifier will recognise it as good content worth extracting \
             from surrounding boilerplate navigation and footer elements.</p>\n",
            i
        ));
    }
    s.push_str(
        "<aside><p>Related: <a href=\"/1\">Link one</a> <a href=\"/2\">Link two</a> \
         <a href=\"/3\">Link three</a></p></aside>\n\
         <footer><p>Copyright &copy; 2024 Corp. All rights reserved.</p></footer>\n\
         </body></html>",
    );
    s
}

/// Large page: ~100 content paragraphs simulating a long-form article.
fn large_html() -> String {
    let mut s = String::from(
        "<!DOCTYPE html><html><head><title>Long Article</title></head><body>\n\
         <nav><a href=\"/\">Home</a><a href=\"/news\">News</a></nav>\n\
         <h1>Long-form Article</h1>\n",
    );
    for i in 1..=100 {
        s.push_str(&format!(
            "<p>Section {} discusses the topic in depth. The paragraph is long enough \
             to exceed the length threshold and contains enough common English stopwords \
             such as the, is, of, and, to, in, a, that, this, it, for, on, are, as, \
             with, they, at, be, from, or, an, have, by, not, what, all, were, we, \
             when, can, there, use, which, do, how, their, if, will, up, other, about, \
             out, many, then, them, these so the classifier marks it as good.</p>\n",
            i
        ));
    }
    s.push_str("<footer><p>Copyright &copy; 2024 Corp.</p></footer>\n</body></html>");
    s
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

/// Full pipeline at three HTML sizes with English stoplist.
fn bench_full_pipeline(c: &mut Criterion) {
    let stoplist = get_stoplist("English").unwrap();
    let config = Config::default();

    let inputs: &[(&str, String)] = &[
        ("small", SMALL_HTML.to_string()),
        ("medium", medium_html()),
        ("large", large_html()),
    ];

    let mut group = c.benchmark_group("full_pipeline");
    for (id, html) in inputs {
        group.bench_with_input(BenchmarkId::from_parameter(id), html, |b, html| {
            b.iter(|| justext(black_box(html), black_box(&stoplist), black_box(&config)))
        });
    }
    group.finish();
}

/// Stoplist access: per-call allocation vs cached merged set.
fn bench_stoplists(c: &mut Criterion) {
    let medium = medium_html();
    let config = Config::default();

    let mut group = c.benchmark_group("stoplists");

    // Per-call: allocates and parses a new HashSet each time.
    group.bench_function("get_stoplist_per_call", |b| {
        b.iter(|| get_stoplist(black_box("English")).unwrap())
    });

    // Cached: LazyLock, pointer-sized return after first call.
    group.bench_function("get_all_stoplists_cached", |b| {
        b.iter(|| get_all_stoplists())
    });

    // Full pipeline: English stoplist vs merged all-languages stoplist.
    let english = get_stoplist("English").unwrap();
    group.bench_function("pipeline_english_stoplist", |b| {
        b.iter(|| justext(black_box(&medium), black_box(&english), black_box(&config)))
    });

    let all = get_all_stoplists();
    group.bench_function("pipeline_all_stoplists", |b| {
        b.iter(|| justext(black_box(&medium), black_box(all), black_box(&config)))
    });

    // Language-independent mode: empty thresholds, empty stoplist.
    let empty_config = Config::default()
        .with_stopwords_low(0.0)
        .with_stopwords_high(0.0);
    let empty: std::collections::HashSet<String> = std::collections::HashSet::new();
    group.bench_function("pipeline_language_independent", |b| {
        b.iter(|| {
            justext(
                black_box(&medium),
                black_box(&empty),
                black_box(&empty_config),
            )
        })
    });

    group.finish();
}

criterion_group!(benches, bench_full_pipeline, bench_stoplists);
criterion_main!(benches);
