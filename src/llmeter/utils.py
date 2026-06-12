from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
import re


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def ns_to_ms(value: int | float | None) -> float | None:
    if value is None:
        return None
    return round(float(value) / 1_000_000.0, 3)


def ns_to_seconds(value: int | float | None) -> float | None:
    if value is None:
        return None
    return float(value) / 1_000_000_000.0


def safe_float(value: object) -> float | None:
    if value is None:
        return None
    try:
        return float(value)
    except (TypeError, ValueError):
        return None


def slugify(value: str) -> str:
    cleaned = re.sub(r"[^A-Za-z0-9._-]+", "-", value.strip())
    cleaned = cleaned.strip("-._")
    return cleaned or "value"


def ensure_dir(path: Path) -> Path:
    path.mkdir(parents=True, exist_ok=True)
    return path
