from __future__ import annotations

from llmeter.benchmarks.base import BenchmarkResultRecord
from llmeter.reporting import build_summary_rows, render_markdown_report
from llmeter.results import BenchmarkRun


def test_markdown_report_contains_summary() -> None:
    run = BenchmarkRun(
        run_id="test-run",
        created_at="2026-01-01T00:00:00Z",
        models=["llama3.2"],
        benchmark_ids=["generation-latency"],
        config={"runs": 1},
        results=[
            BenchmarkResultRecord(
                benchmark_id="generation-latency",
                benchmark_name="Basic generation latency",
                model="llama3.2",
                run_index=1,
                prompt_name="short",
                metrics={"wall_time_ms": 100.0, "time_to_first_token_ms": 10.0, "tokens_per_second": 42.0},
                response_preview="ok",
            )
        ],
    )

    rows = build_summary_rows(run)
    assert rows[0].avg_tokens_per_second == 42.0

    report = render_markdown_report(run)
    assert "LLMeter Report" in report
    assert "generation-latency" in report
    assert "42.00" in report
