from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import json
import os
import platform
import shutil
import signal
import subprocess
import time
from typing import Any

from llmeter.errors import OllamaNotInstalledError, OllamaServerError
from llmeter.ollama.client import OllamaClient
from llmeter.utils import ensure_dir


@dataclass(slots=True)
class ServerStatus:
    installed: bool
    executable: str | None
    running: bool
    version: str | None = None
    tracked_pid: int | None = None


class OllamaServerManager:
    """Detect, start, and stop a local Ollama server process."""

    def __init__(self, client: OllamaClient, state_dir: Path):
        self.client = client
        self.state_dir = state_dir
        self.pid_file = state_dir / "ollama-server.pid.json"
        self.log_file = state_dir / "ollama-server.log"

    def executable(self) -> str | None:
        return shutil.which("ollama")

    def require_executable(self) -> str:
        exe = self.executable()
        if not exe:
            raise OllamaNotInstalledError("The 'ollama' executable was not found on PATH.")
        return exe

    def installed_version_cli(self) -> str | None:
        exe = self.executable()
        if not exe:
            return None
        try:
            completed = subprocess.run(
                [exe, "--version"],
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
        except (OSError, subprocess.SubprocessError):
            return None
        output = (completed.stdout or completed.stderr).strip()
        return output or None

    def _read_pid(self) -> int | None:
        if not self.pid_file.exists():
            return None
        try:
            payload = json.loads(self.pid_file.read_text(encoding="utf-8"))
            pid = payload.get("pid")
            return int(pid) if pid else None
        except (OSError, ValueError, json.JSONDecodeError):
            return None

    def _write_pid(self, pid: int) -> None:
        ensure_dir(self.state_dir)
        self.pid_file.write_text(json.dumps({"pid": pid}, indent=2), encoding="utf-8")

    def _clear_pid(self) -> None:
        try:
            self.pid_file.unlink(missing_ok=True)
        except OSError:
            pass

    def _pid_alive(self, pid: int) -> bool:
        if pid <= 0:
            return False
        if platform.system() == "Windows":
            completed = subprocess.run(
                ["tasklist", "/FI", f"PID eq {pid}"],
                capture_output=True,
                text=True,
                check=False,
            )
            return str(pid) in completed.stdout
        try:
            os.kill(pid, 0)
            return True
        except OSError:
            return False

    def status(self) -> ServerStatus:
        installed = self.executable() is not None
        version: str | None = None
        running = False
        try:
            payload = self.client.version(timeout=1.5)
            version = str(payload.get("version")) if payload.get("version") else None
            running = True
        except OllamaServerError:
            running = False

        pid = self._read_pid()
        if pid is not None and not self._pid_alive(pid):
            self._clear_pid()
            pid = None

        return ServerStatus(
            installed=installed,
            executable=self.executable(),
            running=running,
            version=version,
            tracked_pid=pid,
        )

    def start(self, wait_seconds: float = 10.0) -> ServerStatus:
        if self.client.is_running():
            return self.status()

        exe = self.require_executable()
        ensure_dir(self.state_dir)
        log_handle = self.log_file.open("ab")
        kwargs: dict[str, Any] = {
            "stdout": log_handle,
            "stderr": subprocess.STDOUT,
            "stdin": subprocess.DEVNULL,
        }
        if platform.system() == "Windows":
            kwargs["creationflags"] = getattr(subprocess, "DETACHED_PROCESS", 0) | getattr(subprocess, "CREATE_NEW_PROCESS_GROUP", 0)
        else:
            kwargs["start_new_session"] = True

        try:
            proc = subprocess.Popen([exe, "serve"], **kwargs)
        except OSError as exc:
            raise OllamaServerError(f"Failed to start Ollama: {exc}") from exc
        finally:
            log_handle.close()

        self._write_pid(proc.pid)

        deadline = time.time() + wait_seconds
        last_error: Exception | None = None
        while time.time() < deadline:
            if proc.poll() is not None:
                self._clear_pid()
                raise OllamaServerError(f"Ollama exited early with code {proc.returncode}. See {self.log_file}")
            try:
                if self.client.is_running():
                    return self.status()
            except OllamaServerError as exc:
                last_error = exc
            time.sleep(0.25)

        raise OllamaServerError(f"Ollama did not become ready in {wait_seconds:.1f}s. Last error: {last_error}")

    def stop(self, *, force: bool = False, wait_seconds: float = 5.0) -> str:
        pid = self._read_pid()
        if pid and self._pid_alive(pid):
            if platform.system() == "Windows":
                args = ["taskkill", "/PID", str(pid), "/T"]
                if force:
                    args.append("/F")
                subprocess.run(args, check=False, capture_output=True)
            else:
                os.kill(pid, signal.SIGTERM)
                deadline = time.time() + wait_seconds
                while time.time() < deadline and self._pid_alive(pid):
                    time.sleep(0.2)
                if self._pid_alive(pid) and force:
                    os.kill(pid, signal.SIGKILL)
            self._clear_pid()
            return f"Stopped tracked Ollama server process PID {pid}."

        self._clear_pid()
        if not force:
            return "No tracked Ollama server process found. This CLI only stops servers it started unless --force is used."

        exe = self.executable()
        if not exe:
            raise OllamaNotInstalledError("The 'ollama' executable was not found on PATH.")

        if platform.system() == "Windows":
            subprocess.run(["taskkill", "/IM", "ollama.exe", "/F"], check=False, capture_output=True)
            return "Requested forced termination of ollama.exe processes."

        subprocess.run(["pkill", "-f", "ollama serve"], check=False, capture_output=True)
        return "Requested forced termination of 'ollama serve' processes."
