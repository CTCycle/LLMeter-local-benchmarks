from __future__ import annotations

from collections import defaultdict
from dataclasses import dataclass
from html import escape
from pathlib import Path
from statistics import mean
from typing import Any

from ollama_bench.results import BenchmarkRun


@dataclass(slots=True)
class SummaryRow:
    model: str
    benchmark_id: str
    prompt_name: str | None
    records: int
    errors: int
    avg_wall_time_ms: float | None
    avg_time_to_first_token_ms: float | None
    avg_tokens_per_second: float | None
    avg_total_duration_ms: float | None
    avg_api_prompt_api_eval_duration_ms: float | None
    avg_api_eval_duration_ms: float | None
    mean_similarity: float | None


def build_summary_rows(run: BenchmarkRun) -> list[SummaryRow]:
    grouped: dict[tuple[str, str, str | None], list] = defaultdict(list)
    for record in run.results:
        grouped[(record.model, record.benchmark_id, record.prompt_name)].append(record)

    rows: list[SummaryRow] = []
    for (model, benchmark_id, prompt_name), records in sorted(grouped.items(), key=lambda item: item[0]):
        rows.append(
            SummaryRow(
                model=model,
                benchmark_id=benchmark_id,
                prompt_name=prompt_name,
                records=len(records),
                errors=sum(1 for record in records if record.error),
                avg_wall_time_ms=_metric_mean(records, "wall_time_ms"),
                avg_time_to_first_token_ms=_metric_mean(records, "time_to_first_token_ms"),
                avg_tokens_per_second=_metric_mean(records, "tokens_per_second") or _metric_mean(records, "mean_tokens_per_second"),
                avg_total_duration_ms=_metric_mean(records, "api_total_duration_ms"),
                avg_api_prompt_api_eval_duration_ms=_metric_mean(records, "api_prompt_api_eval_duration_ms"),
                avg_api_eval_duration_ms=_metric_mean(records, "api_eval_duration_ms"),
                mean_similarity=_metric_mean(records, "mean_pairwise_similarity"),
            )
        )
    return rows


def best_throughput_row(rows: list[SummaryRow]) -> SummaryRow | None:
    candidates = [row for row in rows if row.avg_tokens_per_second is not None and row.errors == 0]
    return max(candidates, key=lambda row: row.avg_tokens_per_second or 0, default=None)


def fastest_latency_row(rows: list[SummaryRow]) -> SummaryRow | None:
    candidates = [row for row in rows if row.avg_wall_time_ms is not None and row.errors == 0]
    return min(candidates, key=lambda row: row.avg_wall_time_ms or float("inf"), default=None)


def render_markdown_report(run: BenchmarkRun) -> str:
    rows = build_summary_rows(run)
    error_records = [record for record in run.results if record.error]
    best_tps = best_throughput_row(rows)
    fastest = fastest_latency_row(rows)

    lines: list[str] = []
    lines.append(f"# Ollama Local Bench Report")
    lines.append("")
    lines.append(f"**Run ID:** `{run.run_id}`")
    lines.append(f"**Created:** {run.created_at}")
    lines.append(f"**Models:** {', '.join(run.models) if run.models else 'none'}")
    lines.append(f"**Benchmarks:** {', '.join(run.benchmark_ids) if run.benchmark_ids else 'none'}")
    lines.append("")
    lines.append("## Executive summary")
    lines.append("")
    lines.append(f"- Records: {len(run.results)}")
    lines.append(f"- Successful records: {len(run.results) - len(error_records)}")
    lines.append(f"- Error records: {len(error_records)}")
    if best_tps:
        lines.append(
            f"- Best average throughput: **{_fmt(best_tps.avg_tokens_per_second)} tok/s** "
            f"on `{best_tps.model}` for `{best_tps.benchmark_id}`"
        )
    if fastest:
        lines.append(
            f"- Fastest average wall time: **{_fmt(fastest.avg_wall_time_ms)} ms** "
            f"on `{fastest.model}` for `{fastest.benchmark_id}`"
        )
    lines.append("")

    lines.append("## Run configuration")
    lines.append("")
    lines.append("| Setting | Value |")
    lines.append("|---|---:|")
    for key, value in sorted(run.config.items()):
        lines.append(f"| `{key}` | `{value}` |")
    lines.append("")

    lines.append("## Aggregated benchmark results")
    lines.append("")
    lines.append(
        "| Model | Benchmark | Prompt | Records | Errors | Avg wall ms | Avg TTFT ms | Avg tok/s | Avg total ms | Prompt eval ms | Eval ms | Similarity |"
    )
    lines.append("|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
    for row in rows:
        lines.append(
            "| "
            + " | ".join(
                [
                    _md(row.model),
                    _md(row.benchmark_id),
                    _md(row.prompt_name or "all"),
                    str(row.records),
                    str(row.errors),
                    _fmt(row.avg_wall_time_ms),
                    _fmt(row.avg_time_to_first_token_ms),
                    _fmt(row.avg_tokens_per_second),
                    _fmt(row.avg_total_duration_ms),
                    _fmt(row.avg_api_prompt_api_eval_duration_ms),
                    _fmt(row.avg_api_eval_duration_ms),
                    _fmt(row.mean_similarity, digits=4),
                ]
            )
            + " |"
        )
    lines.append("")

    lines.append("## Detailed records")
    lines.append("")
    lines.append("| Model | Benchmark | Prompt | Run | Wall ms | TTFT ms | Tok/s | Status | Preview |")
    lines.append("|---|---|---|---:|---:|---:|---:|---|---|")
    for record in run.results:
        metrics = record.metrics
        lines.append(
            "| "
            + " | ".join(
                [
                    _md(record.model),
                    _md(record.benchmark_id),
                    _md(record.prompt_name or ""),
                    str(record.run_index or ""),
                    _fmt(metrics.get("wall_time_ms")),
                    _fmt(metrics.get("time_to_first_token_ms")),
                    _fmt(metrics.get("tokens_per_second") or metrics.get("mean_tokens_per_second")),
                    "ERROR" if record.error else "OK",
                    _md(record.response_preview or ""),
                ]
            )
            + " |"
        )
    lines.append("")

    if error_records:
        lines.append("## Errors")
        lines.append("")
        lines.append("| Model | Benchmark | Prompt | Error |")
        lines.append("|---|---|---|---|")
        for record in error_records:
            lines.append(
                f"| {_md(record.model)} | {_md(record.benchmark_id)} | {_md(record.prompt_name or '')} | {_md(record.error or '')} |"
            )
        lines.append("")

    lines.append("## Interpretation notes")
    lines.append("")
    lines.append("- Wall time is measured by the CLI around the request.")
    lines.append("- Time to first token is measured only for streaming benchmark calls.")
    lines.append("- Ollama duration fields come from the final API response when available.")
    lines.append("- Results are local-machine specific. Compare runs from the same host for useful conclusions.")
    lines.append("")
    return "\n".join(lines)


def render_html_report(run: BenchmarkRun) -> str:
    rows = build_summary_rows(run)
    error_records = [record for record in run.results if record.error]
    best_tps = best_throughput_row(rows)
    fastest = fastest_latency_row(rows)

    summary_items = [
        ("Records", str(len(run.results))),
        ("Successful records", str(len(run.results) - len(error_records))),
        ("Error records", str(len(error_records))),
    ]
    if best_tps:
        summary_items.append(
            (
                "Best average throughput",
                f"{_fmt(best_tps.avg_tokens_per_second)} tok/s on {best_tps.model} for {best_tps.benchmark_id}",
            )
        )
    if fastest:
        summary_items.append(
            (
                "Fastest average wall time",
                f"{_fmt(fastest.avg_wall_time_ms)} ms on {fastest.model} for {fastest.benchmark_id}",
            )
        )

    config_rows = "".join(
        f"<tr><th><code>{escape(str(key))}</code></th><td><code>{escape(str(value))}</code></td></tr>"
        for key, value in sorted(run.config.items())
    )
    summary_cards = "".join(
        f"<div class=\"card\"><span>{escape(label)}</span><strong>{escape(value)}</strong></div>" for label, value in summary_items
    )
    aggregate_rows = "".join(
        "<tr>"
        + "".join(
            f"<td>{escape(value)}</td>"
            for value in [
                row.model,
                row.benchmark_id,
                row.prompt_name or "all",
                str(row.records),
                str(row.errors),
                _fmt(row.avg_wall_time_ms),
                _fmt(row.avg_time_to_first_token_ms),
                _fmt(row.avg_tokens_per_second),
                _fmt(row.avg_total_duration_ms),
                _fmt(row.avg_prompt_eval_duration_ms),
                _fmt(row.avg_eval_duration_ms),
                _fmt(row.mean_similarity, digits=4),
            ]
        )
        + "</tr>"
        for row in rows
    )
    detail_rows = "".join(
        "<tr>"
        + "".join(
            f"<td>{escape(value)}</td>"
            for value in [
                record.model,
                record.benchmark_id,
                record.prompt_name or "",
                str(record.run_index or ""),
                _fmt(record.metrics.get("wall_time_ms")),
                _fmt(record.metrics.get("time_to_first_token_ms")),
                _fmt(record.metrics.get("tokens_per_second") or record.metrics.get("mean_tokens_per_second")),
                "ERROR" if record.error else "OK",
                record.response_preview or "",
            ]
        )
        + "</tr>"
        for record in run.results
    )
    error_section = ""
    if error_records:
        error_rows = "".join(
            f"<tr><td>{escape(record.model)}</td><td>{escape(record.benchmark_id)}</td>"
            f"<td>{escape(record.prompt_name or '')}</td><td>{escape(record.error or '')}</td></tr>"
            for record in error_records
        )
        error_section = f"""
<section>
  <h2>Errors</h2>
  <table>
    <thead><tr><th>Model</th><th>Benchmark</th><th>Prompt</th><th>Error</th></tr></thead>
    <tbody>{error_rows}</tbody>
  </table>
</section>
"""

    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Ollama Local Bench Report {escape(run.run_id)}</title>
  <style>
    :root {{ color-scheme: light dark; --border: color-mix(in srgb, CanvasText 20%, Canvas); --soft: color-mix(in srgb, CanvasText 8%, Canvas); }}
    body {{ font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; margin: 0; line-height: 1.5; background: Canvas; color: CanvasText; }}
    main {{ max-width: 1180px; margin: 0 auto; padding: 2rem; }}
    header {{ padding: 1.5rem; border: 1px solid var(--border); border-radius: 18px; background: var(--soft); }}
    h1 {{ margin: 0 0 .5rem; }}
    h2 {{ margin-top: 2rem; }}
    code {{ font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace; }}
    .meta {{ display: grid; grid-template-columns: max-content 1fr; gap: .25rem 1rem; }}
    .cards {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: .75rem; margin: 1rem 0; }}
    .card {{ border: 1px solid var(--border); border-radius: 14px; padding: .9rem; background: var(--soft); }}
    .card span {{ display: block; opacity: .75; font-size: .9rem; }}
    .card strong {{ display: block; margin-top: .25rem; }}
    .table-wrap {{ overflow-x: auto; border: 1px solid var(--border); border-radius: 14px; }}
    table {{ width: 100%; border-collapse: collapse; font-size: .92rem; }}
    th, td {{ padding: .65rem .75rem; border-bottom: 1px solid var(--border); text-align: left; vertical-align: top; }}
    th {{ background: var(--soft); position: sticky; top: 0; }}
    tr:last-child td {{ border-bottom: none; }}
    .notes li {{ margin-bottom: .35rem; }}
  </style>
</head>
<body>
<main>
<header>
  <h1>Ollama Local Bench Report</h1>
  <div class="meta">
    <strong>Run ID</strong><code>{escape(run.run_id)}</code>
    <strong>Created</strong><span>{escape(run.created_at)}</span>
    <strong>Models</strong><span>{escape(', '.join(run.models) if run.models else 'none')}</span>
    <strong>Benchmarks</strong><span>{escape(', '.join(run.benchmark_ids) if run.benchmark_ids else 'none')}</span>
  </div>
</header>
<section>
  <h2>Executive summary</h2>
  <div class="cards">{summary_cards}</div>
</section>
<section>
  <h2>Run configuration</h2>
  <div class="table-wrap"><table><tbody>{config_rows}</tbody></table></div>
</section>
<section>
  <h2>Aggregated benchmark results</h2>
  <div class="table-wrap">
    <table>
      <thead><tr><th>Model</th><th>Benchmark</th><th>Prompt</th><th>Records</th><th>Errors</th><th>Avg wall ms</th><th>Avg TTFT ms</th><th>Avg tok/s</th><th>Avg total ms</th><th>Prompt eval ms</th><th>Eval ms</th><th>Similarity</th></tr></thead>
      <tbody>{aggregate_rows}</tbody>
    </table>
  </div>
</section>
<section>
  <h2>Detailed records</h2>
  <div class="table-wrap">
    <table>
      <thead><tr><th>Model</th><th>Benchmark</th><th>Prompt</th><th>Run</th><th>Wall ms</th><th>TTFT ms</th><th>Tok/s</th><th>Status</th><th>Preview</th></tr></thead>
      <tbody>{detail_rows}</tbody>
    </table>
  </div>
</section>
{error_section}
<section class="notes">
  <h2>Interpretation notes</h2>
  <ul>
    <li>Wall time is measured by the CLI around the request.</li>
    <li>Time to first token is measured only for streaming benchmark calls.</li>
    <li>Ollama duration fields come from the final API response when available.</li>
    <li>Results are local-machine specific. Compare runs from the same host for useful conclusions.</li>
  </ul>
</section>
</main>
</body>
</html>
"""

def save_markdown_report(run: BenchmarkRun, output_dir: Path) -> Path:
    output_dir.mkdir(parents=True, exist_ok=True)
    path = output_dir / f"{run.run_id}.report.md"
    path.write_text(render_markdown_report(run), encoding="utf-8")
    return path


def save_html_report(run: BenchmarkRun, output_dir: Path) -> Path:
    output_dir.mkdir(parents=True, exist_ok=True)
    path = output_dir / f"{run.run_id}.report.html"
    path.write_text(render_html_report(run), encoding="utf-8")
    return path


def _metric_mean(records: list[Any], key: str) -> float | None:
    values = []
    for record in records:
        value = record.metrics.get(key)
        if isinstance(value, int | float):
            values.append(float(value))
    return round(mean(values), 4) if values else None


def _fmt(value: Any, *, digits: int = 2) -> str:
    if value is None or value == "":
        return ""
    if isinstance(value, float):
        return f"{value:.{digits}f}"
    return str(value)


def _md(value: str) -> str:
    return str(value).replace("|", "\\|").replace("\n", " ").strip()
