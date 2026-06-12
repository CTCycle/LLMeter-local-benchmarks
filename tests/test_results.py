from pathlib import Path

from llmeter.benchmarks.base import BenchmarkResultRecord
from llmeter.results import BenchmarkRun, ResultStore


def test_result_store_saves_json_and_csv(tmp_path: Path):
    store = ResultStore(tmp_path)
    run = BenchmarkRun(
        run_id="run-1",
        created_at="2026-01-01T00:00:00+00:00",
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
                metrics={"wall_time_ms": 123.4},
                response_preview="ok",
            )
        ],
    )

    json_path = store.save_json(run)
    csv_path = store.save_csv(run)

    assert json_path.exists()
    assert csv_path.exists()
    assert "generation-latency" in json_path.read_text(encoding="utf-8")
    assert "wall_time_ms" in csv_path.read_text(encoding="utf-8")
