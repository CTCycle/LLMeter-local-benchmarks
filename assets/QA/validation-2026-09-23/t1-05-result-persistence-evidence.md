# T1-05 result persistence validation

Last updated: 2026-09-23

## Validation boundary

| Field | Value |
|---|---|
| Repository | CTCycle/LLMeter-local-benchmarks |
| Branch and tested revision | `develop`, `0f2ea407668a243ff037f84f10cdb459bf1c2edd` |
| Package | llmeter 0.4.0 |
| Environment | Windows 11 Pro x86-64, build 26200; PowerShell 7.6.6; Cargo 1.98.0; Rust 1.98.0 |
| Provider/model | Deterministic loopback mock only; no live provider or model was contacted. |
| Scope | Schema 3.0, JSON/CSV output, run IDs, preview policy, redaction, CSV formula neutralization, and atomic replacement/failure cleanup. |

## Local validation

Formatting and all focused tests passed on the tested revision:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | Passed |
| `cargo test --locked --test test_results -- --test-threads=1` | 6 passed |
| `cargo test --locked --test result_schema_tests -- --test-threads=1` | 3 passed |
| `cargo test --locked --lib atomic_write -- --test-threads=1` | 2 passed |
| `cargo test --locked --lib results::tests -- --test-threads=1` | 4 passed |
| `cargo test --locked --test mock_provider_e2e cli_bench_run_streams_and_generates_report_from_saved_json -- --test-threads=1` | 1 passed |
| `cargo test --locked --test mock_provider_e2e t1_04_official_launcher_captures_multiline_sse_and_accepts_created_responses -- --test-threads=1` | 1 passed |
| `cargo test --locked --test mock_provider_e2e t1_04_official_launcher_auth_is_captured_and_secrets_are_not_persisted -- --test-threads=1` | 1 passed |

Total: 18 focused tests passed, 0 failed.

## Scenarios and results

| Scenario | Evidence | Result |
|---|---|---|
| Real CLI benchmark generated result files against the loopback mock with the default privacy policy. | `cli_bench_run_streams_and_generates_report_from_saved_json` | Saved JSON used schema `3.0`; its run ID matched the JSON filename; configuration recorded previews excluded and redaction applied; the response preview was absent; CSV existed and did not contain the mock response text. JSON, CSV, Markdown, and HTML outputs were generated. |
| Explicit preview opt-in through the official Windows launcher. | `t1_04_official_launcher_captures_multiline_sse_and_accepts_created_responses` | Saved chat and responses results contained the expected preview text when `--include-response-preview` was supplied. |
| Secret redaction in persisted output. | `t1_04_official_launcher_auth_is_captured_and_secrets_are_not_persisted` | The mock captured the synthetic bearer key; the saved error contained truncation and redaction markers. JSON, CSV, Markdown, and HTML files were inspected, with no raw key found. |
| Schema identity and run IDs. | `result_schema_tests`; `results::tests` | New runs serialized the shared schema version; unversioned and schema `2.4` files were rejected; IDs differed for the same timestamp/model set across process IDs. |
| CSV text and output privacy. | `test_results`; `results::tests` | Formula-like text values were prefixed safely. Default preview omission and secret redaction were verified; explicit opt-in and preservation of non-secret token counts were also covered. |
| Atomic replacement and failure cleanup. | `utils::tests::atomic_write_replaces_complete_content_without_temp_files`; `utils::tests::atomic_write_cleans_temporary_file_when_final_rename_fails` | Replacement left the complete new content and only the destination file. A failed final rename left the occupied destination and no temporary residue. |

No runtime defect was found. The T1-05 test-only change adds real-CLI assertions for default preview omission, persisted privacy metadata, CSV response omission, and run-ID/filename identity. T1-04 remains a separate provider-transport evidence boundary.
