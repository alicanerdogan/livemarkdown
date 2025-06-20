use axum::{
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing::{delete, get, post},
    Router,
};
use facet::Facet;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::Stream;

pub mod html_template;
pub mod markdown;
pub mod utils;

#[derive(facet::Facet)]
struct FileChangedResponse {
    html: String,
}

#[derive(Clone, Debug)]
pub enum DocumentEvent {
    FileChanged {
        document_id: String,
    },
    PositionUpdate {
        document_id: String,
        sourcepos: String,
    },
}

pub struct DocumentStore {
    filepath_map: HashMap<String, String>,    // id -> filepath
    document_id_map: HashMap<String, String>, // filepath -> id
    position_map: HashMap<String, String>,    // id -> sourcepos
    event_tx: Option<broadcast::Sender<DocumentEvent>>,
}

#[derive(Clone)]
pub struct AppState {
    store: Arc<Mutex<DocumentStore>>,
    event_tx: broadcast::Sender<DocumentEvent>,
    file_watcher: Arc<Mutex<Option<Debouncer<RecommendedWatcher>>>>,
}

impl AppState {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(100);
        let event_tx_clone = event_tx.clone();

        Self {
            store: Arc::new(Mutex::new(DocumentStore {
                filepath_map: HashMap::new(),
                document_id_map: HashMap::new(),
                position_map: HashMap::new(),
                event_tx: Some(event_tx_clone),
            })),
            event_tx,
            file_watcher: Arc::new(Mutex::new(None)),
        }
    }

    fn init_file_watcher(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut watcher_guard = self.file_watcher.lock().unwrap();

        if watcher_guard.is_some() {
            return Ok(()); // Already initialized
        }

        let event_tx = self.event_tx.clone();
        let store = self.store.clone();

        let debouncer = new_debouncer(
            Duration::from_millis(300),
            move |res: DebounceEventResult| {
                if let Ok(events) = res {
                    for event in events {
                        if let Some(path) = event.path.to_str() {
                            if let Ok(store_guard) = store.lock() {
                                if let Some(doc_id) = store_guard.document_id_map.get(path) {
                                    let _ = event_tx.send(DocumentEvent::FileChanged {
                                        document_id: doc_id.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            },
        )?;

        *watcher_guard = Some(debouncer);
        Ok(())
    }

    pub fn watch_file(
        &self,
        filepath: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Initialize watcher if needed
        self.init_file_watcher()?;

        let mut watcher_guard = self.file_watcher.lock().unwrap();
        if let Some(ref mut debouncer) = *watcher_guard {
            debouncer
                .watcher()
                .watch(std::path::Path::new(filepath), RecursiveMode::NonRecursive)?;
        }

        Ok(())
    }

    pub fn unwatch_file(
        &self,
        filepath: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut watcher_guard = self.file_watcher.lock().unwrap();
        if let Some(ref mut debouncer) = *watcher_guard {
            debouncer
                .watcher()
                .unwatch(std::path::Path::new(filepath))?;
        }

        Ok(())
    }

    pub fn get_id_by_filepath(&self, filepath: &str) -> Option<String> {
        self.store
            .lock()
            .unwrap()
            .document_id_map
            .get(filepath)
            .cloned()
    }

    pub fn get_filepath_by_id(&self, id: &str) -> Option<String> {
        self.store.lock().unwrap().filepath_map.get(id).cloned()
    }

    pub fn add_document(&self, id: String, filepath: String) {
        // Convert to absolute path
        let absolute_path = utils::to_absolute_path(&filepath);

        {
            let mut store = self.store.lock().unwrap();
            store.filepath_map.insert(id.clone(), absolute_path.clone());
            store
                .document_id_map
                .insert(absolute_path.clone(), id.clone());
            store.position_map.insert(id.clone(), "1:1-1:1".to_string()); // Default position
        }

        // Start watching the file
        if let Err(e) = self.watch_file(&absolute_path) {
            eprintln!("Failed to watch file {}: {}", absolute_path, e);
        }
    }

    pub fn remove_document(&self, id: &str) -> Option<String> {
        let filepath = {
            let mut store = self.store.lock().unwrap();
            if let Some(filepath) = store.filepath_map.remove(id) {
                store.document_id_map.remove(&filepath);
                store.position_map.remove(id);
                Some(filepath)
            } else {
                None
            }
        };

        // Stop watching the file if it was removed
        if let Some(ref path) = filepath {
            if let Err(e) = self.unwatch_file(path) {
                eprintln!("Failed to unwatch file {}: {}", path, e);
            }
        }

        filepath
    }

    pub fn update_position(&self, id: &str, sourcepos: String) {
        let mut store = self.store.lock().unwrap();
        store.position_map.insert(id.to_string(), sourcepos.clone());

        // Broadcast position update
        if let Some(ref tx) = store.event_tx {
            let _ = tx.send(DocumentEvent::PositionUpdate {
                document_id: id.to_string(),
                sourcepos,
            });
        }
    }

    pub fn get_position(&self, id: &str) -> Option<String> {
        self.store.lock().unwrap().position_map.get(id).cloned()
    }

    pub fn get_all_documents(&self) -> Vec<(String, String)> {
        self.store
            .lock()
            .unwrap()
            .filepath_map
            .iter()
            .map(|(id, filepath)| (id.clone(), filepath.clone()))
            .collect()
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
    create_app_with_state(AppState::new())
}

pub fn create_app_with_state(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_documents))
        .route("/api/document", post(create_document))
        .route("/api/document/{id}", delete(delete_document))
        .route("/api/document/{id}/open", post(open_document))
        .route("/api/document/{id}/position", post(update_position))
        .route("/document/{id}", get(serve_document))
        .route("/document/{id}/updates", get(document_updates))
        .with_state(state)
}

async fn list_documents(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    let documents = state.get_all_documents();

    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Documents</title>
    <style>{}</style>
</head>
<body>
<main>
<h1>Documents</h1>
<ul>"#,
        html_template::get_styles()
    );

    for (id, filepath) in documents {
        html.push_str(&format!(
            "<li><a href=\"/document/{}\">{}</a></li>\n",
            id, filepath
        ));
    }

    html.push_str("</ul>\n</main>\n</body>\n</html>");

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/html".parse().unwrap());

    (StatusCode::OK, headers, html)
}

async fn create_document(
    axum::extract::State(state): axum::extract::State<AppState>,
    body: String,
) -> impl IntoResponse {
    // Parse the request body using facet
    let request: CreateDocumentRequest =
        facet_json::from_str(&body).unwrap_or_else(|_| CreateDocumentRequest {
            filepath: "unknown".to_string(),
        });

    let filepath = request.filepath;

    // Check if filepath already exists
    if let Some(existing_id) = state.get_id_by_filepath(&filepath) {
        let response = CreateDocumentResponse { id: existing_id };

        let json_body = facet_json::to_string(&response);

        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());

        return (StatusCode::CREATED, headers, json_body);
    }

    // Generate new ID: consistent hash-based
    let doc_id = utils::generate_document_id(&filepath);

    // Store the document
    state.add_document(doc_id.clone(), filepath);

    let response = CreateDocumentResponse { id: doc_id };

    let json_body = facet_json::to_string(&response);

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());

    (StatusCode::CREATED, headers, json_body)
}

async fn delete_document(
    Path(id): Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> StatusCode {
    match state.remove_document(&id) {
        Some(_) => StatusCode::OK,
        None => StatusCode::NOT_FOUND,
    }
}

async fn open_document(Path(id): Path<String>) -> impl IntoResponse {
    println!("Opening document: {}", id);

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/plain".parse().unwrap());

    (StatusCode::CREATED, headers, "Document opened")
}

async fn update_position(
    Path(id): Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
    body: String,
) -> StatusCode {
    // Parse the request body using facet
    let request: UpdatePositionRequest =
        facet_json::from_str(&body).unwrap_or_else(|_| UpdatePositionRequest {
            sourcepos: "unknown".to_string(),
        });

    // Update position in store and broadcast event
    state.update_position(&id, request.sourcepos.clone());

    println!(
        "Updating position for document {}: {}",
        id, request.sourcepos
    );
    StatusCode::CREATED
}

async fn serve_document(
    Path(id): Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    // Check if the document ID exists
    let filepath = match state.get_filepath_by_id(&id) {
        Some(path) => path,
        None => {
            return (StatusCode::NOT_FOUND, "Document not found").into_response();
        }
    };

    // Try to read the file
    let markdown_content = match std::fs::read_to_string(&filepath) {
        Ok(content) => content,
        Err(_) => {
            return (StatusCode::NOT_FOUND, "File not found").into_response();
        }
    };

    // Try to render the markdown
    let markdown_html = match try_render_markdown(&markdown_content) {
        Ok(html) => html,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to parse markdown",
            )
                .into_response();
        }
    };

    // Wrap in HTML template with document title based on filepath
    let title = std::path::Path::new(&filepath)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("Markdown Document");

    let html_content = html_template::wrap_in_html_template(&markdown_html, Some(title));

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/html".parse().unwrap());

    (StatusCode::OK, headers, html_content).into_response()
}

async fn document_updates(
    Path(id): Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    // Check if document exists
    if state.get_filepath_by_id(&id).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Get current position and send it immediately
    let current_position = state
        .get_position(&id)
        .unwrap_or_else(|| "1:1-1:1".to_string());

    // Subscribe to broadcast channel
    let rx = state.event_tx.subscribe();

    // Create stream that starts with current position and then listens for updates
    let stream = async_stream::stream! {
        // Send current position immediately
        yield Ok(Event::default()
            .event("position")
            .data(format!("{{\"sourcepos\":\"{}\"}}", current_position)));

        let mut rx = rx;
        // Listen for updates
        while let Ok(event) = rx.recv().await {
            match event {
                DocumentEvent::FileChanged { document_id } if document_id == id => {
                    let html_content = match state.get_filepath_by_id(&document_id) {
                        Some(filepath) => {
                            match std::fs::read_to_string(&filepath) {
                                Ok(content) => {
                                    match try_render_markdown(&content) {
                                        Ok(html) => html,
                                        Err(_) => String::from("<p>Error rendering markdown</p>"),
                                    }
                                },
                                Err(_) => String::from("<p>Error reading file</p>"),
                            }
                        },
                        None => String::from("<p>Document not found</p>"),
                    };

                    let response = FileChangedResponse { html: html_content };
                    let json_data = facet_json::to_string(&response);

                    yield Ok(Event::default()
                        .event("file_changed")
                        .data(json_data));
                },
                DocumentEvent::PositionUpdate { document_id, sourcepos } if document_id == id => {
                    yield Ok(Event::default()
                        .event("position")
                        .data(format!("{{\"sourcepos\":\"{}\"}}", sourcepos)));
                },
                _ => {
                    // Ignore events for other documents
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("ping"),
    ))
}

fn try_render_markdown(content: &str) -> Result<String, ()> {
    // For now, we'll assume the markdown module always succeeds
    // In a real implementation, you might want to add error handling
    // to the markdown::render_to_html function
    Ok(markdown::render_to_html(content))
}
