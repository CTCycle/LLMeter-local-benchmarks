# LLMeter audit implementation plan

Last updated: 2026-09-10

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

- `77857af` - fallible configuration and bounded provider protocol behavior.
- `fc6545f` - non-TTY CLI contracts and typed output formats.
- `1758c01` - private-by-default saved outputs and redaction.
- `f3d534f` - statistically explicit metrics and bounded telemetry.
- `6bc563d` - support tiers, native CI, lifecycle policy, ephemeral authentication, and interruption contracts.
- `b49a567` - result schema `2.4`, honest streaming and token-usage semantics, fresh model validation, hardened reports, and tag-gated release validation.
- PR #1 (`refactor/canonical-sources-of-truth`) supersedes the active runtime contract with strict result schema `3.0`, canonical provider/model identity, typed output choices, shared performance-profile defaults, and removal of obsolete aliases and runtime compatibility fallbacks. Historical `2.4` references below remain release-history evidence rather than current runtime behavior.

At the original audit snapshot, package state was `0.3.0` on `develop`; that implementation was committed and pushed before hosted release validation.

Earlier local validation evidence (2026-07-30): formatting, locked all-target/all-feature check, Clippy, rustdoc with warnings denied, release build, package dry-run, and the serialized all-target/all-feature suite passed 118 tests. The Windows release binary and extracted archive also passed the mock-provider and CLI smoke checks.

Current release validation evidence (2026-09-10): package version `0.4.0` passes formatting, locked all-target/all-feature check, Clippy, rustdoc with warnings denied, release build, dependency audit, package dry-run, and 132 serialized all-target/all-feature tests on Windows. The release binary passes `--version`, `--help`, the packaged mock-provider E2E suite, and isolated CLI/report smoke checks. A live Ollama pass completed the standard LLM suite (8/8 records), embeddings (1/1), and a performance smoke scenario (1/1); the PowerShell launcher forwards documented flags correctly after a focused fix. Hosted four-platform CI and public release publication remain separate gates.

The validated commands are:

```powershell
cargo fmt --all -- --check
cargo check --locked --target-dir "$env:TEMP\llmeter-codex-final-target" --all-targets --all-features
cargo clippy --locked --target-dir "$env:TEMP\llmeter-codex-final-target" --all-targets --all-features -- -D warnings
cargo test --locked --target-dir "$env:TEMP\llmeter-codex-final-target" --all-targets --all-features -- --test-threads=1
$env:RUSTDOCFLAGS = "-D warnings"
cargo doc --locked --target-dir "$env:TEMP\llmeter-codex-final-target" --no-deps --all-features
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

Status: complete.

### Completed

- `SUPPORTED_PLATFORMS.md` defines Windows x86-64 Tier 1, Ubuntu GNU/Linux Tier 2, compatibility-only macOS/musl targets, and external runtime prerequisites.
- README, user manual, deployment, startup, and release guidance explicitly distinguish GNU/glibc from musl and no longer claim unapproved public artifacts.
- The minimum supported Rust version remains intentionally undeclared until an MSRV support commitment is approved.
- Public package-manager and binary publication remain explicitly deferred pending an owner decision.

### Scope

Define what “supported local CLI” means before adding infrastructure. Keep the primary support target Windows x86-64, retain Linux and macOS build/test coverage where runners are available, and describe GNU Linux versus musl accurately.

### Acceptance

- Documentation does not claim the GNU Linux binary is fully static.
- The supported target list and runtime prerequisites are explicit.
- No package-manager or public-release work is represented as complete without an owner decision.

## Phase 1 - Finish configuration and input validation

Status: complete.

### Completed

- Persisted configuration loading is fallible: missing files use defaults, while read failures, malformed JSON, and unknown providers identify the source path.
- Provider base URLs use parsed `url::Url` validation and preserve the OpenAI-compatible `/v1` contract.
- Invalid `LLMETER_PROVIDER` values are rejected instead of silently selecting Ollama.
- CLI, environment, provider-specific environment, and persisted-file diagnostics identify their configuration source.

Validation evidence (2026-07-30): configuration behavior remains covered by the locked all-target/all-feature suite, including malformed persisted JSON, invalid provider values, URL normalization, timeout bounds, numeric defaults, and provider-specific URL precedence.

The validation boundaries are centralized rather than wrapped in additional domain types: configuration construction validates timeout, positive runs/tokens, finite non-negative temperature, and performance-plan concurrency before values enter execution. Dedicated wrappers were not added because each value has one construction boundary and wrappers would not remove repeated checks. Configuration writes are atomic, malformed persisted files fail with their path, and atomic rename failure cleanup is tested.

### Acceptance

- No invalid CLI, environment, or persisted configuration silently falls back.
- No configuration value can reach a panic path.
- Error messages identify the source, value, and accepted range/format.
- Existing provider-precedence behavior remains covered by serialized tests.

### Likely files

`src/config.rs`, `src/cli.rs`, `src/errors.rs`, `src/providers.rs`, `tests/config_validation.rs`, and `assets/docs/runtime/configuration.md`.

## Phase 2 - Harden the provider protocol boundary

Status: complete.

### Completed

- Chat and responses core request fields cannot be overwritten by arbitrary parameters.
- Internal performance safety controls are separate from provider options.
- Timeout construction is validated.
- Provider model catalog shape and basic SSE metadata handling are tested.
- Direct provider-client construction validates the base URL, uses a LLMeter user agent, separately bounds connection time, disables redirects and proxy inheritance, and retains the actual successful HTTP status in `ApiResult`.
- Non-success response diagnostics and streaming lines are bounded; streaming `data:` fields are assembled into protocol-level SSE events.
- Operational model discovery uses a cached catalog, while status, release validation, and measured provider probes use fresh `/v1/models` requests. The cache has explicit invalidation.
- Optional provider authentication uses an ephemeral sensitive bearer header sourced only from `LLMETER_API_KEY`; it is never persisted or logged.

### Acceptance

- Provider failures are bounded and readable even when a server returns a huge body or event.
- Results contain the actual HTTP status.
- Streaming and non-streaming paths have dedicated protocol tests.
- Authentication material never appears in saved configuration, reports, or diagnostics.

### Likely files

`src/providers.rs`, `src/performance/provider_probe.rs`, `tests/mock_provider_e2e.rs`, and new focused provider protocol tests.

## Phase 3 - Complete safe output and privacy behavior

Status: privacy defaults and atomic persistence complete.

### Completed

- JSON, CSV, Markdown, HTML, and provider configuration writes use same-directory temporary files and rename.
- CSV formula-like text is neutralized in untrusted text cells.
- Response previews are omitted from saved machine output and reports unless `--include-response-preview` is supplied.
- Output preparation redacts credential-shaped diagnostic text, secret-named parameters, nested metadata, and local process/cache selectors without mutating the measured in-memory run.
- Saved configuration records the applied preview and redaction policy.

Failure-path coverage also proves a failed final rename removes the same-directory temporary file.

### Acceptance

- Saved results do not unexpectedly contain full model output, secrets, or sensitive process details.
- JSON and CSV behavior is documented and stable.
- Atomic output failures leave no valid-looking partial final file.

### Likely files

`src/results.rs`, `src/reporting.rs`, `src/utils.rs`, `src/cli.rs`, `assets/docs/architecture/result_storage.md`, `assets/docs/user/reports_and_results.md`, and `tests/test_results.rs`.

## Phase 4 - Stabilize the CLI runtime contract

Status: complete.

### Completed

- Interactive dispatch now requires terminal stdin and stdout; piped/CI invocations print help and return usage code 2.
- Raw export and report formats use typed Clap `ValueEnum` values.
- CLI version output derives from Cargo package metadata.
- Progress rendering already targets stderr, including non-interactive line-oriented rendering.
- CLI contract tests cover no-subcommand non-TTY behavior, explicit non-TTY menu behavior, and version output.
- JSON model output is parseable without stderr decoration; invalid configuration returns documented status 2 without stdout data.
- Ctrl+C is detected at the raw terminal key boundary, the raw-mode guard restores terminal state, and interactive dispatch returns conventional status 130.

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

Status: complete.

### Completed

- Performance records and report tables show the successful latency sample count.
- Percentiles use the documented nearest-rank estimator over successful finite non-negative measurements; P95 requires 20 samples and P99 requires 100.
- Standard deviation is explicitly recorded and documented as population standard deviation.
- Invalid/NaN measurements and zero-duration rates are filtered, and failures remain outside successful latency/token denominators.
- Result metadata records warmups, profile/provider state, telemetry mode, and load-measurement mode.

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

Status: complete.

### Completed

- Telemetry modes are explicitly `off`, `standard` (default), and `detailed`.
- The bounded sample buffer uses `VecDeque` and drops the oldest sample in constant time.
- Sampler shutdown uses an interruptible channel timeout, so stop/drop does not wait for the configured interval.
- Performance request scheduling remains bounded to the configured concurrency, and spawned requests share one immutable `Arc<PerformancePlan>` per scenario instead of cloning the plan for every request.

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

Status: native Ubuntu and Windows CI implemented; hosted execution remains separate from local validation evidence.

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

Current CI uses a non-fail-fast Ubuntu/Windows matrix with locked all-target/all-feature check, Clippy, tests, and rustdoc. Formatting runs once on Ubuntu. MSRV, packaging, macOS release evidence, and action-SHA pinning remain gated on the corresponding support/distribution decisions.

### Acceptance

- The supported local platform matrix has native evidence.
- CI and local validation commands agree with documentation.
- No audit verdict is called runtime proof without an executed check.

## Phase 8 - Lifecycle and distribution policy

Status: complete for the current local-only policy; public distribution remains owner-gated.

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

Deployment, README, and user-manual guidance now label lifecycle commands as local convenience file operations, require the user to verify replacement executables, and explicitly state that LLMeter does not download, authenticate, channel-select, or roll back remote updates.

## Remaining closeout

All locally actionable audit phases are implemented. Hosted release run `32050420660` passed the four-target build, packaged mock-provider, checksum, and provenance gates, and published `v0.3.0` on 2026-08-17. The `0.4.0` source release is locally prepared; hosted four-platform execution, public tag publication, and crates.io publication remain owner-gated steps. MSRV is still not declared.

## Definition of done for the local CLI

LLMeter is ready for controlled local single-user use when:

- Invalid input cannot panic or silently default.
- Provider payload boundaries are explicit and reserved fields are protected.
- Scriptable execution is safe in pipes and CI.
- Saved outputs are atomic, spreadsheet-safe, and privacy-documented.
- Benchmark reports disclose sample limitations and telemetry conditions.
- Windows primary-path tests and at least one non-Windows compatibility path pass.
- Deployment documentation matches the actual binaries and provider prerequisites.
