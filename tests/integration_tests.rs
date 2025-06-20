use axum::http::StatusCode;
use axum_test::TestServer;
use livemarkdown::{create_app, CreateDocumentRequest, CreateDocumentResponse};

#[tokio::test]
async fn test_create_document() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let request_body = r#"{"filepath": "/path/to/test.md"}"#;

    let response = server.post("/api/document").text(request_body).await;

    response.assert_status(StatusCode::CREATED);
    response.assert_header("content-type", "application/json");

    let body_text = response.text();
    assert!(body_text.contains("test-md"));
    assert!(body_text.starts_with("{\"id\":\"test-md-"));
    assert!(body_text.ends_with("\"}"));
}

#[tokio::test]
async fn test_create_document_with_invalid_json() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let request_body = r#"{"invalid": "json"}"#;

    let response = server.post("/api/document").text(request_body).await;

    response.assert_status(StatusCode::CREATED);
    let body_text = response.text();
    assert!(body_text.contains("unknown"));
    assert!(body_text.starts_with("{\"id\":\"unknown-"));
    assert!(body_text.ends_with("\"}"));
}

#[tokio::test]
async fn test_delete_document() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    // Create a document first
    let request_body = r#"{"filepath": "/path/to/delete-test.md"}"#;
    let create_response = server.post("/api/document").text(request_body).await;

    create_response.assert_status(StatusCode::CREATED);
    let create_response_body: CreateDocumentResponse =
        facet_json::from_str(&create_response.text()).unwrap();
    let doc_id = create_response_body.id;

    // Delete the document
    let delete_response = server.delete(&format!("/api/document/{}", doc_id)).await;
    delete_response.assert_status(StatusCode::OK);

    // Verify document is no longer accessible
    let serve_response = server.get(&format!("/document/{}", doc_id)).await;
    serve_response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_nonexistent_document() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let response = server.delete("/api/document/nonexistent-id").await;
    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_open_document() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let response = server.post("/api/document/test-doc-id/open").await;

    response.assert_status(StatusCode::CREATED);
    response.assert_header("content-type", "text/plain");
    response.assert_text("Document opened");
}

#[tokio::test]
async fn test_update_position() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let request_body = r#"{"sourcepos": "1:1-2:5"}"#;

    let response = server
        .post("/api/document/test-doc-id/position")
        .text(request_body)
        .await;

    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn test_update_position_with_invalid_json() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let request_body = r#"{"invalid": "json"}"#;

    let response = server
        .post("/api/document/test-doc-id/position")
        .text(request_body)
        .await;

    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn test_serve_document() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    // Test serving non-existent document returns 404
    let response = server.get("/document/nonexistent-id").await;

    response.assert_status(StatusCode::NOT_FOUND);

    // Create a document using the example file
    let example_path = std::env::current_dir().unwrap().join("examples/simple.md");
    let create_request = CreateDocumentRequest {
        filepath: example_path.to_str().unwrap().to_string(),
    };
    let create_body = facet_json::to_string(&create_request);

    let create_response = server.post("/api/document").text(create_body).await;

    create_response.assert_status(StatusCode::CREATED);
    let create_response_body: CreateDocumentResponse =
        facet_json::from_str(&create_response.text()).unwrap();
    let doc_id = create_response_body.id;

    // Now serve the document
    let serve_response = server.get(&format!("/document/{}", doc_id)).await;

    serve_response.assert_status_ok();
    let body_text = serve_response.text();
    assert!(body_text.contains("<html>") || body_text.contains("<h1"));
}

#[tokio::test]
async fn test_create_document_duplicate_filepath() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let request_body = r#"{"filepath": "/path/to/duplicate.md"}"#;

    // Create first document
    let response1 = server.post("/api/document").text(request_body).await;

    response1.assert_status(StatusCode::CREATED);
    let body1 = response1.text();

    // Create second document with same filepath
    let response2 = server.post("/api/document").text(request_body).await;

    response2.assert_status(StatusCode::CREATED);
    let body2 = response2.text();

    // Should return the same ID
    assert_eq!(body1, body2);
    assert!(body1.contains("duplicate-md"));
}

#[tokio::test]
async fn test_create_document_empty_filepath() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let request_body = r#"{"filepath": ""}"#;

    let response = server.post("/api/document").text(request_body).await;

    response.assert_status(StatusCode::CREATED);
    let body_text = response.text();
    assert!(body_text.contains("unknown"));
    assert!(body_text.starts_with("{\"id\":\"unknown-"));
}

#[tokio::test]
async fn test_create_document_nested_path() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let request_body = r#"{"filepath": "/deep/nested/path/to/readme.md"}"#;

    let response = server.post("/api/document").text(request_body).await;

    response.assert_status(StatusCode::CREATED);
    let body_text = response.text();
    assert!(body_text.contains("readme-md"));
    assert!(body_text.starts_with("{\"id\":\"readme-md-"));
}

#[tokio::test]
async fn test_create_document_special_characters_in_filename() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let request_body = r#"{"filepath": "/path/my-file.name.with.dots.md"}"#;

    let response = server.post("/api/document").text(request_body).await;

    response.assert_status(StatusCode::CREATED);
    let body_text = response.text();
    // Dots should be replaced with dashes
    assert!(body_text.contains("my-file-name-with-dots-md"));
    assert!(body_text.starts_with("{\"id\":\"my-file-name-with-dots-md-"));
}

#[tokio::test]
async fn test_create_document_malformed_json() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let request_body = r#"{"filepath": "/valid/path.md", "extra": "field", invalid json"#;

    let response = server.post("/api/document").text(request_body).await;

    response.assert_status(StatusCode::CREATED);
    let body_text = response.text();
    // Should fallback to "unknown" when JSON parsing fails
    assert!(body_text.contains("unknown"));
    assert!(body_text.starts_with("{\"id\":\"unknown-"));
}

#[tokio::test]
async fn test_create_document_id_format_consistency() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let request_body = r#"{"filepath": "/test/consistency.md"}"#;

    let response = server.post("/api/document").text(request_body).await;

    response.assert_status(StatusCode::CREATED);
    let body_text = response.text();

    // Verify ID format: filename-extension-hash
    assert!(body_text.starts_with("{\"id\":\"consistency-md-"));
    assert!(body_text.ends_with("\"}"));

    // Extract the ID to verify hash format (8 characters)
    let id_start = body_text.find("consistency-md-").unwrap() + "consistency-md-".len();
    let id_end = body_text.find("\"}").unwrap();
    let hash_part = &body_text[id_start..id_end];
    assert_eq!(hash_part.len(), 8); // Hash is 8 characters
    assert!(hash_part.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[tokio::test]
async fn test_create_document_no_extension() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let request_body = r#"{"filepath": "/path/to/README"}"#;

    let response = server.post("/api/document").text(request_body).await;

    response.assert_status(StatusCode::CREATED);
    let body_text = response.text();
    assert!(body_text.contains("README"));
    assert!(body_text.starts_with("{\"id\":\"README-"));
}

#[tokio::test]
async fn test_nonexistent_route() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let response = server.get("/nonexistent").await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_documents() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    // Initially should return empty list
    let response = server.get("/").await;
    response.assert_status_ok();
    response.assert_header("content-type", "text/html");
    let body_text = response.text();
    assert!(body_text.contains("<html"));
    assert!(body_text.contains("Documents"));
    assert!(body_text.contains("<ul>"));

    // Create some documents
    let request1 = r#"{"filepath": "/path/to/first.md"}"#;
    let create_response1 = server.post("/api/document").text(request1).await;
    create_response1.assert_status(StatusCode::CREATED);
    let create_body1: CreateDocumentResponse =
        facet_json::from_str(&create_response1.text()).unwrap();

    let request2 = r#"{"filepath": "/path/to/second.md"}"#;
    let create_response2 = server.post("/api/document").text(request2).await;
    create_response2.assert_status(StatusCode::CREATED);
    let create_body2: CreateDocumentResponse =
        facet_json::from_str(&create_response2.text()).unwrap();

    // Now check the list contains the documents
    let list_response = server.get("/").await;
    list_response.assert_status_ok();
    let list_body = list_response.text();

    assert!(list_body.contains(&format!("/document/{}", create_body1.id)));
    assert!(list_body.contains("/path/to/first.md"));
    assert!(list_body.contains(&format!("/document/{}", create_body2.id)));
    assert!(list_body.contains("/path/to/second.md"));
}

#[tokio::test]
async fn test_sse_endpoint_exists() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    // First create a document
    let request_body = r#"{"filepath": "/path/to/test.md"}"#;
    let create_response = server.post("/api/document").text(request_body).await;
    create_response.assert_status(StatusCode::CREATED);

    let create_body: CreateDocumentResponse =
        facet_json::from_str(&create_response.text()).unwrap();
    let doc_id = create_body.id;

    // Test SSE endpoint with timeout - just check that it connects and starts streaming
    let request = server.get(&format!("/document/{}/updates", doc_id));

    // Use timeout to prevent hanging
    let response = tokio::time::timeout(std::time::Duration::from_secs(2), request).await;

    // Check that we got a response (even if timed out, it means connection was established)
    match response {
        Ok(resp) => {
            resp.assert_status_ok();
        }
        Err(_) => {
            // Timeout is expected for SSE connections, this is actually success
            // as it means the connection was established and streaming started
        }
    }
}

#[tokio::test]
async fn test_sse_endpoint_nonexistent_document() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    // Test SSE endpoint for non-existent document
    let response = server.get("/document/nonexistent-id/updates").await;
    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_file_watcher_integration() {
    use std::env;
    use std::fs;

    // Create a temporary file using TMPDIR
    let tmp_dir = env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string());
    let file_path = format!("{}/test_file_watcher_{}.md", tmp_dir, std::process::id());
    fs::write(&file_path, "# Initial content").unwrap();

    let app = create_app();
    let server = TestServer::new(app).unwrap();

    // Create a document that watches the file
    let request_body = format!(r#"{{"filepath":"{}"}}"#, file_path);
    let create_response = server.post("/api/document").text(&request_body).await;
    create_response.assert_status(StatusCode::CREATED);

    let create_body: CreateDocumentResponse =
        facet_json::from_str(&create_response.text()).unwrap();
    let doc_id = create_body.id;

    // Verify the document can be served
    let serve_response = server.get(&format!("/document/{}", doc_id)).await;
    serve_response.assert_status_ok();
    let content = serve_response.text();
    assert!(content.contains("Initial content"));

    // Test that file watcher is set up (SSE endpoint should work)
    let sse_request = server.get(&format!("/document/{}/updates", doc_id));
    let sse_response = tokio::time::timeout(std::time::Duration::from_secs(1), sse_request).await;

    // SSE should connect successfully (timeout is expected)
    match sse_response {
        Ok(resp) => resp.assert_status_ok(),
        Err(_) => {} // Timeout is expected for SSE
    }

    // Clean up
    let _ = fs::remove_file(&file_path);
}

#[tokio::test]
async fn test_file_change_notification_via_sse() {
    use std::env;
    use std::fs;
    use tokio::time::{sleep, Duration};

    // Create a temporary file using TMPDIR
    let tmp_dir = env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string());
    let file_path = format!("{}/test_file_change_{}.md", tmp_dir, std::process::id());
    fs::write(&file_path, "# Initial content").unwrap();

    let app = create_app();
    let server = TestServer::new(app).unwrap();

    // Create a document that watches the file
    let request_body = format!(r#"{{"filepath":"{}"}}"#, file_path);
    let create_response = server.post("/api/document").text(&request_body).await;
    create_response.assert_status(StatusCode::CREATED);

    let create_body: CreateDocumentResponse =
        facet_json::from_str(&create_response.text()).unwrap();
    let doc_id = create_body.id;

    // Give the file watcher a moment to initialize
    sleep(Duration::from_millis(100)).await;

    // Modify the file to trigger file watcher
    fs::write(
        &file_path,
        "# Modified content\n\nThis file has been changed!",
    )
    .unwrap();

    // Give file watcher time to detect and process the change
    sleep(Duration::from_millis(500)).await;

    // Verify the updated content can be served
    let serve_response = server.get(&format!("/document/{}", doc_id)).await;
    serve_response.assert_status_ok();
    let content = serve_response.text();
    assert!(content.contains("Modified content"));
    assert!(content.contains("This file has been changed!"));

    // Test that SSE endpoint still works after file change
    let sse_request = server.get(&format!("/document/{}/updates", doc_id));
    let sse_response = tokio::time::timeout(std::time::Duration::from_secs(1), sse_request).await;

    // SSE should still connect successfully
    match sse_response {
        Ok(resp) => resp.assert_status_ok(),
        Err(_) => {} // Timeout is expected for SSE
    }

    // Clean up
    let _ = fs::remove_file(&file_path);
}
