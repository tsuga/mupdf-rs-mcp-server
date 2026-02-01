# MuPDF MCP Server

[![CI](https://github.com/tsuga/mupdf-rs-mcp-server/actions/workflows/ci.yml/badge.svg)](https://github.com/tsuga/mupdf-rs-mcp-server/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/tsuga/mupdf-rs-mcp-server/branch/main/graph/badge.svg)](https://codecov.io/gh/tsuga/mupdf-rs-mcp-server)

An MCP (Model Context Protocol) server that exposes PDF reading and manipulation capabilities using [MuPDF](https://mupdf.com/) via [mupdf-rs](https://github.com/messense/mupdf-rs).

## Architecture

This server provides **two usage patterns**:

### STATEFUL API (for multiple operations)
1. `import_document` → receive a `document_id`
2. Perform operations using the `document_id`
3. `close_document` when done
4. **Use when:** You need multiple operations on the same document

### ONESHOT API (for single operations)
1. Call `oneshot_*` tools with file path or base64 directly
2. Get result immediately, no cleanup needed
3. **Use when:** You only need one operation on the document

## Installation

### Using Docker (Recommended)

```bash
# Build release binary
docker compose run --rm build-release

# The binary will be at target/release/mupdf-mcp-server
```

### Native Build

Requires: Rust nightly, libclang, freetype, harfbuzz, libjpeg, openjp2, zlib

```bash
cargo build --release
```

## Development

### Running Tests

```bash
# Run tests via Docker (recommended)
docker compose run --rm test

# Run tests natively (requires dependencies installed)
cargo test
```

### Test Coverage

```bash
# Generate HTML coverage report (output in target/llvm-cov/html/)
docker compose run --rm coverage

# Generate LCOV report (for CI/Codecov)
docker compose run --rm coverage-lcov
```

### Test Fixtures

Test PDF files are located in `tests/fixtures/` and sourced from [mupdf-rs](https://github.com/messense/mupdf-rs/tree/main/tests/files).

## Usage

### With Claude Desktop

Add to your Claude Desktop configuration (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "mupdf": {
      "command": "/path/to/mupdf-mcp-server"
    }
  }
}
```

### Standalone

```bash
./target/release/mupdf-mcp-server
```

The server communicates via STDIO using the MCP protocol.

---

## Feature Checklist

### STATEFUL API

#### Session Management
- [x] `import_document` - Import document (file path or base64) → returns document_id
- [x] `close_document` - Close document and free memory
- [x] `list_documents` - List open documents

#### Document Operations (requires document_id)
- [x] `get_metadata` - Get document metadata (title, author, subject, keywords, creator, producer, creation_date, modification_date)
- [x] `get_page_count` - Get total page count
- [x] `get_outlines` - Get table of contents/bookmarks with page numbers
- [x] `needs_password` - Check if password is required
- [ ] `authenticate` - Unlock document with password
- [x] `is_pdf` - Check if document is PDF format
- [x] `is_reflowable` - Check if document is reflowable (e.g., EPUB)
- [x] `resolve_link` - Resolve link URI to destination page

#### Page Operations (requires document_id + page_number)
- [x] `get_page_bounds` - Get page dimensions (width, height)
- [x] `get_page_links` - Get hyperlinks on page with bounds and URIs
- [x] `search_page` - Find text on page, return match coordinates
- [x] `get_page_text` - Extract text in various formats (plain, html, json, xml)
- [x] `get_page_text_blocks` - Get structured text blocks with positioning
- [x] `render_page` - Render page to PNG (base64 encoded)

#### PDF Modification (requires document_id)
- [ ] `create_blank_pdf` - Create new empty PDF → returns document_id
- [ ] `add_page` - Add new blank page at specified position
- [ ] `delete_page` - Delete page at specified position
- [ ] `set_page_rotation` - Set page rotation (0, 90, 180, 270)
- [ ] `set_outlines` - Set/update bookmarks
- [ ] `save_document` - Save document to file path
- [ ] `export_document` - Export document as base64

### ONESHOT API (no document_id needed)
- [x] `oneshot_get_bookmarks` - Extract all bookmarks with their target page numbers

---

## API Reference

### Session Management

#### `import_document`
Import a document to the server.

**Parameters:**
- `source`: Object with either:
  - `path`: String - File path to PDF
  - `base64`: String - Base64-encoded PDF content
  - `filename`: String (optional) - Filename hint for base64 content
- `password`: String (optional) - Password for encrypted PDFs

**Returns:**
- `document_id`: String - UUID to reference this document

#### `close_document`
Close a document and free its memory.

**Parameters:**
- `document_id`: String - Document ID from import_document

#### `list_documents`
List all open documents.

**Returns:**
- `documents`: Array of objects with:
  - `document_id`: String
  - `page_count`: Number
  - `created_at`: String (ISO timestamp)

### Document Operations

#### `get_page_count`
Get the total number of pages.

**Parameters:**
- `document_id`: String

**Returns:**
- `page_count`: Number

#### `get_metadata`
Get document metadata.

**Parameters:**
- `document_id`: String

**Returns:**
- `title`: String or null
- `author`: String or null
- `subject`: String or null
- `keywords`: String or null
- `creator`: String or null
- `producer`: String or null
- `creation_date`: String or null
- `modification_date`: String or null

#### `get_outlines`
Get table of contents/bookmarks.

**Parameters:**
- `document_id`: String

**Returns:**
- `outlines`: Array of outline entries (recursive structure)
  - `title`: String
  - `page`: Number (0-indexed)
  - `children`: Array of outline entries

### Page Operations

#### `get_page_text`
Extract text from a page.

**Parameters:**
- `document_id`: String
- `page`: Number (0-indexed)
- `format`: String (optional) - "plain" (default), "html", "json", "xml"

**Returns:**
- `text`: String - Extracted text in requested format

#### `render_page`
Render a page to an image.

**Parameters:**
- `document_id`: String
- `page`: Number (0-indexed)
- `scale`: Number (optional, default 1.0)
- `format`: String (optional) - "png" (default) or "svg"

**Returns:**
- `image`: String - Base64-encoded image data
- `width`: Number
- `height`: Number
- `format`: String

### ONESHOT Tools

#### `oneshot_get_bookmarks`
Extract bookmarks with page numbers. No document_id needed.

**Parameters:**
- `source`: Object with either:
  - `path`: String - File path to PDF
  - `base64`: String - Base64-encoded PDF content
- `password`: String (optional)

**Returns:**
- `bookmarks`: Array of bookmark entries
  - `title`: String
  - `page`: Number (0-indexed)
  - `level`: Number (nesting depth, 0 = top level)
- `page_count`: Number

---

## License

AGPL-3.0 (due to MuPDF licensing)

## Credits

- [MuPDF](https://mupdf.com/) - The underlying PDF library
- [mupdf-rs](https://github.com/messense/mupdf-rs) - Rust bindings for MuPDF
- [rmcp](https://github.com/anthropics/rust-sdk) - Rust MCP SDK
