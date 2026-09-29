# Tier 4 quality-planning and distribution evidence

Last updated: 2026-09-29

## Boundary

| Field | Value |
|---|---|
| Validation revision | `4fefea734cc62dac327308a518d5409b4f01a033` |
| Quality scope | LLMeter plan generation only. No external evaluator installation, dataset download, code execution, or SWE-bench run. |
| Distribution scope | Existing public GitHub `v0.4.0` release plus a current local release-binary smoke. crates.io remains separate. |

## T4-03 external quality planning

The focused command passed:

```text
cargo test --locked --all-features --test quality_cli_tests -- --test-threads=1
```

Result: **3 passed, 0 failed**.

The tests cover all four framework adapters:

| Framework | CLI label | Planning result |
|---|---|---|
| LightEval | `lighteval` | CLI parse, task/catalog mapping, model/base URL propagation, serialized dry-run plan |
| Inspect AI | `inspect-ai` | CLI parse, task/catalog mapping, model/base URL propagation, serialized dry-run plan |
| lm-eval-harness | `lm-eval-harness` | CLI parse, task/catalog mapping, model/base URL propagation, serialized dry-run plan |
| SWE-bench | `swe-bench` | CLI parse, task/catalog mapping, model propagation, predictions-file command preview |

The complete quality catalog is also traversed. Its serialized requirement
flags are retained for every entry: `requires_external_tool`,
`requires_dataset`, and `requires_code_execution`. Unknown framework input is
rejected by the parser; an unknown task remains an explicit dry-run plan with a
null catalog mapping rather than being silently treated as a known benchmark.

T4-03 is **PASS** at the planning boundary. External evaluator execution is
outside the currently validated LLMeter runtime contract.

## T4-04 GitHub distribution

The existing [v0.4.0 release report](../release-0.4.0/release-report.md) and
hosted release run `34574075684` verify the separate distribution boundary:

- four native archives for Ubuntu/Linux x86-64, Windows x86-64, macOS Intel,
  and macOS Apple silicon;
- expected archive contents and extracted executable smoke tests;
- `SHA256SUMS` generation and verification;
- GitHub artifact provenance attestations; and
- packaged mock-provider E2E as part of the release workflow.

The current working tree release build also passed:

```text
cargo build --locked --release --all-features
target/release/llmeter.exe --version   # llmeter 0.4.0
target/release/llmeter.exe --help      # exited successfully
```

The current CI baseline remains hosted run `36478241893`, which passed Ubuntu
x86-64, Windows x86-64, macOS Intel, and macOS Apple silicon for the preceding
source/evidence commit. A new hosted run is required after these local test and
documentation changes are pushed; local success is not substituted for hosted
evidence.

## crates.io

Status: **UNVERIFIED / owner-gated**. The expected `llmeter 0.4.0` publication
and clean `cargo install --locked --root` check were not performed. No package
was published and no registry installation was added during this validation.

## Tier 4 disposition

T4-04 is **PASS** for the public GitHub release and separately **UNVERIFIED**
for crates.io. Distribution status does not depend on live provider
availability.
