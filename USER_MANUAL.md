# LLMeter User Manual

LLMeter benchmarks local OpenAI-compatible LLM providers through `/v1` APIs. It supports Ollama, LM Studio, llama.cpp, and custom OpenAI-compatible local servers.

LLMeter does not start or stop provider servers. Start your provider externally, then use LLMeter to check status, list models, run benchmarks, and generate reports.

## Installation

Build from source:

```bash
cargo build --release
```

The binary is at `target/release/llmeter` or `target/release/llmeter.exe`.

On Windows PowerShell, the included launcher builds when needed and runs the release binary:

```powershell
.\run_llmeter.ps1 status
.\run_llmeter.ps1 --provider lmstudio bench run --models all --benchmarks all
```

## Provider Setup

| Provider | Default base URL | Notes |
|---|---|---|
| `ollama` | `http://localhost:11434/v1` | Requires Ollama's OpenAI-compatible API. |
| `lmstudio` | `http://localhost:1234/v1` | Start the LM Studio local server and load a model. |
| `llama-cpp` | `http://localhost:8080/v1` | Start `llama-server` from llama.cpp with a model. |
| `openai-compatible` | `http://localhost:8000/v1` | Use with `--base-url` for any compatible local server. |

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

## Built-In Help

Use the regular CLI help:

```bash
llmeter --help
llmeter bench --help
llmeter bench run --help
```

Use LLMeter's built-in help topics:

```bash
llmeter help
llmeter help providers
llmeter help bench
llmeter help reports
llmeter help examples
llmeter /help examples
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
| `llmeter bench run --suite llm --models all --benchmarks all` | Run benchmarks non-interactively. |
| `llmeter report list` | List saved raw results and generated reports. |
| `llmeter report show [result]` | Render a saved JSON result in the terminal. |
| `llmeter report generate [result] --format both` | Generate Markdown and/or HTML reports. |
| `llmeter help [topic]` | Show built-in help for `providers`, `bench`, `reports`, or `examples`. |
| `llmeter /help [topic]` | Alias for built-in help. |

## Running Benchmarks

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

Default output directory: `benchmark_results/`

```text
<run-id>.json
<run-id>.csv
<run-id>.report.md
<run-id>.report.html
```

Reports include provider, base URL, models, benchmark IDs, aggregate latency/throughput, token usage, structured-output validity, tool-call validity, embedding dimensions, detailed records, and errors.

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

## Configuration

| Variable | Default | Description |
|---|---|---|
| `LLMETER_PROVIDER` | `ollama` | Provider preset. |
| `LLMETER_BASE_URL` | provider default | OpenAI-compatible `/v1` base URL. |
| `OLLAMA_HOST` | unset | If set for Ollama, LLMeter maps it to `<OLLAMA_HOST>/v1`. |
| `LMSTUDIO_BASE_URL` | unset | LM Studio base URL override. |
| `LLAMA_CPP_BASE_URL` | unset | llama.cpp base URL override. |
| `LLMETER_TIMEOUT` | `120` | HTTP timeout in seconds. |
| `LLMETER_OUTPUT_DIR` | `benchmark_results` | Output directory. |
| `LLMETER_RUNS` | `3` | Default benchmark repetitions. |
| `LLMETER_MAX_TOKENS` | `128` | Default output token cap. |
| `LLMETER_TEMPERATURE` | `0.2` | Default sampling temperature. |

CLI flags override environment variables.

Persist the default provider in the user config directory:

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

Last updated: 2026-06-15
