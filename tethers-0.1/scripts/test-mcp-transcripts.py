#!/usr/bin/env python3
"""Run MCP transcript fixtures against the current verified OCaml engine."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))
from verification_support import canonical_json, read_json_lines  # noqa: E402


CASES = [
    "initialization-success", "initialization-success-2025-06-18", "incompatible-mcp-protocol-version",
    "tools-list", "evaluate-matched", "evaluate-not-matched", "evaluate-minimal-tethers-error",
    "evaluate-correlated-tethers-error", "malformed-tool-arguments", "unknown-tool", "call-before-initialization",
    "clean-eof-shutdown", "validate-valid", "validate-invalid", "validate-missing-source", "validate-together",
]


def prop(message: object, name: str) -> object | None:
    return message.get(name) if isinstance(message, dict) else None


def assert_rpc(messages: list[object], direction: str, context: str) -> None:
    for message in messages:
        if prop(message, "jsonrpc") != "2.0":
            raise ValueError(f"{context} must declare jsonrpc 2.0")
        if direction == "stdout":
            has_result = prop(message, "result") is not None
            has_error = prop(message, "error") is not None
            if has_result == has_error or prop(message, "id") is None:
                raise ValueError(f"{context} stdout response shape is invalid")
        elif prop(message, "method") is None:
            raise ValueError(f"{context} stdin request must contain a method")


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    transcript_root = root / "protocol/mcp-transcripts"
    engine = os.environ.get("TETHERS_VERIFIED_ENGINE")
    if engine:
        server = Path(engine)
    else:
        server = root / "engine-ocaml/_build/default/bin/tethers_mcp_main"
        if not server.is_file():
            server = server.with_suffix(".exe")
    try:
        if not server.is_file():
            raise ValueError(f"MCP server executable not found: {server}")
        left = {"b": 2, "a": {"z": 3, "y": [1, 2]}}
        right = {"a": {"y": [1, 2], "z": 3}, "b": 2}
        if canonical_json(left) != canonical_json(right):
            raise ValueError("canonical object-key order self-check failed")
        for case in CASES:
            case_root = transcript_root / case
            stdin_path = case_root / "stdin.jsonl"
            stdout_path = case_root / "stdout.jsonl"
            stdin_messages = read_json_lines(stdin_path)
            expected = read_json_lines(stdout_path)
            assert_rpc(stdin_messages, "stdin", case)
            assert_rpc(expected, "stdout", case)
            completed = subprocess.run(
                [str(server)],
                input=stdin_path.read_text(encoding="utf-8"),
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                timeout=30,
                check=False,
            )
            if completed.returncode:
                raise ValueError(f"{case} server exited {completed.returncode}: {completed.stderr.strip()}")
            actual = [json.loads(line) for line in completed.stdout.splitlines() if line.strip()]
            if len(actual) != len(expected):
                raise ValueError(f"{case} expected {len(expected)} stdout messages, got {len(actual)}")
            for index, (actual_message, expected_message) in enumerate(zip(actual, expected)):
                if canonical_json(actual_message) != canonical_json(expected_message):
                    raise ValueError(f"{case} stdout[{index}] differs from fixture")
            print(f"PASS {case}")
    except (OSError, ValueError, json.JSONDecodeError, subprocess.SubprocessError) as error:
        print(str(error))
        return 1
    print(f"MCP transcript server validation complete ({len(CASES)} cases)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
