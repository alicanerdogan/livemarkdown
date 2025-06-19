use axum::http::StatusCode;
use axum_test::TestServer;
use livemarkdown_rs::create_app;

#[tokio::test]
async fn test_create_document() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let request_body = r#"{"filepath": "/path/to/test.md"}"#;
    
    let response = server
        .post("/api/document")
        .text(request_body)
        .await;

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
    
    let response = server
        .post("/api/document")
        .text(request_body)
        .await;

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

    let response = server
        .delete("/api/document/test-doc-id")
        .await;

    response.assert_status(StatusCode::OK);
}

#[tokio::test]
async fn test_open_document() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/api/document/test-doc-id/open")
        .await;

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

    let response = server
        .get("/document/test-doc-id")
        .await;

    response.assert_status_ok();
    let body_text = response.text();
    assert!(body_text.contains("<html>"));
    assert!(body_text.contains("Document test-doc-id"));
    assert!(body_text.contains("This is a dummy rendered document."));
}

#[tokio::test]
async fn test_create_document_duplicate_filepath() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let request_body = r#"{"filepath": "/path/to/duplicate.md"}"#;
    
    // Create first document
    let response1 = server
        .post("/api/document")
        .text(request_body)
        .await;

    response1.assert_status(StatusCode::CREATED);
    let body1 = response1.text();
    
    // Create second document with same filepath
    let response2 = server
        .post("/api/document")
        .text(request_body)
        .await;

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
    
    let response = server
        .post("/api/document")
        .text(request_body)
        .await;

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
    
    let response = server
        .post("/api/document")
        .text(request_body)
        .await;

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
    
    let response = server
        .post("/api/document")
        .text(request_body)
        .await;

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
    
    let response = server
        .post("/api/document")
        .text(request_body)
        .await;

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
    
    let response = server
        .post("/api/document")
        .text(request_body)
        .await;

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
    
    let response = server
        .post("/api/document")
        .text(request_body)
        .await;

    response.assert_status(StatusCode::CREATED);
    let body_text = response.text();
    assert!(body_text.contains("README"));
    assert!(body_text.starts_with("{\"id\":\"README-"));
}

#[tokio::test]
async fn test_nonexistent_route() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let response = server
        .get("/nonexistent")
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}
