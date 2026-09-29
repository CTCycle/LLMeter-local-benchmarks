# Tier 4 representative live-provider evidence

Last updated: 2026-09-29

## Boundary and current availability

| Field | Value |
|---|---|
| LLMeter validation revision | `10d4238ed3053f26bdd358583b3c4cd83e86fd2e` on `develop` |
| Operating system | Windows `10.0.26200.0` x86-64, PowerShell 7. |
| Current host recheck | Ollama `0.34.0` was listening at `http://localhost:11434/v1`; Docker Desktop's Linux engine was available at client/server `29.8.0` (`linux/amd64`) for the disposable runtimes. |
| Installation/cleanup boundary | No provider executable, host model installation, PATH entry, virtual environment, startup entry, or permanent provider configuration was added. Task-owned containers, images, temporary model/config files, and the Docker Desktop process were removed after capture. |

## Ollama record

Provider: Ollama
Compatibility class: first-class
Provider version: `0.34.0`
Model: `qwen3.5:2b` for generation; `nomic-embed-text:latest` for embeddings
Model identifier returned by `/v1/models`: exact IDs above in the prior live catalog
Operating system: Windows x86-64
Base URL: `http://localhost:11434/v1`
LLMeter revision: `c7438db98812b70c20738ddcd4821d27dab837a3` for the standard suite; `e635459a402414ee880bde42be0efaff0a245acf` for the repeated profile subset

| Boundary | Result |
|---|---|
| `status` | Current recheck passed with API reachable and five exposed models. |
| `models` | Current `models --json` passed and returned five models, including `qwen3.5:2b` and `nomic-embed-text:latest`. |
| Chat Completions | Prior standard suite passed all six LLM benchmark paths with 8/8 records on `qwen3.5:2b`; repeated profile traces returned 24/24 HTTP 200. |
| Streaming | Passed in the standard chat-generation and performance paths. |
| `/v1/responses` | Passed for the selected generation model in the 2026-09-24 standard suite. |
| `/v1/embeddings` | Passed 1/1 for `nomic-embed-text:latest`; the `qwen3.5:2b` capability probe returned HTTP 501, recorded as unsupported. |
| Performance smoke | The current default `latency` and `throughput` profiles passed with 15/15 and 16/16 successful measured traces; the capability-probe smoke also passed its one measured request. See [T3/T4 live default profile evidence](t3-t4-live-default-profiles-evidence.md). |
| Expected limitations | Model-specific unsupported embeddings and provider/model capability variation. |
| Unexpected failures | No request or persistence failure in the current slice. Host swap-pressure warnings limit timing interpretation; the earlier cold non-stream timeout remains historical evidence. |
| Evidence boundary | Exact Ollama/model evidence only; no claim for other presets, models, versions, or hosts. Full sweep, statistical interpretation, and broader live variance remain open. |
| Cleanup performed | No Ollama service was started by this validation; no files or installations were added. |

Source evidence: [Tier 2 standard workflows](../validation-2026-09-24/t2-standard-workflows-evidence.md), [repeated live profiles](../validation-2026-09-26/t3-t5-followup-evidence.md), and [release validation](../release-0.4.0/release-report.md).

## llama.cpp record

Provider: llama.cpp
Compatibility class: first-class
Provider version: `0.5.0-dev`, build `11243`, commit `fc07d781e61f0d23764394e902b88d26a974e202`
Container image: `ghcr.io/ggml-org/llama.cpp:server@sha256:f9115c95639e60abc09d4ea83b26fd4d56c66aa1174594393335a514da00c283`
Model: `ggml-org/gemma-3-270m-it-GGUF/gemma-3-270m-it-Q8_0.gguf`, SHA-256 `0EF57D2C838458A1952664260DCBA38E5BDDA37494F3AF732F06E4ADD24068E3`
Model identifier returned by `/v1/models`: `/models/gemma-3-270m-it-Q8_0.gguf`
Operating system: Windows x86-64
Base URL: `http://127.0.0.1:49977/v1` mapped to container port `8080`
LLMeter revision: `10d4238ed3053f26bdd358583b3c4cd83e86fd2e`

| Boundary | Result |
|---|---|
| `status` | `PASS`; API reachable and the temporary server reported healthy. |
| Fresh `models --json` | `PASS`; one exact GGUF model was returned. |
| Non-streaming generation | `PASS` through the one-measured-request performance smoke. |
| Streaming `chat-generation` | `PASS`; one successful record with TTFT. |
| Capability probes | Models, chat, streaming chat, and `/v1/responses` returned HTTP 200; `/v1/embeddings` returned controlled HTTP 501 and is recorded as unsupported. |
| Performance smoke | `PASS`; one scenario, one warmup-free measured request, one success, zero errors; load measurement and telemetry were disabled. |
| Expected limitations | This certifies the exact image, server build, GGUF, endpoint, and Windows/Docker host recorded here only. |
| Unexpected failures | None in the executed boundary. |
| Evidence boundary | Representative llama.cpp interoperability only; no claim for other llama.cpp versions, models, hosts, or provider presets. |
| Cleanup performed | Task-owned container, image, model directory, and isolated LLMeter roots were removed after evidence capture. |

Run artifacts: [run metadata](t4-02-run-metadata.md), [status](artifacts/t4-02/llama-cpp/status), [fresh models](artifacts/t4-02/llama-cpp/models), [streaming result](artifacts/t4-02/llama-cpp/stream), and [performance smoke](artifacts/t4-02/llama-cpp/perf-smoke).

## LiteLLM record

Selected class: known OpenAI-compatible gateway, LiteLLM.
Provider version: package `1.103.0`, image revision `c991f4b01f5799eb0b0cab0fb63988e15c3a8a9d`
Container image: `docker.litellm.ai/berriai/litellm:latest@sha256:bd089afdcd35b894b14a93f9743cdc8b591f82da1a38dd43a010a7b0c9de5fd7`
Model alias: `qwen3.5:2b`
Backend: `ollama_chat/qwen3.5:2b` through `http://host.docker.internal:11434`; existing Ollama `0.34.0` was reused, not recertified as another implementation.
Operating system: Windows x86-64 with Docker Desktop Linux engine
Base URL: `http://127.0.0.1:64410/v1` mapped to proxy port `4000`
LLMeter revision: `10d4238ed3053f26bdd358583b3c4cd83e86fd2e`

| Boundary | Result |
|---|---|
| `status` | `PASS`; the temporary proxy reported healthy. |
| Fresh `models --json` | `PASS`; the configured alias `qwen3.5:2b` was returned. |
| Non-streaming generation | `PASS` through the one-measured-request performance smoke. |
| Streaming `chat-generation` | `PASS`; one successful record with output timing. |
| Capability probes | Models, chat, streaming chat, and `/v1/responses` returned HTTP 200; `/v1/embeddings` returned controlled HTTP 400 for the unmapped capability and is recorded as unsupported for this alias. |
| Performance smoke | `PASS`; one scenario, one warmup-free measured request, one success, zero errors; load measurement and telemetry were disabled. |
| Expected limitations | This certifies the exact LiteLLM image/package, alias, existing Ollama backend, endpoint, and Windows/Docker host recorded here only. |
| Unexpected failures | None in the executed boundary. |
| Evidence boundary | Representative LiteLLM gateway interoperability only; no claim for other gateways, routes, aliases, provider presets, or models. |
| Cleanup performed | Task-owned container, image, proxy config, and isolated LLMeter roots were removed; the synthetic proxy key was process-only and was not retained. |

Run artifacts: [run metadata](t4-02-run-metadata.md), [status](artifacts/t4-02/litellm/status), [fresh models](artifacts/t4-02/litellm/models), [streaming result](artifacts/t4-02/litellm/stream), and [performance smoke](artifacts/t4-02/litellm/perf-smoke).

The LiteLLM result validates a distinct gateway implementation over the existing
Ollama backend. It does not add a second Ollama certification and does not
generalize to every OpenAI-compatible implementation.

## Best-effort presets

`tgi`, `text-generation-webui`, `jan`, and `mlx-lm` are **preset contract:
validated; live provider: not certified**. They were intentionally not
installed or downloaded for this slice.
