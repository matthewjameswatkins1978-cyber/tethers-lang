"""Small, dependency-free helpers shared by Tethers verification entry points."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
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
    # shutil.which honours PATHEXT on Windows (for example git.exe when asked
    # for git); a hand-rolled Path check silently reports every Windows tool
    # missing even though subprocess can launch it.
    return shutil.which(name)


def discover_ocaml_switch(
    repository_root: Path,
    *,
    requested: str | None = None,
    environ: Mapping[str, str] | None = None,
    run=subprocess.run,
) -> str:
    """Find an installed Tethers OCaml switch without relying on agent PATH lore.

    An explicit request is authoritative and fails closed. Otherwise prefer a
    switch local to this checkout, then inspect opam's registered switches.
    Only the repository's pinned compiler and Dune versions are accepted.
    """
    env = os.environ if environ is None else environ
    explicit = requested or env.get("TETHERS_OCAML_SWITCH")
    if explicit:
        candidates = [explicit]
    else:
        engine_root = repository_root / "tethers-0.1" / "engine-ocaml"
        candidates = [str(engine_root)] if (engine_root / "_opam").is_dir() else []
        try:
            listed = run(
                ["opam", "switch", "list", "--short"],
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                check=False,
            )
        except OSError as error:
            raise RuntimeError(f"Cannot discover OCaml switches because opam is unavailable: {error}") from error
        if listed.returncode != 0:
            detail = (listed.stdout or "").strip()
            raise RuntimeError(f"opam could not list installed switches: {detail or listed.returncode}")
        candidates.extend(line.strip() for line in (listed.stdout or "").splitlines() if line.strip())

    seen: set[str] = set()
    rejected: list[str] = []
    for candidate in candidates:
        if os.name == "nt" and not Path(candidate).is_absolute():
            rejected.append(f"{candidate} (Windows verification requires a project switch path)")
            if explicit:
                break
            continue
        key = os.path.normcase(os.path.normpath(candidate))
        if key in seen:
            continue
        seen.add(key)
        versions: dict[str, str] = {}
        for tool, args in (("ocamlc", ["ocamlc", "-version"]), ("dune", ["dune", "--version"])):
            try:
                result = run(
                    ["opam", "exec", f"--switch={candidate}", "--", *args],
                    text=True,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT,
                    check=False,
                )
            except OSError:
                break
            if result.returncode != 0:
                break
            versions[tool] = (result.stdout or "").strip()
        if versions == {"ocamlc": "5.5.0", "dune": "3.24.0"}:
            try:
                packages = run(
                    ["opam", "list", f"--switch={candidate}", "--installed", "--columns=name,version"],
                    text=True,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT,
                    check=False,
                )
            except OSError:
                packages = None
            installed = {}
            if packages is not None and packages.returncode == 0:
                for line in (packages.stdout or "").splitlines():
                    fields = line.split()
                    if len(fields) >= 2:
                        installed[fields[0]] = fields[1]
            if installed.get("yojson") == "2.2.2" and installed.get("digestif") == "1.3.1":
                return candidate
            rejected.append(f"{candidate} (required packages Yojson 2.2.2 and Digestif 1.3.1 are missing or mismatched)")
            if explicit:
                break
            continue
        rejected.append(f"{candidate} (OCaml {versions.get('ocamlc', 'unavailable')}, Dune {versions.get('dune', 'unavailable')})")
        if explicit:
            break

    if explicit:
        raise RuntimeError(f"Requested OCaml switch is unavailable or incompatible: {rejected[0] if rejected else explicit}")
    inspected = "; ".join(rejected) if rejected else "no registered switches"
    raise RuntimeError(f"No installed Tethers OCaml switch found (requires OCaml 5.5.0 and Dune 3.24.0): {inspected}")


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
