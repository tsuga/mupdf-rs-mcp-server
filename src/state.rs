//! Document store for managing uploaded PDF documents.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use mupdf::Document;
use uuid::Uuid;

use crate::error::{MupdfServerError, Result};

/// Metadata about a stored document.
#[derive(Debug, Clone)]
pub struct DocumentInfo {
    /// Unique identifier for this document.
    pub id: String,
    /// Number of pages in the document.
    pub page_count: i32,
    /// When the document was uploaded.
    pub created_at: Instant,
    /// When the document was last accessed.
    pub last_accessed: Instant,
}

/// A stored document with its metadata.
pub struct StoredDocument {
    /// The MuPDF document handle.
    pub document: Document,
    /// Document metadata.
    pub info: DocumentInfo,
}

impl StoredDocument {
    /// Create a new stored document.
    pub fn new(document: Document) -> Result<Self> {
        let page_count = document.page_count()?;
        let now = Instant::now();
        let id = Uuid::new_v4().to_string();

        Ok(Self {
            document,
            info: DocumentInfo {
                id,
                page_count,
                created_at: now,
                last_accessed: now,
            },
        })
    }

    /// Update the last accessed timestamp.
    pub fn touch(&mut self) {
        self.info.last_accessed = Instant::now();
    }
}

/// Thread-safe document store.
///
/// Note: MuPDF Document is !Send and !Sync, so we need to be careful
/// about how we access documents. All MuPDF operations should be done
/// within the same thread that created the document.
#[derive(Clone)]
pub struct DocumentStore {
    inner: Arc<Mutex<DocumentStoreInner>>,
}

struct DocumentStoreInner {
    documents: HashMap<String, StoredDocument>,
}

// SAFETY: DocumentStoreInner contains MuPDF Document which is !Send because it
// contains raw pointers. However, all access to documents is guarded by a Mutex,
// and documents are never actually moved across threads - they are created and
// used within the critical section. The Arc<Mutex<>> wrapper ensures proper
// synchronization.
unsafe impl Send for DocumentStoreInner {}

impl Default for DocumentStore {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentStore {
    /// Create a new empty document store.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(DocumentStoreInner {
                documents: HashMap::new(),
            })),
        }
    }

    /// Insert a document into the store.
    ///
    /// Returns the document ID.
    pub fn insert(&self, document: Document) -> Result<String> {
        let stored = StoredDocument::new(document)?;
        let id = stored.info.id.clone();

        let mut inner = self.inner.lock().map_err(|e| {
            MupdfServerError::internal(format!("Failed to lock document store: {}", e))
        })?;

        inner.documents.insert(id.clone(), stored);
        Ok(id)
    }

    /// Get document info without accessing the document itself.
    pub fn get_info(&self, id: &str) -> Result<DocumentInfo> {
        let inner = self.inner.lock().map_err(|e| {
            MupdfServerError::internal(format!("Failed to lock document store: {}", e))
        })?;

        inner
            .documents
            .get(id)
            .map(|d| d.info.clone())
            .ok_or_else(|| MupdfServerError::DocumentNotFound(id.to_string()))
    }

    /// Execute a function with access to a document.
    ///
    /// This is the primary way to interact with documents, as it handles
    /// locking and updates the last accessed timestamp.
    pub fn with_document<F, T>(&self, id: &str, f: F) -> Result<T>
    where
        F: FnOnce(&Document) -> Result<T>,
    {
        let mut inner = self.inner.lock().map_err(|e| {
            MupdfServerError::internal(format!("Failed to lock document store: {}", e))
        })?;

        let stored = inner
            .documents
            .get_mut(id)
            .ok_or_else(|| MupdfServerError::DocumentNotFound(id.to_string()))?;

        stored.touch();
        f(&stored.document)
    }

    /// Execute a function with mutable access to a document.
    pub fn with_document_mut<F, T>(&self, id: &str, f: F) -> Result<T>
    where
        F: FnOnce(&mut Document) -> Result<T>,
    {
        let mut inner = self.inner.lock().map_err(|e| {
            MupdfServerError::internal(format!("Failed to lock document store: {}", e))
        })?;

        let stored = inner
            .documents
            .get_mut(id)
            .ok_or_else(|| MupdfServerError::DocumentNotFound(id.to_string()))?;

        stored.touch();
        f(&mut stored.document)
    }

    /// Remove a document from the store.
    pub fn remove(&self, id: &str) -> Result<()> {
        let mut inner = self.inner.lock().map_err(|e| {
            MupdfServerError::internal(format!("Failed to lock document store: {}", e))
        })?;

        if inner.documents.remove(id).is_none() {
            return Err(MupdfServerError::DocumentNotFound(id.to_string()));
        }

        Ok(())
    }

    /// List all documents in the store.
    pub fn list(&self) -> Result<Vec<DocumentInfo>> {
        let inner = self.inner.lock().map_err(|e| {
            MupdfServerError::internal(format!("Failed to lock document store: {}", e))
        })?;

        Ok(inner.documents.values().map(|d| d.info.clone()).collect())
    }

    /// Get the number of documents in the store.
    pub fn len(&self) -> Result<usize> {
        let inner = self.inner.lock().map_err(|e| {
            MupdfServerError::internal(format!("Failed to lock document store: {}", e))
        })?;

        Ok(inner.documents.len())
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require actual PDF files to work.
    // For now, we just test the basic store operations.

    #[test]
    fn test_store_new() {
        let store = DocumentStore::new();
        assert!(store.is_empty().unwrap());
    }

    #[test]
    fn test_store_list_empty() {
        let store = DocumentStore::new();
        let list = store.list().unwrap();
        assert!(list.is_empty());
    }
}
