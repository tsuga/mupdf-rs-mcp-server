//! Integration tests using real PDF files.
//!
//! These tests verify that the MCP tools work correctly with actual PDF documents.

use mupdf_rs_mcp_server::state::DocumentStore;
use mupdf_rs_mcp_server::tools::*;

/// Path to test PDF file.
const DUMMY_PDF: &[u8] = include_bytes!("fixtures/dummy.pdf");

/// Path to encrypted test PDF file (for future password tests).
#[allow(dead_code)]
const DUMMY_ENCRYPTED_PDF: &[u8] = include_bytes!("fixtures/dummy-encrypted.pdf");

// ============== Session Management Tests ==============

mod session {
    use super::*;

    #[test]
    fn test_import_document_from_base64() {
        let store = DocumentStore::new();
        let base64_content =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, DUMMY_PDF);

        let params = ImportDocumentParams {
            source: DocumentSource::Base64 {
                base64: base64_content,
                filename: Some("dummy.pdf".to_string()),
            },
            password: None,
        };

        let result = import_document(&store, params).unwrap();
        assert!(!result.document_id.is_empty());
        assert!(result.page_count > 0);

        // Clean up
        close_document(
            &store,
            CloseDocumentParams {
                document_id: result.document_id,
            },
        )
        .unwrap();
    }

    #[test]
    fn test_list_documents() {
        let store = DocumentStore::new();

        // Initially empty
        let list = list_documents(&store, ListDocumentsParams {}).unwrap();
        assert!(list.documents.is_empty());

        // Import a document
        let base64_content =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, DUMMY_PDF);
        let import_result = import_document(
            &store,
            ImportDocumentParams {
                source: DocumentSource::Base64 {
                    base64: base64_content,
                    filename: Some("dummy.pdf".to_string()),
                },
                password: None,
            },
        )
        .unwrap();

        // Now should have one document
        let list = list_documents(&store, ListDocumentsParams {}).unwrap();
        assert_eq!(list.documents.len(), 1);
        assert_eq!(list.documents[0].document_id, import_result.document_id);

        // Clean up
        close_document(
            &store,
            CloseDocumentParams {
                document_id: import_result.document_id,
            },
        )
        .unwrap();
    }

    #[test]
    fn test_close_document() {
        let store = DocumentStore::new();

        let base64_content =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, DUMMY_PDF);
        let import_result = import_document(
            &store,
            ImportDocumentParams {
                source: DocumentSource::Base64 {
                    base64: base64_content,
                    filename: Some("dummy.pdf".to_string()),
                },
                password: None,
            },
        )
        .unwrap();

        // Close the document
        let close_result = close_document(
            &store,
            CloseDocumentParams {
                document_id: import_result.document_id.clone(),
            },
        )
        .unwrap();
        assert!(close_result.success);

        // Should be empty now
        let list = list_documents(&store, ListDocumentsParams {}).unwrap();
        assert!(list.documents.is_empty());
    }

    #[test]
    fn test_close_nonexistent_document() {
        let store = DocumentStore::new();

        let result = close_document(
            &store,
            CloseDocumentParams {
                document_id: "nonexistent-id".to_string(),
            },
        );

        assert!(result.is_err());
    }
}

// ============== Document Operations Tests ==============

mod document {
    use super::*;

    fn setup_document(store: &DocumentStore) -> String {
        let base64_content =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, DUMMY_PDF);
        import_document(
            store,
            ImportDocumentParams {
                source: DocumentSource::Base64 {
                    base64: base64_content,
                    filename: Some("dummy.pdf".to_string()),
                },
                password: None,
            },
        )
        .unwrap()
        .document_id
    }

    #[test]
    fn test_get_page_count() {
        let store = DocumentStore::new();
        let doc_id = setup_document(&store);

        let result = get_page_count(
            &store,
            GetPageCountParams {
                document_id: doc_id.clone(),
            },
        )
        .unwrap();

        assert!(result.page_count > 0);

        close_document(
            &store,
            CloseDocumentParams {
                document_id: doc_id,
            },
        )
        .unwrap();
    }

    #[test]
    fn test_get_metadata() {
        let store = DocumentStore::new();
        let doc_id = setup_document(&store);

        let result = get_metadata(
            &store,
            GetMetadataParams {
                document_id: doc_id.clone(),
            },
        )
        .unwrap();

        // The metadata may or may not be present, but the call should succeed
        // Just verify it returns a valid result structure
        let _ = result.title;
        let _ = result.author;
        let _ = result.subject;

        close_document(
            &store,
            CloseDocumentParams {
                document_id: doc_id,
            },
        )
        .unwrap();
    }

    #[test]
    fn test_get_outlines() {
        let store = DocumentStore::new();
        let doc_id = setup_document(&store);

        let result = get_outlines(
            &store,
            GetOutlinesParams {
                document_id: doc_id.clone(),
            },
        )
        .unwrap();

        // May or may not have outlines
        let _ = result.outlines;

        close_document(
            &store,
            CloseDocumentParams {
                document_id: doc_id,
            },
        )
        .unwrap();
    }
}

// ============== Page Operations Tests ==============

mod page {
    use super::*;

    fn setup_document(store: &DocumentStore) -> String {
        let base64_content =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, DUMMY_PDF);
        import_document(
            store,
            ImportDocumentParams {
                source: DocumentSource::Base64 {
                    base64: base64_content,
                    filename: Some("dummy.pdf".to_string()),
                },
                password: None,
            },
        )
        .unwrap()
        .document_id
    }

    #[test]
    fn test_get_page_bounds() {
        let store = DocumentStore::new();
        let doc_id = setup_document(&store);

        let result = get_page_bounds(
            &store,
            GetPageBoundsParams {
                document_id: doc_id.clone(),
                page: 0,
            },
        )
        .unwrap();

        assert!(result.width > 0.0);
        assert!(result.height > 0.0);

        close_document(
            &store,
            CloseDocumentParams {
                document_id: doc_id,
            },
        )
        .unwrap();
    }

    #[test]
    fn test_get_page_bounds_invalid_page() {
        let store = DocumentStore::new();
        let doc_id = setup_document(&store);

        let result = get_page_bounds(
            &store,
            GetPageBoundsParams {
                document_id: doc_id.clone(),
                page: 9999, // Invalid page
            },
        );

        assert!(result.is_err());

        close_document(
            &store,
            CloseDocumentParams {
                document_id: doc_id,
            },
        )
        .unwrap();
    }

    #[test]
    fn test_get_page_links() {
        let store = DocumentStore::new();
        let doc_id = setup_document(&store);

        let result = get_page_links(
            &store,
            GetPageLinksParams {
                document_id: doc_id.clone(),
                page: 0,
            },
        )
        .unwrap();

        // May or may not have links
        let _ = result.links;

        close_document(
            &store,
            CloseDocumentParams {
                document_id: doc_id,
            },
        )
        .unwrap();
    }
}

// ============== Text Extraction Tests ==============

mod text {
    use super::*;

    fn setup_document(store: &DocumentStore) -> String {
        let base64_content =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, DUMMY_PDF);
        import_document(
            store,
            ImportDocumentParams {
                source: DocumentSource::Base64 {
                    base64: base64_content,
                    filename: Some("dummy.pdf".to_string()),
                },
                password: None,
            },
        )
        .unwrap()
        .document_id
    }

    #[test]
    fn test_get_page_text_plain() {
        let store = DocumentStore::new();
        let doc_id = setup_document(&store);

        let result = get_page_text(
            &store,
            GetPageTextParams {
                document_id: doc_id.clone(),
                page: 0,
                format: "plain".to_string(),
            },
        )
        .unwrap();

        // Text extraction should succeed
        let _ = result.text;

        close_document(
            &store,
            CloseDocumentParams {
                document_id: doc_id,
            },
        )
        .unwrap();
    }

    #[test]
    fn test_get_page_text_html() {
        let store = DocumentStore::new();
        let doc_id = setup_document(&store);

        let result = get_page_text(
            &store,
            GetPageTextParams {
                document_id: doc_id.clone(),
                page: 0,
                format: "html".to_string(),
            },
        )
        .unwrap();

        // HTML output should contain HTML tags
        assert!(result.text.contains("<") || result.text.is_empty());

        close_document(
            &store,
            CloseDocumentParams {
                document_id: doc_id,
            },
        )
        .unwrap();
    }

    #[test]
    fn test_get_page_text_json() {
        let store = DocumentStore::new();
        let doc_id = setup_document(&store);

        let result = get_page_text(
            &store,
            GetPageTextParams {
                document_id: doc_id.clone(),
                page: 0,
                format: "json".to_string(),
            },
        )
        .unwrap();

        // JSON should be valid
        if !result.text.is_empty() {
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&result.text);
            assert!(parsed.is_ok(), "JSON parsing failed: {}", result.text);
        }

        close_document(
            &store,
            CloseDocumentParams {
                document_id: doc_id,
            },
        )
        .unwrap();
    }

    #[test]
    fn test_search_page() {
        let store = DocumentStore::new();
        let doc_id = setup_document(&store);

        // Search for a common word that might be in the document
        let result = search_page(
            &store,
            SearchPageParams {
                document_id: doc_id.clone(),
                page: 0,
                query: "the".to_string(),
            },
        )
        .unwrap();

        // Results may or may not be found
        let _ = result.hits;

        close_document(
            &store,
            CloseDocumentParams {
                document_id: doc_id,
            },
        )
        .unwrap();
    }

    #[test]
    fn test_get_page_text_blocks() {
        let store = DocumentStore::new();
        let doc_id = setup_document(&store);

        let result = get_page_text_blocks(
            &store,
            GetPageTextBlocksParams {
                document_id: doc_id.clone(),
                page: 0,
            },
        )
        .unwrap();

        // Should have some blocks
        let _ = result.blocks;

        close_document(
            &store,
            CloseDocumentParams {
                document_id: doc_id,
            },
        )
        .unwrap();
    }
}

// ============== Render Tests ==============

mod render {
    use super::*;

    fn setup_document(store: &DocumentStore) -> String {
        let base64_content =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, DUMMY_PDF);
        import_document(
            store,
            ImportDocumentParams {
                source: DocumentSource::Base64 {
                    base64: base64_content,
                    filename: Some("dummy.pdf".to_string()),
                },
                password: None,
            },
        )
        .unwrap()
        .document_id
    }

    #[test]
    fn test_render_page() {
        let store = DocumentStore::new();
        let doc_id = setup_document(&store);

        let result = render_page(
            &store,
            RenderPageParams {
                document_id: doc_id.clone(),
                page: 0,
                scale: 1.0,
            },
        )
        .unwrap();

        // Should return valid PNG data
        assert!(!result.image.is_empty());
        assert!(result.width > 0);
        assert!(result.height > 0);

        // Verify it's valid base64
        let decoded =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &result.image);
        assert!(decoded.is_ok());

        // Verify PNG magic bytes
        let bytes = decoded.unwrap();
        assert!(bytes.len() > 8);
        assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47]); // PNG signature

        close_document(
            &store,
            CloseDocumentParams {
                document_id: doc_id,
            },
        )
        .unwrap();
    }

    #[test]
    fn test_render_page_with_scale() {
        let store = DocumentStore::new();
        let doc_id = setup_document(&store);

        let result_1x = render_page(
            &store,
            RenderPageParams {
                document_id: doc_id.clone(),
                page: 0,
                scale: 1.0,
            },
        )
        .unwrap();

        let result_2x = render_page(
            &store,
            RenderPageParams {
                document_id: doc_id.clone(),
                page: 0,
                scale: 2.0,
            },
        )
        .unwrap();

        // 2x scale should produce larger dimensions
        assert_eq!(result_2x.width, result_1x.width * 2);
        assert_eq!(result_2x.height, result_1x.height * 2);

        close_document(
            &store,
            CloseDocumentParams {
                document_id: doc_id,
            },
        )
        .unwrap();
    }
}

// ============== Oneshot Tests ==============

mod oneshot {
    use super::*;

    #[test]
    fn test_oneshot_get_bookmarks() {
        let base64_content =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, DUMMY_PDF);

        let result = oneshot_get_bookmarks(OneshotGetBookmarksParams {
            source: DocumentSource::Base64 {
                base64: base64_content,
                filename: Some("dummy.pdf".to_string()),
            },
            password: None,
        })
        .unwrap();

        // Should return page count
        assert!(result.page_count > 0);
        // Bookmarks may or may not exist
        let _ = result.bookmarks;
    }
}

// ============== Error Handling Tests ==============

mod errors {
    use super::*;

    #[test]
    fn test_document_not_found() {
        let store = DocumentStore::new();

        let result = get_page_count(
            &store,
            GetPageCountParams {
                document_id: "nonexistent-doc-id".to_string(),
            },
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_base64() {
        let store = DocumentStore::new();

        let result = import_document(
            &store,
            ImportDocumentParams {
                source: DocumentSource::Base64 {
                    base64: "not-valid-base64!!!".to_string(),
                    filename: Some("test.pdf".to_string()),
                },
                password: None,
            },
        );

        assert!(result.is_err());
    }
}
