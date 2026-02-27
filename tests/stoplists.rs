// Port of test_utils.py — stoplist tests

use justext::{available_languages, get_all_stoplists, get_stoplist};

#[test]
fn test_available_languages_count() {
    let languages = available_languages();
    assert_eq!(languages.len(), 100);
}

#[test]
fn test_available_languages_contains_english() {
    let languages = available_languages();
    assert!(languages.contains(&"English"));
}

#[test]
fn test_get_stoplist_english() {
    let stoplist = get_stoplist("English").unwrap();
    assert!(!stoplist.is_empty());
    // "the" should be in the English stoplist
    assert!(stoplist.contains("the"));
    assert!(stoplist.contains("a"));
    assert!(stoplist.contains("is"));
}

#[test]
fn test_get_stoplist_case_insensitive() {
    let lower = get_stoplist("english").unwrap();
    let upper = get_stoplist("ENGLISH").unwrap();
    let mixed = get_stoplist("English").unwrap();
    assert_eq!(lower, upper);
    assert_eq!(lower, mixed);
}

#[test]
fn test_get_stoplist_slovak() {
    let stoplist = get_stoplist("Slovak").unwrap();
    assert!(!stoplist.is_empty());
}

#[test]
fn test_get_stoplist_missing() {
    assert!(get_stoplist("Klingon").is_err());
}

#[test]
fn test_get_all_stoplists_nonempty() {
    let all = get_all_stoplists();
    assert!(!all.is_empty());
    // Should contain words from multiple languages
    assert!(all.contains("the")); // English
}

#[test]
fn test_stoplist_words_are_lowercased() {
    let stoplist = get_stoplist("English").unwrap();
    for word in &stoplist {
        assert_eq!(
            word,
            &word.to_lowercase(),
            "stopword not lowercased: {word}"
        );
    }
}

#[test]
fn test_all_100_languages() {
    let expected = vec![
        "Afrikaans",
        "Albanian",
        "Arabic",
        "Aragonese",
        "Armenian",
        "Aromanian",
        "Asturian",
        "Azerbaijani",
        "Basque",
        "Belarusian",
        "Belarusian_Taraskievica",
        "Bengali",
        "Bishnupriya_Manipuri",
        "Bosnian",
        "Breton",
        "Bulgarian",
        "Catalan",
        "Cebuano",
        "Chuvash",
        "Croatian",
        "Czech",
        "Danish",
        "Dutch",
        "English",
        "Esperanto",
        "Estonian",
        "Finnish",
        "French",
        "Galician",
        "Georgian",
        "German",
        "Greek",
        "Gujarati",
        "Haitian",
        "Hebrew",
        "Hindi",
        "Hungarian",
        "Icelandic",
        "Ido",
        "Igbo",
        "Indonesian",
        "Irish",
        "Italian",
        "Javanese",
        "Kannada",
        "Kazakh",
        "Korean",
        "Kurdish",
        "Kyrgyz",
        "Latin",
        "Latvian",
        "Lithuanian",
        "Lombard",
        "Low_Saxon",
        "Luxembourgish",
        "Macedonian",
        "Malay",
        "Malayalam",
        "Maltese",
        "Marathi",
        "Neapolitan",
        "Nepali",
        "Newar",
        "Norwegian_Bokmal",
        "Norwegian_Nynorsk",
        "Occitan",
        "Persian",
        "Piedmontese",
        "Polish",
        "Portuguese",
        "Quechua",
        "Romanian",
        "Russian",
        "Samogitian",
        "Serbian",
        "Serbo_Croatian",
        "Sicilian",
        "Simple_English",
        "Slovak",
        "Slovenian",
        "Spanish",
        "Sundanese",
        "Swahili",
        "Swedish",
        "Tagalog",
        "Tamil",
        "Telugu",
        "Turkish",
        "Turkmen",
        "Ukrainian",
        "Urdu",
        "Uzbek",
        "Vietnamese",
        "Volapuk",
        "Walloon",
        "Waray_Waray",
        "Welsh",
        "West_Frisian",
        "Western_Panjabi",
        "Yoruba",
    ];

    let languages = available_languages();
    for lang in &expected {
        assert!(languages.contains(lang), "missing language: {lang}");
    }
}
