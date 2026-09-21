# LLMeter 0.3.0 release-readiness validation

Date: 2026-08-17

## Local gates

- `cargo fmt --all -- --check` passed.
- `cargo check --locked --all-targets --all-features` passed.
- `cargo clippy --locked --all-targets --all-features -- -D warnings` passed.
- `cargo test --locked --all-targets --all-features -- --test-threads=1` passed: 122 tests.
- `RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --all-features` passed.
- `cargo audit` passed with no vulnerabilities or unmaintained-crate warnings.
- `cargo tree --duplicates` shows one Crossterm version (`0.29.0`); remaining duplicates are unrelated transitive platform/proc-macro versions.
- `cargo publish --locked --dry-run --allow-dirty` passed; Cargo packaged 90 files (697.0 KiB, 153.8 KiB compressed). `--allow-dirty` is required because this task intentionally does not commit changes.
- `cargo build --locked --release --all-features` passed.

## Runtime and distribution smoke tests

- Release binary `--version`, `--help`, `providers list`, `bench list --suite llm`, and `quality plan` passed.
- Release binary status against a closed loopback port printed the status panel and returned exit code `1`; a reachable provider returned `0`.
- Invalid timeout returned the documented exit code `2`; non-TTY invocation printed help and returned `2`.
- Deterministic mock-provider suite passed against the release binary and an extracted Windows archive: models, standard generation, embeddings, unsupported capabilities, performance smoke, JSON/CSV output, and Markdown/HTML reports.
- A tiny live Ollama smoke run against `qwen3.5:2b` completed successfully with one 8-token prompt/output scenario and JSON/CSV/Markdown/HTML outputs. Provider availability is environment-dependent.
- A temporary Windows archive was extracted outside the repository; its four-file contents, SHA-256 entry, `--version`, and mock-provider suite passed.
- The packaged crate was installed with `cargo install --path target/package/llmeter-0.3.0 --locked --root <temporary-root>` and the installed binary passed `--version` and `bench list --suite llm` outside the repository.

## Hosted gates

- Four-platform CI run `32048610866` passed on Windows x86-64, Linux x86-64 GNU, macOS arm64, and macOS Intel.
- The first tagged release run `32049291985` correctly caught two workflow defects: the v3 attestation action required a missing predicate type, and macOS Intel archive verification depended on locale-specific sort order.
- Corrected release run `32050420660` passed release metadata, all four native builds, packaged mock-provider suites, archive smoke tests, archive attestations, checksum generation/verification, checksum attestation, and GitHub Release publication.
- Public `v0.3.0` assets were downloaded into an isolated temporary directory. All five SHA-256 entries matched, all archives contained the expected binary and `README.md`, `LICENSE`, and `CHANGELOG.md`, the extracted Windows binary reported `llmeter 0.3.0`, and `gh attestation verify` passed for all four archives and `SHA256SUMS`.
- crates.io publication was intentionally not performed; it remains the manual next step from the exact tagged source.
- Post-release documentation and Node 24 Actions upgrades passed the four-platform CI runs `32051322964`, `32051335492`, `32051652998`, and `32051664394` on both main and develop.
- The final Node 24 release-action pin passed CI in runs `32052011788` (main) and `32052023306` (develop); only the hosted macOS Homebrew tap-trust annotation remains non-blocking.
