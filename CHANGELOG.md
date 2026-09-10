# Changelog

All notable changes are documented here.

## 0.4.0 - 2026-09-10

- Establish canonical provider metadata, base-URL resolution, model identity, typed output formats, and shared performance-profile defaults across scriptable and interactive workflows.
- Enforce strict result schema `3.0` and mandatory run metadata; remove obsolete command aliases and legacy runtime compatibility fallbacks.
- Improve fresh provider validation, benchmark planning, performance safety limits, capability probes, load estimates, telemetry boundaries, and report aggregation.
- Preserve non-secret token metadata while redacting sensitive output values and keeping response previews private by default.
- Add regression coverage for canonical quality plans, result compatibility, performance planning, provider behavior, reporting, and interactive menu flows.
- Fix PowerShell launcher argument forwarding so documented flags such as `--provider` and `--version` reach the LLMeter binary unchanged.
- Refresh locked compatible dependencies, including the yanked `chacha20` package update, and validate the Windows release packaging path.

## 0.3.0 - 2026-08-17

- Normalize new benchmark result files to schema version `2.4`.
- Correct streaming metric semantics: chunk arrival timing is separate from ITL, which requires provider-reported output token usage.
- Replace misleading cold/native load modes with an explicitly client-observed first-request estimate.
- Separate cached model catalogs from fresh provider measurements and add explicit invalidation.
- Report token usage sample counts and coverage instead of silently dividing partial usage by all successful requests.
- Correct consistency exact-match ratios to use matching response pairs.
- Centralize standard benchmark input validation at both CLI and runner boundaries.
- Normalize nested prompt cancellation and interruption handling.
- Add performance matrix dry-run estimates and request-budget safety checks.
- Harden Markdown and HTML report escaping for untrusted model, benchmark, error, preview, and metric values.
- Add mock OpenAI-compatible provider E2E coverage for models, streaming chat benchmarks, unsupported endpoint errors, and report generation.
- Add tag-gated checksummed Windows, Linux, and macOS release archives with archive smoke validation and GitHub artifact provenance attestations.
- Publish the first public GitHub distribution and prepare the package for the manual initial crates.io publication (`cargo install llmeter --locked`).
- Make provider status return exit code `1` when unreachable, make performance telemetry opt-in, and harden managed install purge and replacement behavior.
- Refresh compatible dependencies, remove the unmaintained fuzzy-matcher chain, and use release-oriented optimization.

Last updated: 2026-09-10
