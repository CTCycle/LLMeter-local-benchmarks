from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys
from typing import Any

from rich.prompt import Prompt

from ollama_bench.benchmarks.registry import default_registry
from ollama_bench.config import AppConfig
from ollama_bench.errors import OllamaBenchError, OllamaServerError
from ollama_bench.ollama.client import OllamaClient
from ollama_bench.ollama.server import OllamaServerManager
from ollama_bench.reporting import save_html_report, save_markdown_report
from ollama_bench.results import BenchmarkRun, ResultStore
from ollama_bench.runner import installed_model_names, run_benchmarks
from ollama_bench.ui import (
    ask_choice,
    ask_yes_no,
    choose_from_menu,
    choose_number,
    console,
    menu,
    pause,
    print_benchmark_catalog,
    print_file_list,
    print_header,
    print_models,
    print_saved_paths,
    print_status_panel,
    print_terminal_report,
    summarize_run,
)


EXPORT_CHOICES = ["json", "csv", "both", "none"]
REPORT_CHOICES = ["md", "html", "both", "none"]


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="ollama-bench",
        description="Benchmark local Ollama models from a modern interactive CLI.",
    )
    parser.add_argument("--host", default=None, help="Ollama host, default from OLLAMA_HOST or http://localhost:11434")
    parser.add_argument("--timeout", type=float, default=None, help="HTTP request timeout in seconds")
    parser.add_argument("--output-dir", type=Path, default=None, help="Directory for result and report files")

    subparsers = parser.add_subparsers(dest="command")

    subparsers.add_parser("status", help="Show Ollama installation and server status")

    server = subparsers.add_parser("server", help="Manage the local Ollama server")
    server_sub = server.add_subparsers(dest="server_command", required=True)
    server_sub.add_parser("status", help="Show server status")
    server_sub.add_parser("start", help="Start 'ollama serve' if not running")
    stop = server_sub.add_parser("stop", help="Stop a server started by this CLI")
    stop.add_argument("--force", action="store_true", help="Force-stop ollama server processes, not only the tracked process")

    models = subparsers.add_parser("models", help="List installed local Ollama models")
    models.add_argument("--json", action="store_true", help="Print raw JSON")

    show = subparsers.add_parser("show", help="Show model metadata from Ollama")
    show.add_argument("model", help="Installed model name")

    bench = subparsers.add_parser("bench", help="Benchmark menu and commands")
    bench_sub = bench.add_subparsers(dest="bench_command")
    bench_sub.add_parser("list", help="List available benchmarks")

    run = bench_sub.add_parser("run", help="Run benchmarks")
    run.add_argument("--models", required=False, help="Comma-separated model names, or 'all'")
    run.add_argument("--benchmarks", required=False, help="Comma-separated benchmark ids, or 'all'")
    run.add_argument("--runs", type=int, default=None, help="Repeated runs per benchmark")
    run.add_argument("--num-predict", type=int, default=None, help="Ollama num_predict option")
    run.add_argument("--temperature", type=float, default=None, help="Ollama temperature option")
    run.add_argument("--export", choices=EXPORT_CHOICES, default="both", help="Raw result export format")
    run.add_argument("--report", choices=REPORT_CHOICES, default="both", help="Formatted report export format")
    run.add_argument("--start-server", action="store_true", help="Start Ollama if the server is not running")
    run.add_argument("--option", action="append", default=[], help="Extra Ollama option as key=value, repeatable")
    bench_sub.add_parser("menu", help="Open the interactive benchmark menu")

    report = subparsers.add_parser("report", help="View and generate formatted benchmark reports")
    report_sub = report.add_subparsers(dest="report_command")
    report_sub.add_parser("list", help="List recent JSON result files and generated reports")
    show_report = report_sub.add_parser("show", help="Render a saved JSON result as a terminal report")
    show_report.add_argument("result", nargs="?", type=Path, help="JSON result file. Defaults to latest.")
    generate_report = report_sub.add_parser("generate", help="Generate Markdown and/or HTML reports from a saved JSON result")
    generate_report.add_argument("result", nargs="?", type=Path, help="JSON result file. Defaults to latest.")
    generate_report.add_argument("--format", choices=REPORT_CHOICES, default="both", help="Report format")

    subparsers.add_parser("menu", help="Open the interactive main menu")
    return parser


def parse_csv(value: str | None) -> list[str] | None:
    if value is None:
        return None
    return [item.strip() for item in value.split(",") if item.strip()]


def parse_options(values: list[str]) -> dict[str, Any]:
    parsed: dict[str, Any] = {}
    for item in values:
        if "=" not in item:
            raise OllamaBenchError(f"Invalid --option '{item}'. Use key=value.")
        key, value = item.split("=", 1)
        key = key.strip()
        value = value.strip()
        if not key:
            raise OllamaBenchError(f"Invalid --option '{item}'. Empty key.")
        try:
            parsed[key] = json.loads(value)
        except json.JSONDecodeError:
            parsed[key] = value
    return parsed


def make_runtime(args: argparse.Namespace) -> tuple[AppConfig, OllamaClient, OllamaServerManager]:
    config = AppConfig()
    if getattr(args, "host", None):
        config.host = args.host
    if getattr(args, "timeout", None):
        config.timeout = args.timeout
    if getattr(args, "output_dir", None):
        config.output_dir = args.output_dir
    client = OllamaClient(config.api_base_url, timeout=config.timeout)
    manager = OllamaServerManager(client, config.state_dir)
    return config, client, manager


def command_status(manager: OllamaServerManager) -> None:
    print_status_panel(manager.status(), cli_version=manager.installed_version_cli())


def command_bench_run(args: argparse.Namespace, config: AppConfig, client: OllamaClient, manager: OllamaServerManager) -> None:
    if args.start_server and not client.is_running():
        manager.start()

    available_models = installed_model_names(client)
    if args.models is None:
        raise OllamaBenchError("--models is required for non-interactive benchmark runs. Use 'all' or a comma-separated list.")
    selected_models = available_models if args.models.strip().lower() == "all" else parse_csv(args.models) or []

    selected_benchmarks: list[str] | None
    all_benchmarks = False
    if args.benchmarks is None or args.benchmarks.strip().lower() == "all":
        selected_benchmarks = None
        all_benchmarks = True
    else:
        selected_benchmarks = parse_csv(args.benchmarks) or []

    run = execute_benchmarks(
        config=config,
        client=client,
        model_names=selected_models,
        benchmark_ids=selected_benchmarks,
        all_benchmarks=all_benchmarks,
        runs=args.runs or config.default_runs,
        num_predict=args.num_predict or config.default_num_predict,
        temperature=config.default_temperature if args.temperature is None else args.temperature,
        extra_options=parse_options(args.option),
    )
    saved = save_outputs(config, run, export=args.export, report=args.report)
    print_saved_paths(saved)


def execute_benchmarks(
    *,
    config: AppConfig,
    client: OllamaClient,
    model_names: list[str],
    benchmark_ids: list[str] | None,
    all_benchmarks: bool,
    runs: int,
    num_predict: int,
    temperature: float,
    extra_options: dict[str, Any] | None = None,
) -> BenchmarkRun:
    console.print("[bold]Running benchmarks...[/bold]")
    run = run_benchmarks(
        client=client,
        config=config,
        model_names=model_names,
        benchmark_ids=benchmark_ids,
        all_benchmarks=all_benchmarks,
        runs=runs,
        num_predict=num_predict,
        temperature=temperature,
        extra_options=extra_options,
    )
    summarize_run(run)
    return run


def save_outputs(config: AppConfig, run: BenchmarkRun, *, export: str, report: str) -> list[Path]:
    store = ResultStore(config.output_dir)
    saved: list[Path] = []
    if export in {"json", "both"}:
        saved.append(store.save_json(run))
    if export in {"csv", "both"}:
        saved.append(store.save_csv(run))
    if report in {"md", "both"}:
        saved.append(save_markdown_report(run, config.output_dir))
    if report in {"html", "both"}:
        saved.append(save_html_report(run, config.output_dir))
    return saved


def benchmark_menu(config: AppConfig, client: OllamaClient, manager: OllamaServerManager) -> None:
    while True:
        print_header()
        choice = menu(
            "Benchmark workspace",
            [
                "View available benchmark tests",
                "Run a guided benchmark",
                "View latest result as terminal report",
                "Generate report from saved result",
                "Back",
            ],
        )
        if choice == 1:
            print_benchmark_catalog()
            pause()
        elif choice == 2:
            guided_benchmark_run(config, client, manager)
            pause()
        elif choice == 3:
            show_latest_report(config)
            pause()
        elif choice == 4:
            generate_report_interactive(config)
            pause()
        elif choice == 5:
            return


def guided_benchmark_run(config: AppConfig, client: OllamaClient, manager: OllamaServerManager) -> None:
    if not client.is_running():
        if ask_yes_no("Ollama server is not running. Start it now?", default=True):
            try:
                manager.start()
            except OllamaServerError as exc:
                console.print(f"[red]Could not start Ollama:[/red] {exc}")
                return
        else:
            console.print("[yellow]Benchmark cancelled.[/yellow]")
            return

    models = installed_model_names(client)
    selected_models = choose_from_menu("Choose model or models", models, allow_all=True, multi=True)
    if not selected_models:
        console.print("[yellow]No models selected.[/yellow]")
        return

    registry = default_registry()
    benchmark_ids = registry.ids()
    selected_benchmark_ids = choose_from_menu("Choose benchmark or benchmarks", benchmark_ids, allow_all=True, multi=True)
    if not selected_benchmark_ids:
        console.print("[yellow]No benchmarks selected.[/yellow]")
        return
    all_benchmarks = set(selected_benchmark_ids) == set(benchmark_ids)

    runs = choose_number("Runs per benchmark", default=config.default_runs, minimum=1)
    num_predict = choose_number("num_predict", default=config.default_num_predict, minimum=1)
    raw_temperature = Prompt.ask("temperature", default=str(config.default_temperature))
    try:
        temperature = float(raw_temperature)
    except ValueError:
        console.print("[yellow]Invalid temperature, using default.[/yellow]")
        temperature = config.default_temperature

    export = ask_choice("Save raw results", EXPORT_CHOICES, default="both")
    report = ask_choice("Generate formatted report", REPORT_CHOICES, default="both")

    run = execute_benchmarks(
        config=config,
        client=client,
        model_names=selected_models,
        benchmark_ids=None if all_benchmarks else selected_benchmark_ids,
        all_benchmarks=all_benchmarks,
        runs=runs,
        num_predict=num_predict,
        temperature=temperature,
    )
    saved = save_outputs(config, run, export=export, report=report)
    print_saved_paths(saved)


def report_menu(config: AppConfig) -> None:
    while True:
        print_header()
        choice = menu(
            "Reports",
            [
                "List saved result and report files",
                "View latest result as terminal report",
                "Generate Markdown or HTML report",
                "Back",
            ],
        )
        if choice == 1:
            list_report_files(config)
            pause()
        elif choice == 2:
            show_latest_report(config)
            pause()
        elif choice == 3:
            generate_report_interactive(config)
            pause()
        elif choice == 4:
            return


def list_report_files(config: AppConfig) -> None:
    store = ResultStore(config.output_dir)
    print_file_list(store.latest_json_files(), title="Saved JSON results")
    print_file_list(store.latest_report_files(), title="Generated reports")


def show_latest_report(config: AppConfig) -> None:
    store = ResultStore(config.output_dir)
    result_file = choose_result_file(store)
    if not result_file:
        return
    print_terminal_report(store.load_json(result_file))


def generate_report_interactive(config: AppConfig) -> None:
    store = ResultStore(config.output_dir)
    result_file = choose_result_file(store)
    if not result_file:
        return
    report_format = ask_choice("Report format", REPORT_CHOICES, default="both")
    if report_format == "none":
        console.print("[yellow]No report generated.[/yellow]")
        return
    run = store.load_json(result_file)
    saved = save_outputs(config, run, export="none", report=report_format)
    print_saved_paths(saved)


def choose_result_file(store: ResultStore) -> Path | None:
    files = store.latest_json_files()
    if not files:
        console.print("[yellow]No JSON benchmark result files found.[/yellow]")
        return None
    selected = choose_from_menu("Select a result file", [str(path) for path in files], allow_all=False, multi=False)
    if not selected:
        return None
    return Path(selected[0])


def resolve_result_file(config: AppConfig, maybe_path: Path | None) -> Path:
    if maybe_path:
        if not maybe_path.exists():
            raise OllamaBenchError(f"Result file does not exist: {maybe_path}")
        return maybe_path
    store = ResultStore(config.output_dir)
    latest = store.latest_json_files(limit=1)
    if not latest:
        raise OllamaBenchError(f"No JSON result files found in {config.output_dir}")
    return latest[0]


def command_report(args: argparse.Namespace, config: AppConfig) -> None:
    store = ResultStore(config.output_dir)
    if args.report_command in {None, "list"}:
        list_report_files(config)
        return
    if args.report_command == "show":
        result_file = resolve_result_file(config, args.result)
        print_terminal_report(store.load_json(result_file))
        return
    if args.report_command == "generate":
        result_file = resolve_result_file(config, args.result)
        run = store.load_json(result_file)
        saved = save_outputs(config, run, export="none", report=args.format)
        print_saved_paths(saved)
        return


def main_menu(config: AppConfig, client: OllamaClient, manager: OllamaServerManager) -> None:
    while True:
        print_header()
        status = manager.status()
        print_status_panel(status, cli_version=manager.installed_version_cli())
        choice = menu(
            "Main menu",
            [
                "Start Ollama server",
                "Stop Ollama server",
                "List installed models",
                "Benchmark workspace",
                "Reports",
                "Exit",
            ],
        )
        if choice == 1:
            status = manager.start()
            print_status_panel(status, cli_version=manager.installed_version_cli())
            pause()
        elif choice == 2:
            console.print(manager.stop())
            pause()
        elif choice == 3:
            print_models(client.list_models())
            pause()
        elif choice == 4:
            benchmark_menu(config, client, manager)
        elif choice == 5:
            report_menu(config)
        elif choice == 6:
            return


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    config, client, manager = make_runtime(args)

    try:
        if args.command in {None, "menu"}:
            main_menu(config, client, manager)
            return 0
        if args.command == "status":
            command_status(manager)
            return 0
        if args.command == "server":
            if args.server_command == "status":
                command_status(manager)
            elif args.server_command == "start":
                status = manager.start()
                print_status_panel(status, cli_version=manager.installed_version_cli())
            elif args.server_command == "stop":
                console.print(manager.stop(force=args.force))
            return 0
        if args.command == "models":
            models = client.list_models()
            if args.json:
                print(json.dumps(models, indent=2))
            else:
                print_models(models)
            return 0
        if args.command == "show":
            console.print_json(json.dumps(client.show_model(args.model), sort_keys=True))
            return 0
        if args.command == "bench":
            if args.bench_command in {None, "menu"}:
                benchmark_menu(config, client, manager)
            elif args.bench_command == "list":
                print_benchmark_catalog()
            elif args.bench_command == "run":
                command_bench_run(args, config, client, manager)
            return 0
        if args.command == "report":
            command_report(args, config)
            return 0
        parser.print_help()
        return 0
    except KeyboardInterrupt:
        print("\nCancelled.", file=sys.stderr)
        return 130
    except OllamaBenchError as exc:
        print(f"Error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
