# LLMeter validation ledger — 2026-09-23

Last updated: 2026-09-23

## Campaign metadata

| Field | Value |
|---|---|
| Repository | CTCycle/LLMeter-local-benchmarks |
| Branch | develop |
| Revision | 056817717e07fb64c14f13e0b27e83e5ac5b85aa; documentation-only commits beyond source revision e21dd0ec271a9a1e009bf257771715d948c6cf92 |
| Package | llmeter 0.4.0 |
| Environment | Windows x86-64, PowerShell 7.6.6, Cargo 1.98.0, Rust 1.98.0 |
| Build | `cargo build --locked --bin llmeter` passed |
| Hosted CI | [Run 35830958376](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/35830958376) passed Windows, Ubuntu, macOS Intel, and macOS ARM64 at commit `1222ba621e1c921f6a792be5c27688773a8c0625` |
| Provider/model | None; these catalog and help commands did not contact a provider |
| Campaign boundary | Tier 1: application foundations, slice T1-01 |
| Canonical summary | [Project status ledger](../../docs/project_status_ledger.md) |
| Campaign plan | [Validation campaign](../../docs/runtime/validation_campaign.md) |

## Slice ledger

| Slice | Capability | Exists | Exercised | Status | Scenarios executed | Evidence | Remaining gap | Last validated | Commit/revision |
|---|---|---:|---:|---|---|---|---|---|---|
| T1-01 | Provider, benchmark, quality, and built-in help catalogs | YES | YES | PASS | Fourteen real process invocations covered global `--help`, all three catalogs, overview, all advertised help topics, the `benchmarks` and `lifecycle` aliases, and unsupported `quality` and arbitrary topics. All returned exit 0 with empty stderr. Catalogs listed 12 provider presets, 7 built-in benchmarks, and 11 quality tasks. | [CLI stdout snapshots](tier1-evidence.md) | Unsupported topics return the generic overview with exit 0. T1-02 through T1-07 remain unrun at their dedicated campaign boundaries. | 2026-09-23 | 056817717e07fb64c14f13e0b27e83e5ac5b85aa |

## Slice result

T1-01 is PASS at the captured Windows CLI boundary. Unsupported help topics, including `quality`, are accepted and produce the generic overview; the snapshots record this observed behavior without treating it as a documented error contract. Tier 1 is incomplete. Continue with T1-02, the Windows ConPTY menu and interruption slice.
