use thiserror::Error;

#[derive(Debug, Error)]
pub enum JustextError {
    #[error("unknown language: {0}")]
    UnknownLanguage(String),
}
