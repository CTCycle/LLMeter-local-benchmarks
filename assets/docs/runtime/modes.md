# Modes

## Interactive mode

Interactive mode starts when no subcommand is supplied or when `llmeter menu` is used, but only when both standard input and standard output are terminals. A piped or CI invocation prints command help and exits with usage status `2` instead of opening a menu.

Primary characteristics:

- terminal menu navigation through `inquire`
- guided model and benchmark selection
- guided export and report choices
- shared benchmark progress rendering
- shared report viewing and generation flows

This mode is intended for exploratory local usage.

## Scriptable mode

Scriptable mode starts when a subcommand is supplied, such as:

- `status`
- `providers list`
- `models`
- `show <model>`
- `bench list`
- `bench run ...`
- `report list`
- `report show`
- `report generate`

Primary characteristics:

- no interactive prompts
- explicit flags and subcommands
- automation-friendly exit codes
- repeatable output generation for CI or local batch runs
- benchmark progress and diagnostics on standard error, leaving standard output available for command data

## Report-only workflows

Saved results can be inspected without re-running benchmarks:

- `llmeter report list`
- `llmeter report show [result]`
- `llmeter report generate [result] --format md|html|both`

Interactive reports workspace exposes the same operations for users who prefer menu navigation.

## Shared runtime assumptions

All modes assume:

- a provider server is started externally when provider access is required
- the selected provider exposes an OpenAI-compatible `/v1` API
- results and reports are stored under the configured output directory

Last updated: 2026-07-18
