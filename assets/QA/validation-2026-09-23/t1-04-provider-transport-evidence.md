# T1-04 provider transport through the official launcher

Last updated: 2026-09-23

## Validation boundary

| Field | Value |
|---|---|
| Repository | CTCycle/LLMeter-local-benchmarks |
| Branch and revision | `develop`, `5e900ed2fbe17fc3bbab252251e779b4a23839a7` |
| T1-04 implementation commit | `a9787410fa8c6304a9f1fda0d09a257740170ebf` |
| Package | llmeter 0.4.0 |
| Environment | Windows 11 Pro 10.0.26200, PowerShell 7.6.6, Cargo 1.98.0, Rust 1.98.0 |
| Harness | Windows-only integration tests staged the Cargo-built `llmeter.exe` in a disposable root and invoked the official `run_llmeter.ps1`; configuration, home, and result paths were isolated. |
| Provider/model | Deterministic loopback mock only. No live provider or model was contacted. |
| Credential | Synthetic `LLMETER_API_KEY` value only. |

## Local validation

The following passed on the tested Windows tree:

```powershell
cargo fmt --all -- --check
$env:RUSTFLAGS = '-D warnings'; cargo check --locked --all-targets --all-features
$env:RUSTFLAGS = '-D warnings'; cargo clippy --locked --all-targets --all-features -- -D warnings
$env:RUSTFLAGS = '-D warnings'; cargo test --locked --all-targets --all-features -- --test-threads=1
$env:RUSTFLAGS = '-D warnings'; cargo test --locked --test mock_provider_e2e -- --test-threads=1
$env:RUSTFLAGS = '-D warnings'; $env:RUSTDOCFLAGS = '-D warnings'; cargo doc --locked --all-features --no-deps
```

The focused mock-provider command passed 15 tests, 0 failed. The serialized full suite also passed, including all four `t1_04_official_launcher_*` scenarios through the supported launcher.

## Transport scenarios

| Scenario | Test | Observed result |
|---|---|---|
| Successful status and streamed chat | `t1_04_official_launcher_captures_multiline_sse_and_accepts_created_responses` | `status` read the mock model catalog; the launcher ran a chat benchmark, captured `POST /v1/chat/completions`, and persisted the response assembled from multiline SSE data. The normal `model`, `messages`, and `stream` fields were present. |
| Successful created response | Same test | The mock returned HTTP 201 from `POST /v1/responses`; the launcher exited 0 and saved the expected response text. |
| Redirect handling | `t1_04_official_launcher_rejects_redirects_and_unsafe_urls` | HTTP 302 was reported as an error. The mock captured exactly one `GET /v1/models`; its redirect target was not requested. |
| URL rejection before network use | Same test | Query-bearing, credential-bearing, and `file:` base URLs were rejected by the CLI before a provider request. The launcher output did not echo the synthetic query or user-info sentinel values. |
| Response size limits | `t1_04_official_launcher_bounds_json_and_streaming_responses` | Oversized JSON was rejected at the 10 MiB response limit. An SSE event beyond the 1 MiB event limit produced a controlled error saved with the run. |
| API key and saved-output privacy | `t1_04_official_launcher_auth_is_captured_and_secrets_are_not_persisted` | The mock captured `Authorization: Bearer llmeter-t1-04-synthetic-token`. It returned HTTP 401 with the synthetic key in an oversized error body. The saved JSON error showed truncation and redaction markers. The test read every saved file, including JSON, CSV, Markdown, and HTML, and found no raw sentinel key. |
| Reserved chat fields | `reserved_chat_fields_cannot_be_overridden_by_extra_parameters` | ProviderClient unit coverage rejected injection of all six reserved fields: `model`, `messages`, `stream`, `stream_options`, `max_tokens`, and `temperature`. The CLI does not expose this parameter; the launcher scenario verifies normal request fields instead. |

The launcher masks `--base-url` values in its display log while forwarding the actual argument to the child process. The invalid-URL assertions cover query and user-info sentinel non-disclosure.

## Hosted CI (separate gate)

[CI run 35861652500](https://github.com/CTCycle/LLMeter-local-benchmarks/actions/runs/35861652500) passed Windows, Ubuntu, macOS Intel, and macOS Apple silicon on commit `5e900ed2fbe17fc3bbab252251e779b4a23839a7`. The earlier run on implementation commit `a9787410fa8c6304a9f1fda0d09a257740170ebf` exposed non-Windows dead-code warnings in Windows-only fixture evidence fields and scenarios; the platform-scoped lint allowance in `5e900ed` resolved them. Hosted checks confirm cross-platform build/test gates and are recorded separately from the Windows launcher evidence above.

## Remaining boundary

T1-04 passes for the official Windows PowerShell launcher with this deterministic HTTP mock. Live-provider compatibility, other launchers, and T1-05 through T1-07 remain separate validation work.
