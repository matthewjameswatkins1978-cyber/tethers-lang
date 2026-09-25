"""Tiny external host using only the public tethers process protocol."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any

SCHEMA = "tethers.authority/1"


class Gate:
    def __init__(self, executable: Path, config: Path, engine: Path, trail: Path, host_data: Path):
        paths = (config, engine, trail, host_data)
        if any(not path.is_absolute() for path in paths):
            raise ValueError("Gate paths must be absolute")
        self.process = subprocess.Popen(
            [str(executable), "gate", "--stdio", "--config", str(config), "--engine", str(engine),
             "--trail", str(trail), "--host-data-root", str(host_data)],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            text=True, encoding="utf-8", bufsize=1,
        )
        self.sequence = 0
        hello = self.call("hello", {})
        if hello.get("status") != "ok" or hello.get("result", {}).get("protocol") != SCHEMA or hello.get("result", {}).get("provider_invocations") != 0:
            self.close()
            raise RuntimeError(f"Tethers Gate protocol negotiation failed: {hello}")

    def call(self, operation: str, payload: dict[str, Any]) -> dict[str, Any]:
        self.sequence = getattr(self, "sequence", 0) + 1
        request_id = f"external-consumer-{self.sequence}"
        request = {"schema": SCHEMA, "request_id": request_id,
                   "operation": operation, "payload": payload}
        assert self.process.stdin is not None and self.process.stdout is not None
        self.process.stdin.write(json.dumps(request, separators=(",", ":")) + "\n")
        self.process.stdin.flush()
        response = json.loads(self.process.stdout.readline())
        if response.get("schema") != SCHEMA or response.get("request_id") != request_id:
            raise RuntimeError("Tethers returned a mismatched protocol frame")
        return response

    def prepare(self, path: str) -> dict[str, Any]:
        response = self.call("prepare", {
            "action_id": "action_1", "evaluation_id": "external-eval-1",
            "tether": {"id": "external-complete", "version": "1"},
            "event": {"id": "external-event-1", "name": "coding.task_completed",
                      "data": {"task": "external consumer smoke", "path": path}},
            "facts": {"project.type": "software", "task.changed_files": 1},
        })
        if response.get("status") != "ok":
            raise RuntimeError(f"prepare failed: {response.get('error')}")
        return response["result"]

    def close(self) -> None:
        if self.process.poll() is None:
            try:
                self.call("shutdown", {})
            except (OSError, ValueError, RuntimeError):
                pass
            self.process.stdin.close()
            self.process.wait(timeout=10)

    def __enter__(self) -> "Gate":
        return self

    def __exit__(self, *_: object) -> None:
        self.close()


def request(executable: Path, config: Path, engine: Path, trail: Path,
            host_data: Path, operation: str, payload: dict[str, Any],
            schema: str = SCHEMA) -> dict[str, Any]:
    """One-shot raw frame helper for fail-closed protocol checks."""
    proc = subprocess.run(
        [str(executable), "gate", "--stdio", "--config", str(config), "--engine", str(engine),
         "--trail", str(trail), "--host-data-root", str(host_data)],
        input=json.dumps({"schema": schema, "request_id": "external-raw-1",
                          "operation": operation, "payload": payload}) + "\n",
        text=True, capture_output=True, encoding="utf-8", timeout=15,
    )
    if not proc.stdout:
        raise RuntimeError(f"Gate emitted no response: {proc.stderr}")
    return json.loads(proc.stdout.splitlines()[0])
