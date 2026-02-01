//! Document-level operations: metadata, page count, outlines, etc.

use mupdf::MetadataName;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::state::DocumentStore;

// ============== Get Page Count ==============

/// Parameters for getting page count.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPageCountParams {
    /// Document ID.
    pub document_id: String,
}

/// Result of getting page count.
#[derive(Debug, Serialize, JsonSchema)]
pub struct GetPageCountResult {
    /// Total number of pages in the document.
    pub page_count: i32,
}

/// Get the total number of pages in a document.
pub fn get_page_count(
    store: &DocumentStore,
    params: GetPageCountParams,
) -> Result<GetPageCountResult> {
    let info = store.get_info(&params.document_id)?;
    Ok(GetPageCountResult {
        page_count: info.page_count,
    })
}

// ============== Get Metadata ==============

/// Parameters for getting document metadata.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetMetadataParams {
    /// Document ID.
    pub document_id: String,
}

/// Document metadata.
#[derive(Debug, Serialize, JsonSchema)]
pub struct GetMetadataResult {
    /// Document title.
    pub title: Option<String>,
    /// Document author.
    pub author: Option<String>,
    /// Document subject.
    pub subject: Option<String>,
    /// Document keywords.
    pub keywords: Option<String>,
    /// Application that created the document.
    pub creator: Option<String>,
    /// Application that produced the PDF.
    pub producer: Option<String>,
    /// Document creation date.
    pub creation_date: Option<String>,
    /// Document modification date.
    pub modification_date: Option<String>,
}

/// Get document metadata.
pub fn get_metadata(store: &DocumentStore, params: GetMetadataParams) -> Result<GetMetadataResult> {
    store.with_document(&params.document_id, |doc| {
        Ok(GetMetadataResult {
            title: doc
                .metadata(MetadataName::Title)
                .ok()
                .filter(|s| !s.is_empty()),
            author: doc
                .metadata(MetadataName::Author)
                .ok()
                .filter(|s| !s.is_empty()),
            subject: doc
                .metadata(MetadataName::Subject)
                .ok()
                .filter(|s| !s.is_empty()),
            keywords: doc
                .metadata(MetadataName::Keywords)
                .ok()
                .filter(|s| !s.is_empty()),
            creator: doc
                .metadata(MetadataName::Creator)
                .ok()
                .filter(|s| !s.is_empty()),
            producer: doc
                .metadata(MetadataName::Producer)
                .ok()
                .filter(|s| !s.is_empty()),
            creation_date: doc
                .metadata(MetadataName::CreationDate)
                .ok()
                .filter(|s| !s.is_empty()),
            modification_date: doc
                .metadata(MetadataName::ModDate)
                .ok()
                .filter(|s| !s.is_empty()),
        })
    })
}

// ============== Get Outlines (Bookmarks) ==============

/// Parameters for getting document outlines.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetOutlinesParams {
    /// Document ID.
    pub document_id: String,
}

/// A single outline entry (bookmark).
#[derive(Debug, Serialize, JsonSchema)]
pub struct OutlineEntry {
    /// Bookmark title.
    pub title: String,
    /// Target page number (0-indexed).
    pub page: Option<i32>,
    /// URI for external links.
    pub uri: Option<String>,
    /// Child bookmarks.
    pub children: Vec<OutlineEntry>,
}

/// Result of getting document outlines.
#[derive(Debug, Serialize, JsonSchema)]
pub struct GetOutlinesResult {
    /// Root-level outline entries.
    pub outlines: Vec<OutlineEntry>,
}

/// Convert MuPDF outline to our OutlineEntry format.
fn convert_outline(outline: &mupdf::Outline) -> OutlineEntry {
    // Try to get page number from destination
    let page = outline
        .dest
        .as_ref()
        .map(|dest| dest.loc.page_number as i32);

    let uri = outline.uri.as_ref().and_then(|u| {
        // Only include external URIs, not internal page references
        if u.starts_with("http://") || u.starts_with("https://") || u.starts_with("mailto:") {
            Some(u.clone())
        } else {
            None
        }
    });

    // Recursively convert children using 'down' field (it's a Vec)
    let children: Vec<OutlineEntry> = outline.down.iter().map(convert_outline).collect();

    OutlineEntry {
        title: outline.title.clone(),
        page,
        uri,
        children,
    }
}

/// Get document outlines (table of contents).
pub fn get_outlines(store: &DocumentStore, params: GetOutlinesParams) -> Result<GetOutlinesResult> {
    store.with_document(&params.document_id, |doc| {
        let outline_vec = doc.outlines()?;
        let outlines: Vec<OutlineEntry> = outline_vec.iter().map(convert_outline).collect();

        Ok(GetOutlinesResult { outlines })
    })
}

// ============== Needs Password ==============

/// Parameters for checking if document needs password.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct NeedsPasswordParams {
    /// Document ID.
    pub document_id: String,
}

/// Result of password check.
#[derive(Debug, Serialize, JsonSchema)]
pub struct NeedsPasswordResult {
    /// Whether the document requires a password.
    pub needs_password: bool,
}

/// Check if a document requires a password.
pub fn needs_password(
    store: &DocumentStore,
    params: NeedsPasswordParams,
) -> Result<NeedsPasswordResult> {
    store.with_document(&params.document_id, |doc| {
        Ok(NeedsPasswordResult {
            needs_password: doc.needs_password()?,
        })
    })
}

// ============== Is PDF ==============

/// Parameters for checking if document is PDF.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct IsPdfParams {
    /// Document ID.
    pub document_id: String,
}

/// Result of PDF check.
#[derive(Debug, Serialize, JsonSchema)]
pub struct IsPdfResult {
    /// Whether the document is a PDF.
    pub is_pdf: bool,
}

/// Check if a document is a PDF.
pub fn is_pdf(store: &DocumentStore, params: IsPdfParams) -> Result<IsPdfResult> {
    store.with_document(&params.document_id, |doc| {
        Ok(IsPdfResult {
            is_pdf: doc.is_pdf(),
        })
    })
}

// ============== Is Reflowable ==============

/// Parameters for checking if document is reflowable.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct IsReflowableParams {
    /// Document ID.
    pub document_id: String,
}

/// Result of reflowable check.
#[derive(Debug, Serialize, JsonSchema)]
pub struct IsReflowableResult {
    /// Whether the document is reflowable (e.g., EPUB).
    pub is_reflowable: bool,
}

/// Check if a document is reflowable.
pub fn is_reflowable(
    store: &DocumentStore,
    params: IsReflowableParams,
) -> Result<IsReflowableResult> {
    store.with_document(&params.document_id, |doc| {
        Ok(IsReflowableResult {
            is_reflowable: doc.is_reflowable()?,
        })
    })
}

// ============== Resolve Link ==============

/// Parameters for resolving a link.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ResolveLinkParams {
    /// Document ID.
    pub document_id: String,
    /// URI to resolve.
    pub uri: String,
}

/// Result of link resolution.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ResolveLinkResult {
    /// Target page number (0-indexed), if internal link.
    pub page: Option<i32>,
    /// X coordinate on target page.
    pub x: Option<f32>,
    /// Y coordinate on target page.
    pub y: Option<f32>,
}

/// Resolve a link URI to a destination.
pub fn resolve_link(store: &DocumentStore, params: ResolveLinkParams) -> Result<ResolveLinkResult> {
    store.with_document(&params.document_id, |doc| {
        let dest = doc.resolve_link(&params.uri)?;
        match dest {
            Some(d) => {
                // Location has page_number (u32), chapter, page_in_chapter
                // LinkDestination only has loc and kind, no x/y coordinates
                Ok(ResolveLinkResult {
                    page: Some(d.loc.page_number as i32),
                    x: None,
                    y: None,
                })
            }
            None => Ok(ResolveLinkResult {
                page: None,
                x: None,
                y: None,
            }),
        }
    })
}
