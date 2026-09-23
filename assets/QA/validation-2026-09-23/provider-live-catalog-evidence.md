# Live Ollama status and model catalog smoke

Last updated: 2026-09-23

## Validation boundary

| Field | Value |
|---|---|
| Repository | CTCycle/LLMeter-local-benchmarks |
| Branch | `develop` |
| Environment | Windows x86-64; Cargo 1.98.0; Rust 1.98.0 |
| Provider | Ollama at `http://localhost:11434/v1` |
| Scope | Current live provider status and fresh model discovery through the LLMeter CLI. No benchmark request was sent. |

## Commands and results

| Command | Result |
|---|---|
| `target\debug\llmeter.exe --provider ollama status` | Exit 0; API reachable; 8 models exposed. |
| `target\debug\llmeter.exe --provider ollama models --json` | Exit 0; returned the same 8 model IDs as JSON. |

The catalog included `qwen3.5:2b` and `nomic-embed-text:latest`, which are available for a future bounded live benchmark slice.

## Remaining boundary

This confirms current status and model listing for one Ollama endpoint. It does not validate chat, responses, embeddings, optional capabilities, best-effort provider presets, performance workloads, or cross-platform behavior. `provider.presets` remains `PARTIAL`, `provider.best-effort.live` remains `UNVALIDATED`, and cross-provider optional-capability debt remains open.
