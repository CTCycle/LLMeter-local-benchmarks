# LLMeter

Last updated: 2026-09-10

[![CI](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/workflows/ci.yml/badge.svg?branch=develop)](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/workflows/ci.yml?query=branch%3Adevelop) [![Rust](https://img.shields.io/badge/rust-2021-orange?logo=rust&logoColor=white)](./Cargo.toml) [![License](https://img.shields.io/badge/license-MIT-lightgrey)](./LICENSE)

LLMeter is a single-binary Rust CLI for measuring local OpenAI-compatible LLM providers. It combines a guided terminal workflow with scriptable commands for repeatable benchmark runs, native performance scenarios, and report generation.

Current source release: `0.4.0`.

It can benchmark providers such as Ollama, LM Studio, llama.cpp, and other servers that expose a compatible `/v1` API. LLMeter does not start provider servers, install models, or manage provider processes: start the provider externally, expose at least one model, and then point LLMeter at its base URL.

## What LLMeter provides

- Interactive menus for provider setup, model inventory, benchmark selection, and saved reports.
- Scriptable `status`, `models`, `bench`, `report`, and `quality` commands for local automation and CI workflows.
- Standard `llm` and `embeddings` suites covering generation, consistency, prompt sizes, structured output, tool calling, responses, and embeddings.
- Native `bench perf` profiles for latency, throughput, warmups, concurrency sweeps, capability probes, telemetry, and request-budget planning.
- JSON and CSV raw exports plus Markdown and self-contained HTML reports.
- Persisted result schema `3.0`, with mandatory schema identity and run kind and no implicit legacy-result normalization.
- Privacy-aware output defaults: response previews are omitted unless requested, and credential-shaped values are redacted before persistence.

Quality commands are intentionally a planning surface. `llmeter quality plan` prints dry-run adapter commands for external tools; it does not install or execute those evaluators inside LLMeter.

## Before you start

You need:

- The current stable Rust toolchain when building from source.
- A running provider server with an OpenAI-compatible `/v1` API.
- At least one model loaded or exposed by that provider.

Common provider presets are:

| Preset | Default base URL | Typical use |
| --- | --- | --- |
| `ollama` | `http://localhost:11434/v1` | Local Ollama models. |
| `lmstudio` | `http://localhost:1234/v1` | Models served by LM Studio. |
| `llama-cpp` | `http://localhost:8080/v1` | `llama-server` from llama.cpp. |
| `openai-compatible` | `http://localhost:8000/v1` | A custom compatible server. |

Additional presets include `vllm`, `sglang`, `localai`, `litellm`, `tgi`, `text-generation-webui`, `jan`, and `mlx-lm`. Run `llmeter providers list` for the complete catalog and compatibility tiers.

For authenticated local endpoints, set `LLMETER_API_KEY` for the process that launches LLMeter. The bearer token is used by the active HTTP client only; it is not persisted or printed.

## Install

### Prebuilt releases

The [GitHub Releases](https://github.com/CTCycle/LLMeter-local-benchmarks/releases) page is the preferred portable installation path. Each release publishes these archives:

| Target | Archive |
| --- | --- |
| Windows x86-64 (MSVC) | `llmeter-v<version>-x86_64-pc-windows-msvc.zip` |
| Linux x86-64 (GNU) | `llmeter-v<version>-x86_64-unknown-linux-gnu.tar.gz` |
| macOS Apple silicon | `llmeter-v<version>-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `llmeter-v<version>-x86_64-apple-darwin.tar.gz` |

Extract an archive and run the binary directly, or place it in a user-owned directory on `PATH`. The Linux archive is dynamically linked and requires a compatible glibc runtime; it is not a fully static Linux build.

Verify a download from the directory containing the assets:

```bash
sha256sum --check SHA256SUMS
gh attestation verify llmeter-v<version>-x86_64-unknown-linux-gnu.tar.gz \
  --repo CTCycle/LLMeter-local-benchmarks
```

On Windows, use `Get-FileHash .\llmeter-v<version>-x86_64-pc-windows-msvc.zip -Algorithm SHA256` and compare it with the matching line in `SHA256SUMS`. GitHub records provenance attestations for the archives and checksum file; verify them with `gh attestation verify`.

### Install with Cargo

For a published release, Cargo provides the conventional installation path:

```bash
cargo install llmeter --locked
```

The crates.io publication remains a manual owner-gated step. Until `llmeter` is published for this release, install the checked-out source with `cargo install --path . --locked` or use the matching GitHub release archive after the tag-gated workflow completes.

### Build from source

```bash
git clone https://github.com/CTCycle/LLMeter-local-benchmarks.git
cd LLMeter-local-benchmarks
cargo build --release
```

The binary is written to `target/release/llmeter` on Unix-like systems or `target/release/llmeter.exe` on Windows.

You can also install a user-local copy with Cargo:

```bash
cargo install --path . --locked
```

On Windows, `run_llmeter.ps1` can build when needed and forward command arguments to the release binary:

```powershell
.\run_llmeter.ps1 status
.\run_llmeter.ps1 --provider lmstudio bench run --models all --benchmarks all
```

## Quick start

After installing `llmeter` or adding the release binary to `PATH`, start your provider and verify its catalog:

```bash
llmeter providers list
llmeter --provider ollama status
llmeter --provider ollama models
llmeter bench list --suite llm
```

Run the complete standard LLM suite against all exposed Ollama models:

```bash
llmeter --provider ollama bench run \
  --suite llm \
  --models all \
  --benchmarks all \
  --export both \
  --report both
```

To explore the guided workflow instead, run:

```bash
llmeter
```

The interactive menu covers provider setup, model inventory, benchmark workspaces, reports, built-in help, and exit. A subcommand should be supplied for non-interactive callers; a piped or CI invocation without a subcommand prints help and exits with usage status `2`. `status` always prints its panel: it exits `0` when the selected provider is reachable and `1` when the provider cannot be reached.

## Provider setup

Check a provider and use a custom endpoint when necessary:

```bash
llmeter --provider lmstudio --base-url http://localhost:1234/v1 status
llmeter --provider openai-compatible --base-url http://localhost:9000/v1 models
```

LLMeter accepts a base URL with or without the `/v1` suffix and normalizes it internally. Provider selection, provider-specific environment variables, and provider default URLs resolve through the provider catalog. An explicit global `LLMETER_BASE_URL` remains authoritative when a command overrides only the provider. `status`, `models`, benchmark model validation, and measured performance probes use fresh `/v1/models` reads. Ordinary interactive model navigation may use the client-local catalog cache; choose its explicit refresh action after loading or unloading a model.

Persist a default provider for later commands:

```bash
llmeter providers set ollama
```

## Run benchmarks

### Standard suites

The standard `llm` suite includes:

| Benchmark | Measures |
| --- | --- |
| `chat-generation` | Streaming chat latency, client-observed TTFT, usage, and throughput. |
| `responses-generation` | `/v1/responses` generation when the provider supports it. |
| `consistency` | Repeated responses, exact-match behavior, and pairwise text similarity. |
| `prompt-sizes` | Client-observed timing across short, medium, and long prompts. |
| `structured-output` | JSON-schema output requests and response validation. |
| `tool-calling` | Tool/function-call name and argument validation. |

The separate `embeddings` suite measures `/v1/embeddings` latency, vector count, and dimensions. A provider or model may not support every capability; unsupported benchmark calls are recorded as per-record errors so the rest of a run can continue.

Run selected benchmarks with explicit settings:

```bash
llmeter --provider lmstudio bench run \
  --suite llm \
  --models all \
  --benchmarks chat-generation,structured-output,tool-calling \
  --runs 5 \
  --max-tokens 256 \
  --temperature 0.2 \
  --export both \
  --report both
```

### Native performance profiles

`bench perf` is the canonical native scenario-based performance surface:

```bash
llmeter --provider ollama bench perf \
  --models all \
  --profile smoke \
  --export both \
  --report both
```

Available profiles are `smoke`, `latency`, `throughput`, and `sweep`. Preview a matrix before sending requests:

```bash
llmeter bench perf \
  --models llama3.1 \
  --profile sweep \
  --prompt-tokens 128,512 \
  --output-tokens 64,128 \
  --concurrency 1,2 \
  --dry-run
```

Performance plans show the selected models, matrix dimensions, warmup and measured request counts, total request estimate, and active request limit before execution. The default matrix budget is `500` requests; larger matrices require deliberate review and `--allow-large-matrix`. Prompt values above the documented bounds likewise require `--allow-large-prompt`.

Capability probes and telemetry are opt-in. Telemetry is disabled by default so sampling does not distort benchmark timings. The client-observed first-request versus warm-request load estimate is enabled by default and can be disabled with `--load-measurement off`:

```bash
llmeter --provider ollama bench perf \
  --models all \
  --profile latency \
  --runs 3 \
  --warmup 1 \
  --probe-capabilities \
  --telemetry detailed \
  --report both \
  --export both
```

### Interpreting timing results

- TTFT is the client-observed time to the first non-empty streamed content chunk. It is not tokenizer-confirmed.
- Inter-chunk latency measures streamed chunk arrival; it is not the same as inter-token latency (ITL).
- ITL is reported only when streaming timing and provider-reported output usage for at least two tokens are available.
- `estimated_load_overhead_ms` is a client-side first-request versus warm-request estimate. It does not measure provider restart, cache eviction, model loading, or native lifecycle telemetry.
- Compare runs on the same machine under similar provider, model, quantization, thermal, and memory conditions. Smoke runs are exploratory rather than statistically conclusive.

## Reports and saved results

By default, LLMeter writes state and benchmark outputs under:

- Windows: `%USERPROFILE%\.llmeter\benchmark_results`
- Unix: `~/.llmeter/benchmark_results`

Set `LLMETER_HOME` to move the state tree, or use `--output-dir` / `LLMETER_OUTPUT_DIR` for results:

```bash
llmeter --output-dir ./benchmark-runs bench run --models all --benchmarks all
llmeter report list
llmeter report show
llmeter report generate --format both
```

Each run can produce:

| File | Purpose |
| --- | --- |
| `<run-id>.json` | Canonical result data, including performance metadata and request traces where retained. |
| `<run-id>.csv` | Flattened records for spreadsheet analysis. |
| `<run-id>.report.md` | Human-readable Markdown summary. |
| `<run-id>.report.html` | Self-contained browser report. |

New runs use result schema `3.0`. Schema identity and run kind are mandatory, and older result schemas are rejected rather than silently upgraded. Reports include benchmark summaries, latency and throughput measures, token-usage coverage, capability errors, and performance sections when applicable.

Response previews are omitted by default. Use `--include-response-preview` only when model output is safe to retain. Saved artifacts also redact credential-shaped values and explicitly secret-named provider parameters before writing, while preserving non-secret benchmark metadata such as token-count settings.

## Configuration

CLI flags take precedence over environment variables and persisted configuration. The most useful settings are:

| Variable | Default | Purpose |
| --- | --- | --- |
| `LLMETER_HOME` | `%USERPROFILE%\.llmeter` / `~/.llmeter` | Root for persisted configuration and default outputs. |
| `LLMETER_PROVIDER` | `ollama` | Default provider preset. |
| `LLMETER_BASE_URL` | Provider default | OpenAI-compatible base URL override. |
| `LLMETER_OUTPUT_DIR` | `<LLMETER_HOME>/benchmark_results` | Default result and report directory. |
| `LLMETER_CONFIG_DIR` | `<LLMETER_HOME>/config` | Persisted configuration directory override. |
| `LLMETER_TIMEOUT` | `120` seconds | HTTP request timeout. |
| `LLMETER_RUNS` | `3` | Default repeated runs per benchmark. |
| `LLMETER_MAX_TOKENS` | `128` | Default generation output cap. |
| `LLMETER_TEMPERATURE` | `0.2` | Default sampling temperature. |
| `LLMETER_API_KEY` | Unset | Ephemeral bearer token for authenticated providers. |

Provider-specific base URL variables such as `OLLAMA_HOST`, `LMSTUDIO_BASE_URL`, and `LLAMA_CPP_BASE_URL` are also supported. See [configuration documentation](assets/docs/runtime/configuration.md) for precedence and validation rules.

## Troubleshooting

| Symptom | What to check |
| --- | --- |
| Provider is unreachable | Start the provider externally, confirm the base URL, and run `llmeter status`. |
| No models are listed | Load or expose a model in the provider, then run `llmeter models` or refresh the interactive inventory. |
| A benchmark records endpoint errors | Check the provider's supported `/v1` capabilities; unsupported calls remain visible as error records. |
| A performance plan is rejected | Use `--dry-run` to inspect request counts and reduce the matrix, or deliberately review `--allow-large-prompt` / `--allow-large-matrix`. |
| A report cannot be loaded | Confirm that the JSON result uses the current schema and is complete, rather than truncated, malformed, or from an unsupported older schema. |
| Concerned about secrets in output | Keep response previews disabled and pass `LLMETER_API_KEY` through the process environment rather than command arguments. |

## Development

Run the repository quality checks from its root:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -- --test-threads=1
cargo doc --no-deps --all-features
```

Tests should use one thread because configuration tests mutate process environment variables. The main source areas are:

- `src/cli.rs`, `src/config.rs`, and `src/ui.rs` for command parsing, configuration, and menus.
- `src/providers.rs` for OpenAI-compatible provider access.
- `src/benchmarks/` for standard benchmark implementations.
- `src/performance/` for native performance planning and execution.
- `src/results.rs` and `src/reporting.rs` for persistence and report generation.
- `tests/` for integration and mock-provider coverage.

## Documentation

- [USER_MANUAL.md](USER_MANUAL.md): end-user command reference, provider setup, benchmarks, reports, and troubleshooting.
- [SUPPORTED_PLATFORMS.md](SUPPORTED_PLATFORMS.md): support tiers and distribution boundaries.
- [assets/docs/project_index.md](assets/docs/project_index.md): entry point for the maintained documentation tree.
- [assets/docs/user/interactive_usage.md](assets/docs/user/interactive_usage.md): guided menu behavior.
- [assets/docs/user/scriptable_usage.md](assets/docs/user/scriptable_usage.md): non-interactive commands and automation examples.
- [assets/docs/user/benchmarks.md](assets/docs/user/benchmarks.md): benchmark families and metric definitions.
- [assets/docs/user/reports_and_results.md](assets/docs/user/reports_and_results.md): saved formats, privacy policy, and interpretation guidance.
- [assets/docs/runtime/release_checklist.md](assets/docs/runtime/release_checklist.md): release validation and artifact checklist.

## License

LLMeter is distributed under the MIT License. See [LICENSE](LICENSE).
