use std::collections::HashSet;
use std::sync::LazyLock;

/// All embedded stoplists: (language_name, file_contents).
const STOPLISTS: &[(&str, &str)] = &[
    ("Afrikaans", include_str!("Afrikaans.txt")),
    ("Albanian", include_str!("Albanian.txt")),
    ("Arabic", include_str!("Arabic.txt")),
    ("Aragonese", include_str!("Aragonese.txt")),
    ("Armenian", include_str!("Armenian.txt")),
    ("Aromanian", include_str!("Aromanian.txt")),
    ("Asturian", include_str!("Asturian.txt")),
    ("Azerbaijani", include_str!("Azerbaijani.txt")),
    ("Basque", include_str!("Basque.txt")),
    ("Belarusian", include_str!("Belarusian.txt")),
    (
        "Belarusian_Taraskievica",
        include_str!("Belarusian_Taraskievica.txt"),
    ),
    ("Bengali", include_str!("Bengali.txt")),
    (
        "Bishnupriya_Manipuri",
        include_str!("Bishnupriya_Manipuri.txt"),
    ),
    ("Bosnian", include_str!("Bosnian.txt")),
    ("Breton", include_str!("Breton.txt")),
    ("Bulgarian", include_str!("Bulgarian.txt")),
    ("Catalan", include_str!("Catalan.txt")),
    ("Cebuano", include_str!("Cebuano.txt")),
    ("Chuvash", include_str!("Chuvash.txt")),
    ("Croatian", include_str!("Croatian.txt")),
    ("Czech", include_str!("Czech.txt")),
    ("Danish", include_str!("Danish.txt")),
    ("Dutch", include_str!("Dutch.txt")),
    ("English", include_str!("English.txt")),
    ("Esperanto", include_str!("Esperanto.txt")),
    ("Estonian", include_str!("Estonian.txt")),
    ("Finnish", include_str!("Finnish.txt")),
    ("French", include_str!("French.txt")),
    ("Galician", include_str!("Galician.txt")),
    ("Georgian", include_str!("Georgian.txt")),
    ("German", include_str!("German.txt")),
    ("Greek", include_str!("Greek.txt")),
    ("Gujarati", include_str!("Gujarati.txt")),
    ("Haitian", include_str!("Haitian.txt")),
    ("Hebrew", include_str!("Hebrew.txt")),
    ("Hindi", include_str!("Hindi.txt")),
    ("Hungarian", include_str!("Hungarian.txt")),
    ("Icelandic", include_str!("Icelandic.txt")),
    ("Ido", include_str!("Ido.txt")),
    ("Igbo", include_str!("Igbo.txt")),
    ("Indonesian", include_str!("Indonesian.txt")),
    ("Irish", include_str!("Irish.txt")),
    ("Italian", include_str!("Italian.txt")),
    ("Javanese", include_str!("Javanese.txt")),
    ("Kannada", include_str!("Kannada.txt")),
    ("Kazakh", include_str!("Kazakh.txt")),
    ("Korean", include_str!("Korean.txt")),
    ("Kurdish", include_str!("Kurdish.txt")),
    ("Kyrgyz", include_str!("Kyrgyz.txt")),
    ("Latin", include_str!("Latin.txt")),
    ("Latvian", include_str!("Latvian.txt")),
    ("Lithuanian", include_str!("Lithuanian.txt")),
    ("Lombard", include_str!("Lombard.txt")),
    ("Low_Saxon", include_str!("Low_Saxon.txt")),
    ("Luxembourgish", include_str!("Luxembourgish.txt")),
    ("Macedonian", include_str!("Macedonian.txt")),
    ("Malay", include_str!("Malay.txt")),
    ("Malayalam", include_str!("Malayalam.txt")),
    ("Maltese", include_str!("Maltese.txt")),
    ("Marathi", include_str!("Marathi.txt")),
    ("Neapolitan", include_str!("Neapolitan.txt")),
    ("Nepali", include_str!("Nepali.txt")),
    ("Newar", include_str!("Newar.txt")),
    ("Norwegian_Bokmal", include_str!("Norwegian_Bokmal.txt")),
    ("Norwegian_Nynorsk", include_str!("Norwegian_Nynorsk.txt")),
    ("Occitan", include_str!("Occitan.txt")),
    ("Persian", include_str!("Persian.txt")),
    ("Piedmontese", include_str!("Piedmontese.txt")),
    ("Polish", include_str!("Polish.txt")),
    ("Portuguese", include_str!("Portuguese.txt")),
    ("Quechua", include_str!("Quechua.txt")),
    ("Romanian", include_str!("Romanian.txt")),
    ("Russian", include_str!("Russian.txt")),
    ("Samogitian", include_str!("Samogitian.txt")),
    ("Serbian", include_str!("Serbian.txt")),
    ("Serbo_Croatian", include_str!("Serbo_Croatian.txt")),
    ("Sicilian", include_str!("Sicilian.txt")),
    ("Simple_English", include_str!("Simple_English.txt")),
    ("Slovak", include_str!("Slovak.txt")),
    ("Slovenian", include_str!("Slovenian.txt")),
    ("Spanish", include_str!("Spanish.txt")),
    ("Sundanese", include_str!("Sundanese.txt")),
    ("Swahili", include_str!("Swahili.txt")),
    ("Swedish", include_str!("Swedish.txt")),
    ("Tagalog", include_str!("Tagalog.txt")),
    ("Tamil", include_str!("Tamil.txt")),
    ("Telugu", include_str!("Telugu.txt")),
    ("Turkish", include_str!("Turkish.txt")),
    ("Turkmen", include_str!("Turkmen.txt")),
    ("Ukrainian", include_str!("Ukrainian.txt")),
    ("Urdu", include_str!("Urdu.txt")),
    ("Uzbek", include_str!("Uzbek.txt")),
    ("Vietnamese", include_str!("Vietnamese.txt")),
    ("Volapuk", include_str!("Volapuk.txt")),
    ("Walloon", include_str!("Walloon.txt")),
    ("Waray_Waray", include_str!("Waray_Waray.txt")),
    ("Welsh", include_str!("Welsh.txt")),
    ("West_Frisian", include_str!("West_Frisian.txt")),
    ("Western_Panjabi", include_str!("Western_Panjabi.txt")),
    ("Yoruba", include_str!("Yoruba.txt")),
];

/// Merged set of all stopwords from every language, cached.
static ALL_STOPLISTS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    for (_, contents) in STOPLISTS {
        for word in parse_stoplist(contents) {
            set.insert(word);
        }
    }
    set
});

/// Parse a stoplist file: one word per line, lowercased, blank lines skipped.
fn parse_stoplist(contents: &str) -> HashSet<String> {
    contents
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|word| word.to_lowercase())
        .collect()
}

/// Return the stoplist for a given language (case-insensitive match).
///
/// Returns `None` if the language is not recognized.
pub fn get_stoplist(language: &str) -> Option<HashSet<String>> {
    let language_lower = language.to_lowercase();
    STOPLISTS
        .iter()
        .find(|(name, _)| name.to_lowercase() == language_lower)
        .map(|(_, contents)| parse_stoplist(contents))
}

/// Return the merged set of all stopwords from every language.
pub fn get_all_stoplists() -> &'static HashSet<String> {
    &ALL_STOPLISTS
}

/// Return the list of available language names.
pub fn available_languages() -> Vec<&'static str> {
    STOPLISTS.iter().map(|(name, _)| *name).collect()
}
