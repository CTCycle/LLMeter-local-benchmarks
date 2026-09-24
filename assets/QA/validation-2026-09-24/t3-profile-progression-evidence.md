# T3-03 performance profile progression and telemetry fixture evidence

Last updated: 2026-09-24

## Scope and boundary

This follow-on slice validates profile planning and execution through the real CLI against the deterministic OpenAI-compatible fixture. It covers `smoke`, `latency`, `throughput`, and `sweep`; it does not claim live-provider performance or production-sized workload behavior.

Environment: Windows x86-64, PowerShell 7.6.6, Cargo 1.98.0, Rust 1.98.0. The performance implementation under test is from `c7438db98812b70c20738ddcd4821d27dab837a3`; this slice adds focused profile-default and CLI E2E regression coverage.

## Profile default planning

`performance_profiles_keep_distinct_default_matrices_with_bounded_request_totals` verified each built-in profile's default prompt/output/concurrency matrix, warmup, run count, and request estimate for one model:

| Profile | Scenarios | Total requests | Result |
|---|---:|---:|---|
| `smoke` | 2 | 8 | PASS |
| `latency` | 3 | 18 | PASS |
| `throughput` | 4 | 20 | PASS |
| `sweep` | 27 | 108 | PASS |

All default totals stayed below the 500-request safety limit.

## Real CLI fixture progression

`performance_profiles_persist_bounded_scenarios_telemetry_and_capabilities` ran each profile through the CLI with a bounded matrix: prompt estimate 1, output size 2, concurrency levels 1 and 2, zero warmup requests, two measured requests per scenario, and standard telemetry at a 100 ms sampling interval.

Each run persisted two scenario rows. Each row retained two successful HTTP 200 traces at its selected concurrency level, with matching request counts. Across the four profiles, 16 measured chat requests were captured. The runs also persisted system telemetry, an environment snapshot, and model inventory metadata. The `smoke` run additionally exercised the complete capability probe: model listing and streaming/non-streaming chat succeeded; the fixture's embeddings and responses endpoints returned controlled unsupported results.

The fixture proves plan-to-request/result accounting and metadata persistence. Its response timing is synthetic and the fixture accepts requests serially, so these results do not prove provider-side parallel throughput or comparative performance.

## Bounded live Ollama progression

After the initial connection refusal, the local Ollama service became available. LLMeter `status` then returned `API reachable: yes` and eight exposed models. The live profile runs used the existing Ollama `qwen3.5:2b` model with one estimated prompt token, eight requested output tokens, concurrency `1,2`, zero warmups, two measured requests per scenario, a four-request safety budget, load measurement off, and standard telemetry at a 1000 ms interval. The `smoke` run also enabled all endpoint probes.

| Profile | Run ID | Scenario records | Request counts | Telemetry samples | Outcome |
|---|---|---:|---|---:|---|
| `smoke` | `2026-09-24T074759.010971Z-p35080-qwen3.5-2b` | 2 | concurrency 1: 2/2 success; concurrency 2: 2/2 success | 2 | PASS |
| `latency` | `2026-09-24T075058.960623Z-p28260-qwen3.5-2b` | 2 | concurrency 1: 2/2 success; concurrency 2: 2/2 success | 2 | PASS |
| `throughput` | `2026-09-24T075114.568708Z-p11580-qwen3.5-2b` | 2 | concurrency 1: 2/2 success; concurrency 2: 2/2 success | 2 | PASS |
| `sweep` | `2026-09-24T075116.770121Z-p25232-qwen3.5-2b` | 2 | concurrency 1: 2/2 success; concurrency 2: 2/2 success | 2 | PASS |

These four runs persisted 16 successful measured requests, model inventory for `qwen3.5:2b`, and environment snapshots. The smoke capability probe reported models, streaming chat, and Responses as HTTP 200. Its first non-streaming chat probe timed out while the model was cold; a subsequent warmed smoke probe, run `2026-09-24T075229.313945Z-p23148-qwen3.5-2b`, returned HTTP 200 for non-streaming chat. Embeddings returned the provider's explicit HTTP 501 unsupported response on both probes. This evidence is specific to the exercised Ollama configuration.

The generated JSON and Markdown/HTML reports were inspected from the QA artifact subdirectory. Local process/host metadata from those volatile outputs is omitted from the committed evidence; the run IDs and outcomes above are the durable summary.

Focused commands passed:

```powershell
cargo test --locked --test performance_cli_tests performance_profiles_keep_distinct_default_matrices_with_bounded_request_totals -- --exact
cargo test --locked --test mock_provider_e2e performance_profiles_persist_bounded_scenarios_telemetry_and_capabilities -- --exact
```

The complete local gate set also passed on this tree: formatting, locked all-target/all-feature check, warning-denied Clippy, serialized all-target/all-feature tests (146 passed), warning-denied rustdoc, and the locked `llmeter` binary build.

## Live-provider gate

The endpoint initially refused connection. After the local service became available, LLMeter status and all four bounded profile runs passed. The first cold non-stream chat probe timed out but succeeded on a warmed repeat; the error was recorded, not treated as proof that the endpoint is unsupported. The earlier Tier 2 live runs and one-request performance smoke remain valid at their recorded scope.

## Final status

The default-plan, deterministic fixture, and bounded one-model Ollama progression boundaries for T3-03 are `PASS`. `benchmark.performance` and Tier 3 remain `PARTIAL`: the live runs used small token sizes and only two measured requests at concurrency 1 and 2. Full default-size workload progression, statistically useful repeated samples, broader model/provider/host coverage, and production-sized interpretation remain open. Do not infer a performance comparison from these runs.
