# LLMeter validation ledger — 2026-09-23

Last updated: 2026-09-23

## Campaign metadata

| Field | Value |
|---|---|
| Repository | CTCycle/LLMeter-local-benchmarks |
| Branch | develop |
| Revision | T1-01: `056817717e07fb64c14f13e0b27e83e5ac5b85aa`; T1-02: `46be1d2879fd5da6bd4b4f2f4e4615bd2e6c4f1f`; T1-03: `eeb10fc81b4e7924a9b491be704923db5a92e7c2`; T1-04: `5e900ed2fbe17fc3bbab252251e779b4a23839a7`. |
| Package | llmeter 0.4.0 |
| Environment | Windows x86-64, PowerShell 7.6.6, Cargo 1.98.0, Rust 1.98.0 |
| Build / validation | T1-01: `cargo build --locked --bin llmeter` passed. T1-02: focused PTY suite passed 9/9; manual CLI menu exited 0. T1-03: mock-provider suite passed 11/11. T1-04: format, locked check, warning-denied Clippy, serialized all-target/all-feature tests, and warning-denied rustdoc passed; focused mock-provider suite passed 15/15. |
| Hosted CI | [Run 35861652500](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/35861652500) passed Windows, Ubuntu, macOS Intel, and macOS Apple silicon on `5e900ed2fbe17fc3bbab252251e779b4a23839a7`. This hosted gate is separate from the Windows T1-04 evidence. |
| Provider/model | T1-01: none. T1-02: no live provider or model; isolated Ollama status used the intentionally unavailable loopback URL `http://127.0.0.1:9/v1`. T1-03 and T1-04: deterministic loopback mock only. T1-04 used a synthetic API key. |
| Campaign boundary | Tier 1: application foundations, slices T1-01 through T1-04; Tier 1 remains incomplete. |
| Canonical summary | [Project status ledger](../../docs/project_status_ledger.md) |
| Campaign plan | [Validation campaign](../../docs/runtime/validation_campaign.md) |

## Slice ledger

| Slice | Capability | Exists | Exercised | Status | Scenarios executed | Evidence | Remaining gap | Last validated | Commit/revision |
|---|---|---:|---:|---|---|---|---|---|---|
| T1-01 | Provider, benchmark, quality, and built-in help catalogs | YES | YES | PASS | Fourteen real process invocations covered global `--help`, all three catalogs, overview, all advertised help topics, the `benchmarks` and `lifecycle` aliases, and unsupported `quality` and arbitrary topics. All returned exit 0 with empty stderr. Catalogs listed 12 provider presets, 7 built-in benchmarks, and 11 quality tasks. | [CLI stdout snapshots](tier1-evidence.md) | Unsupported topics return the generic overview with exit 0. T1-05 through T1-07 remain unrun at their dedicated campaign boundaries. | 2026-09-23 | 056817717e07fb64c14f13e0b27e83e5ac5b85aa |
| T1-02 | Main menu, nested workspaces, Back/Escape, EOF, Ctrl+C, and confirmation interruption | YES | YES | PASS | Nine focused Windows ConPTY tests passed, covering interruption, EOF, nested cancel/back, pause cancel/interruption, numeric prompt cancel, and confirmation interruption. The manual route opened Provider setup, Model inventory, Benchmark workspace, Reports, and Help; Escape returned from the workspaces, Help resumed at the menu, and Exit returned code 0. | [T1-02 interactive evidence](t1-02-interactive-evidence.md), [PTY test source](../../../tests/pty_menu_e2e.rs) | Windows-local ConPTY boundary; non-Windows terminal behavior remains unvalidated. T1-05 through T1-07 remain unrun. | 2026-09-23 | 46be1d2879fd5da6bd4b4f2f4e4615bd2e6c4f1f |
| T1-03 | `/v1/models`, cached navigation, fresh operational validation, and missing models | YES | YES | PASS | The 11-test mock-provider E2E suite passed. Cached listing/details shared one request; fresh benchmark validation bypassed a seeded cache; `models --json` returned the mock catalog without stderr; and a missing model returned exit 2 with a `Model not found` diagnostic. | [T1-03 model catalog evidence](t1-03-model-catalog-evidence.md) | Mock-provider boundary only; live provider compatibility remains separate. T1-05 through T1-07 remain unrun. | 2026-09-23 | eeb10fc81b4e7924a9b491be704923db5a92e7c2 |
| T1-04 | HTTP status, redirects, reserved fields, SSE assembly, response limits, auth, and URL safety through the official launcher | YES | YES | PASS | Four Windows-only T1-04 launcher scenarios passed with a disposable official-launcher root and deterministic loopback provider. They covered successful model status and HTTP 201 responses, multiline SSE, redirect rejection, query/credential/file URL rejection before provider requests, 10 MiB JSON and 1 MiB SSE-event limits, captured bearer authorization, bounded 401 diagnostics, and secret inspection across saved JSON/CSV/Markdown/HTML. Reserved-field injection passed at the ProviderClient unit boundary. | [T1-04 provider transport evidence](t1-04-provider-transport-evidence.md) | Windows official-launcher plus deterministic mock boundary only; live-provider compatibility remains separate. T1-05 through T1-07 remain unrun. | 2026-09-23 | 5e900ed2fbe17fc3bbab252251e779b4a23839a7 |

## Slice result

T1-01 through T1-04 are PASS at their captured boundaries. Unsupported help topics, including `quality`, are accepted and produce the generic overview; the snapshots record this observed behavior without treating it as a documented error contract. T1-02's manual route and nine-test ConPTY result are recorded in [the interactive evidence note](t1-02-interactive-evidence.md); T1-03's mock catalog sequence and CLI output/error assertions are recorded in [the model catalog evidence note](t1-03-model-catalog-evidence.md); and T1-04's launcher transport, HTTP, and saved-secret assertions are recorded in [the provider transport evidence note](t1-04-provider-transport-evidence.md). Tier 1 remains incomplete. Continue with T1-05 through T1-07.
