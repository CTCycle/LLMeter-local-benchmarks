# Implementation plan

## Phase 1, repository bootstrap

- Create Python 3.14 project metadata.
- Add `src/` package layout.
- Add `pytest` test setup.
- Add GitHub Actions CI targeting Python 3.14.
- Add `.python-version`, `.gitignore`, README, and MIT license.

Status: implemented.

## Phase 2, Ollama integration

- Add a small REST client for `/api/version`, `/api/tags`, `/api/show`, `/api/generate`, and `/api/ps`.
- Add server manager for executable detection, server health, start, and stop.
- Track only the PID started by this CLI unless forced.

Status: implemented.

## Phase 3, benchmark core

- Define benchmark context and result dataclasses.
- Add registry-based benchmark discovery.
- Implement generation latency benchmark.
- Implement consistency benchmark.
- Implement short, medium, and long prompt benchmark.
- Convert Ollama duration fields into milliseconds.
- Derive tokens per second when possible.

Status: implemented.

## Phase 4, modern interactive CLI

- Make the default command open an interactive main menu.
- Add a benchmark workspace.
- Add guided model and benchmark selection.
- Add Rich tables, panels, prompts, and terminal summaries.
- Keep non-interactive subcommands for repeatable benchmark runs.

Status: implemented.

## Phase 5, reporting

- Save raw JSON and CSV files.
- Generate Markdown reports suitable for GitHub.
- Generate HTML reports suitable for local browser viewing.
- Add terminal report rendering for saved JSON result files.
- Add report subcommands and interactive report menu.

Status: implemented.

## Phase 6, next useful improvements

These are deliberately not part of the first commit unless needed immediately:

- Add warmup runs separate from measured runs.
- Add model unload option before or after each benchmark.
- Add system metadata capture, CPU, GPU, RAM, OS, and Ollama version.
- Add benchmark comparison across multiple saved runs.
- Add optional charts in HTML reports.
- Add an optional machine-readable schema version.
- Add concurrent runs only after serial correctness is reliable.
- Add provider abstraction only if a second provider is actually implemented.
