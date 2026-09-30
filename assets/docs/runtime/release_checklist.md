# Release checklist

Last updated: 2026-09-30

## Release-candidate condition

Before tagging a release candidate, confirm that:

- the [project status ledger](../project_status_ledger.md) has been reviewed and the Tier 0 through Tier 5 boundaries are satisfied at their documented scopes;
- no unresolved release-blocking application-validation entry remains;
- the exact candidate commit has green hosted CI for Ubuntu x86-64, Windows x86-64, macOS Intel, and macOS Apple silicon;
- the current live-provider boundary, configured 500-request ceiling, interruption/restart evidence, native PTY evidence, privacy checks, and release artifact checks are complete;
- the working tree is clean and the candidate SHA is recorded in the release notes and ledger;
- any PARTIAL or BLOCKED result is explicitly classified as a non-blocking environment, optional-provider, or out-of-scope limitation, with the claims it prevents.

Do not use a package dry-run or a GitHub release alone to mark crates.io distribution validated. Publication and clean registry installation are separate owner-gated steps.

## Current v0.5.0 release-candidate boundary

The public [v0.4.0 GitHub release](https://github.com/CTCycle/LLMeter-local-benchmarks/releases/tag/v0.4.0) passed the four-target release workflow, packaged mock-provider checks, checksums, provenance, and publication in [run 34574075684](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/34574075684). It is the verified archive distribution path.

The develop line contains the committed ConPTY interruption repair and integration-fixture cleanup. The focused regressions pass, but the exact `v0.5.0` candidate must still pass the complete local release gate and four-platform hosted CI before main is synchronized or a tag is created.

## Non-blocking evidence limitations

A host or provider limitation does not block a scoped release when the functional, safety, quality, and exact-candidate hosted gates pass, the limitation is not a known product defect, and the release record states the claims that remain out of scope.

Swap pressure invalidates numeric comparative timing and model-ranking claims, and therefore keeps the comprehensive-validation claim open. It does not invalidate functional benchmark execution, persistence, reporting, privacy, or safety evidence. An unavailable optional provider route, such as a disposable LiteLLM proxy, follows the same boundary.

## Before tagging

1. Confirm the version in Cargo.toml.
2. Update CHANGELOG.md.
3. Run:

```
cargo fmt --all -- --check
cargo check --locked --all-targets --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features -- --test-threads=1
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features
cargo build --locked --release --all-features
cargo audit
cargo tree --locked --duplicates
cargo publish --locked --dry-run
```

On Windows, use an isolated target directory if the shared target/ tree is locked:

```
cargo build --locked --release --all-features --target-dir "$env:TEMP\llmeter-release-target"
```

## Release artifacts and trust

The authorized v* release workflow gates publication on the Cargo version, changelog heading, locked all-target/all-feature quality suite, dependency audit, package dry-run, native platform builds, packaged mock-provider tests, archive extraction/content checks, and extracted-binary --version/--help smoke checks. It publishes four archives, filename-only SHA256SUMS, and GitHub artifact provenance attestations.

Users should verify checksums and provenance before running a downloaded binary. The GNU/Linux archive requires a compatible glibc runtime and is not fully static.

## Publishing procedure

1. Review and commit the prepared changes, push develop, and require green four-platform CI.
2. Merge or fast-forward the verified commit to main and require CI there.
3. Confirm Cargo.toml and CHANGELOG.md identify the version, the worktree is clean, and no matching tag or release exists.
4. Create and push an annotated tag from that main commit.
5. Verify all four archives, archive contents, checksums, extracted-binary smoke tests, and provenance attestations.
6. Independently download and verify the public assets.
7. If approved, run cargo publish --locked from the exact tagged source.
8. Verify cargo install llmeter --version <version> --locked --root <clean-temp-root> and the installed binary.
9. Stop publication if any hosted or registry verification fails.
