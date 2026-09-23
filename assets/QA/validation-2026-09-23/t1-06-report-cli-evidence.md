# T1-06 report CLI validation

Last updated: 2026-09-23

## Validation boundary

| Field | Value |
|---|---|
| Repository | CTCycle/LLMeter-local-benchmarks |
| Branch | `develop` |
| Package | llmeter 0.4.0 |
| Environment | Windows x86-64; Cargo 1.98.0; Rust 1.98.0 |
| Provider/model | Synthetic schema 3.0 fixtures only; the T1-06 report scenario made no provider calls. A separate live Ollama status/catalog probe is recorded in [provider live evidence](provider-live-catalog-evidence.md). |
| Scope | Real CLI `report list`, `report show`, and `report generate` flows for standard, performance, error, and adversarial values. |

## Local validation

| Command | Result |
|---|---|
| `cargo test --locked --all-features --test report_cli_e2e -- --test-threads=1` | 1 passed |
| `cargo test --locked --all-targets --all-features -- --test-threads=1` | Passed; includes the report CLI scenario and existing report, result-store, schema, and mock-provider regressions. |

The fixtures and output directory were isolated under a temporary `assets/QA/` directory and removed by the test harness.

## Scenarios and results

| Scenario | Evidence | Result |
|---|---|---|
| List saved JSON results before and generated reports afterward. | `report_cli_lists_shows_and_generates_standard_performance_and_adversarial_runs` | Both schema 3.0 fixtures appeared in `report list`; after generation, all four Markdown and HTML reports appeared in the report list. |
| Show standard and performance results in the terminal. | Same real-CLI scenario | Both invocations returned exit 0 and displayed the expected run IDs. The standard report retained its controlled error row. |
| Generate Markdown and HTML for standard and performance results. | Same real-CLI scenario | All four files were created. Performance output included its performance summary and timing model sections. |
| Render errors and hostile user text safely. | Same real-CLI scenario | Error text remained visible; Markdown table separators were escaped, and HTML rendered script-shaped text as escaped content. |
| Apply output privacy while generating reports. | Same real-CLI scenario | The synthetic API key and response-preview sentinels were absent from generated Markdown and HTML. |

No application defect was found in this slice. The schema loader correctly rejected the initial fixture until it included required benchmark metadata and a registered standard benchmark ID; the final fixture represents valid current-schema input.

## Remaining boundary

This slice validates report handling with deterministic saved results. It does not validate benchmark execution, production performance workloads, or provider-specific optional capabilities. Those boundaries remain separately represented in the project ledger; the report fixture does not clear `benchmark.performance` validation debt.
