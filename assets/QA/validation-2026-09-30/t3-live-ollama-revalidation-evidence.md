# T3 live Ollama revalidation evidence

Last updated: 2026-09-30

## Boundary

- Base checkout revision: `922da56ebf96834d905d918400f2dee0ea3c3491`; candidate remains uncommitted.
- Package / binary: `llmeter 0.4.0`, existing `target/release/llmeter.exe`.
- Host: Windows x86-64, Rust `1.98.0`, Cargo `1.98.0`.
- Provider: Ollama `0.34.0`, `http://localhost:11434/v1`.
- Fresh status: API reachable `yes`; five models exposed, including `qwen3.5:2b` and `qwen3.5:9b`.
- No provider, model, container, PATH, startup, or permanent configuration was added by this validation.

All authoritative runs used isolated `LLMETER_HOME`, `LLMETER_CONFIG_DIR`, and
`--output-dir` roots, the documented `--timeout 120`,
`--load-measurement off`, standard telemetry at a 1000 ms sample interval,
`--max-requests 500`, both raw exports, both formatted reports, and full trace
detail. Response previews were not requested.

## Full default profiles

| Profile | Run ID | Scenarios | Warmup | Measured | Successful | Errors | Result |
|---|---|---:|---:|---:|---:|---:|---|
| `latency` / `qwen3.5:2b` | `2026-09-30T081918.314568Z-p29856-qwen3.5-2b` | 3 | 3 | 15 | 15 | 0 | PASS |
| `throughput` / `qwen3.5:2b` | `2026-09-30T082243.510489Z-p7184-qwen3.5-2b` | 4 | 4 | 16 | 16 | 0 | PASS |
| `sweep` / `qwen3.5:2b` | `2026-09-30T083252.238636Z-p4948-qwen3.5-2b` | 27 | 27 | 81 | 81 | 0 | PASS |

Warmup counts are one request per scenario. The sweep covered prompt sizes
`128,512,2048`, output sizes `64,128,256`, and concurrency `1,2,4`.

## Capability smoke

The current `qwen3.5:2b` capability probe (`2026-09-30T090231.167943Z-p14928-qwen3.5-2b`)
passed the real CLI smoke and recorded:

- `/v1/models`: supported, HTTP 200;
- non-streaming chat: supported, HTTP 200;
- streaming chat: supported, HTTP 200;
- `/v1/responses`: supported, HTTP 200;
- `/v1/embeddings`: unsupported, controlled HTTP 501 because Ollama was not
  started with embeddings enabled.

## Matched two-model matrix

Run ID: `2026-09-30T084529.645691Z-p19796-qwen3.5-2b-qwen3.5-9b`.

The exact same non-streaming matrix ran for both `qwen3.5:2b` and
`qwen3.5:9b`: prompt `32`, output `16`, concurrency `1,2`, one warmup and
20 measured requests per scenario, standard telemetry, load measurement off,
and the 500-request cap. The plan contained 4 scenarios, 4 warmups, 80
measured requests, and 84 total requests. All 4 records and all 80 measured
traces succeeded; every scenario has enough samples for P95 representation.

The saved schema `3.0` result reported these scenario averages:

| Model | Concurrency | Average wall time |
|---|---:|---:|
| `qwen3.5:2b` | 1 | 965.47 ms |
| `qwen3.5:2b` | 2 | 1389.03 ms |
| `qwen3.5:9b` | 1 | 4717.02 ms |
| `qwen3.5:9b` | 2 | 9133.58 ms |

These values are host-specific observations only. Every telemetry run emitted
the documented swap-pressure warning; the matched run reached approximately
0.98 maximum swap-used ratio. Functional completion and persistence pass, but
comparative timing interpretation is disqualified by host pressure.

## Persistence and privacy

Fresh release-binary processes ran `report list` and `report show` for each
of the four saved run directories. All returned exit code `0`. Each run
produced JSON, CSV, Markdown, and HTML outputs. The checks found:

- schema `3.0`, run kind `performance`, and zero result errors;
- complete expected measured trace counts and no temporary files;
- `response_previews_included: false` and zero non-null response previews;
- no credential-shaped values in the four output formats.

## Remaining live/hosted boundary

The optional LiteLLM-over-Ollama path was checked without starting a proxy:
`http://localhost:4000/v1` was unreachable, so no second-provider claim is
made. Exact-candidate four-platform hosted CI is also still pending because
the working tree is uncommitted.

## Timeout qualification

A preliminary run with an explicit 30-second timeout recorded two controlled
timeouts on the same default latency matrix. The authoritative retry used the
documented 120-second timeout and completed all 15 measured requests with zero
errors. This is recorded as host/runtime timing context, not as a product
failure.

## Disposition

`T3-05: PARTIAL`. Current Ollama status, full default latency/throughput/sweep
profiles, the two-model matched functional matrix, persistence/reload, and
privacy boundaries pass. LiteLLM availability, timing interpretation under
swap pressure, and exact-candidate hosted CI remain open.
