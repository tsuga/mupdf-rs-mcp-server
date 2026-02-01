//! Oneshot tools for stateless one-shot operations.
//!
//! These tools don't require document_id - they open, process, and close
//! the document in a single call. Convenient for one-off operations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::tools::session::DocumentSource;

// ============== Oneshot Get Bookmarks ==============

/// Parameters for extracting bookmarks with page numbers (oneshot).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OneshotGetBookmarksParams {
    /// Document source (file path or base64 content).
    pub source: DocumentSource,
    /// Password for encrypted documents (optional).
    #[serde(default)]
    pub password: Option<String>,
}

/// A bookmark entry with its page number.
#[derive(Debug, Serialize, JsonSchema)]
pub struct BookmarkEntry {
    /// Bookmark title.
    pub title: String,
    /// Target page number (0-indexed).
    pub page: Option<i32>,
    /// Nesting level (0 = top level).
    pub level: i32,
}

/// Result of extracting bookmarks.
#[derive(Debug, Serialize, JsonSchema)]
pub struct OneshotGetBookmarksResult {
    /// List of bookmarks with their page numbers.
    pub bookmarks: Vec<BookmarkEntry>,
    /// Total number of pages in the document.
    pub page_count: i32,
}

/// Recursively collect bookmarks from an outline.
fn collect_bookmarks(
    outline: &mupdf::Outline,
    level: i32,
    result: &mut Vec<BookmarkEntry>,
) {
    // Try to get page number from destination
    let page = outline.dest.as_ref().map(|dest| dest.loc.page_number as i32);

    result.push(BookmarkEntry {
        title: outline.title.clone(),
        page,
        level,
    });

    // Recursively process children via 'down' field (it's a Vec)
    for child in &outline.down {
        collect_bookmarks(child, level + 1, result);
    }
}

/// Extract all bookmarks with their target page numbers.
///
/// This is a oneshot (stateless) operation - it opens the document,
/// extracts bookmarks, and closes it in a single call.
pub fn oneshot_get_bookmarks(
    params: OneshotGetBookmarksParams,
) -> Result<OneshotGetBookmarksResult> {
    let doc = params.source.open(params.password.as_deref())?;
    let page_count = doc.page_count()?;

    let mut bookmarks = Vec::new();

    let outlines = doc.outlines()?;
    for outline in &outlines {
        collect_bookmarks(outline, 0, &mut bookmarks);
    }

    Ok(OneshotGetBookmarksResult {
        bookmarks,
        page_count,
    })
}
