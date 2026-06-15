# Shared rules

## Scope

- Keep changes focused on the CLI, benchmark pipeline, provider integration, reporting, or result storage directly involved in the task.
- Prefer small edits to existing modules over broad structural rewrites.
- Do not add runtime services, background daemons, or external orchestration layers unless the product direction changes explicitly.

## Data shape rules

- Use strongly typed Rust structs for stable entities such as config, provider status, benchmark runs, and result records.
- Use `serde_json::Value` only where the payload is intentionally provider-defined or metric-defined.
- Keep persisted result shapes backward-readable when possible within the current codebase, but do not preserve obsolete branches or unused fields.

## CLI behavior rules

- Scriptable commands must produce deterministic, automation-friendly behavior.
- Interactive flows should remain thin wrappers around shared runner and report logic.
- New benchmark or report features should be accessible from both scriptable and interactive surfaces unless there is a clear reason not to expose them in both.

## Provider boundary rules

- Treat providers as external local services.
- Do not assume LLMeter controls provider startup, shutdown, model loading, or model installation.
- Keep API usage within documented OpenAI-compatible endpoints already supported by the client unless a new capability is being added intentionally.

## Persistence rules

- Save benchmark artifacts under the configured output directory only.
- Keep JSON as the canonical full-fidelity run format.
- Ensure new report or export features derive from `BenchmarkRun` rather than introducing disconnected storage formats.

## Cleanup rules

- Remove dead benchmark branches, unused prompt variants, and obsolete report fields when replacing them.
- Keep `assets/docs/project_index.md` aligned with any doc additions, deletions, or renames.

Last updated: 2026-06-15
