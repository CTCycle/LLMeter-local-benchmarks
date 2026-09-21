# LLMeter audit validation — 2026-07-30

## Passed

- `cargo fmt --all -- --check`
- `cargo test --locked --all-targets --all-features -- --test-threads=1` — 118 passed across library, integration, PTY, reporting, and resource suites
- `cargo clippy --locked --all-targets --all-features -- -D warnings`
- `RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --all-features`
- `cargo package --locked --allow-dirty --list`
- `cargo check --locked --all-targets --all-features`
- `cargo audit` — passed with no vulnerabilities; two non-blocking unmaintained transitive warnings remain for `fxhash 0.2.1` and `proc-macro-error2 2.0.1`.
- `cargo build --release --locked --all-features`
- `cargo publish --locked --dry-run --allow-dirty` — passed after excluding generated alternate target directories; 90 files, 669.6 KiB unpacked, 149.4 KiB compressed.
- Final all-features Windows release binary rebuilt after the latest CLI/UI and fresh-validation changes; its mock-provider integration suite passed 11 tests, and the binary reported `llmeter 0.3.0` with updated provider-preset help text.
- Mock-provider export coverage now verifies JSON and CSV raw outputs plus Markdown and HTML reports in one CLI run.
- Live Ollama validation passed: the release binary reported the local endpoint reachable with 8 exposed models; one real `qwen3.5:2b` standard chat run completed with 1/1 success, schema 2.4, 32 output tokens, 85,930.426 ms wall time, and independently recalculated wall-time output rate 0.372394 tok/s.
- Live Ollama performance smoke passed: one streamed `qwen3.5:2b` scenario completed with 1/1 success, 636.878 ms scenario wall time, 4 output tokens, stored output rate 6.275249 tok/s, independently recalculated rate 6.280638 tok/s, and P50/P90 wall time 636.878 ms. TTFT, inter-chunk latency, and ITL were unavailable because the provider returned only empty-content reasoning chunks; the raw SSE response was inspected and no token timings were claimed.
- A fresh streamed `qwen3.5:9b` smoke also completed successfully (66,286.294 ms wall time), but returned no non-empty content chunks under the requested thinking setting; TTFT, inter-chunk latency, ITL, and content-based output rates remained unavailable rather than being fabricated.
- Live Ollama first-request estimate passed: first probe 712.814 ms, warm probe mean 395.144 ms, estimated client-observed overhead 317.670 ms, with persisted notes explicitly excluding provider restart, cache eviction, model loading, and native lifecycle telemetry.
- Scriptable mock-provider status, model listing, and model-details smoke paths passed.
- Scriptable provider-catalog smoke passed with compatibility-tier output and endpoint-dependent preset wording.
- Corrected the Windows archive to preserve its versioned top-level directory, then verified extraction contained only `llmeter.exe`, `README.md`, `LICENSE`, and `CHANGELOG.md`; extracted `--version`, `--help`, and invalid-timeout checks passed.
- The release workflow now runs the mock-provider suite against the extracted Linux/Windows archive binary itself; the extracted Windows binary path was revalidated locally with all 11 mock tests passing.
- Nested Windows ConPTY cancellation/navigation test — 9 PTY tests passed, including top-level Ctrl+C, explicit EOF termination, nested provider/model cancellation, numeric-prompt cancellation, pause-prompt interruption, performance-plan confirmation interruption, Reports-list cancellation, submenu Back, and clean Exit.
- Output-directory failure handling test — a non-directory result path now returns an error instead of appearing empty.
- Windows release binary smoke check: `llmeter 0.3.0` from the release build and extracted archive; `--help` also completed.
- Release workflow statically checked and tightened to run audit/package gates, all-feature release builds, extracted-binary smoke tests, archive-content checks, and packaged mock-provider tests on versioned Linux/Windows artifacts.
- Release workflow mock-provider jobs now also use `--locked --all-features` for both the built and extracted archive binaries.
- Unix lifecycle path now installs the executable directly without emitting Windows-only launcher files.
- Final terminology and contract search found no stale TPOT alias, provider-native load claim, or legacy model-list call; native wording remains limited to the performance mode and explicit unsupported-telemetry caveats.
- Re-audit corrected the remaining `schema_version` 2.3 documentation reference and synchronized the README/changelog update markers with the 2.4 contract.

## Implemented audit corrections

- Renamed chunk timing fields and separated inter-chunk latency from ITL.
- ITL now requires streaming, TTFT, and at least two provider-reported output tokens, using post-TTFT wall time divided by `output_tokens - 1`.
- Added token usage sample counts/coverage and suppressed aggregate token rates when successful usage is partial.
- Split cached and fresh model catalog access; ordinary navigation uses cached discovery while benchmark validation and measured/status paths use fresh discovery.
- Added an explicit model-catalog refresh action that invalidates the cache before fresh discovery.
- Replaced unsupported cold/native load claims with an explicit first-request estimate.
- Made cache sizing a single provider-cache-directory scan with symlink and traversal bounds.
- Added a Unix regression test proving symlinked cache directories are skipped without double-counting.
- Corrected consistency scoring to pairwise matching frequency and reject fewer than two successes.
- Added CLI and runner-boundary validation for standard benchmark numeric options.
- Unified prompt cancellation/interruption outcomes and preserved menu navigation behavior.
- Propagated interruption from pause prompts instead of discarding the shared prompt outcome.
- Distinguished mixed failures from total scenario errors and distinguished omitted from truncated request traces.
- Clarified TTFT as time to the first non-empty content chunk and made invalid TTFT/wall-time ordering unavailable.
- Made lifecycle update messaging explicit about deferred copy verification.
- Made result/report discovery surface directory and metadata errors.
- Kept scenario aggregate throughput separate from standard per-request throughput in reports.
- Reworded provider catalog/help/manual surfaces from unsupported "supported providers" claims to provider presets with compatibility tiers and endpoint-dependent availability.
- Bumped result schema to 2.4 and aligned release documentation/workflow.

## Unverified or deferred

- Live Ollama behavior is validated for the observed `qwen3.5:2b` model and endpoint; other providers, models, and provider-specific lifecycle behavior remain unverified.
- GitHub-hosted Linux and Windows release jobs remain unexecuted; `gh run list --workflow release.yml` returned no runs. The workflow was statically reviewed and the Windows artifact was built and smoke-tested locally.
- macOS native packaging remains deferred and is not advertised by the release workflow.
- Full nested interactive PTY coverage across every prompt and cancellation path remains unverified; nine exercised ConPTY paths now pass, including EOF, Back, clean Exit, pause-prompt interruption, performance-plan confirmation interruption, and Reports-list cancellation, while broader prompt coverage still needs a separate run.
- No release tag, public publication, or commit/push was performed.
- The crates.io API was rechecked and returned 404 for `llmeter`, so the exact crate name appears unclaimed at validation time; no publication was attempted.
