# LLMeter

**Benchmark local OpenAI-compatible language-model providers from a single Rust command-line tool.**

[![License](https://img.shields.io/github/license/CTCycle/LLMeter-local-benchmarks?style=flat-square)](LICENSE) [![Release](https://img.shields.io/github/v/release/CTCycle/LLMeter-local-benchmarks?style=flat-square)](https://github.com/CTCycle/LLMeter-local-benchmarks/releases) [![CI](https://img.shields.io/github/actions/workflow/status/CTCycle/LLMeter-local-benchmarks/ci.yml?branch=develop&style=flat-square)](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/workflows/ci.yml?query=branch%3Adevelop) [![Rust 2021](https://img.shields.io/badge/Rust-2021-000000?logo=rust&logoColor=white&style=flat-square)](Cargo.toml) [![Platforms](https://img.shields.io/badge/platforms-Windows%20%7C%20Linux%20%7C%20macOS-informational?style=flat-square)](SUPPORTED_PLATFORMS.md)

## Overview

LLMeter is a single-binary CLI for benchmarking local and custom OpenAI-compatible providers. Start the provider separately and expose at least one model through its `/v1` API; LLMeter does not start provider servers or load models.

## Key Features

- Interactive terminal workflows and scriptable commands for `llm` and `embeddings` benchmark suites.
- Native performance scenarios for smoke, latency, throughput, and parameter sweeps.
- JSON and CSV measurements plus Markdown and self-contained HTML reports.
- Dry-run quality plans that print commands or serialized plans for external evaluators without installing or running them.
- Secret redaction and opt-in response previews for persisted results.

Performance results describe the selected provider, model, host, and run conditions. They are not hardware-independent rankings.

## Technology Stack

LLMeter is written in Rust 2021 and builds as a native CLI for the platforms listed in [Supported Platforms](SUPPORTED_PLATFORMS.md).

## Installation and Usage

Download a release archive from [GitHub Releases](https://github.com/CTCycle/LLMeter-local-benchmarks/releases). Archives are published for Windows x86-64, GNU/Linux x86-64, macOS Intel, and macOS Apple silicon with checksums and provenance attestations. Verify the archive before running it. GNU/Linux archives require a compatible glibc runtime.

To build and install from a source checkout, install the stable Rust toolchain, then run:

```bash
git clone https://github.com/CTCycle/LLMeter-local-benchmarks.git
cd LLMeter-local-benchmarks
cargo install --path . --locked
```

Start a provider and then check its catalog and status:

```bash
llmeter providers list
llmeter --provider ollama status
llmeter --provider ollama models
```

Run the standard LLM suite and save measurements and reports:

```bash
llmeter --provider ollama bench run \
  --suite llm \
  --models all \
  --benchmarks all \
  --export both \
  --report both
```

On Windows, the repository also includes `run_llmeter.ps1`, which builds when needed and forwards CLI arguments to the release binary.

## Configuration

- Select a provider preset with `--provider`; custom OpenAI-compatible endpoints and provider setup are covered in the [User Manual](USER_MANUAL.md).
- Set `LLMETER_API_KEY` in the current process for providers that require authentication. The key is not persisted.
- By default, LLMeter stores its configuration and results under `%USERPROFILE%\.llmeter` on Windows or `~/.llmeter` on Unix; result files are placed in `benchmark_results/`. Set `LLMETER_HOME` to move this state tree.
- Set `LLMETER_OUTPUT_DIR` or use `--output-dir` to choose a result location.
- Response previews are omitted by default, and credential-shaped values are redacted before persistence.

## Documentation

- [User Manual](USER_MANUAL.md)
- [Supported Platforms](SUPPORTED_PLATFORMS.md)

## License

Licensed under the MIT License. See [LICENSE](LICENSE).
