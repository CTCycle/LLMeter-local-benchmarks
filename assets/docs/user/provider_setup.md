# Provider setup

## Supported presets

LLMeter supports these provider presets:

- `ollama`
- `lmstudio`
- `llama-cpp`
- `openai-compatible`

Default base URLs:

| Provider | Default base URL |
|---|---|
| `ollama` | `http://localhost:11434/v1` |
| `lmstudio` | `http://localhost:1234/v1` |
| `llama-cpp` | `http://localhost:8080/v1` |
| `openai-compatible` | `http://localhost:8000/v1` |

## General rule

Start the provider server first, then point LLMeter at the provider. LLMeter does not launch provider processes or load models on your behalf.

## Basic checks

List provider presets:

```bash
llmeter providers list
```

Check provider reachability:

```bash
llmeter --provider ollama status
```

List exposed models:

```bash
llmeter --provider ollama models
```

## Custom base URL

If the provider is running on a different host or port:

```bash
llmeter --provider openai-compatible --base-url http://localhost:9000/v1 status
```

LLMeter accepts a base URL with or without the `/v1` suffix and normalizes it internally.

## Provider-specific notes

### Ollama

- Ensure the Ollama service is running.
- Ensure the target model has been pulled or created.
- `OLLAMA_HOST` can be used as an environment compatibility shortcut.

### LM Studio

- Start the local server in LM Studio.
- Load or expose a model before benchmarking.

### llama.cpp

- Start `llama-server` with a model file and OpenAI-compatible server mode.
- Confirm the server exposes `/v1/models`, `/v1/chat/completions`, and any extra endpoints needed by selected benchmarks.

### Custom OpenAI-compatible server

- Ensure the server is local and exposes the required `/v1` endpoints.
- Capability coverage varies by implementation, so some benchmark types may report per-record errors.

Last updated: 2026-06-15
