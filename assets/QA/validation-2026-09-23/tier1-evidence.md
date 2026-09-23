# T1-01 CLI catalog stdout snapshots

Captured from the current source build at commit 056817717e07fb64c14f13e0b27e83e5ac5b85aa on Windows x86-64 with PowerShell 7.6.6, Cargo 1.98.0, and Rust 1.98.0. No provider was contacted.

Each fenced block contains the process standard output. Exit status and standard error are recorded separately.

## global-help

Command: llmeter --help
Exit code: 0
Standard error: empty

Standard output:
~~~text
Benchmark local OpenAI-compatible LLM providers from a modern CLI.

Usage: llmeter.exe [OPTIONS] [COMMAND]

Commands:
  status     Check provider /v1 endpoint reachability, health, and exposed model count
  providers  List provider presets with compatibility tiers and default URLs
  models     List local models exposed by the selected provider
  show       Show model metadata from the selected provider
  bench      Run and manage LLM benchmarks: generation, latency, performance, and quality
  report     View saved results and export formatted Markdown or HTML reports
  quality    Quality benchmark plans for lighteval, inspect-ai, lm-eval-harness, and SWE-bench
  install    Install LLMeter into the managed CLI home bin directory
  update     Update the managed LLMeter install
  uninstall  Uninstall the managed LLMeter CLI
  help       Show built-in help. Use a topic such as providers, bench, reports, install, or examples
  menu       Open the interactive main menu

Options:
      --provider <PROVIDER>      Provider preset. Use `llmeter providers list` for the full catalog. [possible values: ollama, lmstudio, llama-cpp, openai-compatible, vllm, sglang, localai, litellm, tgi, text-generation-webui, jan, mlx-lm]
      --base-url <BASE_URL>      OpenAI-compatible /v1 base URL
      --timeout <TIMEOUT>        HTTP request timeout in seconds
      --output-dir <OUTPUT_DIR>  Directory for result and report files
  -h, --help                     Print help
  -V, --version                  Print version
~~~

## providers-list

Command: llmeter providers list
Exit code: 0
Standard error: empty

Standard output:
~~~text
╭───────────────────────┬─────────────────────────┬───────────────────────────┬─────────────────────────────────────────╮
│ Provider presets and compatibility tiers                                                                              │
├───────────────────────┼─────────────────────────┼───────────────────────────┼─────────────────────────────────────────┤
│ Provider              │ Tier                    │ Default /v1 base URL      │ Notes                                   │
│ ollama                │ first-class             │ http://localhost:11434/v1 │ Existing first-class target             │
│ lmstudio              │ first-class             │ http://localhost:1234/v1  │ Existing first-class target             │
│ llama-cpp             │ first-class             │ http://localhost:8080/v1  │ Existing first-class target             │
│ openai-compatible     │ custom                  │ http://localhost:8000/v1  │ User-supplied local /v1 server          │
│ vllm                  │ known OpenAI-compatible │ http://localhost:8000/v1  │ OpenAI-compatible server preset         │
│ sglang                │ known OpenAI-compatible │ http://localhost:30000/v1 │ OpenAI-compatible server preset         │
│ localai               │ known OpenAI-compatible │ http://localhost:8080/v1  │ OpenAI-compatible server preset         │
│ litellm               │ known OpenAI-compatible │ http://localhost:4000/v1  │ OpenAI-compatible proxy preset          │
│ tgi                   │ best effort             │ http://localhost:8080/v1  │ Requires OpenAI-compatible router mode  │
│ text-generation-webui │ best effort             │ http://localhost:5000/v1  │ API shape can vary by extension         │
│ jan                   │ best effort             │ http://localhost:1337/v1  │ Local API behavior can vary by version  │
│ mlx-lm                │ best effort             │ http://localhost:8080/v1  │ OpenAI-compatible server shape can vary │
╰───────────────────────┴─────────────────────────┴───────────────────────────┴─────────────────────────────────────────╯
~~~

## bench-list

Command: llmeter bench list
Exit code: 0
Standard error: empty

Standard output:
~~~text
╭───┬────────────┬──────────────────────┬─────────────────────────────────┬─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮
│ Benchmark catalog                                                                                                                                                                                                                         │
├───┼────────────┼──────────────────────┼─────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│ # │ Suite      │ ID                   │ Name                            │ Description                                                                                                                                                     │
│ 1 │ llm        │ chat-generation      │ Basic generation latency        │ Measures wall time, time to first token, usage fields, and output token throughput.                                                                             │
│ 2 │ llm        │ responses-generation │ Responses API generation        │ Measures generation through /v1/responses when the provider supports it.                                                                                        │
│ 3 │ llm        │ consistency          │ Response consistency            │ Repeats the same prompt and reports exact-match and pairwise text similarity.                                                                                   │
│ 4 │ llm        │ prompt-sizes         │ Performance across prompt sizes │ Runs short, medium, and long prompts to compare client-observed end-to-end timing and generation metrics. Prompt-processing time is not measured independently. │
│ 5 │ llm        │ structured-output    │ Structured JSON output          │ Requests schema-constrained JSON and validates the returned object shape.                                                                                       │
│ 6 │ llm        │ tool-calling         │ Function/tool calling           │ Requests a tool call and validates the selected function and arguments.                                                                                         │
│ 7 │ embeddings │ embeddings           │ Embeddings API                  │ Measures /v1/embeddings latency and returned vector dimensions.                                                                                                 │
╰───┴────────────┴──────────────────────┴─────────────────────────────────┴─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯
~~~

## quality-list

Command: llmeter quality list
Exit code: 0
Standard error: empty

Standard output:
~~~text
╭────────────────────────┬────────────────────────┬─────────────────────┬─────────────────┬──────────────┬───────────╮
│ Quality benchmark catalog                                                                                          │
├────────────────────────┼────────────────────────┼─────────────────────┼─────────────────┼──────────────┼───────────┤
│ ID                     │ Name                   │ Family              │ Framework       │ Metric       │ Code exec │
│ mmlu                   │ MMLU                   │ Knowledge           │ lighteval       │ accuracy     │ no        │
│ gsm8k                  │ GSM8K                  │ Reasoning           │ lighteval       │ accuracy     │ no        │
│ arc-challenge          │ ARC Challenge          │ Reasoning           │ lm-eval-harness │ accuracy     │ no        │
│ hellaswag              │ HellaSwag              │ Reasoning           │ lm-eval-harness │ accuracy     │ no        │
│ truthfulqa             │ TruthfulQA             │ Truthfulness        │ inspect-ai      │ truthfulness │ no        │
│ winogrande             │ Winogrande             │ Reasoning           │ lm-eval-harness │ accuracy     │ no        │
│ humaneval              │ HumanEval              │ Code                │ inspect-ai      │ pass@1       │ yes       │
│ swe-bench-lite         │ SWE-bench Lite         │ SoftwareEngineering │ swe-bench       │ % resolved   │ yes       │
│ swe-bench-verified     │ SWE-bench Verified     │ SoftwareEngineering │ swe-bench       │ % resolved   │ yes       │
│ swe-bench-full         │ SWE-bench Full         │ SoftwareEngineering │ swe-bench       │ % resolved   │ yes       │
│ swe-bench-multilingual │ SWE-bench Multilingual │ SoftwareEngineering │ swe-bench       │ % resolved   │ yes       │
╰────────────────────────┴────────────────────────┴─────────────────────┴─────────────────┴──────────────┴───────────╯
~~~

## help-overview

Command: llmeter help
Exit code: 0
Standard error: empty

Standard output:
~~~text
LLMeter — Benchmark local OpenAI-compatible LLM providers.
Help topics: providers, bench, reports, install, examples
Use `llmeter help <topic>` for details, or `llmeter --help` for CLI reference.
~~~

## help-providers

Command: llmeter help providers
Exit code: 0
Standard error: empty

Standard output:
~~~text
Providers — OpenAI-compatible provider presets:
  Use `llmeter providers list` for the full catalog with compatibility tiers
  and default /v1 base URLs (Ollama, LM Studio, llama.cpp, vLLM, etc.).
  Use `llmeter providers set <name>` to persist a default provider.
~~~

## help-bench

Command: llmeter help bench
Exit code: 0
Standard error: empty

Standard output:
~~~text
Benchmarks — Generation latency, consistency, performance under load:
  Standard benchmarks  : llmeter bench run (chat, JSON, tool calls, embeddings)
  Performance bench    : llmeter bench perf (latency/throughput/TTFT profiles)
  Interactive menu     : llmeter menu

Examples:
  llmeter bench list --suite llm
  llmeter --provider lmstudio bench run --suite llm --models all --benchmarks all
  llmeter bench run --provider ollama --suite embeddings --models all --benchmarks all
  llmeter bench perf --models all --profile smoke --export json --report md
  llmeter bench perf --models all --profile latency --probe-capabilities --telemetry detailed
~~~

## help-benchmarks-alias

Command: llmeter help benchmarks
Exit code: 0
Standard error: empty

Standard output:
~~~text
Benchmarks — Generation latency, consistency, performance under load:
  Standard benchmarks  : llmeter bench run (chat, JSON, tool calls, embeddings)
  Performance bench    : llmeter bench perf (latency/throughput/TTFT profiles)
  Interactive menu     : llmeter menu

Examples:
  llmeter bench list --suite llm
  llmeter --provider lmstudio bench run --suite llm --models all --benchmarks all
  llmeter bench run --provider ollama --suite embeddings --models all --benchmarks all
  llmeter bench perf --models all --profile smoke --export json --report md
  llmeter bench perf --models all --profile latency --probe-capabilities --telemetry detailed
~~~

## help-reports

Command: llmeter help reports
Exit code: 0
Standard error: empty

Standard output:
~~~text
Reports — View and export saved benchmark results:
  llmeter report list       List recent JSON result files and generated reports
  llmeter report show       Display a saved result as a terminal report
  llmeter report generate   Export a saved result as Markdown or HTML
~~~

## help-install

Command: llmeter help install
Exit code: 0
Standard error: empty

Standard output:
~~~text
Install and lifecycle — Manage the LLMeter binary:
  llmeter install           Install to the managed CLI home directory
  llmeter update            Replace with a newer binary
  llmeter uninstall         Remove the managed install

Examples:
  llmeter install
  llmeter install --force
  llmeter update --source C:\path\to\llmeter.exe
  llmeter uninstall
  llmeter uninstall --purge-home
~~~

## help-lifecycle-alias

Command: llmeter help lifecycle
Exit code: 0
Standard error: empty

Standard output:
~~~text
Install and lifecycle — Manage the LLMeter binary:
  llmeter install           Install to the managed CLI home directory
  llmeter update            Replace with a newer binary
  llmeter uninstall         Remove the managed install

Examples:
  llmeter install
  llmeter install --force
  llmeter update --source C:\path\to\llmeter.exe
  llmeter uninstall
  llmeter uninstall --purge-home
~~~

## help-examples

Command: llmeter help examples
Exit code: 0
Standard error: empty

Standard output:
~~~text
Quick examples — Common workflows:
  llmeter status                                Check your provider is reachable
  llmeter providers set ollama                  Set Ollama as the default
  llmeter models                                List local models
  llmeter menu                                  Open the interactive menu
  llmeter bench run --suite llm --models all \
    --benchmarks all --runs 3 --max-tokens 128  Run all LLM benchmarks
  llmeter bench perf --models all --profile smoke  Quick performance check
  llmeter quality list                          Browse quality tasks
~~~

## help-quality

Command: llmeter help quality
Exit code: 0
Standard error: empty

Standard output:
~~~text
LLMeter — Benchmark local OpenAI-compatible LLM providers.
Help topics: providers, bench, reports, install, examples
Use `llmeter help <topic>` for details, or `llmeter --help` for CLI reference.
~~~

## help-invalid

Command: llmeter help definitely-invalid
Exit code: 0
Standard error: empty

Standard output:
~~~text
LLMeter — Benchmark local OpenAI-compatible LLM providers.
Help topics: providers, bench, reports, install, examples
Use `llmeter help <topic>` for details, or `llmeter --help` for CLI reference.
~~~
