from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
import os

DEFAULT_HOST = "http://localhost:11434"
DEFAULT_OUTPUT_DIR = Path("benchmark_results")


@dataclass(slots=True)
class AppConfig:
    host: str = field(default_factory=lambda: os.getenv("OLLAMA_HOST", DEFAULT_HOST))
    timeout: float = field(default_factory=lambda: float(os.getenv("LLMETER_TIMEOUT", "120")))
    output_dir: Path = field(default_factory=lambda: Path(os.getenv("LLMETER_OUTPUT_DIR", DEFAULT_OUTPUT_DIR)))
    state_dir: Path = field(default_factory=lambda: Path(os.getenv("LLMETER_STATE_DIR", Path.home() / ".llmeter")))
    default_runs: int = field(default_factory=lambda: int(os.getenv("LLMETER_RUNS", "3")))
    default_num_predict: int = field(default_factory=lambda: int(os.getenv("LLMETER_NUM_PREDICT", "128")))
    default_temperature: float = field(default_factory=lambda: float(os.getenv("LLMETER_TEMPERATURE", "0.2")))

    @property
    def api_base_url(self) -> str:
        host = self.host.rstrip("/")
        return host if host.endswith("/api") else f"{host}/api"
