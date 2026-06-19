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

Unsupported provider capabilities are recorded as benchmark error records instead of aborting the entire run.

Performance capability probing checks `/v1/models`, chat completions, optional streaming chat completions, and optionally embeddings and responses. Optional endpoint failures are captured in the capability report and do not fail the benchmark by themselves.

## Lifecycle

LLMeter does not start or stop provider servers. Users start Ollama, LM Studio, llama.cpp, or custom local servers externally and pass provider/base URL settings to LLMeter.

Last updated: 2026-06-18
