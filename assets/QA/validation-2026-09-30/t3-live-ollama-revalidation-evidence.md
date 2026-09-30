# T3 live Ollama revalidation evidence

Last updated: 2026-09-30

## Boundary

- Starting source revision: `e0a4723d76c0836eff83b6190ebb575fc5562c2d` on `develop`.
- Candidate revision: to be assigned after the scoped changes are committed.
- Package / binary: `llmeter 0.4.0`, rebuilt `target/release/llmeter.exe`.
- Host: Windows 11 Pro `10.0.26200`, x86-64, Rust `1.98.0`, Cargo `1.98.0`.
- Ollama client: `0.34.0`; endpoint `http://localhost:11434/v1`.
- Current model inventory: five models, including `qwen3.5:2b` and `qwen3.5:9b`.
- No provider, model, container, PATH, startup, or permanent configuration was added by this validation.

The current runs were executed after the scoped performance trace-identity
fix. Every run used isolated `LLMETER_HOME`, `LLMETER_CONFIG_DIR`, and output
roots, `--timeout 120`, load measurement off, standard telemetry at a 1000 ms
sample interval, `--max-requests 500`, both raw exports, both formatted
reports, full request traces, and response previews disabled. The disposable
roots were removed after the artifact audit.

## Full default profiles

| Profile | Run ID | Scenarios | Warmup | Measured | Total | Successful | Errors | Result |
|---|---|---:|---:|---:|---:|---:|---:|---|
| `latency` / `qwen3.5:2b` | `2026-09-30T123739.621475Z-p9424-qwen3.5-2b` | 3 | 3 | 15 | 18 | 15 | 0 | PASS |
| `throughput` / `qwen3.5:2b` | `2026-09-30T123850.947857Z-p31096-qwen3.5-2b` | 4 | 4 | 16 | 20 | 16 | 0 | PASS |
| `sweep` / `qwen3.5:2b` | `2026-09-30T124048.622209Z-p10960-qwen3.5-2b` | 27 | 27 | 81 | 108 | 81 | 0 | PASS |

Warmups were one request per scenario. The sweep covered prompt sizes
`128,512,2048`, output sizes `64,128,256`, and concurrency `1,2,4`.

The fixed sweep retained 81 successful measured traces and 81 unique request
IDs. This matters because the pre-fix request ID omitted output size and
collided across the three output-size branches. The current request ID includes
model, prompt, output size, concurrency, and run index; the deterministic
default-matrix test asserts the 81-ID boundary as well.

## Matched two-model matrix

Run ID: `2026-09-30T124647.076747Z-p17212-qwen3.5-2b-qwen3.5-9b`.

The exact same non-streaming matrix ran for both models: prompt tokens `32`,
output tokens `16`, concurrency `1,2`, one warmup and 20 measured requests per
scenario, standard telemetry, load measurement off, and the 500-request cap.
The plan contained 4 scenarios, 4 warmups, 80 measured requests, and 84 total
requests. All 4 records and all 80 measured traces succeeded, with 80 unique
request IDs and at least 20 observations per scenario for P95 representation.

| Model | Concurrency | Average wall time |
|---|---:|---:|
| `qwen3.5:2b` | 1 | 319.02 ms |
| `qwen3.5:2b` | 2 | 718.78 ms |
| `qwen3.5:9b` | 1 | 3254.10 ms |
| `qwen3.5:9b` | 2 | 7626.93 ms |

The matched run reached approximately 0.95 maximum swap-used ratio and
approximately 0.81 maximum memory ratio. Functional completion and
persistence pass, but comparative timing interpretation is unvalidated and no
model ranking is claimed.

## Persistence and privacy

Fresh release-binary `report list` and `report show` processes returned exit
code `0` for all four current run directories. Each run produced exactly one
JSON, CSV, Markdown, and HTML artifact. The audit found schema `3.0`, run kind
`performance`, zero result errors, successful HTTP 200 traces, complete
non-truncated traces, no response previews, no credential-shaped values, and
no `.tmp` or `.partial` files. The disposable output roots were removed after
the checks, so they are not retained as repository evidence.

## Second-provider boundary

LiteLLM at `http://localhost:4000/v1` was unavailable. Docker was installed,
but its Linux engine was not reachable, so no temporary proxy was started and
no new LiteLLM smoke was claimed. The earlier Tier 4 LiteLLM-over-Ollama
record remains representative historical coverage at its own revision and
does not replace this current-provider check.

## Disposition

`T3-04: PASS` for the recorded repeated live Ollama execution boundary, with
functional current profile and matched-matrix evidence. `T3-05: PARTIAL`:
full current profiles, the matched two-model functional matrix,
persistence/reload, trace identity, and privacy checks pass; comparative timing
is disqualified by swap pressure, LiteLLM is unavailable, and exact-candidate
four-platform hosted CI remains open.
