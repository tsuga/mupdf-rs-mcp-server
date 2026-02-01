//! Error types for the MuPDF MCP server.

use thiserror::Error;

/// Errors that can occur in the MuPDF MCP server.
#[derive(Debug, Error)]
pub enum MupdfServerError {
    /// Document with the given ID was not found in the store.
    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    /// Invalid page number (out of bounds).
    #[error("Invalid page number: {page} (document has {total} pages, valid range: 0-{max})")]
    InvalidPageNumber { page: i32, total: i32, max: i32 },

    /// Document requires a password to open.
    #[error("Password required for this document")]
    PasswordRequired,

    /// The provided password is incorrect.
    #[error("Invalid password")]
    InvalidPassword,

    /// The document is not a PDF (for PDF-specific operations).
    #[error("Document is not a PDF")]
    NotAPdf,

    /// Invalid text format requested.
    #[error("Invalid text format: {0} (valid formats: plain, html, json, xml)")]
    InvalidTextFormat(String),

    /// Invalid image format requested.
    #[error("Invalid image format: {0} (valid formats: png, svg)")]
    InvalidImageFormat(String),

    /// Base64 decoding error.
    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    /// IO error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// MuPDF library error.
    #[error("MuPDF error: {0}")]
    MupdfError(#[from] mupdf::Error),

    /// JSON serialization error.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Internal error (unexpected state).
    #[error("Internal error: {0}")]
    Internal(String),
}

impl MupdfServerError {
    /// Create an internal error with a message.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

/// Result type for MuPDF MCP server operations.
pub type Result<T> = std::result::Result<T, MupdfServerError>;
