# Tier 3 release-performance evidence

Last updated: 2026-09-30

## Revision and environment

- Repository: `CTCycle/LLMeter-local-benchmarks`
- Final candidate revision: `1fb3dbd1f3211e25280481f5bbdab439a850a2f9` on `develop`.
- Package: `llmeter 0.4.0`
- Host: Windows 11 Pro `10.0.26200`, x86-64; Nitro AN517-41; AMD Ryzen 5 5600H, 12 logical CPUs; 31.36 GB RAM; NVIDIA GeForce RTX 3060 Laptop GPU.
- Rust / Cargo: `1.98.0` / `1.98.0`
- Provider: Ollama `0.34.0`; endpoint `http://localhost:11434/v1`; five exposed models.

All live runs used an isolated home, configuration directory, and output root,
`--timeout 120`, load measurement off, standard telemetry, a 1000 ms sample
interval, `--max-requests 500`, JSON and CSV exports, Markdown and HTML
reports, full request traces, and no response previews. No provider lifecycle
or model-download configuration was added.

## Deterministic release ceiling

The final-revision focused mock-provider suite passed 23/23. The ceiling case
executed exactly 500 planned requests with 25 scenarios and complete persisted
measured accounting. The adjacent 525-request matrix was rejected before any
performance chat request and wrote no result artifact. Schema `3.0`, unique
request IDs, report reload, default privacy, and temporary-file cleanup passed.

The final focused regressions also passed:

- `performance_cli_tests`: 7/7;
- `performance_metrics_tests`: 8/8;
- `resilience_e2e`: 1/1;
- `pty_menu_e2e`: 10/10.

The serialized `cargo test --locked --all-targets --all-features --
--test-threads=1` suite passed 156 tests with zero failures.

## Final live default profiles

| Profile / model | Run ID | Scenarios | Warmup | Measured | Total | Errors | Result |
|---|---|---:|---:|---:|---:|---:|---|
| `latency` / `qwen3.5:2b` | `2026-09-30T144728.525184Z-p25032-qwen3.5-2b` | 3 | 3 | 15 | 18 | 0 | PASS |
| `throughput` / `qwen3.5:2b` | `2026-09-30T144905.995869Z-p16524-qwen3.5-2b` | 4 | 4 | 16 | 20 | 0 | PASS |
| `sweep` / `qwen3.5:2b` | `2026-09-30T145111.909196Z-p32580-qwen3.5-2b` | 27 | 27 | 81 | 108 | 0 | PASS |

The sweep retained 81 successful measured traces and 81 unique request IDs.

## Matched two-model matrix

Run ID: `2026-09-30T145749.738916Z-p5564-qwen3.5-2b-qwen3.5-9b`.

The identical non-streaming matrix used prompt tokens `32`, output tokens
`16`, concurrency `1,2`, one warmup, 20 measured requests per scenario,
standard telemetry, load measurement off, and the 500-request cap. It produced
4 scenarios, 4 warmups, 80 measured requests, 84 total requests, 4 successful
records, 80 successful traces, and 80 unique request IDs.

| Model | Concurrency | Average wall time |
|---|---:|---:|
| `qwen3.5:2b` | 1 | 409.88 ms |
| `qwen3.5:2b` | 2 | 985.77 ms |
| `qwen3.5:9b` | 1 | 3200.84 ms |
| `qwen3.5:9b` | 2 | 6505.08 ms |

Every live run emitted the telemetry warning that swap-used ratio exceeded
0.20. Maximum swap-used ratios were 0.624, 0.941, 0.738, and 0.958 for the
latency, throughput, sweep, and matched runs respectively. The matched run
reached a maximum memory-used ratio of 0.845. These runs therefore qualify
functional completion and accounting, but not comparative timing or model
ranking.

## Persistence, reporting, and privacy

Release-binary `report list` and `report show` returned exit code `0` for all
four run directories. The audited outputs contained schema `3.0`, run kind
`performance`, 192 successful measured traces, 192 unique request IDs,
complete non-truncated traces, JSON/CSV/Markdown/HTML artifacts, zero non-null
response previews, zero credential-shaped values, and no `.tmp` or `.partial`
files. The isolated raw-output root was removed after the audit.

## Second-provider and hosted boundaries

LiteLLM at `http://localhost:4000/v1` was unavailable. Docker `29.8.0` was
installed, but its Linux engine was unreachable at
`npipe:////./pipe/dockerDesktopLinuxEngine`; no temporary proxy or permanent
configuration was created. The previous Tier 4 LiteLLM result remains
historical representative evidence only.

Hosted [CI run `36722865383`](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/36722865383)
passed all four jobs for the exact candidate. Native Unix PTY passed 4/4 on
Ubuntu x86-64, macOS Apple silicon, and macOS Intel; Windows ConPTY passed
10/10. The run also passed formatting/check, Clippy, serialized tests, and
rustdoc workflow steps.

## Disposition

`T3-05: PARTIAL`. Deterministic accounting, the 500-request ceiling and
refusal, full current Ollama profiles, the matched two-model functional matrix,
persistence, reload, trace identity, reporting, privacy, and exact-candidate
four-platform CI pass. Comparative timing remains unvalidated under swap
pressure and the optional LiteLLM smoke was unavailable. No universal,
cross-host, or hardware-independent numeric performance claim is made.
