//! Text extraction tools.

use mupdf::TextPageFlags;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{MupdfServerError, Result};
use crate::state::DocumentStore;

/// Validate page number.
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

// ============== Get Page Text ==============

/// Parameters for extracting page text.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPageTextParams {
    /// Document ID.
    pub document_id: String,
    /// Page number (0-indexed).
    pub page: i32,
    /// Output format: "plain", "html", "json", "xml".
    #[serde(default = "default_text_format")]
    pub format: String,
}

fn default_text_format() -> String {
    "plain".to_string()
}

/// Result of text extraction.
#[derive(Debug, Serialize, JsonSchema)]
pub struct GetPageTextResult {
    /// Extracted text in the requested format.
    pub text: String,
    /// Format of the text.
    pub format: String,
}

/// Extract text from a page in the specified format.
pub fn get_page_text(
    store: &DocumentStore,
    params: GetPageTextParams,
) -> Result<GetPageTextResult> {
    store.with_document(&params.document_id, |doc| {
        validate_page_number(doc, params.page)?;
        let page = doc.load_page(params.page)?;
        let text_page = page.to_text_page(TextPageFlags::empty())?;

        let text = match params.format.as_str() {
            "plain" => {
                // Extract plain text by iterating through blocks
                let mut result = String::new();
                for block in text_page.blocks() {
                    for line in block.lines() {
                        for ch in line.chars() {
                            if let Some(c) = ch.char() {
                                result.push(c);
                            }
                        }
                        result.push('\n');
                    }
                    result.push('\n');
                }
                result
            }
            "html" => text_page.to_html(0, true)?,
            "json" => text_page.to_json(1.0)?,
            "xml" => text_page.to_xml(0)?,
            other => return Err(MupdfServerError::InvalidTextFormat(other.to_string())),
        };

        Ok(GetPageTextResult {
            text,
            format: params.format,
        })
    })
}

// ============== Get Page Text Blocks ==============

/// Parameters for extracting structured text blocks.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPageTextBlocksParams {
    /// Document ID.
    pub document_id: String,
    /// Page number (0-indexed).
    pub page: i32,
}

/// A text block on a page.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TextBlock {
    /// Block bounding box.
    pub bounds: BlockBounds,
    /// Lines in this block.
    pub lines: Vec<TextLine>,
}

/// Bounding box for a text block.
#[derive(Debug, Serialize, JsonSchema)]
pub struct BlockBounds {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

/// A line of text.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TextLine {
    /// Line bounding box.
    pub bounds: BlockBounds,
    /// Text content of the line.
    pub text: String,
}

/// Result of extracting text blocks.
#[derive(Debug, Serialize, JsonSchema)]
pub struct GetPageTextBlocksResult {
    /// Text blocks on the page.
    pub blocks: Vec<TextBlock>,
}

/// Extract structured text blocks from a page.
pub fn get_page_text_blocks(
    store: &DocumentStore,
    params: GetPageTextBlocksParams,
) -> Result<GetPageTextBlocksResult> {
    store.with_document(&params.document_id, |doc| {
        validate_page_number(doc, params.page)?;
        let page = doc.load_page(params.page)?;
        let text_page = page.to_text_page(TextPageFlags::empty())?;

        let mut blocks = Vec::new();

        for block in text_page.blocks() {
            let block_bounds = block.bounds();
            let mut lines = Vec::new();

            for line in block.lines() {
                let line_bounds = line.bounds();
                let text: String = line.chars().filter_map(|c| c.char()).collect();

                lines.push(TextLine {
                    bounds: BlockBounds {
                        x0: line_bounds.x0,
                        y0: line_bounds.y0,
                        x1: line_bounds.x1,
                        y1: line_bounds.y1,
                    },
                    text,
                });
            }

            blocks.push(TextBlock {
                bounds: BlockBounds {
                    x0: block_bounds.x0,
                    y0: block_bounds.y0,
                    x1: block_bounds.x1,
                    y1: block_bounds.y1,
                },
                lines,
            });
        }

        Ok(GetPageTextBlocksResult { blocks })
    })
}
