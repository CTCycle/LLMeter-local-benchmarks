# System overview

## Design goals

LLMeter is a single-binary Rust CLI for benchmarking local OpenAI-compatible LLM providers. It supports interactive terminal workflows, scriptable subcommands, and formatted report generation.

Primary goals:

1. **Provider-neutral local benchmarking** - provide presets and OpenAI-compatible request handling for Ollama, LM Studio, llama.cpp, and custom `/v1` servers; runtime support is confirmed by endpoint checks rather than preset selection alone.
2. **Interactive and scriptable** - rich terminal UI plus automation-friendly subcommands.
3. **Zero runtime deps** - everything is compiled into the binary.
4. **Clear result persistence** - JSON and CSV for raw data, Markdown and HTML for reports.
5. **Modular benchmark registration** - add benchmarks by implementing a single trait.
6. **Maximum portability** - `rustls` instead of `openssl`.
7. **Capability coverage** - measure text generation, prompt sizes, consistency, structured output, tool calls, responses, and embeddings.

## Stack

| Area | Crate | Notes |
|---|---|---|
| Language | Rust (edition 2021) | Single binary via `cargo build --release` |
| CLI parser | `clap` (derive) | Auto-generated help plus built-in help topics |
| Interactive UI | `inquire` | Select, MultiSelect, CustomType, Text |
| Terminal tables | `tabled` | Formatted output for providers, models, summaries, files |
| Terminal colors | `colored` | Lightweight ANSI terminal coloring |
| HTTP client | `reqwest` + `rustls-tls` | Blocking client with streaming response parsing |
| JSON | `serde` + `serde_json` | Derived serialization and dynamic metric dictionaries |
| CSV | `csv` | Native Rust CSV writer |
| Timestamps | `chrono` | ISO-style UTC run IDs |
| Text similarity | `similar` | Pairwise sequence matching for consistency benchmark |
| Error handling | `anyhow` + `thiserror` | CLI-level `anyhow`, library-level `thiserror` |

## Module map

```text
src/
  main.rs              Entry point. Parses CLI and dispatches commands.
  lib.rs               Public crate root for integration tests.
  cli.rs               Clap derive API, provider flags, subcommands, help alias.
  config.rs            AppConfig, provider/base URL/env defaults.
  errors.rs            thiserror enum hierarchy.
  providers.rs         OpenAI-compatible provider client and provider presets.
  utils.rs             Timestamps, slugify, preview, filesystem helpers.
  prompts.rs           Built-in benchmark prompt strings.
  benchmarks/
    base.rs            Benchmark trait, context, result record.
    registry.rs        Default benchmark registry and selection.
    generation.rs      Streaming chat generation latency benchmark.
    consistency.rs     Response consistency benchmark.
    prompt_sizes.rs    Prompt size performance benchmark.
    api_calls.rs       Responses, structured output, tool calling, embeddings.
    metrics.rs         Metric extraction helpers and pairwise similarity.
  runner.rs            Model validation, benchmark orchestration, output saving.
  results.rs           BenchmarkRun model, JSON/CSV persistence.
  reporting.rs         Markdown and HTML report generation.
  performance/         Scenario plans, load estimates, fresh probes, inventory, metrics, telemetry, and concurrency.
  quality/             External quality-framework catalog and dry-run plan adapters.
  lifecycle.rs         Local convenience install, update, and uninstall operations.
  ui.rs                Interactive menus and terminal output.
```

Last updated: 2026-07-30
