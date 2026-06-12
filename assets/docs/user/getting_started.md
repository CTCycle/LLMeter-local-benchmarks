# Getting started

## Requirements

- Rust toolchain (stable) — only needed to build from source.
- Prebuilt binary — no requirements beyond the OS.
- Ollama installed and available on `PATH`.
- At least one local Ollama model.

## Installation

### Using a prebuilt binary

Download the binary for your platform from the releases page, rename it to `llmeter` (or `llmeter.exe` on Windows), and place it in a directory on your `PATH`.

### Building from source

```bash
git clone <repo-url>
cd llmeter
cargo build --release
```

The binary is at `target/release/llmeter` (or `target/release/llmeter.exe` on Windows).

## First run

```bash
llmeter
```

This opens the interactive main menu. Choose option 1 to start the Ollama server, then explore the available options.

Or run a quick benchmark non-interactively:

```bash
llmeter bench run --models all --benchmarks all --start-server
```

## Verify it works

```bash
llmeter status
```

Should show Ollama installed, server running, and API version.

Last updated: 2026-06-12
