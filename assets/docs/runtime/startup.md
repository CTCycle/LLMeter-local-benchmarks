# Startup
Last updated: 2026-09-10

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

Or use the repo launcher, which builds when needed, reports build and binary
status, passes the remaining arguments to the binary unchanged, and falls back
to a temp Cargo target directory if the workspace `target\release` tree is
locked on Windows:

```powershell
.\run_llmeter.ps1 --provider ollama status
.\run_llmeter.ps1 --provider ollama models
.\run_llmeter.ps1 --provider ollama
```

The wrapper also exposes storage-aware maintenance actions:

```powershell
.\run_llmeter.ps1 -Action Clean
.\run_llmeter.ps1 -Action RemoveAllData
.\run_llmeter.ps1 -Action Uninstall
```

Each destructive action requires an affirmative `[y/N]` confirmation and fails
closed when input is redirected. `Clean` removes the repository or fallback
Cargo build trees and repository-local legacy cache paths such as `.uv-cache`.
`RemoveAllData` additionally removes LLMeter-owned `bin`,
`config`, and `benchmark_results` data under the configured `LLMETER_HOME`.
`Uninstall` removes only the managed installation, cleans wrapper-owned build/cache paths, and preserves home data.
Provider servers, model caches, externally selected output directories, and
repository lockfiles are not removed. Normal remaining arguments continue to
pass through to the binary unchanged; forwarded `uninstall` and
`--purge-home` commands are confirmed by the wrapper as well.

The wrapper accepts normal LLMeter flags and subcommands as positional
pass-through arguments, including arguments that begin with `--`:

```powershell
.\run_llmeter.ps1 --version
.\run_llmeter.ps1 --provider ollama status
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

`status`, `models`, benchmark model validation, and measured performance probes refresh `/v1/models` from the provider. Interactive model inventory can list the current client-local cache or explicitly refresh it.

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
llmeter --provider ollama bench run --suite llm --models all --benchmarks all --export both --report both
```

This performs provider validation, builds a benchmark plan, runs benchmarks serially, saves raw outputs, and then generates formatted reports.
