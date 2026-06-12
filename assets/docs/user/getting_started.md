# Getting started

## Requirements

- Rust toolchain only when building from source.
- A running local provider server with an OpenAI-compatible `/v1` API.
- At least one model exposed by that provider.

Supported presets:

- `ollama`
- `lmstudio`
- `llama-cpp`
- `openai-compatible`

## Installation

### Using a prebuilt binary

Download the binary for your platform, rename it to `llmeter` or `llmeter.exe`, and place it on your `PATH`.

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
llmeter --provider ollama bench run --models all --benchmarks all
```

Open the interactive menu:

```bash
llmeter --provider lmstudio
```

## Verify it works

`llmeter status` should show the selected provider, base URL, reachable API status, and model count.

Last updated: 2026-06-12
