# Contributing

## Development Setup

Install the stable Rust toolchain, then run:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -- --test-threads=1
```

Use `--test-threads=1` because some configuration tests intentionally mutate process environment variables.

## Scope

Keep changes focused on local OpenAI-compatible provider benchmarking, report generation, provider capability checks, and reproducible local runs. Quality framework commands are dry-run planning adapters unless a change explicitly expands that contract.

## Pull Request Checklist

- Update docs under `assets/docs/` when behavior changes.
- Add or update tests for CLI parsing, provider behavior, schema changes, report rendering, or benchmark math.
- Avoid committing benchmark outputs, local model data, generated target directories, or provider logs.
- Do not include API keys, private prompts, internal hostnames, or sensitive model outputs in examples.

Last updated: 2026-07-01
