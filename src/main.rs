use axum::{
    extract::Path,
    http::StatusCode,
    routing::{delete, get, post},
    Router,
};
use clap::Parser;
use std::process;
use tokio::net::TcpListener;

#[derive(Parser)]
#[command(name = "livemarkdown")]
#[command(about = "A markdown live preview server")]
struct Args {
    #[arg(short = 'p', long = "port")]
    #[arg(help = "Port number to run the server on")]
    #[arg(value_parser = validate_port)]
    port: u16,
}

fn validate_port(s: &str) -> Result<u16, String> {
    match s.parse::<u16>() {
        Ok(port) => {
            if port == 0 {
                Err("Port number must be greater than 0".to_string())
            } else {
                Ok(port)
            }
        }
        Err(_) => Err("Port must be a valid number between 1 and 65535".to_string()),
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let app = Router::new()
        .route("/api/document", post(create_document))
        .route("/api/document/{id}", delete(delete_document))
        .route("/api/document/{id}/open", post(open_document))
        .route("/api/document/{id}/position", post(update_position))
        .route("/document/{id}", get(serve_document));

    let addr = format!("127.0.0.1:{}", args.port);
    
    match TcpListener::bind(&addr).await {
        Ok(listener) => {
            println!("Starting livemarkdown server on port {}", args.port);
            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("Server error: {}", e);
                process::exit(1);
            }
        }
        Err(_) => {
            eprintln!("Port {} is already in use", args.port);
            process::exit(30);
        }
    }
}

async fn create_document() -> String {
    r#"{"id": "dummy-doc-123"}"#.to_string()
}

async fn delete_document(Path(id): Path<String>) -> StatusCode {
    println!("Deleting document: {}", id);
    StatusCode::OK
}

async fn open_document(Path(id): Path<String>) -> String {
    println!("Opening document: {}", id);
    "Document opened".to_string()
}

async fn update_position(Path(id): Path<String>) -> String {
    println!("Updating position for document: {}", id);
    "Position updated".to_string()
}

async fn serve_document(Path(id): Path<String>) -> String {
    println!("Serving document: {}", id);
    format!("<html><body><h1>Document {}</h1><p>This is a dummy rendered document.</p></body></html>", id)
}
