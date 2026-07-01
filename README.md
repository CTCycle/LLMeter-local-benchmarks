# LLMeter

[![Rust](https://img.shields.io/badge/rust-2021-orange?logo=rust&logoColor=white)](./Cargo.toml) [![License](https://img.shields.io/badge/license-MIT-lightgrey)](./LICENSE)

## 1. Project Overview
LLMeter is a self-contained Rust CLI for benchmarking local OpenAI-compatible LLM providers. It targets local `/v1` endpoints such as Ollama, LM Studio, llama.cpp, and custom OpenAI-compatible servers, and it combines guided terminal flows with scriptable commands for repeatable benchmark runs.

Key capabilities:
- interactive menu and scriptable subcommands
- persisted default provider selection plus run-specific provider overrides
- split benchmark suites for standard `llm` runs and separate `embeddings` runs
- live benchmark progress with current phase, current step, and completion percentage
- JSON and CSV raw exports plus Markdown and HTML reports

LLMeter does not start or stop provider servers. Start the local provider first, then point LLMeter at its `/v1` base URL.

## 2. Quick Start

### 2.1 Build
```bash
cargo build --release
```

### 2.2 Check Provider Status
```bash
llmeter --provider ollama status
llmeter --provider ollama models
```

### 2.3 Run Benchmarks
Scripted run:
```bash
llmeter --provider ollama bench run --suite llm --models all --benchmarks all --export both --report both
```

Interactive run:
```bash
llmeter
```

During benchmark execution, LLMeter shows live progress for validation, planning, benchmark steps, raw result saving, and report generation.

## 3. Using the CLI
Typical workflow:
1. Select or configure the provider and base URL.
2. Verify the provider is reachable and exposes models.
3. Run either the guided benchmark flow or `bench run`.
4. Review the terminal summary.
5. Open saved JSON, CSV, Markdown, or HTML outputs from the configured output directory.

Common commands:
```bash
llmeter providers list
llmeter providers set ollama
llmeter install
llmeter update --source C:\path\to\llmeter.exe
llmeter uninstall
llmeter bench list --suite llm
llmeter report list
llmeter report show
llmeter help bench
```

## 4. Benchmark Progress and Outputs
Benchmark runs now report:
- current lifecycle phase
- current model and benchmark
- current run or prompt step where applicable
- completion percentage based on planned benchmark work units

Saved outputs can include:
- `.json` raw benchmark run data
- `.csv` flattened result exports
- `.md` Markdown reports
- `.html` formatted reports

Default output location:
- `%USERPROFILE%\\.llmeter\\benchmark_results` on Windows
- `~/.llmeter/benchmark_results` on Unix

Set `LLMETER_HOME` to move both persisted config and the default output tree into another folder.

## 5. Development
Run:
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -- --test-threads=1
```

Use a single test thread because some configuration tests mutate process environment variables.

The codebase is organized under:
- `src`: CLI, provider client, benchmark implementations, runner, reporting, and UI
- `tests`: integration tests for metrics, registry, reporting, and results
- `assets/docs`: maintainers' documentation tree

## 6. Documentation Map
- [USER_MANUAL.md](USER_MANUAL.md): end-user installation, commands, provider setup, reports, and troubleshooting.
- [assets/docs/project_index.md](assets/docs/project_index.md): entry point for the internal documentation tree.
- [assets/docs/architecture/cli_flow.md](assets/docs/architecture/cli_flow.md): menu structure and execution flow.
- [assets/docs/user/interactive_usage.md](assets/docs/user/interactive_usage.md): guided menu behavior.
- [assets/docs/user/scriptable_usage.md](assets/docs/user/scriptable_usage.md): non-interactive command usage.
- [assets/docs/runtime/release_checklist.md](assets/docs/runtime/release_checklist.md): release validation and artifact checklist.

## 7. License
Distributed under the MIT License. See [LICENSE](LICENSE).

Last updated: 2026-07-01
