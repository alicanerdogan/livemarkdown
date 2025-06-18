use axum::{
    Router,
    extract::Path,
    http::{StatusCode, HeaderMap, header},
    routing::{delete, get, post},
    response::IntoResponse,
};
use facet::Facet;

// API request/response structures
#[derive(Facet)]
pub struct CreateDocumentRequest {
    pub filepath: String,
}

#[derive(Facet)]
pub struct CreateDocumentResponse {
    pub id: String,
}

#[derive(Facet)]
pub struct UpdatePositionRequest {
    pub sourcepos: String,
}

pub fn create_app() -> Router {
    Router::new()
        .route("/api/document", post(create_document))
        .route("/api/document/{id}", delete(delete_document))
        .route("/api/document/{id}/open", post(open_document))
        .route("/api/document/{id}/position", post(update_position))
        .route("/document/{id}", get(serve_document))
}

async fn create_document(body: String) -> impl IntoResponse {
    // Parse the request body using facet
    let request: CreateDocumentRequest = facet_json::from_str(&body).unwrap_or_else(|_| {
        CreateDocumentRequest {
            filepath: "unknown".to_string(),
        }
    });
    
    // Generate a simple document ID based on filepath
    let doc_id = format!("doc-{}", request.filepath.replace('/', "-").replace('.', "-"));
    
    let response = CreateDocumentResponse {
        id: doc_id,
    };
    
    let json_body = facet_json::to_string(&response);
    
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
    
    (StatusCode::CREATED, headers, json_body)
}

async fn delete_document(Path(id): Path<String>) -> StatusCode {
    println!("Deleting document: {}", id);
    StatusCode::OK
}

async fn open_document(Path(id): Path<String>) -> impl IntoResponse {
    println!("Opening document: {}", id);
    
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/plain".parse().unwrap());
    
    (StatusCode::CREATED, headers, "Document opened")
}

async fn update_position(Path(id): Path<String>, body: String) -> StatusCode {
    // Parse the request body using facet
    let request: UpdatePositionRequest = facet_json::from_str(&body).unwrap_or_else(|_| {
        UpdatePositionRequest {
            sourcepos: "unknown".to_string(),
        }
    });
    
    println!("Updating position for document {}: {}", id, request.sourcepos);
    StatusCode::CREATED
}

async fn serve_document(Path(id): Path<String>) -> String {
    println!("Serving document: {}", id);
    format!(
        "<html><body><h1>Document {}</h1><p>This is a dummy rendered document.</p></body></html>",
        id
    )
}