# Tier 4 representative live-provider evidence

Last updated: 2026-09-29

## Boundary and current availability

| Field | Value |
|---|---|
| LLMeter validation revision | `e65948cbc53d2acb9e507d24b3411193a74adcce` (runtime baseline unchanged; evidence-only validation follows) |
| Operating system | Windows x86-64, PowerShell. |
| Current host recheck | 2026-09-29: Ollama `0.34.0` was listening at `http://localhost:11434/v1`; LLMeter status returned API reachable `yes` and five exposed models after the live slice. `llama-server` was not installed. Docker was installed but its Linux engine was not running. |
| Installation/cleanup boundary | No provider binary, model, container, virtual environment, PATH entry, or startup entry was added by this validation. |

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
Provider version: not available
Model: not run; no disposable GGUF model was available
Model identifier returned by `/v1/models`: not run
Operating system: Windows x86-64
Base URL: planned isolated temporary URL; no server was started
LLMeter revision: `e65948cbc53d2acb9e507d24b3411193a74adcce`

| Boundary | Result |
|---|---|
| `status` | `BLOCKED` before execution: `llama-server` was not installed. |
| `models` | `BLOCKED`; no server/model was started or downloaded. |
| Chat/streaming/responses/embeddings | Not run; no live claim. |
| Performance smoke | Not run; no live claim. |
| Expected limitations | Provider installation and model acquisition are external QA setup, not LLMeter runtime behavior. |
| Unexpected failures | None in LLMeter; host lacked the disposable runtime. |
| Evidence boundary | No llama.cpp certification. |
| Cleanup performed | No temporary llama.cpp environment was created. |

## Known OpenAI-compatible implementation record

Selected class: one of vLLM, SGLang, LocalAI, or LiteLLM, chosen only if a
frictionless temporary runtime was available.
Provider/version/model/host: not applicable; no eligible runtime was available.
LLMeter revision: `4fefea734cc62dac327308a518d5409b4f01a033`

`status`, `models`, capability probing, standard generation, and performance
smoke are **BLOCKED** by host availability. Docker's Linux engine was not
running and no local provider installation was introduced. This is not a
functional failure and does not generalize the Ollama result to the known
OpenAI-compatible class.

## Best-effort presets

`tgi`, `text-generation-webui`, `jan`, and `mlx-lm` are **preset contract:
validated; live provider: not certified**. They were intentionally not
installed or downloaded for this slice.
