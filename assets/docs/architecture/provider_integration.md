# Provider integration

LLMeter benchmarks local providers through OpenAI-compatible `/v1` HTTP APIs.

## Supported presets

| Provider | Default base URL |
|---|---|
| `ollama` | `http://localhost:11434/v1` |
| `lmstudio` | `http://localhost:1234/v1` |
| `llama-cpp` | `http://localhost:8080/v1` |
| `openai-compatible` | `http://localhost:8000/v1` |

`llama-cpp` refers to the llama.cpp project and its `llama-server` OpenAI-compatible API.

## API endpoints

`ProviderClient` in `src/providers.rs` uses:

| Endpoint | Use |
|---|---|
| `GET /v1/models` | Provider health check and model listing. |
| `POST /v1/chat/completions` | Streaming and non-streaming chat generation, structured output, and tool calling. |
| `POST /v1/responses` | Responses API benchmark when supported. |
| `POST /v1/embeddings` | Embeddings latency and vector dimension benchmark. |

Unsupported provider capabilities are recorded as benchmark error records instead of aborting the entire run.

## Lifecycle

LLMeter does not start or stop provider servers. Users start Ollama, LM Studio, llama.cpp, or custom local servers externally and pass provider/base URL settings to LLMeter.

Last updated: 2026-06-12
