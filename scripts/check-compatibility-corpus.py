#!/usr/bin/env python3
"""Validate the durable 0.7 compatibility seed corpus."""

from __future__ import annotations

import json
import sys
from pathlib import Path

from verification_support import read_json_lines


def read_json(root: Path, relative: str) -> object:
    path = root / relative
    if not path.is_file():
        raise ValueError(f"Missing compatibility fixture: {relative}")
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        raise ValueError(f"Invalid compatibility JSON {relative}: {error}") from error


def main() -> int:
    root = Path(__file__).resolve().parents[1] / "compat/0.7"
    required = [
        "README.md",
        "config/evaluation-request.json",
        "cli/evaluation-request.json",
        "manifests/fixture-ping.json",
        "plans/record-completed-task.json",
        "trails/record-completed-task.json",
        "mcp/validate-valid.stdin.jsonl",
        "mcp/validate-valid.stdout.jsonl",
    ]
    try:
        for relative in required:
            if not (root / relative).is_file():
                raise ValueError(f"Missing compatibility fixture: {relative}")
        request = read_json(root, "config/evaluation-request.json")
        manifest = read_json(root, "manifests/fixture-ping.json")
        plan = read_json(root, "plans/record-completed-task.json")
        if request.get("protocol_version") != "0.1" or request.get("language_version") != "0.1":
            raise ValueError("Compatibility request is missing its historical version identifiers.")
        if not request.get("tether", {}).get("source"):
            raise ValueError("Human Tether source is missing from compatibility request.")
        if manifest.get("manifest_format_version") != "1.0" or not manifest.get("digest"):
            raise ValueError("Capability manifest identity is incomplete.")
        if plan.get("protocol_version") != "0.1" or len(plan.get("plan", {}).get("actions", [])) != 1:
            raise ValueError("Plan compatibility fixture is not the expected ordered-action shape.")
        read_json_lines(root / "mcp/validate-valid.stdin.jsonl")
        read_json_lines(root / "mcp/validate-valid.stdout.jsonl")
    except (ValueError, AttributeError, TypeError) as error:
        print(str(error))
        return 1
    print("PASS compatibility corpus seed (0.7 fixtures, provenance and required version identifiers)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
