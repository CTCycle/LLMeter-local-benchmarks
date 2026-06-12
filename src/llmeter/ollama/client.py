from __future__ import annotations

from dataclasses import dataclass
import json
import time
from typing import Any, Iterable
from urllib import error, request

from llmeter.errors import OllamaServerError


@dataclass(slots=True)
class GenerateResult:
    model: str
    prompt: str
    response: str
    raw: dict[str, Any]
    wall_time_ns: int
    time_to_first_token_ns: int | None = None

    @property
    def total_duration_ns(self) -> int | None:
        return self.raw.get("total_duration")

    @property
    def eval_count(self) -> int | None:
        return self.raw.get("eval_count")

    @property
    def eval_duration_ns(self) -> int | None:
        return self.raw.get("eval_duration")

    @property
    def prompt_eval_count(self) -> int | None:
        return self.raw.get("prompt_eval_count")

    @property
    def prompt_eval_duration_ns(self) -> int | None:
        return self.raw.get("prompt_eval_duration")

    @property
    def tokens_per_second(self) -> float | None:
        if not self.eval_count or not self.eval_duration_ns:
            return None
        seconds = self.eval_duration_ns / 1_000_000_000.0
        if seconds <= 0:
            return None
        return round(self.eval_count / seconds, 3)


class OllamaClient:
    """Small Ollama HTTP client using only the Python standard library."""

    def __init__(self, api_base_url: str = "http://localhost:11434/api", timeout: float = 120.0):
        self.api_base_url = api_base_url.rstrip("/")
        self.timeout = timeout

    def _url(self, path: str) -> str:
        return f"{self.api_base_url}/{path.lstrip('/')}"

    def _decode_error(self, exc: error.HTTPError) -> str:
        try:
            body = exc.read().decode("utf-8", errors="replace")
        except Exception:
            body = ""
        return f"HTTP {exc.code}: {body or exc.reason}"

    def get_json(self, path: str, timeout: float | None = None) -> dict[str, Any]:
        req = request.Request(self._url(path), method="GET")
        try:
            with request.urlopen(req, timeout=timeout or self.timeout) as resp:
                payload = resp.read().decode("utf-8")
                return json.loads(payload or "{}")
        except error.HTTPError as exc:
            raise OllamaServerError(self._decode_error(exc)) from exc
        except error.URLError as exc:
            raise OllamaServerError(str(exc.reason)) from exc
        except TimeoutError as exc:
            raise OllamaServerError("Request timed out") from exc
        except json.JSONDecodeError as exc:
            raise OllamaServerError(f"Invalid JSON response: {exc}") from exc

    def post_json(self, path: str, payload: dict[str, Any], timeout: float | None = None) -> dict[str, Any]:
        data = json.dumps(payload).encode("utf-8")
        req = request.Request(
            self._url(path),
            data=data,
            method="POST",
            headers={"Content-Type": "application/json"},
        )
        try:
            with request.urlopen(req, timeout=timeout or self.timeout) as resp:
                response_payload = resp.read().decode("utf-8")
                return json.loads(response_payload or "{}")
        except error.HTTPError as exc:
            raise OllamaServerError(self._decode_error(exc)) from exc
        except error.URLError as exc:
            raise OllamaServerError(str(exc.reason)) from exc
        except TimeoutError as exc:
            raise OllamaServerError("Request timed out") from exc
        except json.JSONDecodeError as exc:
            raise OllamaServerError(f"Invalid JSON response: {exc}") from exc

    def version(self, timeout: float | None = 2.0) -> dict[str, Any]:
        return self.get_json("version", timeout=timeout)

    def is_running(self) -> bool:
        try:
            self.version(timeout=1.5)
            return True
        except OllamaServerError:
            return False

    def list_models(self) -> list[dict[str, Any]]:
        payload = self.get_json("tags")
        models = payload.get("models", [])
        return models if isinstance(models, list) else []

    def show_model(self, model: str) -> dict[str, Any]:
        return self.post_json("show", {"model": model})

    def running_models(self) -> list[dict[str, Any]]:
        payload = self.get_json("ps")
        models = payload.get("models", [])
        return models if isinstance(models, list) else []

    def generate(
        self,
        model: str,
        prompt: str,
        *,
        options: dict[str, Any] | None = None,
        keep_alive: str | int | None = "5m",
        timeout: float | None = None,
    ) -> GenerateResult:
        body: dict[str, Any] = {
            "model": model,
            "prompt": prompt,
            "stream": False,
        }
        if options:
            body["options"] = options
        if keep_alive is not None:
            body["keep_alive"] = keep_alive

        started = time.perf_counter_ns()
        payload = self.post_json("generate", body, timeout=timeout)
        ended = time.perf_counter_ns()
        return GenerateResult(
            model=model,
            prompt=prompt,
            response=str(payload.get("response", "")),
            raw=payload,
            wall_time_ns=ended - started,
            time_to_first_token_ns=None,
        )

    def generate_stream(
        self,
        model: str,
        prompt: str,
        *,
        options: dict[str, Any] | None = None,
        keep_alive: str | int | None = "5m",
        timeout: float | None = None,
    ) -> GenerateResult:
        body: dict[str, Any] = {
            "model": model,
            "prompt": prompt,
            "stream": True,
        }
        if options:
            body["options"] = options
        if keep_alive is not None:
            body["keep_alive"] = keep_alive

        data = json.dumps(body).encode("utf-8")
        req = request.Request(
            self._url("generate"),
            data=data,
            method="POST",
            headers={"Content-Type": "application/json"},
        )

        chunks: list[str] = []
        final_payload: dict[str, Any] = {}
        first_token_at: int | None = None
        started = time.perf_counter_ns()

        try:
            with request.urlopen(req, timeout=timeout or self.timeout) as resp:
                for raw_line in resp:
                    line = raw_line.decode("utf-8", errors="replace").strip()
                    if not line:
                        continue
                    try:
                        chunk = json.loads(line)
                    except json.JSONDecodeError as exc:
                        raise OllamaServerError(f"Invalid streaming JSON chunk: {exc}") from exc

                    token = str(chunk.get("response", ""))
                    if token:
                        if first_token_at is None:
                            first_token_at = time.perf_counter_ns()
                        chunks.append(token)
                    if chunk.get("done") is True:
                        final_payload = chunk
                        break
        except error.HTTPError as exc:
            raise OllamaServerError(self._decode_error(exc)) from exc
        except error.URLError as exc:
            raise OllamaServerError(str(exc.reason)) from exc
        except TimeoutError as exc:
            raise OllamaServerError("Request timed out") from exc

        ended = time.perf_counter_ns()
        response_text = "".join(chunks)
        if not final_payload:
            final_payload = {"response": response_text, "done": False}
        final_payload.setdefault("response", response_text)
        return GenerateResult(
            model=model,
            prompt=prompt,
            response=response_text,
            raw=final_payload,
            wall_time_ns=ended - started,
            time_to_first_token_ns=None if first_token_at is None else first_token_at - started,
        )

    def unload_model(self, model: str) -> dict[str, Any]:
        return self.post_json("generate", {"model": model, "keep_alive": 0})
