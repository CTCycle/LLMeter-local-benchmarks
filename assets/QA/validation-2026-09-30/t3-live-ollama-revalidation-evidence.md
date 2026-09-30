# T3 live Ollama revalidation evidence

Last updated: 2026-09-30

## Boundary

- Final source revision: `1fb3dbd1f3211e25280481f5bbdab439a850a2f9` on `develop`.
- Package / binary: `llmeter 0.4.0`, rebuilt `target/release/llmeter.exe`.
- Host: Windows 11 Pro `10.0.26200`, x86-64; Nitro AN517-41; AMD Ryzen 5 5600H, 12 logical CPUs; 31.36 GB RAM; NVIDIA GeForce RTX 3060 Laptop GPU.
- Provider: Ollama `0.34.0`; endpoint `http://localhost:11434/v1`; five exposed models.
- Models: `qwen3.5:2b` (`324d162be6ca`, 2.7 GB) and `qwen3.5:9b` (`6488c96fa5fa`, 6.6 GB).
- Starting free physical memory was approximately 11.57 GB. No deliberate competing workload was introduced; ambient desktop processes remained.
- No provider, model, container, PATH, startup, or permanent configuration was added by this validation.

Every run used isolated `LLMETER_HOME`, `LLMETER_CONFIG_DIR`, and output roots,
`--timeout 120`, load measurement off, standard telemetry at a 1000 ms sample
interval, `--max-requests 500`, both raw exports, both formatted reports, full
request traces, and response previews disabled. The raw JSON traces were
audited in the isolated root before that root was removed.

## Full default profiles

| Profile | Run ID | Scenarios | Warmup | Measured | Total | Successful | Errors | Result |
|---|---|---:|---:|---:|---:|---:|---:|---|
| `latency` / `qwen3.5:2b` | `2026-09-30T144728.525184Z-p25032-qwen3.5-2b` | 3 | 3 | 15 | 18 | 15 | 0 | PASS |
| `throughput` / `qwen3.5:2b` | `2026-09-30T144905.995869Z-p16524-qwen3.5-2b` | 4 | 4 | 16 | 20 | 16 | 0 | PASS |
| `sweep` / `qwen3.5:2b` | `2026-09-30T145111.909196Z-p32580-qwen3.5-2b` | 27 | 27 | 81 | 108 | 81 | 0 | PASS |

The sweep covered prompt sizes `128,512,2048`, output sizes `64,128,256`,
and concurrency `1,2,4`. It retained 81 successful measured traces and 81
unique request IDs. All profile outputs were schema `3.0` performance results.

## Matched two-model matrix

Run ID: `2026-09-30T145749.738916Z-p5564-qwen3.5-2b-qwen3.5-9b`.

The same non-streaming matrix ran for both models: prompt tokens `32`, output
tokens `16`, concurrency `1,2`, one warmup and 20 measured requests per
scenario, standard telemetry, load measurement off, and the 500-request cap.
The plan contained 4 scenarios, 4 warmups, 80 measured requests, and 84 total
requests. All 4 records and all 80 measured traces succeeded; all 80 request
IDs were unique and every scenario had enough samples for P95 representation.

| Model | Concurrency | Average wall time |
|---|---:|---:|
| `qwen3.5:2b` | 1 | 409.88 ms |
| `qwen3.5:2b` | 2 | 985.77 ms |
| `qwen3.5:9b` | 1 | 3200.84 ms |
| `qwen3.5:9b` | 2 | 6505.08 ms |

Telemetry reported swap warnings for every live run. Maximum swap-used ratios
were 0.624 for latency, 0.941 for throughput, 0.738 for sweep, and 0.958 for
the matched matrix; the matched run also reached a maximum memory-used ratio of
0.845. The functional results and persistence pass, but comparative timing is
not validated on this host and no model ranking is claimed.

## Persistence and privacy

Fresh release-binary `report list` and `report show` processes returned exit
code `0` for all four run directories. The audit found schema `3.0`, run kind
`performance`, 192 successful measured traces, 192 unique request IDs, no
errors, complete non-truncated traces, JSON/CSV/Markdown/HTML outputs, zero
non-null response previews, zero credential-shaped values, and no `.tmp` or
`.partial` files.

## Second-provider boundary

LiteLLM at `http://localhost:4000/v1` was unavailable. Docker `29.8.0` was
installed, but its Linux engine was not reachable at
`npipe:////./pipe/dockerDesktopLinuxEngine`, so no temporary proxy was started
and no provider, model, container, PATH, or permanent configuration was
changed. The earlier Tier 4 LiteLLM-over-Ollama record remains representative
historical coverage and does not replace this current smoke.

## Final-revision hosted boundary

Hosted [CI run `36722865383`](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/36722865383)
passed all four jobs for this exact revision. The native Unix PTY suite passed
4/4 on Ubuntu x86-64, macOS Apple silicon, and macOS Intel; Windows ConPTY
passed 10/10 in the same all-target test workflow.

## Disposition

`T3-04: PASS` for the recorded live Ollama functional boundary.
`T3-05: PARTIAL`: deterministic accounting, the 500-request ceiling and
refusal, full default profiles, the matched functional matrix, persistence,
reload, trace identity, reporting, privacy, and exact-candidate hosted CI
pass. Comparative timing remains unvalidated under swap pressure, and the
optional LiteLLM smoke was unavailable. No universal or cross-host numeric
performance claim is made.
