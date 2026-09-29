# Provider integration

LLMeter benchmarks local providers through OpenAI-compatible `/v1` HTTP APIs.

## Supported presets

| Provider | Default base URL |
|---|---|
| `ollama` | `http://localhost:11434/v1` |
| `lmstudio` | `http://localhost:1234/v1` |
| `llama-cpp` | `http://localhost:8080/v1` |
| `openai-compatible` | `http://localhost:8000/v1` |
| `vllm` | `http://localhost:8000/v1` |
| `sglang` | `http://localhost:30000/v1` |
| `localai` | `http://localhost:8080/v1` |
| `litellm` | `http://localhost:4000/v1` |
| `tgi` | `http://localhost:8080/v1` |
| `text-generation-webui` | `http://localhost:5000/v1` |
| `jan` | `http://localhost:1337/v1` |
| `mlx-lm` | `http://localhost:8080/v1` |

`llama-cpp` refers to the llama.cpp project and its `llama-server` OpenAI-compatible API.

Provider presets carry compatibility tiers: first-class, known OpenAI-compatible, best-effort OpenAI-compatible, or custom. The catalog is centralized in `ProviderKind::catalog()` and is used by CLI/UI surfaces.

## API endpoints

`ProviderClient` in `src/providers.rs` uses:

| Endpoint | Use |
|---|---|
| `GET /v1/models` | Provider health check and model listing. |
| `POST /v1/chat/completions` | Streaming and non-streaming chat generation, structured output, and tool calling. |
| `POST /v1/responses` | Responses API benchmark when supported. |
| `POST /v1/embeddings` | Embeddings latency and vector dimension benchmark. |

Client construction accepts only absolute HTTP(S) `/v1` URLs without embedded credentials, queries, or fragments. Requests identify LLMeter with its Cargo-derived user agent, use a separately bounded connection timeout, do not follow redirects, and do not inherit proxy settings. Non-success response bodies are capped before diagnostics are rendered; streaming lines are bounded and multi-line SSE `data:` fields are assembled as one event.

Each `ProviderClient` retains the first validated `/v1/models` response as a command-local catalog cache. Ordinary interactive listing and metadata lookup can reuse it; `refresh_model_catalog()` invalidates and replaces it. Status, the scriptable `models` command, benchmark model validation, load measurement, and capability probing use fresh `/v1/models` requests so operational checks are not satisfied by stale model data.

Optional authentication uses only the `LLMETER_API_KEY` process environment variable. The client converts it to a sensitive in-memory `Authorization: Bearer` header. The value is never written to persisted provider configuration, benchmark configuration, results, reports, progress output, or error diagnostics. Empty values mean no authentication header; invalid header values fail without echoing the secret.

Unsupported provider capabilities are recorded as benchmark error records instead of aborting the entire run.

## Compatibility evidence

Tier 4 uses four separate evidence lanes:

1. The deterministic preset contract matrix derives its provider list from
   `ProviderKind::catalog()` and exercises every registered preset against an
   ephemeral loopback fixture. It covers CLI selection, explicit base-URL
   override, `/v1/models`, streaming and non-streaming Chat Completions,
   request/response parsing, and controlled optional-endpoint failures.
2. The representative live matrix records exact provider implementation,
   version, model, host, base URL, and endpoint capabilities for only the real
   providers that are available for a bounded run.
3. Quality integrations stop at dry-run plan generation; plan evidence does
   not mean that LightEval, Inspect AI, lm-eval-harness, or SWE-bench was
   installed or executed.
4. GitHub release and registry-install evidence are distribution gates,
   independent of provider compatibility.

The fixture matrix proves LLMeter's request construction, SSE parsing, usage
extraction, and diagnostics for all preset names. It does not live-verify every
server version, model, extension, or optional endpoint. The Tier 4 claim is
therefore: LLMeter regression-tests all registered provider presets against its
OpenAI-compatible contract, while live-provider validation is representative
and does not imply universal provider certification.

Performance capability probing checks `/v1/models`, chat completions, optional streaming chat completions, and optionally embeddings and responses. The probe emits per-endpoint progress updates through the shared terminal progress sink so interactive and scriptable runs show visible validation progress before timed scenarios start. Optional endpoint failures are captured in the capability report and do not fail the benchmark by themselves.

## Lifecycle

LLMeter does not start or stop provider servers. Users start Ollama, LM Studio, llama.cpp, or custom local servers externally and pass provider/base URL settings to LLMeter.

Last updated: 2026-09-29
