# LLMeter User Manual

LLMeter benchmarks local OpenAI-compatible LLM providers through `/v1` APIs. It provides presets for Ollama, LM Studio, llama.cpp, and custom OpenAI-compatible local servers; actual availability and capability support depend on the server you run and the endpoints it exposes.

LLMeter does not start or stop provider servers. Start your provider externally, then use LLMeter to check status, list models, run benchmarks, and generate reports.

Windows x86-64 is the primary supported platform. GNU/Linux requires a compatible glibc runtime. Tagged releases also publish macOS Intel and Apple silicon archives; see `SUPPORTED_PLATFORMS.md` for the maintained support tiers.

## Installation

Build from source:

```bash
cargo build --release
```

The binary is at `target/release/llmeter` or `target/release/llmeter.exe`.

By default, LLMeter stores its runtime state in a single home folder:

- Windows: `%USERPROFILE%\\.llmeter`
- Unix: `~/.llmeter`
- Override: set `LLMETER_HOME`

On Windows PowerShell, the included launcher builds when needed, retries in a temp Cargo target directory if the workspace `target\release` tree is locked, and then runs the release binary:

```powershell
.\run_llmeter.ps1 status
.\run_llmeter.ps1 --provider lmstudio bench run --models all --benchmarks all
```

Create a managed install usable from `cmd.exe`:

```cmd
llmeter install
```

Update that managed install from a newer binary:

```cmd
llmeter update --source C:\downloads\llmeter.exe
```

The install/update/uninstall commands are local convenience operations. LLMeter does not download or authenticate remote updates; verify the replacement executable yourself before using `update --source`.

Uninstall it:

```cmd
llmeter uninstall
```

## Provider Setup

| Provider | Default base URL | Notes |
|---|---|---|
| `ollama` | `http://localhost:11434/v1` | Requires Ollama's OpenAI-compatible API. |
| `lmstudio` | `http://localhost:1234/v1` | Start the LM Studio local server and load a model. |
| `llama-cpp` | `http://localhost:8080/v1` | Start `llama-server` from llama.cpp with a model. |
| `openai-compatible` | `http://localhost:8000/v1` | Use with `--base-url` for any compatible local server. |

Provider metadata, canonical labels, provider-specific base URL environment variables, and provider default URLs are resolved through the provider catalog. An explicit `LLMETER_BASE_URL` remains authoritative when a command changes only the provider.

Check provider status:

```bash
llmeter --provider ollama status
llmeter --provider lmstudio --base-url http://localhost:1234/v1 status
llmeter --provider llama-cpp --base-url http://localhost:8080/v1 status
```

List models:

```bash
llmeter --provider ollama models
llmeter --provider lmstudio models
```

Model identity is the OpenAI-compatible `/v1/models` `id` field. Alternate historical model-name fields are not used as identity fallbacks.

## Built-In Help

Use the regular CLI help:

```bash
llmeter --help
llmeter bench --help
llmeter bench run --help
llmeter bench perf --help
```

Use LLMeter's built-in help topics:

```bash
llmeter help
llmeter help providers
llmeter help bench
llmeter help reports
llmeter help examples
```

## Command Reference

| Command | Description |
|---|---|
| `llmeter` | Open the interactive menu. |
| `llmeter status` | Check selected provider reachability and model count. |
| `llmeter providers list` | Show provider presets and default URLs. |
| `llmeter providers set <provider>` | Persist the default provider for future runs. |
| `llmeter models` | List models exposed by the selected provider. |
| `llmeter show <model>` | Show model metadata from `/v1/models`. |
| `llmeter bench list [--suite <suite>]` | List benchmark IDs and descriptions. |
| `llmeter bench run --suite llm --models all --benchmarks all` | Run standard benchmarks non-interactively. |
| `llmeter bench perf --models all --profile <profile>` | Run native performance scenarios. |
| `llmeter report list` | List saved raw results and generated reports. |
| `llmeter report show [result]` | Render a saved JSON result in the terminal. |
| `llmeter report generate [result] --format both` | Generate Markdown and/or HTML reports. |
| `llmeter quality list` | List external quality benchmark tasks. |
| `llmeter quality plan [options]` | Build a dry-run external quality benchmark plan. |
| `llmeter install [--force]` | Install a managed CLI copy into `<LLMETER_HOME>/bin`. |
| `llmeter update [--source <exe>]` | Update the managed CLI copy from a newer executable. |
| `llmeter uninstall [--purge-home]` | Remove the managed CLI copy and optionally remove `LLMETER_HOME`. |
| `llmeter help [topic]` | Show built-in help for `providers`, `bench`, `reports`, `install`, or `examples`. |

## Running Standard Benchmarks

Run every benchmark against every exposed model:

```bash
llmeter --provider ollama bench run --suite llm --models all --benchmarks all
```

Run selected benchmarks against LM Studio:

```bash
llmeter --provider lmstudio bench run \
  --suite llm \
  --models all \
  --benchmarks chat-generation,structured-output,tool-calling \
  --runs 3 \
  --max-tokens 128 \
  --temperature 0.2 \
  --export both \
  --report both
```

Run llama.cpp on a custom port:

```bash
llmeter --provider llama-cpp --base-url http://localhost:8081/v1 bench run \
  --suite embeddings \
  --models all \
  --benchmarks all
```

Add extra provider request parameters:

```bash
llmeter bench run --models all --benchmarks chat-generation --param top_p=0.9
```

Performance safety controls are dedicated CLI flags and cannot be supplied through generic `--param` values.

## Native Performance Benchmarks

`llmeter bench perf` is the canonical native performance command. Available profiles are `smoke`, `latency`, `throughput`, and `sweep`.

Run a smoke profile:

```bash
llmeter --provider ollama bench perf --models all --profile smoke --export both --report both
```

Preview a matrix without issuing benchmark requests:

```bash
llmeter bench perf \
  --models llama3.1 \
  --profile sweep \
  --prompt-tokens 128,512 \
  --output-tokens 64,128 \
  --concurrency 1,2 \
  --dry-run
```

The profile owns the default prompt sizes, output sizes, concurrency, warmup count, and measured run count. The same profile defaults are used by scriptable and guided performance flows.

Capability probing and telemetry are opt-in. Telemetry defaults to `off`. The client-observed first-request versus warm-request load estimate defaults to `first-request-estimate`; disable it explicitly with `--load-measurement off` when it is not wanted.

The default performance request budget is 500 warmup-plus-measured requests. Larger matrices require the explicit `--allow-large-matrix` override. Prompt or output sizes above the documented limits require `--allow-large-prompt`.

## Built-In Benchmarks

Standard `llm` suite:

| ID | Purpose |
|---|---|
| `chat-generation` | Streaming chat completion latency, TTFT, token usage, and throughput. |
| `responses-generation` | `/v1/responses` generation latency and usage when supported. |
| `consistency` | Repeated deterministic prompt similarity and exact-match behavior. |
| `prompt-sizes` | Short, medium, and long prompt performance comparison. |
| `structured-output` | JSON schema output request and validation. |
| `tool-calling` | Tool/function call request and argument validation. |

Separate `embeddings` suite:

| ID | Purpose |
|---|---|
| `embeddings` | Embeddings latency, vector count, and vector dimensions. |

Some providers or models may not support every endpoint or capability. Unsupported calls are recorded as error records in the result and report rather than stopping the entire run.

## Reports And Output Files

Default output directory: `<LLMETER_HOME>/benchmark_results/`

```text
<run-id>.json
<run-id>.csv
<run-id>.report.md
<run-id>.report.html
```

Current benchmark results use schema `3.0`. `schema_version` and `run_kind` are mandatory. Older or unversioned result schemas are rejected on load rather than silently normalized into the current representation.

Reports include provider, base URL, models, benchmark IDs, aggregate latency/throughput, token usage, structured-output validity, tool-call validity, embedding dimensions, detailed records, and errors.

Response previews are omitted from persisted output by default. Credential-shaped values and explicitly secret-named provider parameters are redacted, while non-secret benchmark metadata such as token-count settings is preserved.

Report commands:

```bash
llmeter report list
llmeter report show
llmeter report generate --format both
```

Use a custom output directory:

```bash
llmeter --output-dir ./benchmark-runs bench run --models all --benchmarks all
```

## Quality Planning

`llmeter quality` is a planning surface for external evaluators rather than a second benchmark-result implementation. Quality plans are emitted as dry-run planning output and are not persisted as an alternate `BenchmarkRun` shape.

```bash
llmeter quality list
llmeter quality plan --framework lighteval --task "leaderboard|mmlu|5" --model llama3.1
```

## Configuration

| Variable | Default | Description |
|---|---|---|
| `LLMETER_HOME` | `%USERPROFILE%\\.llmeter` on Windows, `~/.llmeter` on Unix | Root directory for LLMeter state. |
| `LLMETER_PROVIDER` | `ollama` | Provider preset. |
| `LLMETER_BASE_URL` | provider default | OpenAI-compatible `/v1` base URL. |
| `OLLAMA_HOST` | unset | If set for Ollama, LLMeter maps it to `<OLLAMA_HOST>/v1`. |
| `LMSTUDIO_BASE_URL` | unset | LM Studio base URL override. |
| `LLAMA_CPP_BASE_URL` | unset | llama.cpp base URL override. |
| `LLMETER_TIMEOUT` | `120` | HTTP timeout in seconds. |
| `LLMETER_OUTPUT_DIR` | `<LLMETER_HOME>/benchmark_results` | Output directory. Relative or absolute values are used as provided. |
| `LLMETER_CONFIG_DIR` | `<LLMETER_HOME>/config` | Override the persisted config directory. |
| `LLMETER_RUNS` | `3` | Default benchmark repetitions. |
| `LLMETER_MAX_TOKENS` | `128` | Default output token cap. |
| `LLMETER_TEMPERATURE` | `0.2` | Default sampling temperature. |
| `LLMETER_API_KEY` | unset | Optional ephemeral bearer token for providers that require authentication. It is never persisted to configuration or results. |

CLI flags override environment variables.

Persist the default provider in `<LLMETER_HOME>/config/config.json` unless `LLMETER_CONFIG_DIR` is set:

```bash
llmeter providers set ollama
```

## Troubleshooting

### Provider is not reachable

Check provider and URL:

```bash
llmeter providers list
llmeter --provider openai-compatible --base-url http://localhost:9000/v1 status
```

Start the provider server externally and confirm its `/v1/models` endpoint is available.

### No models found

Verify that the provider has a loaded or exposed model:

```bash
llmeter models
```

For LM Studio, load a model and start the local server. For llama.cpp, start `llama-server` with a model file. For Ollama, pull or create a model before benchmarking.

### Benchmark records show endpoint errors

Not every provider/model supports every OpenAI-compatible capability. `responses-generation`, `structured-output`, `tool-calling`, and `embeddings` may fail independently. These failures are saved as error records in JSON/CSV and shown in reports.

### A saved result cannot be loaded

Confirm that the selected JSON file uses the current schema `3.0` and contains mandatory `schema_version` and `run_kind` fields. Historical result schemas are not upgraded implicitly at runtime.

Last updated: 2026-09-10
