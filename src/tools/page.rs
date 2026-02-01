//! Page-level operations: bounds, links, search, render.

use base64::Engine;
use mupdf::{Colorspace, Matrix};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{MupdfServerError, Result};
use crate::state::DocumentStore;

/// Validate page number and return the page.
fn validate_page_number(doc: &mupdf::Document, page: i32) -> Result<()> {
    let page_count = doc.page_count()?;
    if page < 0 || page >= page_count {
        return Err(MupdfServerError::InvalidPageNumber {
            page,
            total: page_count,
            max: page_count - 1,
        });
    }
    Ok(())
}

// ============== Get Page Bounds ==============

/// Parameters for getting page bounds.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPageBoundsParams {
    /// Document ID.
    pub document_id: String,
    /// Page number (0-indexed).
    pub page: i32,
}

/// Page dimensions.
#[derive(Debug, Serialize, JsonSchema)]
pub struct GetPageBoundsResult {
    /// Page width in points.
    pub width: f32,
    /// Page height in points.
    pub height: f32,
    /// X origin (usually 0).
    pub x0: f32,
    /// Y origin (usually 0).
    pub y0: f32,
}

/// Get the dimensions of a page.
pub fn get_page_bounds(
    store: &DocumentStore,
    params: GetPageBoundsParams,
) -> Result<GetPageBoundsResult> {
    store.with_document(&params.document_id, |doc| {
        validate_page_number(doc, params.page)?;
        let page = doc.load_page(params.page)?;
        let bounds = page.bounds()?;

        Ok(GetPageBoundsResult {
            width: bounds.width(),
            height: bounds.height(),
            x0: bounds.x0,
            y0: bounds.y0,
        })
    })
}

// ============== Get Page Links ==============

/// Parameters for getting page links.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPageLinksParams {
    /// Document ID.
    pub document_id: String,
    /// Page number (0-indexed).
    pub page: i32,
}

/// A hyperlink on a page.
#[derive(Debug, Serialize, JsonSchema)]
pub struct PageLink {
    /// Link bounding box.
    pub bounds: LinkBounds,
    /// Link URI (internal or external).
    pub uri: Option<String>,
    /// Target page number (for internal links).
    pub target_page: Option<i32>,
}

/// Bounding box for a link.
#[derive(Debug, Serialize, JsonSchema)]
pub struct LinkBounds {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

/// Result of getting page links.
#[derive(Debug, Serialize, JsonSchema)]
pub struct GetPageLinksResult {
    /// Links found on the page.
    pub links: Vec<PageLink>,
}

/// Get all hyperlinks on a page.
pub fn get_page_links(
    store: &DocumentStore,
    params: GetPageLinksParams,
) -> Result<GetPageLinksResult> {
    store.with_document(&params.document_id, |doc| {
        validate_page_number(doc, params.page)?;
        let page = doc.load_page(params.page)?;

        let mut links = Vec::new();
        for link in page.links()? {
            let target_page = doc.resolve_link(&link.uri)
                .ok()
                .flatten()
                .map(|dest| dest.loc.page_number as i32);

            links.push(PageLink {
                bounds: LinkBounds {
                    x0: link.bounds.x0,
                    y0: link.bounds.y0,
                    x1: link.bounds.x1,
                    y1: link.bounds.y1,
                },
                uri: Some(link.uri.clone()),
                target_page,
            });
        }

        Ok(GetPageLinksResult { links })
    })
}

// ============== Search Page ==============

/// Parameters for searching a page.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchPageParams {
    /// Document ID.
    pub document_id: String,
    /// Page number (0-indexed).
    pub page: i32,
    /// Text to search for.
    pub query: String,
}

/// A search hit with its bounding quad.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SearchHit {
    /// Upper-left corner.
    pub ul: Point,
    /// Upper-right corner.
    pub ur: Point,
    /// Lower-left corner.
    pub ll: Point,
    /// Lower-right corner.
    pub lr: Point,
}

/// A 2D point.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// Result of searching a page.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SearchPageResult {
    /// Search hits with their bounding quads.
    pub hits: Vec<SearchHit>,
}

/// Search for text on a page.
pub fn search_page(
    store: &DocumentStore,
    params: SearchPageParams,
) -> Result<SearchPageResult> {
    store.with_document(&params.document_id, |doc| {
        validate_page_number(doc, params.page)?;
        let page = doc.load_page(params.page)?;

        // Search with a reasonable hit limit
        let hits: Vec<SearchHit> = page
            .search(&params.query, 100)?
            .iter()
            .map(|quad| SearchHit {
                ul: Point {
                    x: quad.ul.x,
                    y: quad.ul.y,
                },
                ur: Point {
                    x: quad.ur.x,
                    y: quad.ur.y,
                },
                ll: Point {
                    x: quad.ll.x,
                    y: quad.ll.y,
                },
                lr: Point {
                    x: quad.lr.x,
                    y: quad.lr.y,
                },
            })
            .collect();

        Ok(SearchPageResult { hits })
    })
}

// ============== Render Page ==============

/// Parameters for rendering a page.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RenderPageParams {
    /// Document ID.
    pub document_id: String,
    /// Page number (0-indexed).
    pub page: i32,
    /// Scale factor (default 1.0 = 72 DPI).
    #[serde(default = "default_scale")]
    pub scale: f32,
}

fn default_scale() -> f32 {
    1.0
}

/// Result of rendering a page.
#[derive(Debug, Serialize, JsonSchema)]
pub struct RenderPageResult {
    /// Base64-encoded PNG image data.
    pub image: String,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Image format.
    pub format: String,
}

/// Render a page to a PNG image.
pub fn render_page(
    store: &DocumentStore,
    params: RenderPageParams,
) -> Result<RenderPageResult> {
    store.with_document(&params.document_id, |doc| {
        validate_page_number(doc, params.page)?;
        let page = doc.load_page(params.page)?;

        let matrix = Matrix::new_scale(params.scale, params.scale);
        let pixmap = page.to_pixmap(&matrix, &Colorspace::device_rgb(), false, true)?;

        let width = pixmap.width();
        let height = pixmap.height();

        // Write to PNG bytes using the pixmap's write method
        let mut png_buffer = Vec::new();
        pixmap.write_to(&mut png_buffer, mupdf::ImageFormat::PNG)?;
        let image = base64::engine::general_purpose::STANDARD.encode(&png_buffer);

        Ok(RenderPageResult {
            image,
            width,
            height,
            format: "png".to_string(),
        })
    })
}
