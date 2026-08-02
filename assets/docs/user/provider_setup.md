# Provider setup

## Supported presets

LLMeter supports these provider presets:

- `ollama`
- `lmstudio`
- `llama-cpp`
- `openai-compatible`
- `vllm`
- `sglang`
- `localai`
- `litellm`
- `tgi`
- `text-generation-webui`
- `jan`
- `mlx-lm`

Default base URLs:

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

The provider list includes compatibility tiers. Ollama, LM Studio, and llama.cpp are first-class local targets. vLLM, SGLang, LocalAI, and LiteLLM are known OpenAI-compatible presets. TGI, text-generation-webui, Jan, and MLX-LM are best-effort because endpoint shape can vary by configuration or version.

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

`status` and the scriptable `models` command perform fresh `/v1/models` requests. Interactive **List exposed models** may use the current client-local cache; **Refresh exposed models** invalidates it first.

Probe provider capabilities before a performance benchmark:

```bash
llmeter --provider ollama bench perf --models all --profile smoke --probe-capabilities --runs 1 --export none --report none
```

In interactive mode, use **Provider setup** then **Probe provider capabilities**.

For authenticated local endpoints, set `LLMETER_API_KEY` in the environment before launching LLMeter. The bearer value is held only by the active HTTP client and is never persisted or printed.

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

Last updated: 2026-08-02
