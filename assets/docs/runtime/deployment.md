# Deployment

## Installation

### From source

```bash
git clone <repo-url>
cd llmeter
cargo build --release
```

The binary is at `target/release/llmeter` (or `target/release/llmeter.exe` on Windows). Add it to your `PATH`, or copy it to a directory in your `PATH`:

```bash
cp target/release/llmeter ~/.local/bin/
```

On Windows PowerShell:

```powershell
copy target\release\llmeter.exe C:\Users\<you>\bin\
```

### Via cargo install

```bash
cargo install --path .
```

Or directly from a published crate (when available):

```bash
cargo install llmeter
```

### Prebuilt binaries

Prebuilt binaries for Linux (x86_64, aarch64 with musl), macOS (x86_64, arm64), and Windows (x86_64) are available from the releases page. These are statically linked with `rustls` — no `openssl` or system libraries required.

## Dependencies

- Runtime: **None**. The binary is self-contained.
- Build-time: Rust toolchain (stable), listed in `Cargo.toml`.

## Versioning

Current version: `0.2.0`. Follows semantic versioning. Defined in `Cargo.toml`.

## Platforms

Cross-platform (Windows, macOS, Linux). Platform-specific behavior:

- **Process management**: `taskkill` on Windows, `kill` on Unix — handled in `ollama/server.rs`.
- **Process detachment**: `DETACHED_PROCESS` flag on Windows, process group on Unix.
- **Binary naming**: `llmeter.exe` on Windows, `llmeter` on Unix.

The binary is compiled with `rustls` (no `openssl`), making it fully statically linkable for musl targets.

Last updated: 2026-06-12
