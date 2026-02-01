//! MCP server implementation with tool routing.

use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, Implementation, InitializeResult,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData as McpError, ServerHandler};
use serde_json::Value;
use std::borrow::Cow;
use std::sync::Arc;

use crate::state::DocumentStore;
use crate::tools;

/// MuPDF MCP Server.
///
/// Provides PDF reading and manipulation capabilities via MCP.
pub struct MupdfServer {
    /// Document store for stateful operations.
    store: DocumentStore,
}

impl MupdfServer {
    /// Create a new MuPDF MCP server.
    pub fn new() -> Self {
        Self {
            store: DocumentStore::new(),
        }
    }

    fn make_tool(name: &str, description: &str, schema: Value) -> Tool {
        Tool {
            name: Cow::Owned(name.to_string()),
            title: None,
            description: Some(Cow::Owned(description.to_string())),
            input_schema: Arc::new(serde_json::from_value(schema).unwrap_or_default()),
            output_schema: None,
            annotations: None,
            icons: None,
            meta: None,
        }
    }
}

impl Default for MupdfServer {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerHandler for MupdfServer {
    fn get_info(&self) -> InitializeResult {
        InitializeResult {
            protocol_version: Default::default(),
            server_info: Implementation {
                name: "mupdf-mcp-server".to_string(),
                title: Some("MuPDF MCP Server".to_string()),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "MuPDF-based PDF processing server. \
                 \
                 TWO USAGE PATTERNS: \
                 \
                 1. STATEFUL (for multiple operations on same document): \
                    import_document → get document_id → use document_id with other tools → close_document. \
                 \
                 2. ONESHOT (for single operation, no state management): \
                    Use tools prefixed with 'oneshot_' - they accept file path or base64 directly and handle everything in one call. \
                 \
                 Choose ONESHOT when you only need one operation. Choose STATEFUL when you need multiple operations on the same document."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_
    {
        async move {
            let tools = vec![
                // Session Management (STATEFUL API - requires document_id)
                Self::make_tool(
                    "import_document",
                    "[STATEFUL] Import a document to the server. Returns a document_id for subsequent operations. Use this when you need multiple operations on the same document. Remember to call close_document when done.",
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "source": {
                                "oneOf": [
                                    {
                                        "type": "object",
                                        "properties": {
                                            "path": { "type": "string", "description": "File path to PDF" }
                                        },
                                        "required": ["path"]
                                    },
                                    {
                                        "type": "object",
                                        "properties": {
                                            "base64": { "type": "string", "description": "Base64-encoded PDF content" },
                                            "filename": { "type": "string", "description": "Optional filename hint" }
                                        },
                                        "required": ["base64"]
                                    }
                                ]
                            },
                            "password": { "type": "string", "description": "Password for encrypted documents" }
                        },
                        "required": ["source"]
                    }),
                ),
                Self::make_tool(
                    "close_document",
                    "[STATEFUL] Close a document and free its memory. Always call this after you're done with a document imported via import_document.",
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "document_id": { "type": "string" }
                        },
                        "required": ["document_id"]
                    }),
                ),
                Self::make_tool(
                    "list_documents",
                    "[STATEFUL] List all open documents with their IDs and page counts.",
                    serde_json::json!({
                        "type": "object",
                        "properties": {}
                    }),
                ),
                // Document Operations (STATEFUL API - requires document_id)
                Self::make_tool(
                    "get_page_count",
                    "[STATEFUL] Get the total number of pages. Requires document_id from import_document.",
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "document_id": { "type": "string" }
                        },
                        "required": ["document_id"]
                    }),
                ),
                Self::make_tool(
                    "get_metadata",
                    "[STATEFUL] Get document metadata (title, author, subject, keywords, etc.). Requires document_id from import_document.",
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "document_id": { "type": "string" }
                        },
                        "required": ["document_id"]
                    }),
                ),
                Self::make_tool(
                    "get_outlines",
                    "[STATEFUL] Get document outlines (table of contents/bookmarks) with page numbers. Requires document_id from import_document.",
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "document_id": { "type": "string" }
                        },
                        "required": ["document_id"]
                    }),
                ),
                // Page Operations (STATEFUL API - requires document_id)
                Self::make_tool(
                    "get_page_bounds",
                    "[STATEFUL] Get the dimensions (width, height) of a page. Requires document_id from import_document.",
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "document_id": { "type": "string" },
                            "page": { "type": "integer", "description": "Page number (0-indexed)" }
                        },
                        "required": ["document_id", "page"]
                    }),
                ),
                Self::make_tool(
                    "get_page_text",
                    "[STATEFUL] Extract text from a page in various formats (plain, html, json, xml). Requires document_id from import_document.",
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "document_id": { "type": "string" },
                            "page": { "type": "integer", "description": "Page number (0-indexed)" },
                            "format": { "type": "string", "enum": ["plain", "html", "json", "xml"], "default": "plain" }
                        },
                        "required": ["document_id", "page"]
                    }),
                ),
                Self::make_tool(
                    "search_page",
                    "[STATEFUL] Search for text on a page. Returns coordinates of all matches. Requires document_id from import_document.",
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "document_id": { "type": "string" },
                            "page": { "type": "integer", "description": "Page number (0-indexed)" },
                            "query": { "type": "string", "description": "Text to search for" }
                        },
                        "required": ["document_id", "page", "query"]
                    }),
                ),
                Self::make_tool(
                    "render_page",
                    "[STATEFUL] Render a page to an image (PNG). Returns base64-encoded data. Requires document_id from import_document.",
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "document_id": { "type": "string" },
                            "page": { "type": "integer", "description": "Page number (0-indexed)" },
                            "scale": { "type": "number", "default": 1.0, "description": "Scale factor (1.0 = 72 DPI)" }
                        },
                        "required": ["document_id", "page"]
                    }),
                ),
                // ONESHOT tools (stateless - no document_id needed)
                Self::make_tool(
                    "oneshot_get_bookmarks",
                    "[ONESHOT] Extract all bookmarks with their target page numbers. No document_id needed - pass file path or base64 directly. Use this for a single operation; use STATEFUL API if you need multiple operations on the same document.",
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "source": {
                                "oneOf": [
                                    {
                                        "type": "object",
                                        "properties": {
                                            "path": { "type": "string", "description": "File path to PDF" }
                                        },
                                        "required": ["path"]
                                    },
                                    {
                                        "type": "object",
                                        "properties": {
                                            "base64": { "type": "string", "description": "Base64-encoded PDF content" },
                                            "filename": { "type": "string", "description": "Optional filename hint" }
                                        },
                                        "required": ["base64"]
                                    }
                                ]
                            },
                            "password": { "type": "string", "description": "Password for encrypted documents" }
                        },
                        "required": ["source"]
                    }),
                ),
            ];

            Ok(ListToolsResult {
                tools,
                next_cursor: None,
                meta: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        let name = request.name.clone();
        let args = request.arguments.clone().unwrap_or_default();

        async move {
            let result = match name.as_ref() {
                "import_document" => {
                    let params: tools::ImportDocumentParams =
                        serde_json::from_value(Value::Object(args))
                            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    tools::import_document(&self.store, params)
                        .map(|r| serde_json::to_string(&r).unwrap())
                }
                "close_document" => {
                    let params: tools::CloseDocumentParams =
                        serde_json::from_value(Value::Object(args))
                            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    tools::close_document(&self.store, params)
                        .map(|r| serde_json::to_string(&r).unwrap())
                }
                "list_documents" => {
                    let params: tools::ListDocumentsParams =
                        serde_json::from_value(Value::Object(args))
                            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    tools::list_documents(&self.store, params)
                        .map(|r| serde_json::to_string(&r).unwrap())
                }
                "get_page_count" => {
                    let params: tools::GetPageCountParams =
                        serde_json::from_value(Value::Object(args))
                            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    tools::get_page_count(&self.store, params)
                        .map(|r| serde_json::to_string(&r).unwrap())
                }
                "get_metadata" => {
                    let params: tools::GetMetadataParams =
                        serde_json::from_value(Value::Object(args))
                            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    tools::get_metadata(&self.store, params)
                        .map(|r| serde_json::to_string(&r).unwrap())
                }
                "get_outlines" => {
                    let params: tools::GetOutlinesParams =
                        serde_json::from_value(Value::Object(args))
                            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    tools::get_outlines(&self.store, params)
                        .map(|r| serde_json::to_string(&r).unwrap())
                }
                "get_page_bounds" => {
                    let params: tools::GetPageBoundsParams =
                        serde_json::from_value(Value::Object(args))
                            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    tools::get_page_bounds(&self.store, params)
                        .map(|r| serde_json::to_string(&r).unwrap())
                }
                "get_page_text" => {
                    let params: tools::GetPageTextParams =
                        serde_json::from_value(Value::Object(args))
                            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    tools::get_page_text(&self.store, params)
                        .map(|r| serde_json::to_string(&r).unwrap())
                }
                "search_page" => {
                    let params: tools::SearchPageParams =
                        serde_json::from_value(Value::Object(args))
                            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    tools::search_page(&self.store, params)
                        .map(|r| serde_json::to_string(&r).unwrap())
                }
                "render_page" => {
                    let params: tools::RenderPageParams =
                        serde_json::from_value(Value::Object(args))
                            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    tools::render_page(&self.store, params)
                        .map(|r| serde_json::to_string(&r).unwrap())
                }
                "oneshot_get_bookmarks" => {
                    let params: tools::OneshotGetBookmarksParams =
                        serde_json::from_value(Value::Object(args))
                            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                    tools::oneshot_get_bookmarks(params)
                        .map(|r| serde_json::to_string(&r).unwrap())
                }
                _ => return Err(McpError::invalid_params(format!("Unknown tool: {}", name), None)),
            };

            match result {
                Ok(json) => Ok(CallToolResult::success(vec![Content::text(json)])),
                Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
            }
        }
    }
}
