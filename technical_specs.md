# Technical Specifications & Constraints: Project Redclaw

## 1. Architectural Philosophy
"Zero-Cost Abstractions". Every byte of RAM must be justified. We prioritize binary size and startup time over development convenience.

## 2. Language & Runtime
-   **Language**: Rust (Latest Stable).
-   **Runtime**: Prefer synchronous/blocking I/O where possible to avoid the overhead of a full async runtime (`tokio`).
    -   *Exception*: If async is strictly required for specific concurrency patterns, use `smol` or a minimal `tokio` feature set (`rt`, `macros`).
-   **Linking**: Static linking with `musl` is mandatory for portability.

## 3. Dependency Whitelist (Strict)
Use these crates (or lighter alternatives). Do NOT use heavy frameworks.
-   `serde`, `serde_json`: For JSON handling.
-   `clap` (feature `derive` disabled if possible, or use `lexopt`) for CLI.
-   `ureq` OR `minreq`: For HTTP requests (avoid `reqwest` unless stripped down).
-   `chrono`: For time management.
-   `anyhow`: For error handling.
-   *Banned*: `tokio-full`, `actix`, `rocket`, `sqlx`.

## 4. Memory Management Strategy
-   **No Vector DB**: Semantic search is expensive. Use exact keyword matching or simple grep-like logic via `cmd_exec` if needed.
-   **Streaming**: When reading large files, use buffered readers. Do not load entire files into `String` unless necessary.
-   **String Handling**: Prefer `&str` over `String` in internal function calls to reduce heap allocations.

## 5. Directory Structure
```text
src/
├── main.rs          # Entry point, CLI parsing
├── config.rs        # Struct definitions for JSON config
├── agent/
│   ├── mod.rs       # Agent Loop logic
│   ├── llm.rs       # HTTP Client wrapper for OpenAI/compatible APIs
│   └── memory.rs    # Flat-file manipulation
├── tools/
│   ├── mod.rs       # Tool trait definition
│   ├── builtin.rs   # fs, cmd, search implementations
│   └── registry.rs  # Tool routing
└── utils.rs         # Helper functions
