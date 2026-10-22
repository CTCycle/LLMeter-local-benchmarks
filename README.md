<div align="center">

# LLMeter

Benchmark local OpenAI-compatible language-model providers from a single Rust command-line tool.

[![GitHub stars](https://img.shields.io/github/stars/CTCycle/LLMeter-local-benchmarks?style=flat-square)](https://github.com/CTCycle/LLMeter-local-benchmarks/stargazers)
[![Last commit](https://img.shields.io/github/last-commit/CTCycle/LLMeter-local-benchmarks?style=flat-square)](https://github.com/CTCycle/LLMeter-local-benchmarks/commits)
[![Release](https://img.shields.io/github/v/release/CTCycle/LLMeter-local-benchmarks?style=flat-square)](https://github.com/CTCycle/LLMeter-local-benchmarks/releases)
[![License](https://img.shields.io/github/license/CTCycle/LLMeter-local-benchmarks?style=flat-square)](LICENSE)

[![Rust 2021](https://img.shields.io/badge/Rust-2021-000000?logo=rust&logoColor=white&style=flat-square)](Cargo.toml)
[![Platforms](https://img.shields.io/badge/platforms-Windows%20%7C%20Linux%20%7C%20macOS-informational?style=flat-square)](SUPPORTED_PLATFORMS.md)

</div>

## Overview

LLMeter is a single-binary CLI for benchmarking local and custom OpenAI-compatible providers. Start the provider separately and expose at least one model through its `/v1` API; LLMeter does not start provider servers or load models.

Performance results describe the selected provider, model, host, and run conditions. They are not hardware-independent rankings.

## Quick Start

Check a provider catalog and status:

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

Sample output:

```text
╭───────────────────────┬─────────────────────────┬───────────────────────────┬─────────────────────────────────────────╮
│ Provider presets and compatibility tiers                                                                              │
├───────────────────────┼─────────────────────────┼───────────────────────────┼─────────────────────────────────────────┤
│ Provider              │ Tier                    │ Default /v1 base URL      │ Notes                                   │
│ ollama                │ first-class             │ http://localhost:11434/v1 │ Existing first-class target             │
│ lmstudio              │ first-class             │ http://localhost:1234/v1  │ Existing first-class target             │
│ llama-cpp             │ first-class             │ http://localhost:8080/v1  │ Existing first-class target             │
╰───────────────────────┴─────────────────────────┴───────────────────────────┴─────────────────────────────────────────╯
```

## Key Features

- Interactive terminal workflows and scriptable commands for `llm` and `embeddings` benchmark suites.
- Native performance scenarios for smoke, latency, throughput, and parameter sweeps.
- JSON and CSV measurements plus Markdown and self-contained HTML reports.
- Dry-run quality plans that print commands or serialized plans for external evaluators without installing or running them.
- Secret redaction and opt-in response previews for persisted results.

## Screenshots/TUI

LLMeter renders provider catalogs, benchmark catalogs, and run summaries as rounded terminal tables. The sample above is representative of `llmeter providers list`.

## Technology Stack

| Component | Technology |
| --- | --- |
| Language | Rust 2021 |
| CLI parsing | clap 4 |
| HTTP client | reqwest with rustls |
| Async runtime | tokio |
| Serialization | serde, serde_json |
| Terminal tables | tabled |
| Interactive prompts | inquire |
| Terminal control | crossterm |

## Installation and Usage

Download a release archive from GitHub Releases. Archives are published for Windows x86-64, GNU/Linux x86-64, macOS Intel, and macOS Apple silicon with checksums and provenance attestations. Verify the archive before running it. GNU/Linux archives require a compatible glibc runtime.

To build and install from a source checkout, install the stable Rust toolchain, then run:

```bash
git clone https://github.com/CTCycle/LLMeter-local-benchmarks.git
cd LLMeter-local-benchmarks
cargo install --path . --locked
```

On Windows, the repository also includes `run_llmeter.ps1`, which builds when needed and forwards CLI arguments to the release binary.

Start a provider and then check its catalog and status:

```bash
llmeter providers list
llmeter --provider ollama status
llmeter --provider ollama models
```

## Configuration

Environment variables and persisted state control provider selection, endpoints, timeouts, and output locations.

| Variable or file | Purpose |
| --- | --- |
| `--provider` / `LLMETER_PROVIDER` | Select the provider preset |
| `--base-url` / `LLMETER_BASE_URL` | Override the OpenAI-compatible `/v1` base URL |
| Provider base URL variables | `OLLAMA_HOST`, `LMSTUDIO_BASE_URL`, `LLAMA_CPP_BASE_URL`, `VLLM_BASE_URL`, `SGLANG_BASE_URL`, `LOCALAI_BASE_URL`, `LITELLM_BASE_URL`, `TGI_BASE_URL`, `TEXT_GENERATION_WEBUI_BASE_URL`, `JAN_BASE_URL`, `MLX_LM_BASE_URL` |
| `LLMETER_API_KEY` | Ephemeral bearer token for providers that require authentication. Not persisted. |
| `LLMETER_HOME` | Move the state tree. Defaults to `~/.llmeter` or `%USERPROFILE%\.llmeter`. |
| `LLMETER_CONFIG_DIR` | Override the config directory. Defaults to `<LLMETER_HOME>/config`. |
| `LLMETER_OUTPUT_DIR` / `--output-dir` | Choose the result location. Defaults to `<LLMETER_HOME>/benchmark_results`. |
| `LLMETER_TIMEOUT` / `--timeout` | HTTP request timeout in seconds. Default 120. |
| `LLMETER_RUNS` | Default repeated runs per benchmark. Default 3. |
| `LLMETER_MAX_TOKENS` | Default maximum output tokens. Default 128. |
| `LLMETER_TEMPERATURE` | Default sampling temperature. Default 0.2. |
| `config.json` | Persisted default provider only, stored under `<LLMETER_HOME>/config/config.json`. |

Response previews are omitted by default. Credential-shaped values are redacted before persistence. See [SECURITY.md](SECURITY.md) for artifact handling.

## Provider Compatibility

| Provider | Tier | Default `/v1` base URL |
| --- | --- | --- |
| Ollama | first-class | http://localhost:11434/v1 |
| LM Studio | first-class | http://localhost:1234/v1 |
| llama.cpp | first-class | http://localhost:8080/v1 |
| OpenAI-compatible | custom | http://localhost:8000/v1 |
| vLLM | known OpenAI-compatible | http://localhost:8000/v1 |
| SGLang | known OpenAI-compatible | http://localhost:30000/v1 |
| LocalAI | known OpenAI-compatible | http://localhost:8080/v1 |
| LiteLLM | known OpenAI-compatible | http://localhost:4000/v1 |
| TGI | best effort | http://localhost:8080/v1 |
| text-generation-webui | best effort | http://localhost:5000/v1 |
| Jan | best effort | http://localhost:1337/v1 |
| MLX-LM | best effort | http://localhost:8080/v1 |

## Documentation

- [User Manual](USER_MANUAL.md)
- [Supported Platforms](SUPPORTED_PLATFORMS.md)
- [Security Policy](SECURITY.md)
- [Contributing Guide](CONTRIBUTING.md)
- [Project Index](assets/docs/project_index.md)

## Limitations

- LLMeter does not manage provider processes or install models. A running OpenAI-compatible `/v1` endpoint is required.
- GNU/Linux archives require a compatible glibc runtime. No musl artifact is currently promised.
- The minimum supported Rust version is not declared. Builds use the current stable toolchain.
- Quality framework commands produce dry-run plans only. They do not execute external evaluators.
- System and process telemetry is disabled by default.
- Results reflect the selected provider, model, host, and run conditions.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, scope, and pull request expectations.

## License

Licensed under the MIT License. See [LICENSE](LICENSE).
