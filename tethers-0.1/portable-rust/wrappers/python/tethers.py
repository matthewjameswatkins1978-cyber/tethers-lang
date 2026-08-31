"""Tiny subprocess adapter for the canonical Tethers JSON protocol."""
import json
import subprocess
from typing import Any

DECISIONS = {"ALLOW", "ASK", "DENY"}

def evaluate(binary: str, request: dict[str, Any], timeout: float = 5.0) -> dict[str, Any]:
    try:
        completed = subprocess.run([binary, "evaluate"], input=json.dumps(request), text=True,
                                   capture_output=True, timeout=timeout, check=False)
    except (OSError, subprocess.TimeoutExpired) as exc:
        return {"decision": "DENY", "error": f"Tethers unavailable: {exc}"}
    if completed.returncode != 0:
        return {"decision": "DENY", "error": f"Tethers exited with {completed.returncode}"}
    try:
        result = json.loads(completed.stdout)
    except (TypeError, json.JSONDecodeError) as exc:
        return {"decision": "DENY", "error": f"invalid Tethers response: {exc}"}
    if result.get("decision") not in DECISIONS:
        return {"decision": "DENY", "error": "unknown Tethers decision"}
    return result
