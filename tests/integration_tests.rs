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
    assert!(body_text.contains("doc--path-to-test-md"));
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
    assert!(body_text.contains("doc-unknown"));
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
async fn test_nonexistent_route() {
    let app = create_app();
    let server = TestServer::new(app).unwrap();

    let response = server
        .get("/nonexistent")
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}
