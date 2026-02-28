/// Run justext on a directory of HTML files and emit JSONL, mirroring scripts/compare_python.py.
///
/// Usage:
///   cargo run --bin compare -- <html-dir>
///
/// Output (stdout): one JSON object per file: {"file": "...", "text": "..."}
/// Errors (stderr): {"file": "...", "error": "..."}
/// Summary (stderr): "Done: N ok, M errors"
use std::env;
use std::fs;
use std::path::Path;

fn json_str(s: &str) -> String {
    // Minimal JSON string escaping — no external deps.
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: compare <html-dir>");
        std::process::exit(1);
    }
    let html_dir = Path::new(&args[1]);

    let stoplist = justext::get_stoplist("English").expect("English stoplist missing");
    let config = justext::Config::default();

    let mut entries: Vec<_> = fs::read_dir(html_dir)
        .expect("cannot read html-dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "html"))
        .map(|e| e.path())
        .collect();
    entries.sort();

    let mut ok = 0usize;
    let mut errors = 0usize;

    for path in &entries {
        let filename = path.file_name().unwrap().to_string_lossy();

        let raw = match fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!(
                    "{{\"file\": {}, \"error\": {}}}",
                    json_str(&filename),
                    json_str(&e.to_string())
                );
                errors += 1;
                continue;
            }
        };

        // Match Python's UTF-8 → latin-1 fallback.
        let html = match String::from_utf8(raw.clone()) {
            Ok(s) => s,
            Err(_) => raw.iter().map(|&b| b as char).collect(),
        };

        let paragraphs = justext::justext(&html, &stoplist, &config);
        let text: String = paragraphs
            .iter()
            .filter(|p| !p.is_boilerplate())
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        println!(
            "{{\"file\": {}, \"text\": {}}}",
            json_str(&filename),
            json_str(&text)
        );
        ok += 1;
    }

    eprintln!("Done: {ok} ok, {errors} errors  (total {})", ok + errors);
}
