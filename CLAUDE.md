# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.


## Project Overview

This is a Rust project called `livemarkdown-rs` - appears to be a markdown-related tool based on the name. Currently in early development stage with minimal implementation.

## Development Commands

### Building and Running
- `cargo build` - Compile the project
- `cargo run -- --port=3030` - Build and run the application
- `cargo check` - Check code without building (faster feedback)

### Testing and Quality
- `cargo test` - Run all tests (using axum_test for integration tests)
- `cargo clippy` - Run Rust linter for code quality
- `cargo fmt` - Format code according to Rust standards

### Dependencies
- `cargo add <crate_name>` - Add a new dependency
- `cargo update` - Update dependencies to latest compatible versions

## Architecture

Currently a single-file Rust application with:
- `src/main.rs` - Entry point with basic Hello World implementation
- `Cargo.toml` - Project configuration and dependencies

## Notes

- Project uses Rust 2024 edition
- It uses `facet` for JSON serialization. And it is forbidden to use `serde` in this project. `lib.rs` has example of how to use `facet` for JSON serialization.

# Product requirements
A rust application that watches changes in a markdown file and renders into an html by serving it over local http server.

# Requirements

- The application should be an executable binary.
- The application should accept a `--port` option to run the server.
    - If the port is used, it should return the exit code 30.
- The application should accept an optional filepath as a parameter to run the server with a markdown file.
    - e.g. `--port 8080 ./path/to/file.md`
- The http server should expose the following HTTP calls:
    - `POST /api/document` which creates a markdown that is watched with `{ filepath: string; }`. And returns a document id with `{ id: string }`
    - `DELETE /api/document/:id` removes a file that is watched.
    - `POST /api/document/:id/open` opens the file in the browser.
    - `POST /api/document/:id/position` sends a desired document position that `{ sourcepos: string; }`
    - `GET /` should return the list of documents as an HTML page with links to the documents.
- The http server should serve the documents that are rendered with `GET /document/:id`
- The http server should also have a SSE API (`GET /document/:id/updates`) which updates the browser clients for the latest active document position and and update event when the file contents are changed

# Rust libraries

- `comrak`: Render markdown with github flavor and sourcepos
- `notify` and `notify-debouncer-mini`: File watcher
- `axum`: HTTP server
- `facet`: JSON serialization

# Remaining tasks

[x] Implement CLI args parsing
[x] Implement basic HTTP server with `axum`
[x] Integrate `facet` for JSON serialization
[x] Add integration tests using `axum_test`
[x] Modularize server code into `src/lib.rs`
[x] Integrate `comrak` to render markdown to HTML with source position
[x] Add CLI arg to specify a document to render at start
[x] Implement SSE for real-time updates
[x] Add tests for SSE
[x] Implement file watching using `notify` and `notify-debouncer-mini`
[x] CSS markup for markdown (light and dark mode)
[ ] Debug file watching
[ ] Replacing the content of the file with the new content in the client
[ ] Prevent injection attacks for HTML rendering at root
[ ] Add error handling and logging
[ ] Use file streams for reading files
