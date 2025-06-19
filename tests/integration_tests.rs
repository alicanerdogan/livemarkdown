use axum::http::StatusCode;
use axum_test::TestServer;
use livemarkdown_rs::{create_app, CreateDocumentRequest, CreateDocumentResponse};

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

    // Verify ID format: filename-extension-ULID
    assert!(body_text.starts_with("{\"id\":\"consistency-md-"));
    assert!(body_text.ends_with("\"}"));

    // Extract the ID to verify ULID format (26 characters)
    let id_start = body_text.find("consistency-md-").unwrap() + "consistency-md-".len();
    let id_end = body_text.find("\"}").unwrap();
    let ulid_part = &body_text[id_start..id_end];
    assert_eq!(ulid_part.len(), 26); // ULID is 26 characters
    assert!(ulid_part.chars().all(|c| c.is_ascii_alphanumeric()));
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
