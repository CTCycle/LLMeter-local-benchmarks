# T4-02 disposable live-provider run metadata

Last updated: 2026-09-29

## Host and toolchain

| Field | Value |
|---|---|
| LLMeter revision | `10d4238ed3053f26bdd358583b3c4cd83e86fd2e` on `develop` |
| Package | `llmeter 0.4.0` |
| Operating system | Windows `10.0.26200.0`, x86-64 |
| Shell | PowerShell `7.6.6` |
| Rust/Cargo | `rustc 1.98.0`, `cargo 1.98.0` |
| Docker Desktop engine | Client/server `29.8.0`, Linux `amd64` |
| Existing backend | Ollama `0.34.0` at `http://localhost:11434/v1` |
| Ollama model reused by LiteLLM | `qwen3.5:2b` |

The LLMeter commands used isolated temporary `LLMETER_HOME` and
`LLMETER_CONFIG_DIR` roots and explicit output directories under the retained
QA artifact tree. No provider executable, model installation, PATH entry, or
startup entry was added to the host.

## llama.cpp runtime

| Field | Value |
|---|---|
| Image | `ghcr.io/ggml-org/llama.cpp:server` |
| Image digest | `sha256:f9115c95639e60abc09d4ea83b26fd4d56c66aa1174594393335a514da00c283` |
| Image/server version | `0.5.0-dev`, build `11243`, commit `fc07d781e61f0d23764394e902b88d26a974e202` |
| Container | `llmeter-t4-02-llama-cpp-20260929` (`079606965cc91ba5059e6cbb3d44320596eeb6319916eb93d5f18cb6d5da53bb`) |
| Endpoint | `http://127.0.0.1:49977/v1` mapped to container port `8080` |
| Model source | `ggml-org/gemma-3-270m-it-GGUF` |
| GGUF file | `gemma-3-270m-it-Q8_0.gguf` |
| GGUF SHA-256 | `0EF57D2C838458A1952664260DCBA38E5BDDA37494F3AF732F06E4ADD24068E3` |
| Fresh model ID | `/models/gemma-3-270m-it-Q8_0.gguf` |
| Performance run | `2026-09-29T133815.469646Z-p29392-models-gemma-3-270m-it-Q8_0.gguf` |
| Streaming run | `2026-09-29T133856.185143Z-p26996-models-gemma-3-270m-it-Q8_0.gguf` |

The one-scenario non-streaming performance smoke completed with one measured
request and no errors. The streaming chat generation completed with one
successful record and reported TTFT. Capability results were models 200, chat
200, streaming chat 200, Responses 200, and controlled embeddings 501.

## LiteLLM runtime

| Field | Value |
|---|---|
| Image | `docker.litellm.ai/berriai/litellm:latest` |
| Image digest | `sha256:bd089afdcd35b894b14a93f9743cdc8b591f82da1a38dd43a010a7b0c9de5fd7` |
| Image revision | `c991f4b01f5799eb0b0cab0fb63988e15c3a8a9d` |
| LiteLLM package | `1.103.0` |
| Container | `llmeter-t4-02-litellm-20260929` (`511052027aeefc9a1682091757993833a1b7dd0d536e1c39238c295a67078b9e`) |
| Endpoint | `http://127.0.0.1:64410/v1` mapped to container port `4000` |
| Proxy model alias | `qwen3.5:2b` |
| Backend route | `ollama_chat/qwen3.5:2b` via `http://host.docker.internal:11434` |
| Performance run | `2026-09-29T134228.139869Z-p24544-qwen3.5-2b` |
| Streaming run | `2026-09-29T134326.966056Z-p22012-qwen3.5-2b` |

The one-scenario non-streaming performance smoke completed with one measured
request and no errors. The streaming chat generation completed with one
successful record and reported output timing. Capability results were models
200, chat 200, streaming chat 200, Responses 200, and controlled embeddings
400 from LiteLLM's unmapped-model response.

An ephemeral synthetic proxy key was injected only through runtime environment
variables. It is intentionally not recorded here. A recursive artifact scan
found no proxy key, API key, authorization header, or Bearer token in the
retained QA outputs.

## Cleanup confirmation

The two task-owned containers and their exact pulled image digests were removed
after capture. The temporary model/config and isolated `LLMETER_HOME` roots
were removed from the host, the two empty staging directories were removed, and
the Docker Desktop process started for this run was stopped. Retained files are
the sanitized JSON/CSV/Markdown/HTML results, command outputs, and sanitized
container metadata under `artifacts/t4-02/`.

## Focused regression boundary

| Suite | Result |
|---|---:|
| `mock_provider_e2e` | 21 passed, 0 failed |
| `provider_probe_tests` | 2 passed, 0 failed |
| `performance_cli_tests` | 7 passed, 0 failed |

## Evidence boundary

This is representative interoperability evidence for the exact provider
images, versions, models, endpoints, and Windows/Docker host recorded above.
It does not certify other provider versions, models, extensions, hosts, or
best-effort presets, and the tiny performance smokes are not statistical
performance certification.

## Source references

- [llama.cpp Docker documentation](https://github.com/ggml-org/llama.cpp/blob/master/docs/docker.md)
- [LiteLLM Docker quick start](https://github.com/BerriAI/litellm-docs/blob/main/docs/proxy/docker_quick_start.md)
- [GGML Gemma GGUF model collection](https://huggingface.co/ggml-org/gemma-3-270m-it-GGUF/tree/main)
