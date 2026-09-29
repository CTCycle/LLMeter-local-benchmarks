# Tier 4 provider preset contract evidence

Last updated: 2026-09-29

## Boundary

| Field | Value |
|---|---|
| Validation revision | `4fefea734cc62dac327308a518d5409b4f01a033` |
| Environment | Windows x86-64, PowerShell, locked Rust toolchain, deterministic loopback TCP fixture. |
| Provider installation | None. The fixture binds an ephemeral `127.0.0.1` port and never uses a provider default port. |
| Evidence lane | T4-01 preset contract matrix only; it is not live-provider certification. |

## Results

The catalog-derived fixture covered all 12 registered presets:

| Provider | Compatibility tier | Default `/v1` base URL |
|---|---|---|
| `ollama` | first-class | `http://localhost:11434/v1` |
| `lmstudio` | first-class | `http://localhost:1234/v1` |
| `llama-cpp` | first-class | `http://localhost:8080/v1` |
| `openai-compatible` | custom | `http://localhost:8000/v1` |
| `vllm` | known OpenAI-compatible | `http://localhost:8000/v1` |
| `sglang` | known OpenAI-compatible | `http://localhost:30000/v1` |
| `localai` | known OpenAI-compatible | `http://localhost:8080/v1` |
| `litellm` | known OpenAI-compatible | `http://localhost:4000/v1` |
| `tgi` | best effort | `http://localhost:8080/v1` |
| `text-generation-webui` | best effort | `http://localhost:5000/v1` |
| `jan` | best effort | `http://localhost:1337/v1` |
| `mlx-lm` | best effort | `http://localhost:8080/v1` |

The focused command passed:

```text
cargo test --locked --all-features --test mock_provider_e2e -- --test-threads=1
```

Result: **21 passed, 0 failed**.

The two catalog-derived T4-01 tests passed alongside the existing fixture
coverage:

- `every_registered_preset_obeys_the_baseline_openai_contract_fixture`
  validated catalog parsing, tier/default-URL metadata, model discovery,
  provider identity after explicit base-URL override, streaming and
  non-streaming Chat Completions, request paths/shapes, and controlled HTTP 501
  errors for unsupported Responses and Embeddings endpoints.
- `every_registered_preset_is_cli_selectable_with_an_explicit_base_url`
  selected every preset through the real CLI, used the same random fixture
  base URL, checked the selected provider identity in `status`, and verified a
  synthetic API key was absent from stdout, stderr, and fixture files.

Existing T1-04 transport evidence additionally covers authorization capture,
multiline SSE, bounded error bodies, URL safety, and persisted JSON/CSV/Markdown/
HTML secret inspection. Those checks remain shared transport evidence and are
not duplicated per preset.

## Interpretation

T4-01 is **PASS**. Every current preset is deterministic-contract validated,
and the catalog-derived loop makes a future `ProviderKind` addition require
corresponding fixture maintenance. This result proves protocol/preset behavior
only. It does not certify any provider server version, model, extension,
optional endpoint, or best-effort deployment.
