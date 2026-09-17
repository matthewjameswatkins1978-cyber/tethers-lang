"""Small, dependency-free helpers shared by Tethers verification entry points."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Mapping, Sequence


OUTCOME_STATUSES = ("PASS", "FAIL", "SKIPPED WITH REASON", "NOT APPLICABLE")


def canonical_json(value: object) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def first_meaningful_line(output: str) -> str | None:
    for line in output.splitlines():
        line = line.strip()
        if line:
            return line[:1000]
    return None


@dataclass
class StepResult:
    name: str
    category: str
    status: str
    exit_code: int | None
    duration_ms: int
    first_failure: str | None = None
    output: str = ""

    def report(self) -> dict[str, object]:
        return {
            "name": self.name,
            "category": self.category,
            "status": self.status,
            "exit_code": self.exit_code,
            "duration_ms": self.duration_ms,
            "first_failure": self.first_failure,
        }


def run_step(
    name: str,
    argv: Sequence[str],
    cwd: Path,
    *,
    category: str = "required",
    env: Mapping[str, str] | None = None,
    timeout: int | None = None,
) -> StepResult:
    started = time.monotonic()
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)
    try:
        completed = subprocess.run(
            list(argv),
            cwd=str(cwd),
            env=merged_env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            timeout=timeout,
            check=False,
        )
        output = completed.stdout or ""
        exit_code = completed.returncode
    except FileNotFoundError as error:
        output = str(error)
        exit_code = 127
    except subprocess.TimeoutExpired as error:
        output = (error.stdout or "") + "\nverification step timed out"
        exit_code = 124
    duration_ms = int((time.monotonic() - started) * 1000)
    status = "PASS" if exit_code == 0 else "FAIL"
    return StepResult(
        name=name,
        category=category,
        status=status,
        exit_code=exit_code,
        duration_ms=duration_ms,
        first_failure=None if status == "PASS" else first_meaningful_line(output),
        output=output[-12000:],
    )


def skipped_step(name: str, reason: str, *, category: str = "required") -> StepResult:
    return StepResult(
        name=name,
        category=category,
        status="SKIPPED WITH REASON",
        exit_code=None,
        duration_ms=0,
        first_failure=reason,
    )


def command_path(name: str) -> str | None:
    for directory in os.environ.get("PATH", "").split(os.pathsep):
        candidate = Path(directory) / name
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return str(candidate)
    return None


def read_json_lines(path: Path) -> list[object]:
    if not path.is_file():
        raise ValueError(f"Missing JSONL file: {path}")
    messages: list[object] = []
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip():
            raise ValueError(f"Blank JSONL line at {path}:{line_number}")
        try:
            messages.append(json.loads(line))
        except json.JSONDecodeError as error:
            raise ValueError(f"Invalid JSON at {path}:{line_number}: {error}") from error
    return messages


def json_files(root: Path, suffix: str) -> Iterable[Path]:
    return sorted(path for path in root.rglob(f"*{suffix}") if path.is_file())
