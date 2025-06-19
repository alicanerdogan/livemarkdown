use axum::{
    Router,
    extract::Path,
    http::{StatusCode, HeaderMap, header},
    routing::{delete, get, post},
    response::IntoResponse,
};
use facet::Facet;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use ulid::Ulid;

pub mod markdown;

// Global state for document management
type DocumentStore = Arc<Mutex<HashMap<String, String>>>; // id -> filepath
type FilepathToIdMap = Arc<Mutex<HashMap<String, String>>>; // filepath -> id

#[derive(Clone)]
pub struct AppState {
    documents: DocumentStore,
    filepath_to_id: FilepathToIdMap,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(Mutex::new(HashMap::new())),
            filepath_to_id: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn get_id_by_filepath(&self, filepath: &str) -> Option<String> {
        let filepath_to_id = self.filepath_to_id.lock().unwrap();
        filepath_to_id.get(filepath).cloned()
    }
    
    pub fn get_filepath_by_id(&self, id: &str) -> Option<String> {
        let documents = self.documents.lock().unwrap();
        documents.get(id).cloned()
    }
    
    pub fn add_document(&self, id: String, filepath: String) {
        let mut documents = self.documents.lock().unwrap();
        let mut filepath_to_id = self.filepath_to_id.lock().unwrap();
        
        documents.insert(id.clone(), filepath.clone());
        filepath_to_id.insert(filepath, id);
    }
    
    pub fn remove_document(&self, id: &str) -> Option<String> {
        let mut documents = self.documents.lock().unwrap();
        let mut filepath_to_id = self.filepath_to_id.lock().unwrap();
        
        if let Some(filepath) = documents.remove(id) {
            filepath_to_id.remove(&filepath);
            Some(filepath)
        } else {
            None
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

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
    let state = AppState::new();
    Router::new()
        .route("/api/document", post(create_document))
        .route("/api/document/{id}", delete(delete_document))
        .route("/api/document/{id}/open", post(open_document))
        .route("/api/document/{id}/position", post(update_position))
        .route("/document/{id}", get(serve_document))
        .with_state(state)
}

async fn create_document(
    axum::extract::State(state): axum::extract::State<AppState>,
    body: String
) -> impl IntoResponse {
    // Parse the request body using facet
    let request: CreateDocumentRequest = facet_json::from_str(&body).unwrap_or_else(|_| {
        CreateDocumentRequest {
            filepath: "unknown".to_string(),
        }
    });
    
    let filepath = request.filepath;
    
    // Check if filepath already exists
    if let Some(existing_id) = state.get_id_by_filepath(&filepath) {
        let response = CreateDocumentResponse {
            id: existing_id,
        };
        
        let json_body = facet_json::to_string(&response);
        
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
        
        return (StatusCode::CREATED, headers, json_body);
    }
    
    // Generate new ID: filename + ULID
    let filename = std::path::Path::new(&filepath)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .replace('.', "-");
    
    let ulid = Ulid::new();
    let doc_id = format!("{}-{}", filename, ulid);
    
    // Store the document
    state.add_document(doc_id.clone(), filepath);
    
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