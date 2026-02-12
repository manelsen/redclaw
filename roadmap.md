# Project Roadmap: Project Redclaw (PicoClaw in Rust)

## Mission
Port the Go-based PicoClaw to Rust to achieve <2MB RAM footprint and static binary portability for embedded Linux (RISC-V/ARM).

## Phase 1: The Exoskeleton (Zero-Allocation Goal)
- [ ] **Core Structure**: Initialize Rust project with `cargo new`.
- [ ] **CLI Argument Parsing**: Implement `clap` (using `derive` feature only if strict size limits allow, otherwise `lexopt`) to handle flags like `-m` (message), `-s` (server), `--onboard`.
- [ ] **Config Loader**: Create `config.rs` to deserialize `config.json` using `serde` + `serde_json`.
    - *Constraint*: Fail fast if config is missing. No default magic strings.
- [ ] **Logger**: Implement a zero-cost logger (formatting to stdout/stderr) without heavy crates like `log4rs`. Use `env_logger` or a simple `eprintln!` wrapper.

## Phase 2: The Brain (HTTP & Providers)
- [ ] **HTTP Client**: Implement `client.rs` using `ureq` (blocking, lightweight) or `reqwest` (blocking feature enabled, default-features=false) to minimize async runtime overhead.
    - *Target*: Support OpenAI-compatible API schema (used by OpenRouter/Zhipu).
- [ ] **Provider Trait**: Define a `LLMProvider` trait.
- [ ] **Context Manager**: Implement `context.rs` to assemble the System Prompt + History + Tools definitions.

## Phase 3: The Memory (File System over Vector DB)
- [ ] **Markdown IO**: Implement `memory.rs`.
    - Function: `read_long_term()` (reads `MEMORY.md`).
    - Function: `append_daily_log()` (appends to `YYYY/MM/DD.md`).
- [ ] **Workspace Manager**: Ensure directory structure creation (`workspace/sessions`, `workspace/memory`) on boot.

## Phase 4: The Pincers (Tools System)
- [ ] **Tool Traits**: Define the `Tool` trait with `name()`, `description()`, and `execute(args)`.
- [ ] **Core Tools Implementation**:
    - `cmd_exec` (via `std::process::Command`).
    - `file_read` / `file_write`.
    - `web_search` (API call to Brave/Tavily).
- [ ] **Tool Parser**: Implement logic to parse LLM JSON tool calls and route to the correct struct.

## Phase 5: The Nervous System (Event Loop)
- [ ] **Agent Loop**: Port `loop.go` to `agent.rs`.
    - Logic: User Input -> Context Build -> LLM Call -> Tool Execution -> Output -> Recursive Loop (if tool used).
- [ ] **Signal Handling**: Graceful shutdown on Ctrl+C.

## Phase 6: Molting (Optimization & Release)
- [ ] **Binary Stripping**: Configure `Cargo.toml` for `opt-level = "z"`, `lto = true`, `panic = "abort"`.
- [ ] **Cross-Compilation**: Verify builds for `riscv64gc-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`.
