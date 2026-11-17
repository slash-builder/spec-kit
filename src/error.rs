//! Error type for Spec Kit operations.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpecKitError {
    #[error("parse error: {0}")]
    Parse(String),

    #[error("invalid spec version: {0}")]
    Version(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
