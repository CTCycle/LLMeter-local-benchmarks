from __future__ import annotations

from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any
import csv
import json

from llmeter.benchmarks.base import BenchmarkResultRecord
from llmeter.utils import ensure_dir, slugify, utc_now_iso


@dataclass(slots=True)
class BenchmarkRun:
    run_id: str
    created_at: str
    models: list[str]
    benchmark_ids: list[str]
    config: dict[str, Any]
    results: list[BenchmarkResultRecord] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        return {
            "run_id": self.run_id,
            "created_at": self.created_at,
            "models": self.models,
            "benchmark_ids": self.benchmark_ids,
            "config": self.config,
            "results": [asdict(record) for record in self.results],
        }


class ResultStore:
    def __init__(self, output_dir: Path):
        self.output_dir = output_dir

    def new_run_id(self, models: list[str]) -> str:
        stamp = utc_now_iso().replace(":", "").replace("+0000", "Z").replace("+00:00", "Z")
        model_part = slugify("-".join(models[:2]))[:60] if models else "models"
        return f"{stamp}-{model_part}"

    def save_json(self, run: BenchmarkRun, path: Path | None = None) -> Path:
        if path is None:
            path = self.output_dir / f"{run.run_id}.json"
        ensure_dir(path.parent)
        path.write_text(json.dumps(run.to_dict(), indent=2, sort_keys=True), encoding="utf-8")
        return path

    def save_csv(self, run: BenchmarkRun, path: Path | None = None) -> Path:
        if path is None:
            path = self.output_dir / f"{run.run_id}.csv"
        ensure_dir(path.parent)

        metric_keys = sorted({key for record in run.results for key in record.metrics.keys()})
        fieldnames = [
            "run_id",
            "created_at",
            "benchmark_id",
            "benchmark_name",
            "model",
            "run_index",
            "prompt_name",
            "error",
            "response_preview",
            *metric_keys,
        ]
        with path.open("w", newline="", encoding="utf-8") as handle:
            writer = csv.DictWriter(handle, fieldnames=fieldnames)
            writer.writeheader()
            for record in run.results:
                row: dict[str, Any] = {
                    "run_id": run.run_id,
                    "created_at": run.created_at,
                    "benchmark_id": record.benchmark_id,
                    "benchmark_name": record.benchmark_name,
                    "model": record.model,
                    "run_index": record.run_index,
                    "prompt_name": record.prompt_name,
                    "error": record.error,
                    "response_preview": record.response_preview,
                }
                row.update(record.metrics)
                writer.writerow(row)
        return path

    def latest_json_files(self, limit: int = 10) -> list[Path]:
        if not self.output_dir.exists():
            return []
        return sorted(self.output_dir.glob("*.json"), key=lambda p: p.stat().st_mtime, reverse=True)[:limit]

    def latest_report_files(self, limit: int = 10) -> list[Path]:
        if not self.output_dir.exists():
            return []
        patterns = ["*.report.md", "*.report.html"]
        files: list[Path] = []
        for pattern in patterns:
            files.extend(self.output_dir.glob(pattern))
        return sorted(files, key=lambda p: p.stat().st_mtime, reverse=True)[:limit]

    def load_json(self, path: Path) -> BenchmarkRun:
        payload = json.loads(path.read_text(encoding="utf-8"))
        records = [BenchmarkResultRecord(**item) for item in payload.get("results", [])]
        return BenchmarkRun(
            run_id=payload["run_id"],
            created_at=payload["created_at"],
            models=list(payload.get("models", [])),
            benchmark_ids=list(payload.get("benchmark_ids", [])),
            config=dict(payload.get("config", {})),
            results=records,
        )
