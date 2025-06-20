use clap::Parser;
use livemarkdown::{create_app, create_app_with_state, utils, AppState};
use std::process;
use tokio::net::TcpListener;

#[derive(Parser)]
#[command(name = "livemarkdown")]
#[command(about = "A markdown live preview server")]
struct Args {
    #[arg(short = 'p', long = "port")]
    #[arg(help = "Port number to run the server on (defaults to 3030)")]
    #[arg(value_parser = validate_port)]
    port: Option<u16>,

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

    // Find an available port starting from 3030
    let port = find_available_port(args.port.unwrap_or(3030)).await;

    let app = if let Some(filepath) = &args.file {
        // Convert to absolute path for consistency
        let absolute_filepath = utils::to_absolute_path(filepath);

        // Check if file exists
        if !std::path::Path::new(&absolute_filepath).exists() {
            eprintln!("File not found: {}", filepath);
            process::exit(1);
        }

        let state = AppState::new();
        let doc_id = create_initial_document(&state, filepath.clone());

        println!("Starting livemarkdown server on port {}", port);
        println!("Serving file: {}", filepath);
        println!(
            "Document URL: http://127.0.0.1:{}/document/{}",
            port, doc_id
        );

        create_app_with_state(state)
    } else {
        println!("Starting livemarkdown server on port {}", port);
        create_app()
    };

    let addr = format!("127.0.0.1:{}", port);

    match TcpListener::bind(&addr).await {
        Ok(listener) => {
            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("Server error: {}", e);
                process::exit(1);
            }
        }
        Err(_) => {
            eprintln!("Port {} is already in use", port);
            process::exit(30);
        }
    }
}

async fn find_available_port(start_port: u16) -> u16 {
    for port in start_port..start_port + 100 {
        let addr = format!("127.0.0.1:{}", port);
        if TcpListener::bind(&addr).await.is_ok() {
            return port;
        }
    }

    eprintln!(
        "No available ports found in range {}-{}",
        start_port,
        start_port + 99
    );
    process::exit(30);
}

fn create_initial_document(state: &AppState, filepath: String) -> String {
    // Generate new ID: consistent hash-based
    let doc_id = utils::generate_document_id(&filepath);

    // Store the document
    state.add_document(doc_id.clone(), filepath);

    doc_id
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use tokio::net::TcpListener;

    #[test]
    fn test_validate_port() {
        assert!(validate_port("3030").is_ok());
        assert!(validate_port("8080").is_ok());
        assert!(validate_port("65535").is_ok());
        assert!(validate_port("0").is_err());
        assert!(validate_port("abc").is_err());
        assert!(validate_port("70000").is_err());
    }

    #[test]
    fn test_args_parsing_with_port() {
        let args = Args::try_parse_from(&["livemarkdown", "--port", "8080"]).unwrap();
        assert_eq!(args.port, Some(8080));
        assert_eq!(args.file, None);
    }

    #[test]
    fn test_args_parsing_without_port() {
        let args = Args::try_parse_from(&["livemarkdown"]).unwrap();
        assert_eq!(args.port, None);
        assert_eq!(args.file, None);
    }

    #[test]
    fn test_args_parsing_with_file() {
        let args = Args::try_parse_from(&["livemarkdown", "test.md"]).unwrap();
        assert_eq!(args.port, None);
        assert_eq!(args.file, Some("test.md".to_string()));
    }

    #[test]
    fn test_args_parsing_with_port_and_file() {
        let args = Args::try_parse_from(&["livemarkdown", "--port", "3030", "test.md"]).unwrap();
        assert_eq!(args.port, Some(3030));
        assert_eq!(args.file, Some("test.md".to_string()));
    }

    #[tokio::test]
    async fn test_find_available_port_with_free_port() {
        // Test with a high port number that's likely to be available
        let test_port = 50000;
        let found_port = find_available_port(test_port).await;
        assert!(found_port >= test_port);
        assert!(found_port < test_port + 100);

        // Verify the port is actually available
        let listener = TcpListener::bind(format!("127.0.0.1:{}", found_port)).await;
        assert!(listener.is_ok());
    }

    #[tokio::test]
    async fn test_find_available_port_with_occupied_port() {
        // Bind to a port to occupy it
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let occupied_port = addr.port();

        // Test should find the next available port
        let found_port = find_available_port(occupied_port).await;
        assert!(found_port >= occupied_port);
        assert!(found_port < occupied_port + 100);

        drop(listener);
    }

    #[tokio::test]
    async fn test_find_available_port_default_3030() {
        // Test that it defaults to trying from 3030
        let port = find_available_port(3030).await;
        assert!(port >= 3030);
        assert!(port < 3130);
    }
}
