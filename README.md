# LLMeter

Last updated: 2026-09-30

[![CI](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/workflows/ci.yml/badge.svg?branch=develop)](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/workflows/ci.yml?query=branch%3Adevelop) [![Rust](https://img.shields.io/badge/rust-2021-orange?logo=rust&logoColor=white)](./Cargo.toml) [![License](https://img.shields.io/badge/license-MIT-lightgrey)](./LICENSE)

LLMeter is a single-binary Rust CLI for measuring local OpenAI-compatible LLM providers. It offers a guided terminal workflow, scriptable benchmark commands, native performance scenarios, and Markdown/HTML reports.

The current package version is `0.5.0` release candidate on `develop`. The public [GitHub release](https://github.com/CTCycle/LLMeter-local-benchmarks/releases/tag/v0.4.0) remains the verified public release until the `v0.5.0` tag workflow succeeds; the first crates.io publication remains owner-gated.

LLMeter does not start provider servers, install models, or manage provider processes. Start a provider externally, expose at least one model, and give LLMeter its OpenAI-compatible `/v1` base URL.

## Before you start

- Rust stable, when building from source.
- A running provider with an OpenAI-compatible `/v1` API.
- At least one loaded or exposed model.

Common presets include `ollama`, `lmstudio`, `llama-cpp`, and `openai-compatible`. Run `llmeter providers list` for the complete catalog and compatibility tiers. Provider and model capabilities remain endpoint- and version-dependent.

## Install

### Verified release archive

Download the appropriate archive from [GitHub Releases](https://github.com/CTCycle/LLMeter-local-benchmarks/releases). Authorized tags publish Windows x86-64, GNU/Linux x86-64, macOS Intel, and macOS Apple silicon archives with `SHA256SUMS` and GitHub artifact provenance.

Verify an archive before running it:

```bash
sha256sum --check SHA256SUMS
gh attestation verify llmeter-v<version>-<target>.<archive> \
  --repo CTCycle/LLMeter-local-benchmarks
```

On Windows, compare `Get-FileHash <archive> -Algorithm SHA256` with the matching checksum entry. The GNU/Linux archive requires a compatible glibc runtime and is not fully static. See [supported platforms](SUPPORTED_PLATFORMS.md) for the maintained boundaries.

### Build from source

```bash
git clone https://github.com/CTCycle/LLMeter-local-benchmarks.git
cd LLMeter-local-benchmarks
cargo build --release
```

Until the crate is published, install a checked-out source tree with:

```bash
cargo install --path . --locked
```

On Windows, the repository launcher can build when needed and forward arguments to the release binary:

```powershell
.\run_llmeter.ps1 status
.\run_llmeter.ps1 --provider lmstudio bench run --models all --benchmarks all
```

The launcher and the `install`, `update`, and `uninstall` commands are local convenience operations. They do not download, authenticate, verify, or select remote updates. Verify any replacement executable yourself.

## Quick start

Start the provider, then inspect its preset and model catalog:

```bash
llmeter providers list
llmeter --provider ollama status
llmeter --provider ollama models
llmeter bench list --suite llm
```

Run the standard LLM suite:

```bash
llmeter --provider ollama bench run \
  --suite llm \
  --models all \
  --benchmarks all \
  --export both \
  --report both
```

Run the guided workflow with:

```bash
llmeter
```

Interactive mode requires a terminal. A piped or CI invocation without a subcommand prints help and returns usage status `2`. `status` prints its panel and returns `0` for a reachable provider or `1` when the provider cannot be reached.

## Native performance

`llmeter bench perf` runs the native `smoke`, `latency`, `throughput`, and `sweep` profiles:

```bash
llmeter --provider ollama bench perf \
  --models all \
  --profile smoke \
  --export both \
  --report both
```

Preview a matrix without sending requests:

```bash
llmeter bench perf \
  --models llama3.1 \
  --profile sweep \
  --prompt-tokens 128,512 \
  --output-tokens 64,128 \
  --concurrency 1,2 \
  --dry-run
```

The default performance request ceiling is `500`, counting warmups and measured requests. Larger matrices require `--allow-large-matrix`; prompt or output sizes above the documented bounds require `--allow-large-prompt`.

Telemetry is `off` by default. `--telemetry standard` or `--telemetry detailed` samples host state during execution. The default load mode is the client-observed `first-request-estimate`; use `--load-measurement off` to disable it. Neither signal measures provider restart, cache eviction, model loading, or native lifecycle telemetry.

Performance results describe the observed provider, model, host, sample count, and telemetry conditions. They are not universal hardware-independent rankings, and timing comparisons are not meaningful when the result reports material swap pressure.

## Results and privacy

By default, LLMeter stores state and results under:

- Windows: `%USERPROFILE%\.llmeter\benchmark_results`
- Unix: `~/.llmeter/benchmark_results`

Set `LLMETER_HOME` to move the state tree, or use `--output-dir` / `LLMETER_OUTPUT_DIR` for results. A run may produce JSON, CSV, Markdown, and self-contained HTML files. Current result files use schema `3.0`; `schema_version` and `run_kind` are mandatory, and older or malformed schemas are rejected rather than silently normalized.

Response previews are omitted from saved output unless `--include-response-preview` is requested. Credential-shaped values and explicitly secret-named provider parameters are redacted before persistence. `LLMETER_API_KEY` is process-local and is not persisted or printed.

## Quality planning

`llmeter quality` is a dry-run planning surface for external evaluators. It lists supported plan adapters and prints commands or serialized plans; LLMeter does not install or execute those frameworks and does not produce their scores.

## Development

Run the repository gates from its root:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -- --test-threads=1
cargo doc --no-deps --all-features
```

Use one test thread because configuration tests mutate process environment variables. The [testing and quality guide](assets/docs/coding/testing_and_quality.md) describes the full locked and hosted checks.

## Documentation

- [User manual](USER_MANUAL.md): command reference, provider setup, benchmarks, reports, and troubleshooting.
- [Supported platforms](SUPPORTED_PLATFORMS.md): support tiers and distribution boundaries.
- [Project index](assets/docs/project_index.md): maintained documentation ontology and reading order.
- [Project status ledger](assets/docs/project_status_ledger.md): current validation status, evidence boundaries, and open work.
- [Validation campaign](assets/docs/runtime/validation_campaign.md): ordered validation scope and claim boundaries.
- [Release checklist](assets/docs/runtime/release_checklist.md): maintainer gates, artifacts, and trust model.

Durable QA knowledge belongs in the documents above, tests, and linked hosted CI; dated per-run reports are not maintained as parallel documentation.

## License

LLMeter is distributed under the MIT License. See [LICENSE](LICENSE).
