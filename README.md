# livemarkdown-rs

A Rust application that watches markdown files for changes and renders them as HTML, served via a local HTTP server with real-time updates.

## Features

- Watch markdown files for changes and auto-reload
- Live preview with real-time updates via Server-Sent Events (SSE)
- GitHub-flavored markdown rendering with source position mapping
- Multiple document management
- Light and dark mode support
- Browser integration for opening documents

## Installation

Clone the repository and build the project:

```bash
git clone <repository-url>
cd livemarkdown
cargo build --release
```

## Usage

### Basic Usage

Start the server on default port with a markdown file:

```bash
cargo run -- --port 3030 ./path/to/your/file.md
```

### Command Line Options

- `--port <PORT>` - Specify the port to run the server on
- `[FILE]` - Optional path to a markdown file to watch at startup

### API Endpoints

- `GET /` - List all watched documents
- `GET /document/:id` - View rendered markdown document
- `GET /document/:id/updates` - SSE endpoint for real-time updates
- `POST /api/document` - Create a new watched document
- `DELETE /api/document/:id` - Remove a watched document
- `POST /api/document/:id/open` - Open document in browser
- `POST /api/document/:id/position` - Update document position

### Example API Usage

Create a new document to watch:
```bash
curl -X POST http://localhost:3030/api/document \
  -H "Content-Type: application/json" \
  -d '{"filepath": "./example.md"}'
```

## Development

### Building and Running
```bash
cargo build          # Compile the project
cargo run -- --port=3030  # Build and run
cargo check          # Check code without building
```

### Testing and Quality
```bash
cargo test           # Run all tests
cargo clippy         # Run linter
cargo fmt            # Format code
```

### Dependencies Management
```bash
cargo add <crate>    # Add dependency
cargo update         # Update dependencies
```

## License

This project is in early development stage.