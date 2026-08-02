# Getting started

## Requirements

- Rust toolchain only when building from source.
- A running local provider server with an OpenAI-compatible `/v1` API.
- At least one model exposed by that provider.

The catalog includes first-class local presets (`ollama`, `lmstudio`, and `llama-cpp`) plus additional OpenAI-compatible presets. Run `llmeter providers list` for the complete catalog, default URLs, and compatibility tiers.

## Installation

### Using an authorized prebuilt binary

The release workflow publishes validated GNU/Linux x86-64 and Windows x86-64 archives only from authorized `v*` tags. macOS is currently source-compatible but has no release artifact. Extract the binary, verify the archive and checksum independently, rename it to `llmeter` or `llmeter.exe` if needed, and place it on your `PATH`.

By default, LLMeter stores its config and outputs under `%USERPROFILE%\\.llmeter` on Windows and `~/.llmeter` on Unix. Set `LLMETER_HOME` to move that state into another portable folder.

### Building from source

```bash
git clone <repo-url>
cd llmeter
cargo build --release
```

The binary is at `target/release/llmeter` or `target/release/llmeter.exe`.

## First run

Start your provider server externally, then run:

```bash
llmeter providers list
llmeter --provider ollama status
llmeter --provider ollama models
```

Run a benchmark:

```bash
llmeter --provider ollama bench run --suite llm --models all --benchmarks all
```

Open the interactive menu:

```bash
llmeter --provider lmstudio
```

Create a managed install for `cmd.exe` usage after adding `<LLMETER_HOME>\bin` to `PATH`:

```cmd
llmeter install
```

## Verify it works

`llmeter status` should show the selected provider, base URL, reachable API status, and model count.

For providers that require authentication, set `LLMETER_API_KEY` for the process. The key is used ephemerally and is not saved in the LLMeter configuration or result files.

Last updated: 2026-08-02
