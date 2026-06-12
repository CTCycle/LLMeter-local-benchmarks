from __future__ import annotations

from collections.abc import Iterable, Sequence
from pathlib import Path
from typing import Any

from rich import box
from rich.console import Console, Group
from rich.markdown import Markdown
from rich.panel import Panel
from rich.prompt import Confirm, IntPrompt, Prompt
from rich.rule import Rule
from rich.table import Table
from rich.text import Text

from llmeter.benchmarks.registry import default_registry
from llmeter.ollama.server import ServerStatus
from llmeter.reporting import build_summary_rows, render_markdown_report
from llmeter.results import BenchmarkRun

console = Console()


APP_TITLE = "LLMeter"
APP_SUBTITLE = "Local LLM benchmarking for Ollama"


def human_size(num_bytes: int | None) -> str:
    if num_bytes is None:
        return ""
    units = ["B", "KB", "MB", "GB", "TB"]
    value = float(num_bytes)
    for unit in units:
        if value < 1024 or unit == units[-1]:
            return f"{value:.1f} {unit}" if unit != "B" else f"{int(value)} B"
        value /= 1024
    return f"{num_bytes} B"


def human_bool(value: bool) -> str:
    return "yes" if value else "no"


def section(title: str) -> None:
    console.print(Rule(title, style="dim"))


def print_header() -> None:
    title = Text(APP_TITLE, style="bold")
    subtitle = Text(APP_SUBTITLE, style="dim")
    console.print(Panel(Group(title, subtitle), box=box.ROUNDED, padding=(1, 2)))


def print_status_panel(status: ServerStatus, *, cli_version: str | None = None) -> None:
    table = Table(box=box.SIMPLE, show_header=False, pad_edge=False)
    table.add_column("Check", style="dim")
    table.add_column("Value")
    table.add_row("Ollama installed", human_bool(status.installed))
    table.add_row("Executable", status.executable or "not found")
    table.add_row("Server running", human_bool(status.running))
    table.add_row("API version", status.version or "unknown")
    table.add_row("Tracked PID", str(status.tracked_pid) if status.tracked_pid else "none")
    if cli_version:
        table.add_row("Ollama CLI", cli_version)
    console.print(Panel(table, title="Status", box=box.ROUNDED))


def print_table(headers: list[str], rows: Iterable[Iterable[Any]], *, title: str | None = None) -> None:
    table = Table(title=title, box=box.SIMPLE_HEAVY, show_lines=False)
    for header in headers:
        table.add_column(header)
    count = 0
    for row in rows:
        table.add_row(*["" if cell is None else str(cell) for cell in row])
        count += 1
    if count == 0:
        console.print("[dim]No rows.[/dim]")
        return
    console.print(table)


def print_models(models: list[dict[str, Any]]) -> None:
    if not models:
        console.print(Panel("No local Ollama models found. Install one with: [bold]ollama pull <model>[/bold]", title="Models"))
        return
    table = Table(title="Installed Ollama models", box=box.ROUNDED)
    table.add_column("#", justify="right", style="dim")
    table.add_column("Model", style="bold")
    table.add_column("Size", justify="right")
    table.add_column("Params")
    table.add_column("Quant")
    table.add_column("Family")
    table.add_column("Modified", style="dim")
    for index, model in enumerate(models, start=1):
        details = model.get("details") or {}
        table.add_row(
            str(index),
            str(model.get("name") or model.get("model") or ""),
            human_size(model.get("size")),
            str(details.get("parameter_size", "")),
            str(details.get("quantization_level", "")),
            str(details.get("family", "")),
            str(model.get("modified_at", "")),
        )
    console.print(table)


def print_benchmark_catalog() -> None:
    registry = default_registry()
    table = Table(title="Benchmark catalog", box=box.ROUNDED)
    table.add_column("#", justify="right", style="dim")
    table.add_column("ID", style="bold")
    table.add_column("Name")
    table.add_column("Description", overflow="fold")
    for index, benchmark in enumerate(registry.all(), start=1):
        table.add_row(str(index), benchmark.id, benchmark.name, benchmark.description)
    console.print(table)


def summarize_run(run: BenchmarkRun) -> None:
    errors = [record for record in run.results if record.error]
    ok = len(run.results) - len(errors)
    header = Table.grid(expand=True)
    header.add_column()
    header.add_column(justify="right")
    header.add_row("Run ID", run.run_id)
    header.add_row("Created", run.created_at)
    header.add_row("Models", ", ".join(run.models))
    header.add_row("Records", f"{len(run.results)} total, {ok} ok, {len(errors)} errors")
    console.print(Panel(header, title="Benchmark run", box=box.ROUNDED))

    summary_rows = build_summary_rows(run)
    table = Table(title="Aggregated summary", box=box.ROUNDED)
    table.add_column("Model", style="bold")
    table.add_column("Benchmark")
    table.add_column("Prompt")
    table.add_column("Runs", justify="right")
    table.add_column("Errors", justify="right")
    table.add_column("Avg wall ms", justify="right")
    table.add_column("Avg TTFT ms", justify="right")
    table.add_column("Avg tok/s", justify="right")
    table.add_column("Similarity", justify="right")
    for row in summary_rows:
        table.add_row(
            row.model,
            row.benchmark_id,
            row.prompt_name or "all",
            str(row.records),
            str(row.errors),
            _fmt(row.avg_wall_time_ms),
            _fmt(row.avg_time_to_first_token_ms),
            _fmt(row.avg_tokens_per_second),
            _fmt(row.mean_similarity, digits=4),
        )
    console.print(table)

    if errors:
        error_table = Table(title="Errors", box=box.SIMPLE_HEAVY)
        error_table.add_column("Model")
        error_table.add_column("Benchmark")
        error_table.add_column("Prompt")
        error_table.add_column("Error", overflow="fold")
        for record in errors[:20]:
            error_table.add_row(record.model, record.benchmark_id, record.prompt_name or "", record.error or "")
        console.print(error_table)


def print_terminal_report(run: BenchmarkRun) -> None:
    console.print(Markdown(render_markdown_report(run)))


def choose_from_menu(prompt: str, choices: Sequence[str], *, allow_all: bool = True, multi: bool = True) -> list[str]:
    if not choices:
        return []
    table = Table(title=prompt, box=box.SIMPLE)
    table.add_column("#", justify="right", style="dim")
    table.add_column("Option")
    for index, choice in enumerate(choices, start=1):
        table.add_row(str(index), choice)
    if allow_all:
        table.add_row("all", "All")
    console.print(table)

    hint = "comma-separated numbers" if multi else "number"
    raw = Prompt.ask(f"Select {hint}", default="all" if allow_all else "1").strip()
    if allow_all and raw.lower() in {"all", "a", "*"}:
        return list(choices)
    selected: list[str] = []
    tokens = raw.replace(" ", "").split(",") if multi else [raw]
    for token in tokens:
        if not token:
            continue
        if token.isdigit():
            idx = int(token)
            if 1 <= idx <= len(choices):
                selected.append(choices[idx - 1])
        elif token in choices:
            selected.append(token)
    return list(dict.fromkeys(selected))


def choose_number(prompt: str, *, default: int, minimum: int = 1, maximum: int | None = None) -> int:
    while True:
        value = IntPrompt.ask(prompt, default=default)
        if value < minimum:
            console.print(f"[red]Value must be at least {minimum}.[/red]")
            continue
        if maximum is not None and value > maximum:
            console.print(f"[red]Value must be at most {maximum}.[/red]")
            continue
        return value


def ask_yes_no(prompt: str, *, default: bool = True) -> bool:
    return Confirm.ask(prompt, default=default)


def ask_choice(prompt: str, choices: Sequence[str], *, default: str) -> str:
    return Prompt.ask(prompt, choices=list(choices), default=default)


def pause() -> None:
    Prompt.ask("Press Enter to continue", default="")


def print_saved_paths(paths: list[Path]) -> None:
    if not paths:
        return
    table = Table(title="Saved files", box=box.SIMPLE)
    table.add_column("Type")
    table.add_column("Path", overflow="fold")
    for path in paths:
        table.add_row(path.suffix.lstrip(".") or "file", str(path))
    console.print(table)


def print_file_list(paths: list[Path], *, title: str) -> None:
    if not paths:
        console.print(Panel("No files found.", title=title))
        return
    table = Table(title=title, box=box.ROUNDED)
    table.add_column("#", justify="right", style="dim")
    table.add_column("File", overflow="fold")
    table.add_column("Modified", style="dim")
    for index, path in enumerate(paths, start=1):
        modified = ""
        try:
            modified = str(path.stat().st_mtime)
        except OSError:
            pass
        table.add_row(str(index), str(path), modified)
    console.print(table)


def menu(title: str, options: Sequence[str]) -> int:
    table = Table(title=title, box=box.ROUNDED, show_header=False)
    table.add_column("#", justify="right", style="dim")
    table.add_column("Action")
    for index, option in enumerate(options, start=1):
        table.add_row(str(index), option)
    console.print(table)
    return choose_number("Selection", default=1, minimum=1, maximum=len(options))


def _fmt(value: Any, *, digits: int = 2) -> str:
    if value is None or value == "":
        return ""
    if isinstance(value, float):
        return f"{value:.{digits}f}"
    return str(value)
