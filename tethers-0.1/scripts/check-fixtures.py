#!/usr/bin/env python3
"""Validate first-party JSON and JSONL protocol fixtures."""

from __future__ import annotations

import json
import sys
from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    json_paths = [root / "protocol/request.json", root / "protocol/expected-response.json"]
    cases = root / "protocol/cases"
    if cases.is_dir():
        json_paths.extend(sorted(cases.rglob("*.json")))
    jsonl_paths = sorted((root / "protocol/mcp-transcripts").rglob("*.jsonl")) if (root / "protocol/mcp-transcripts").is_dir() else []
    try:
        for path in json_paths:
            if not path.is_file():
                raise ValueError(f"Missing JSON fixture: {path.relative_to(root)}")
            json.loads(path.read_text(encoding="utf-8"))
        for path in jsonl_paths:
            for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
                if not line.strip():
                    raise ValueError(f"Invalid JSONL fixture '{path.relative_to(root)}': blank line at {number}")
                json.loads(line)
    except (OSError, json.JSONDecodeError, ValueError) as error:
        print(str(error))
        return 1
    print(f"JSON fixtures are valid ({len(json_paths)} JSON files, {len(jsonl_paths)} JSONL files)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
