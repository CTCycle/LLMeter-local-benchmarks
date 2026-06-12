# System overview

## Design goals

LLMeter is a single-binary Rust CLI for benchmarking local Ollama models. It provides an interactive terminal UI, scriptable subcommands, and formatted report generation — all with zero runtime dependencies beyond the OS.

Primary goals:

1. **Rust-first** — single statically-linked binary, no runtime required.
2. **Interactive by default** — rich terminal UI with `inquire` prompts and `tabled` tables.
3. **Zero runtime deps** — everything is compiled into the binary.
4. **Clear result persistence** — JSON and CSV for raw data, Markdown and HTML for reports.
5. **Modular benchmark registration** — add benchmarks by implementing a single trait.
6. **Maximum portability** — `rustls` instead of `openssl`, full static linking.
7. **Maximum performance** — async HTTP with streaming TTFT measurement via `reqwest`.

## Stack

| Area | Crate | Notes |
|---|---|---|
| Language | Rust (edition 2021) | Single binary via `cargo build --release` |
| CLI parser | `clap` (derive) | Auto-generated help, shell completions |
| Interactive UI | `inquire` | Confirm, Select, MultiSelect, CustomType, Text |
| Terminal tables | `tabled` | Formatted output for models, summaries, file lists |
| Terminal colors | `colored` | Lightweight ANSI terminal coloring |
| Markdown report | `comrak` | Terminal markdown rendering for `report show` |
| HTTP client | `reqwest` + `rustls-tls` | No `openssl` → fully static binary |
| JSON | `serde` + `serde_json` | Derive-based serialization |
| CSV | `csv` | Native Rust CSV writer |
| Timestamps | `chrono` | ISO-8601 UTC |
| Process info | `sysinfo` | Cross-platform PID alive checks |
| Text similarity | `similar` | Pairwise sequence matching for consistency benchmark |
| Error handling | `anyhow` + `thiserror` | CLI-level `anyhow`, library-level `thiserror` |

## Module map

```text
src/
  main.rs              Entry point. Parses CLI, dispatches to interactive or scriptable mode.
  lib.rs               Public crate root for integration tests.
  cli.rs               Clap derive API — all subcommands and option parsing.
  config.rs            AppConfig struct — reads LLMETER_* env vars at runtime.
  errors.rs            thiserror enum hierarchy (LLMeterError).
  utils.rs             ISO timestamps, nanosecond conversions, slugify.
  prompts.rs           Built-in prompt strings (short, medium, long, consistency).
  ollama/
    client.rs          Async (blocking) Ollama REST client. Streaming generate for TTFT.
    server.rs          Executable detection, server start/stop, PID tracking.
  benchmarks/
    base.rs            Benchmark trait, BenchmarkContext, BenchmarkResultRecord.
    registry.rs        Default benchmark registry and selection.
    generation.rs      Basic generation latency benchmark (streaming).
    consistency.rs     Response consistency benchmark (pairwise similarity).
    prompt_sizes.rs    Prompt size performance benchmark.
    metrics.rs         Metric extraction helpers, pairwise similarity utility.
  runner.rs            Validates models, selects benchmarks, orchestrates execution, saves outputs.
  results.rs           BenchmarkRun model, JSON/CSV persistence, ResultStore.
  reporting.rs         Markdown and HTML report generation from BenchmarkRun.
  ui.rs                inquire/tabled/comrak based interactive menus and terminal output.
```

Last updated: 2026-06-12
