# Ollama integration

## API endpoints

`OllamaClient` in `ollama/client.rs` talks to the local Ollama API using these endpoints:

| Endpoint | Use |
|---|---|
| `GET /api/version` | Health check and API version. |
| `GET /api/tags` | Installed local model list. |
| `POST /api/show` | Model metadata. |
| `POST /api/generate` | Text generation and benchmark measurements. |
| `GET /api/ps` | Running models (available for future UI expansion). |

The latency benchmark uses streaming generation (`"stream": true` in the POST body) to measure client-side time to first token via nanosecond-precision `std::time::Instant`. The final Ollama response contains the duration fields used for throughput and prompt processing metrics when available.

## Server lifecycle

`OllamaServerManager` in `ollama/server.rs` checks whether the `ollama` executable exists on `PATH`, checks the API health endpoint, and starts `ollama serve` as a detached background process.

When this CLI starts the server, it stores the PID in:

```text
~/.llmeter/ollama-server.pid.json
```

By default, `llmeter server stop` stops only that tracked process. This is safer than killing a server started by another terminal or by the operating system service manager. `--force` can be used for broader process termination.

Last updated: 2026-06-12
