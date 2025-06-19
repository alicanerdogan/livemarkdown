use clap::Parser;
use livemarkdown_rs::{create_app, create_app_with_state, AppState};
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
    
    #[arg(help = "Markdown file to serve")]
    file: Option<String>,
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

    let app = if let Some(filepath) = &args.file {
        // Check if file exists
        if !std::path::Path::new(filepath).exists() {
            eprintln!("File not found: {}", filepath);
            process::exit(1);
        }

        let state = AppState::new();
        let doc_id = create_initial_document(&state, filepath.clone());
        
        println!("Starting livemarkdown server on port {}", args.port);
        println!("Serving file: {}", filepath);
        println!("Document URL: http://127.0.0.1:{}/document/{}", args.port, doc_id);
        
        create_app_with_state(state)
    } else {
        println!("Starting livemarkdown server on port {}", args.port);
        create_app()
    };

    let addr = format!("127.0.0.1:{}", args.port);

    match TcpListener::bind(&addr).await {
        Ok(listener) => {
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

fn create_initial_document(state: &AppState, filepath: String) -> String {
    // Generate new ID: filename + ULID
    let filename = std::path::Path::new(&filepath)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .replace('.', "-");

    let ulid = ulid::Ulid::new();
    let doc_id = format!("{}-{}", filename, ulid);

    // Store the document
    state.add_document(doc_id.clone(), filepath);

    doc_id
}
