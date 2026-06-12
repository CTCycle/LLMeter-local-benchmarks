# Implementation plan — Rust refactor

## Completed

- **Phase 1 — Skeleton**: `Cargo.toml`, module tree, `clap` CLI, `AppConfig`, error types, utils, prompts.
- **Phase 2 — Ollama**: `reqwest`-based HTTP client with streaming TTFT, `sysinfo`-based PID management, cross-platform server start/stop.
- **Phase 3 — Benchmarks**: `Benchmark` trait, registry, three built-in benchmarks (generation, consistency, prompt sizes), metric helpers.
- **Phase 4 — Results & Reporting**: JSON/CSV persistence, Markdown/HTML report generation matching Python output.
- **Phase 5 — UI**: Interactive menus via `inquire`, terminal tables via `tabled`, markdown rendering via `comrak`.
- **Phase 6 — Tests**: 20 integration tests across 4 test files.
- **Phase 7 — Cleanup**: Removed Python sources, updated CI to Rust toolchain, updated `.gitignore`.
- **Phase 8 — Ontology**: Updated all 13 documentation files for the Rust stack.

## Current state

- **Stack**: Rust 2021 edition, `clap`, `reqwest` (blocking, `rustls-tls`), `serde+serde_json`, `csv`, `chrono`, `inquire`, `tabled`, `comrak`, `colored`, `sysinfo`, `similar`, `dirs`.
- **Build**: `cargo check` passes. `cargo test` — 20/20 pass. `cargo build --release` produces a single static binary.
- **Binary**: Self-contained, no Python or runtime dependencies, no `openssl`.
- **Platforms**: Windows, macOS, Linux — with platform-specific process management.

## Future improvements (not yet implemented)

- Warmup runs before measured runs.
- System metadata capture (CPU, GPU, RAM, OS).
- Multi-run comparison across saved results.
- Optional charts in HTML reports.
- Shell completion generation.
- Provider abstraction (if second provider is needed).
