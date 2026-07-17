# LLMeter audit implementation plan

Last updated: 2026-07-18

## Purpose

This is the durable implementation roadmap for the Rust and CLI audit. It is intentionally scoped to LLMeter as a local, single-user benchmarking CLI. Future sessions should use this document to resume work, update the current phase, and record validation evidence without repeating the original audit.

## Product boundary

LLMeter is a local executable that talks to provider servers started by the user. It does not need accounts, roles, multi-tenant isolation, a hosted control plane, or application-managed provider lifecycles.

The plan therefore prioritizes:

- Correct, bounded, scriptable local execution.
- Honest benchmark measurements and result interpretation.
- Safe handling of local configuration, provider responses, and saved output.
- Windows-first development with Linux compatibility and practical cross-platform CI.
- Reproducible release artifacts only when binary distribution is explicitly resumed.

The plan does not require, for the local-single-user product:

- User authentication, authorization, sessions, or a database server.
- Cloud deployment abstractions or a service-to-service security model.
- Package-manager publication, Homebrew, WinGet, Scoop, or crates.io publication before public distribution is approved.
- Signed remote self-updates. The application should not become its own package manager.
- SBOM, provenance, signing, or public-release attestations for private local builds. These become release gates only when public binary distribution is approved.

## Current baseline

Completed and pushed:

- `58527a1` - validated timeout and numeric configuration, dedicated performance safety flags, and reserved provider fields.
- `8508c11` - atomic persistence for JSON, CSV, Markdown, HTML, and provider configuration.

Completed in the current source state:

- CSV formula-injection mitigation for untrusted text cells.
- Regression coverage for formula-like CSV values.

The existing validation baseline is green when run with an isolated temporary Cargo target directory:

```powershell
cargo fmt --all -- --check
cargo check --target-dir "$env:TEMP\llmeter-codex-check-target" --all-targets --all-features
cargo clippy --target-dir "$env:TEMP\llmeter-codex-clippy-target" --all-targets --all-features -- -D warnings
cargo test --target-dir "$env:TEMP\llmeter-codex-test-target" --all-targets --all-features -- --test-threads=1
```

## Work status convention

At the beginning of each continuation:

1. Read this plan and `assets/docs/project_index.md`.
2. Run `git status -sb` and preserve unrelated worktree changes.
3. Select one bounded slice from the next recommended phase.
4. Add or update focused tests before broad validation.
5. Update this plan only when phase status or acceptance evidence materially changes.
6. Commit coherent slices separately and record the commit here when the user requests commit/push cadence.

## Phase 0 - Maintain the release and support contract

Status: partially complete.

### Scope

Define what “supported local CLI” means before adding infrastructure. Keep the primary support target Windows x86-64, retain Linux and macOS build/test coverage where runners are available, and describe GNU Linux versus musl accurately.

### Remaining work

- Add or finalize `SUPPORTED_PLATFORMS.md` with Tier 1 and Tier 2 targets.
- Declare the minimum supported Rust version only when the project is ready to support it.
- Align README, user manual, deployment, startup, and release-checklist claims.
- State that provider servers and optional `nvidia-smi` telemetry are external prerequisites.
- Decide whether public source or binary distribution is approved before adding publication automation.

### Acceptance

- Documentation does not claim the GNU Linux binary is fully static.
- The supported target list and runtime prerequisites are explicit.
- No package-manager or public-release work is represented as complete without an owner decision.

## Phase 1 - Finish configuration and input validation

Status: in progress.

### Remaining implementation

- Make persisted configuration loading fallible. A missing file may use defaults; malformed JSON or an unknown provider must produce a source-specific configuration error.
- Replace string URL normalization with parsed `url::Url` validation while preserving the OpenAI-compatible `/v1` contract.
- Add small validated domain types where they reduce repeated checks: request timeout, positive run count, positive token limit, non-negative finite temperature, and positive concurrency.
- Reject invalid provider environment values instead of silently selecting the default.
- Preserve CLI source context in every configuration diagnostic.
- Keep configuration writes atomic and add failure-path tests for malformed files and invalid values.

### Acceptance

- No invalid CLI, environment, or persisted configuration silently falls back.
- No configuration value can reach a panic path.
- Error messages identify the source, value, and accepted range/format.
- Existing provider-precedence behavior remains covered by serialized tests.

### Likely files

`src/config.rs`, `src/cli.rs`, `src/errors.rs`, `src/providers.rs`, `tests/config_validation.rs`, and `assets/docs/runtime/configuration.md`.

## Phase 2 - Harden the provider protocol boundary

Status: partially complete.

### Completed

- Chat and responses core request fields cannot be overwritten by arbitrary parameters.
- Internal performance safety controls are separate from provider options.
- Timeout construction is validated.
- Provider model catalog shape and basic SSE metadata handling are tested.

### Remaining implementation

- Cap non-success error-body size and streaming line/event size.
- Preserve the actual successful HTTP status in `ApiResult`.
- Add a clear LLMeter user agent.
- Add separate connect and total request timeouts.
- Parse and validate the base URL before constructing a client.
- Define an explicit policy for redirects, proxies, and optional authentication headers. Keep secrets ephemeral and out of persisted results/logs.
- Assemble multi-line SSE events according to the protocol rather than treating every line as a complete event.
- Avoid duplicate `/v1/models` calls within one command by reusing a catalog snapshot.

### Acceptance

- Provider failures are bounded and readable even when a server returns a huge body or event.
- Results contain the actual HTTP status.
- Streaming and non-streaming paths have dedicated protocol tests.
- Authentication material never appears in saved configuration, reports, or diagnostics.

### Likely files

`src/providers.rs`, `src/performance/provider_probe.rs`, `tests/mock_provider_e2e.rs`, and new focused provider protocol tests.

## Phase 3 - Complete safe output and privacy behavior

Status: atomic persistence complete; CSV formula mitigation pending commit.

### Completed

- JSON, CSV, Markdown, HTML, and provider configuration writes use same-directory temporary files and rename.
- CSV formula-like text is neutralized in untrusted text cells.

### Remaining implementation

- Add explicit output controls such as `--redact` and `--include-response-preview`.
- Keep response previews opt-in for machine output and reports where practical.
- Redact secrets from URLs, headers, provider errors, and diagnostic text.
- Define which fields are safe, sensitive, or environment-specific.
- Add failure-path tests proving stale temporary files are cleaned up where possible.

### Acceptance

- Saved results do not unexpectedly contain full model output, secrets, or sensitive process details.
- JSON and CSV behavior is documented and stable.
- Atomic output failures leave no valid-looking partial final file.

### Likely files

`src/results.rs`, `src/reporting.rs`, `src/utils.rs`, `src/cli.rs`, `assets/docs/architecture/result_storage.md`, `assets/docs/user/reports_and_results.md`, and `tests/test_results.rs`.

## Phase 4 - Stabilize the CLI runtime contract

Status: not started.

### Implementation

- Detect terminal capabilities explicitly.
- With no subcommand: open the menu only when stdin and stdout are terminals; otherwise print help and return a documented usage code.
- Keep data on stdout and progress/diagnostics on stderr.
- Add a shared output policy for human, JSON, quiet, color, and progress modes only where it improves scriptability without bloating the local CLI.
- Ensure JSON mode never receives decoration or progress text.
- Ensure Ctrl+C restores terminal state and returns a conventional interruption status.
- Replace stringly typed export/report formats with `ValueEnum` values.
- Derive the CLI version from Cargo metadata.
- Add CLI contract tests for help, invalid configuration, non-TTY invocation, JSON cleanliness, and exit codes.

### Local-single-user decision

Shell completions and a man page are useful but not blocking for local development. Implement them after stdout/stderr and non-TTY behavior are stable.

### Acceptance

- Piped and CI execution never opens an interactive menu.
- Scriptable commands have stable exit and output behavior.
- Interactive terminal state is restored after normal exit, Escape, and Ctrl+C.

### Likely files

`src/main.rs`, `src/cli.rs`, `src/ui.rs`, new terminal/output modules, `tests/pty_menu_e2e.rs`, and new CLI contract tests.

## Phase 5 - Make benchmark results statistically honest

Status: partially complete.

### Implementation

- Include successful sample count beside each latency percentile.
- Mark P95/P99 as insufficient or omit them when the sample count cannot support the requested interpretation.
- Document the percentile estimator and population versus sample standard deviation.
- Distinguish smoke/exploratory profiles from statistically meaningful profiles.
- Record warmup count, provider state, telemetry mode, and load-measurement mode in result metadata.
- Add tests for small samples, NaN filtering, zero durations, failed requests, and concurrency aggregates.

### Acceptance

- Reports do not imply confidence unsupported by the sample count.
- Performance summaries remain reproducible from saved metadata.
- Failure counts never contaminate successful-request latency denominators.

### Likely files

`src/performance/metrics.rs`, `src/performance/runner.rs`, `src/results.rs`, `src/reporting.rs`, `tests/performance_metrics_tests.rs`, and result-storage docs.

## Phase 6 - Control telemetry and concurrency overhead

Status: not started.

### Implementation

- Offer telemetry off/standard/detailed modes with an explicit default.
- Replace front-removal from `Vec` with a bounded `VecDeque` or ring buffer.
- Make telemetry shutdown interruptible and non-blocking at drop.
- Record telemetry configuration and overhead mode in results.
- Avoid cloning the entire performance plan for every request where an immutable shared context is sufficient.
- Keep the current blocking provider model unless profiling demonstrates that async conversion is necessary. If converted, use one runtime, bounded futures, semaphore limits, and cancellation propagation.

### Acceptance

- Telemetry overhead is controllable and disclosed.
- No request fan-out can exceed the configured concurrency.
- Ctrl+C cancels or drains outstanding work predictably.

### Likely files

`src/performance/telemetry.rs`, `src/performance/runner.rs`, `src/performance/resource.rs`, and focused telemetry/concurrency tests.

## Phase 7 - CI and maintainer validation

Status: Ubuntu-only CI exists; broader validation is pending.

### Required local/repository gates

```powershell
cargo fmt --all -- --check
cargo check --locked --all-targets --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features -- --test-threads=1
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features
```

On Windows, use an isolated target directory when the workspace target is locked:

```powershell
cargo test --target-dir "$env:TEMP\llmeter-codex-test-target" --all-targets --all-features -- --test-threads=1
```

### CI implementation

- Keep Ubuntu stable as the fast baseline.
- Add native Windows testing because PTY behavior and the primary development environment are Windows-specific.
- Add macOS testing only for supported release targets or when a real macOS runner is available.
- Add MSRV validation only after `rust-version` is intentionally declared.
- Add package validation (`cargo package`, extracted-package build) only when crates.io or source packaging is approved.
- Pin third-party actions to reviewed commit SHAs when public release trust becomes in scope.

### Acceptance

- The supported local platform matrix has native evidence.
- CI and local validation commands agree with documentation.
- No audit verdict is called runtime proof without an executed check.

## Phase 8 - Lifecycle and distribution policy

Status: policy decision pending.

### Recommended local-single-user policy

Treat `llmeter install`, `llmeter update`, and `llmeter uninstall` as local convenience commands, not a general cross-platform updater. Add deprecation guidance and direct users to copying a verified binary or using the eventual package manager.

Do not implement remote update metadata, rollback, signatures, or release channels inside the application unless public binary distribution is approved and the lifecycle commands are retained intentionally.

### If public binary distribution is later approved

- Define Tier 1/Tier 2 targets.
- Build from protected version tags.
- Validate tag, Cargo version, CLI version, and changelog heading.
- Build artifacts, execute `--version` and `--help`, run mock-provider smoke tests, generate checksums, and publish only after verification.
- Add SBOM/provenance/signatures according to the repository’s hosting and trust requirements.
- Prefer generated release tooling over hand-maintained installer mechanics.

### Acceptance for current local scope

- Documentation clearly labels managed lifecycle commands as convenience behavior.
- No claim suggests the CLI can securely self-update from arbitrary local or remote executables.
- Public distribution work remains visibly deferred rather than half-implemented.

## Recommended next-session order

1. Commit and push the current CSV hardening slice.
2. Finish fallible persisted configuration and URL validation.
3. Add provider response-size limits and actual HTTP status capture.
4. Stabilize non-TTY/output contracts.
5. Add privacy/redaction controls.
6. Add percentile/sample guardrails.
7. Refactor telemetry only after measuring its overhead.
8. Expand native CI and revisit distribution only after the local CLI contract is stable.

## Definition of done for the local CLI

LLMeter is ready for controlled local single-user use when:

- Invalid input cannot panic or silently default.
- Provider payload boundaries are explicit and reserved fields are protected.
- Scriptable execution is safe in pipes and CI.
- Saved outputs are atomic, spreadsheet-safe, and privacy-documented.
- Benchmark reports disclose sample limitations and telemetry conditions.
- Windows primary-path tests and at least one non-Windows compatibility path pass.
- Deployment documentation matches the actual binaries and provider prerequisites.
