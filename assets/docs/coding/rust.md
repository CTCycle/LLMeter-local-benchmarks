# Rust

## Edition

Rust edition 2021 is used, targeting stable Rust.

## Conventions

- Use `clap` derive API for all CLI argument parsing.
- Use `serde` derive (`#[derive(Serialize, Deserialize)]`) for all data structures.
- Use `thiserror` with `#[derive(Error)]` for library error types.
- Use `anyhow::Result` for CLI entry-point and orchestration functions.
- Prefer `PathBuf` over `String` for filesystem paths.
- Use `HashMap<String, Value>` (serde_json::Value) for dynamic metric dictionaries.
- Use `pub` module declarations in `lib.rs` for integration test access.

## Crate dependencies

Only the crates listed in `Cargo.toml` under `[dependencies]`. No runtime dependencies beyond the compiled binary — `rustls` replaces `openssl` for fully static linking on all platforms.

## Style

- Formatted with `cargo fmt` (default settings, 100-char line width).
- Linted with `cargo clippy` — deny warnings in CI.
- No unsafe code except where platform APIs require it (Windows process creation flags).

## Module structure

- One module per file. Modules declared in `lib.rs`.
- Internal benchmark modules live under `benchmarks/`; provider integration lives in `providers.rs`.
- Tests live in `tests/` as integration tests, plus inline `#[cfg(test)] mod tests` where appropriate.

Last updated: 2026-06-12
