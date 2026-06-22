# Startup

## Prerequisites

Before launching LLMeter:

1. Install or build the `llmeter` binary.
2. Start the local provider server externally.
3. Ensure at least one model is exposed by that provider.

LLMeter does not manage provider startup or model loading.

## Typical startup sequence

PowerShell:

```powershell
.\target\release\llmeter.exe --provider ollama status
.\target\release\llmeter.exe --provider ollama models
.\target\release\llmeter.exe --provider ollama
```

Or use the repo launcher, which builds when needed and falls back to a temp Cargo target directory if the workspace `target\release` tree is locked on Windows:

```powershell
.\run_llmeter.ps1 --provider ollama status
.\run_llmeter.ps1 --provider ollama models
.\run_llmeter.ps1 --provider ollama
```

CMD:

```cmd
cargo build --release
target\release\llmeter.exe --provider ollama status
target\release\llmeter.exe --provider ollama models
target\release\llmeter.exe --provider ollama
```

Installed binary:

```bash
llmeter --provider ollama status
llmeter --provider ollama models
llmeter --provider ollama
```

## Startup path inside the binary

At launch:

1. `clap` parses the CLI shape.
2. `AppConfig::from_env()` resolves provider, base URL, timeout, output directory, and defaults.
3. `ProviderClient::new()` builds the HTTP client.
4. `main.rs` dispatches to interactive menu, provider command, benchmark command, report command, or help topic.

## Recommended prechecks

Run these before a full benchmark:

```bash
llmeter providers list
llmeter --provider ollama status
llmeter --provider ollama models
```

If those checks fail, benchmark execution will also fail.

## Scripted benchmark startup

Example:

```bash
llmeter --provider ollama bench run --models all --benchmarks all --export both --report both
```

This performs provider validation, builds a benchmark plan, runs benchmarks serially, saves raw outputs, and then generates formatted reports.

Last updated: 2026-06-21
