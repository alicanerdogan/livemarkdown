use clap::Parser;
use livemarkdown_rs::create_app;
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

    let app = create_app();

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
