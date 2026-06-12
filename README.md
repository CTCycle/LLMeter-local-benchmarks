# LLMeter

A self-contained Rust CLI for benchmarking locally installed Ollama models — with zero runtime dependencies.

Single statically-linked binary. No Python. No `node_modules`. No virtualenvs.

## Features

- Modern interactive terminal menu built with `inquire`.
- Scriptable subcommands for automation and CI.
- Ollama installation and server status checks, start/stop lifecycle management.
- List installed models with size, quantization, and family info.
- Three built-in benchmarks:
  - **generation-latency** — wall time, TTFT, token throughput.
  - **consistency** — exact-match ratio, pairwise text similarity.
  - **prompt-sizes** — short/medium/long prompt comparison.
- Save raw results as JSON and CSV.
- Generate formatted Markdown and HTML reports with dark mode support.
- `rustls`-based HTTP — fully static binary, no `openssl` dependency.

## Quick start

```bash
# Download binary, then:
llmeter status                     # check Ollama
llmeter bench run --models all --benchmarks all --start-server   # run everything
```

Or open the interactive menu:

```bash
llmeter
```

## Requirements

- Ollama installed and available on `PATH`
- At least one local Ollama model (`ollama pull llama3.2`)

No Python or runtime dependencies required.

## Installation

### Prebuilt binary

Download from the releases page for your platform.

### From source

```bash
cargo build --release
cp target/release/llmeter ~/.local/bin/   # or your PATH directory
```

## Usage

| Command | Description |
|---|---|
| `llmeter` | Interactive main menu |
| `llmeter status` | Show Ollama status |
| `llmeter server start` | Start `ollama serve` |
| `llmeter server stop` | Stop tracked server |
| `llmeter models` | List installed models |
| `llmeter show <model>` | Show model metadata |
| `llmeter bench list` | List available benchmarks |
| `llmeter bench run --models all --benchmarks all` | Run all benchmarks |
| `llmeter report list` | List saved results |
| `llmeter report show` | View latest result |
| `llmeter report generate` | Generate MD/HTML report |

## Benchmark tests

| ID | Purpose |
|---|---|
| `generation-latency` | Wall time, TTFT, Ollama duration fields, token throughput |
| `consistency` | Repeated prompt: exact-match ratio, pairwise similarity |
| `prompt-sizes` | Short, medium, and long prompt comparison |

## Output files

Default: `benchmark_results/`

```text
<run-id>.json
<run-id>.csv
<run-id>.report.md
<run-id>.report.html
```

## Configuration

| Variable | Default | Description |
|---|---|---|
| `OLLAMA_HOST` | `http://localhost:11434` | Ollama server URL |
| `LLMETER_TIMEOUT` | `120` | HTTP timeout (seconds) |
| `LLMETER_OUTPUT_DIR` | `benchmark_results` | Output directory |
| `LLMETER_STATE_DIR` | `~/.llmeter` | State directory |
| `LLMETER_RUNS` | `3` | Default repetitions |
| `LLMETER_NUM_PREDICT` | `128` | Default token cap |
| `LLMETER_TEMPERATURE` | `0.2` | Default temperature |

## Architecture

```text
src/
  main.rs              Entry point
  lib.rs               Crate root
  cli.rs               Clap CLI definition
  config.rs            AppConfig (env + CLI overrides)
  errors.rs            Error types
  utils.rs             Helpers (timestamps, slugify)
  prompts.rs           Built-in prompts
  ollama/
    client.rs          Ollama REST client (reqwest, streaming)
    server.rs          Server lifecycle management
  benchmarks/
    base.rs            Benchmark trait and data structs
    registry.rs        Benchmark registration
    metrics.rs         Metric helpers, pairwise similarity
    generation.rs      Latency benchmark
    consistency.rs     Consistency benchmark
    prompt_sizes.rs    Prompt size benchmark
  runner.rs            Benchmark orchestration
  results.rs           JSON/CSV persistence
  reporting.rs         Markdown/HTML report generation
  ui.rs                Interactive terminal UI
```

## Adding a benchmark

Implement the `Benchmark` trait and register it:

```rust
struct MyBenchmark;

impl Benchmark for MyBenchmark {
    fn id(&self) -> &str { "my-benchmark" }
    fn name(&self) -> &str { "My benchmark" }
    fn description(&self) -> &str { "What it measures." }
    fn run(&self, client: &OllamaClient, model: &str, context: &BenchmarkContext) -> Vec<BenchmarkResultRecord> {
        // ...
    }
}

// In registry.rs:
registry.register(Box::new(MyBenchmark));
```

## Development

```bash
cargo test              # run tests
cargo clippy            # lint
cargo fmt --check       # format check
cargo build --release   # release build
```

## Limitations

- Ollama only. No other providers.
- No concurrent benchmark execution.
- Results are machine-specific.
- HTML reports are intentionally minimal.

Last updated: 2026-06-12
