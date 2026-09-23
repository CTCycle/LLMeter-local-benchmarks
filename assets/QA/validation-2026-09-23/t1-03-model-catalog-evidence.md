# T1-03 model catalog freshness evidence

Last updated: 2026-09-23

## Validation boundary

| Field | Value |
|---|---|
| Repository | CTCycle/LLMeter-local-benchmarks |
| Branch and revision | `develop`, `eeb10fc81b4e7924a9b491be704923db5a92e7c2` |
| Package | llmeter 0.4.0 |
| Environment | Windows x86-64, PowerShell 7.6.6, Cargo 1.98.0, Rust 1.98.0 |
| Command | `cargo test --locked --test mock_provider_e2e` |
| Result | 11 passed; 0 failed; 0 ignored; 0 filtered out |
| Provider/model | Deterministic loopback mock only; no live provider or model contacted |
| Hosted CI | [Run 35840649779](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/35840649779) passed the Windows, Ubuntu, macOS Intel, and macOS Apple silicon jobs at this revision. |

## Scenarios and evidence

The test executable captures CLI stdout and stderr and inspects the mock server's request paths. The observations below summarize the assertions from the passing run.

| Scenario | Test | Observed result |
|---|---|---|
| Cached catalog reuse and explicit refresh | `provider_client_reuses_a_model_catalog_snapshot` | `model_names()` followed by `show_model("mock-model")` made one `GET /v1/models`. A fresh read, explicit refresh, invalidation, and subsequent cached read brought the total to four requests. |
| Fresh benchmark validation despite a seeded cache | `benchmark_model_validation_bypasses_cached_catalog` | After seeding the cache, `validate_models()` returned `mock-model` and issued a second `GET /v1/models`. |
| Operational request order | `performance_load_estimate_runs_before_capability_chat_probes` | The first load chat request followed cached model selection, fresh model validation, and fresh provider status; the warm load request followed immediately. |
| Scriptable model listing | `cli_models_json_reads_mock_provider_catalog` | `llmeter models --json` exited successfully; captured stdout parsed as JSON with `mock-model` as the first model ID, stderr was empty, and `/v1/models` was requested. |
| Missing model error | `cli_handles_unavailable_provider_and_missing_model` | `llmeter show missing-model` returned exit code 2 and stderr containing `Model not found`. The same test confirms unavailable-provider status returns exit code 1. |

This is fixture evidence for the stated request, output, and error boundaries. It does not certify any live provider or model.
