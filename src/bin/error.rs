// use anyhow::{Context, Result};
use thiserror::Error;

use fancy_regex;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Read error")]
    ReadError {source: std::io::Error},

    #[error("Write error")]
    WriteError {source: std::io::Error},

    // All other cases of "std::io::Error"
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    // Regex errors
    #[error(transparent)]
    RegexError(#[from] fancy_regex::Error),
}