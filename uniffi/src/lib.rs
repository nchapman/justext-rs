uniffi::setup_scaffolding!();

/// Errors returned by functions that take a language name.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum JustextError {
    #[error("{reason}")]
    UnknownLanguage { reason: String },
}

impl From<justext::JustextError> for JustextError {
    fn from(e: justext::JustextError) -> Self {
        match e {
            justext::JustextError::UnknownLanguage(lang) => JustextError::UnknownLanguage {
                reason: format!("unknown language: {lang}"),
            },
        }
    }
}

/// Classification label for a paragraph.
#[derive(uniffi::Enum)]
pub enum ClassType {
    Good,
    Bad,
    Short,
    NearGood,
}

/// A classified text paragraph extracted from HTML.
#[derive(uniffi::Record)]
pub struct Paragraph {
    /// Dot-separated DOM path (e.g., "body.div.p").
    pub dom_path: String,
    /// XPath with ordinals (e.g., "/html[1]/body[1]/div[2]/p[1]").
    pub xpath: String,
    /// Normalized text content.
    pub text: String,
    /// Word count.
    pub word_count: i64,
    /// Character count inside `<a>` tags.
    pub link_char_count: i64,
    /// Inline tag count.
    pub tag_count: i64,
    /// Final classification.
    pub class_type: ClassType,
    /// Classification before neighbor-based revision.
    pub initial_class: ClassType,
    /// Whether this paragraph is a heading.
    pub heading: bool,
}

/// Configuration for the JusText algorithm.
#[derive(uniffi::Record)]
pub struct Config {
    pub length_low: i64,
    pub length_high: i64,
    pub stopwords_low: f64,
    pub stopwords_high: f64,
    pub max_link_density: f64,
    pub max_heading_distance: i64,
    pub no_headings: bool,
}

/// Returns the default configuration (matches Python JusText 3.0.2).
#[uniffi::export]
pub fn default_config() -> Config {
    let d = justext::Config::default();
    Config {
        length_low: d.length_low as i64,
        length_high: d.length_high as i64,
        stopwords_low: d.stopwords_low,
        stopwords_high: d.stopwords_high,
        max_link_density: d.max_link_density,
        max_heading_distance: d.max_heading_distance as i64,
        no_headings: d.no_headings,
    }
}

/// Returns the list of available language names.
#[uniffi::export]
pub fn available_languages() -> Vec<String> {
    justext::available_languages()
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

/// Extract only the good paragraph text using a language name and default config.
#[uniffi::export]
pub fn extract_text(html: String, language: String) -> Result<String, JustextError> {
    Ok(justext::extract_text_lang(&html, &language, &justext::Config::default())?)
}

/// Extract only the good paragraph text with custom config.
#[uniffi::export]
pub fn extract_text_with(
    html: String,
    language: String,
    config: Config,
) -> Result<String, JustextError> {
    let core_config = to_core_config(&config);
    Ok(justext::extract_text_lang(&html, &language, &core_config)?)
}

/// Classify all paragraphs in HTML using a language name and default config.
#[uniffi::export]
pub fn classify_paragraphs(
    html: String,
    language: String,
) -> Result<Vec<Paragraph>, JustextError> {
    let paragraphs = justext::justext_lang(&html, &language, &justext::Config::default())?;
    Ok(paragraphs.into_iter().map(to_ffi_paragraph).collect())
}

/// Classify all paragraphs in HTML with custom config.
#[uniffi::export]
pub fn classify_paragraphs_with(
    html: String,
    language: String,
    config: Config,
) -> Result<Vec<Paragraph>, JustextError> {
    let core_config = to_core_config(&config);
    let paragraphs = justext::justext_lang(&html, &language, &core_config)?;
    Ok(paragraphs.into_iter().map(to_ffi_paragraph).collect())
}

// --- Internal conversion helpers ---

fn to_core_config(c: &Config) -> justext::Config {
    justext::Config::default()
        .with_length_low(c.length_low.max(0) as usize)
        .with_length_high(c.length_high.max(0) as usize)
        .with_stopwords_low(c.stopwords_low)
        .with_stopwords_high(c.stopwords_high)
        .with_max_link_density(c.max_link_density)
        .with_max_heading_distance(c.max_heading_distance.max(0) as usize)
        .with_no_headings(c.no_headings)
}

fn convert_class_type(ct: justext::ClassType) -> ClassType {
    match ct {
        justext::ClassType::Good => ClassType::Good,
        justext::ClassType::Bad => ClassType::Bad,
        justext::ClassType::Short => ClassType::Short,
        justext::ClassType::NearGood => ClassType::NearGood,
    }
}

fn to_ffi_paragraph(p: justext::Paragraph) -> Paragraph {
    Paragraph {
        dom_path: p.dom_path,
        xpath: p.xpath,
        text: p.text,
        word_count: p.words_count as i64,
        link_char_count: p.chars_count_in_links as i64,
        tag_count: p.tags_count as i64,
        class_type: convert_class_type(p.class_type),
        initial_class: convert_class_type(p.initial_class),
        heading: p.heading,
    }
}
