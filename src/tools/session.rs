//! Session management tools: upload, close, list documents.

use base64::Engine;
use mupdf::Document;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{MupdfServerError, Result};
use crate::state::DocumentStore;

/// Source for a document: either a file path or base64 content.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum DocumentSource {
    /// Load document from a file path.
    FilePath {
        /// Path to the PDF file.
        path: String,
    },
    /// Load document from base64-encoded content.
    Base64 {
        /// Base64-encoded document content.
        base64: String,
        /// Optional filename hint (for format detection).
        #[serde(default)]
        filename: Option<String>,
    },
}

impl DocumentSource {
    /// Open a document from this source.
    pub fn open(&self, password: Option<&str>) -> Result<Document> {
        let mut doc = match self {
            DocumentSource::FilePath { path } => Document::open(path)?,
            DocumentSource::Base64 { base64, filename } => {
                let bytes = base64::engine::general_purpose::STANDARD.decode(base64)?;
                let magic = filename.as_deref().unwrap_or("application/pdf");
                Document::from_bytes(&bytes, magic)?
            }
        };

        // Handle password protection
        if doc.needs_password()? {
            match password {
                Some(pw) => {
                    if !doc.authenticate(pw)? {
                        return Err(MupdfServerError::InvalidPassword);
                    }
                }
                None => return Err(MupdfServerError::PasswordRequired),
            }
        }

        Ok(doc)
    }
}

// ============== Import Document ==============

/// Parameters for importing a document.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImportDocumentParams {
    /// Document source (file path or base64 content).
    pub source: DocumentSource,
    /// Password for encrypted documents (optional).
    #[serde(default)]
    pub password: Option<String>,
}

/// Result of importing a document.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ImportDocumentResult {
    /// Unique identifier for the imported document.
    pub document_id: String,
    /// Number of pages in the document.
    pub page_count: i32,
}

/// Import a document to the server.
pub fn import_document(
    store: &DocumentStore,
    params: ImportDocumentParams,
) -> Result<ImportDocumentResult> {
    let doc = params.source.open(params.password.as_deref())?;
    let page_count = doc.page_count()?;
    let document_id = store.insert(doc)?;

    Ok(ImportDocumentResult {
        document_id,
        page_count,
    })
}

// ============== Close Document ==============

/// Parameters for closing a document.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CloseDocumentParams {
    /// Document ID to close.
    pub document_id: String,
}

/// Result of closing a document.
#[derive(Debug, Serialize, JsonSchema)]
pub struct CloseDocumentResult {
    /// Whether the document was successfully closed.
    pub success: bool,
}

/// Close a document and free its memory.
pub fn close_document(
    store: &DocumentStore,
    params: CloseDocumentParams,
) -> Result<CloseDocumentResult> {
    store.remove(&params.document_id)?;
    Ok(CloseDocumentResult { success: true })
}

// ============== List Documents ==============

/// Parameters for listing documents (none required).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListDocumentsParams {}

/// Information about a single document.
#[derive(Debug, Serialize, JsonSchema)]
pub struct DocumentListEntry {
    /// Document ID.
    pub document_id: String,
    /// Number of pages.
    pub page_count: i32,
    /// Seconds since the document was uploaded.
    pub age_seconds: u64,
}

/// Result of listing documents.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ListDocumentsResult {
    /// List of open documents.
    pub documents: Vec<DocumentListEntry>,
}

/// List all open documents.
pub fn list_documents(
    store: &DocumentStore,
    _params: ListDocumentsParams,
) -> Result<ListDocumentsResult> {
    let docs = store.list()?;
    let documents = docs
        .into_iter()
        .map(|info| DocumentListEntry {
            document_id: info.id,
            page_count: info.page_count,
            age_seconds: info.created_at.elapsed().as_secs(),
        })
        .collect();

    Ok(ListDocumentsResult { documents })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_source_deserialize_path() {
        let json = r#"{"path": "/tmp/test.pdf"}"#;
        let source: DocumentSource = serde_json::from_str(json).unwrap();
        match source {
            DocumentSource::FilePath { path } => assert_eq!(path, "/tmp/test.pdf"),
            _ => panic!("Expected FilePath variant"),
        }
    }

    #[test]
    fn test_document_source_deserialize_base64() {
        let json = r#"{"base64": "SGVsbG8=", "filename": "test.pdf"}"#;
        let source: DocumentSource = serde_json::from_str(json).unwrap();
        match source {
            DocumentSource::Base64 { base64, filename } => {
                assert_eq!(base64, "SGVsbG8=");
                assert_eq!(filename, Some("test.pdf".to_string()));
            }
            _ => panic!("Expected Base64 variant"),
        }
    }
}
