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

Each `ProviderClient` retains the first validated `/v1/models` response as an immutable command-local catalog snapshot. Model selection, lookup, and capability probing reuse that snapshot rather than repeatedly contacting the provider during a single invocation.

Optional authentication uses only the `LLMETER_API_KEY` process environment variable. The client converts it to a sensitive in-memory `Authorization: Bearer` header. The value is never written to persisted provider configuration, benchmark configuration, results, reports, progress output, or error diagnostics. Empty values mean no authentication header; invalid header values fail without echoing the secret.

Unsupported provider capabilities are recorded as benchmark error records instead of aborting the entire run.

## Compatibility evidence

The test suite runs the baseline `/v1/models` and streaming Chat Completions contract against every registered preset through deterministic fixtures. This proves LLMeter's request construction, SSE parsing, usage extraction, and diagnostics for all preset names without implying that every server version or model is live-verified. Responses and embeddings remain capability-probed because provider deployments vary.

Performance capability probing checks `/v1/models`, chat completions, optional streaming chat completions, and optionally embeddings and responses. The probe emits per-endpoint progress updates through the shared terminal progress sink so interactive and scriptable runs show visible validation progress before timed scenarios start. Optional endpoint failures are captured in the capability report and do not fail the benchmark by themselves.

## Lifecycle

LLMeter does not start or stop provider servers. Users start Ollama, LM Studio, llama.cpp, or custom local servers externally and pass provider/base URL settings to LLMeter.

Last updated: 2026-07-18
