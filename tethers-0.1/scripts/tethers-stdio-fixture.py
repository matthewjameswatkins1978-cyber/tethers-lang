#!/usr/bin/env python3
"""Native Linux equivalent of tethers-stdio-fixture.ps1.

This is test infrastructure only.  It deliberately mirrors the protocol
behaviour used by the Windows fixture without changing product semantics.
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import time
import uuid
from pathlib import Path


PROTOCOL_VERSION = "2025-11-25"
RUN_MODES = {"run-success", "run-explicit-error", "run-invalid-output", "run-hang-call"}


def parse_args(argv: list[str]) -> dict[str, str]:
    values = {
        "Mode": "valid",
        "MarkerFile": "",
        "CwdMarkerFile": "",
        "BarrierDirectory": "",
    }
    index = 0
    while index < len(argv):
        key = argv[index].lstrip("-")
        if key in values and index + 1 < len(argv):
            values[key] = argv[index + 1]
            index += 2
        else:
            index += 1
    return values


def write_json(value: object) -> None:
    sys.stdout.write(json.dumps(value, separators=(",", ":")) + "\n")
    sys.stdout.flush()


def write_error(request_id: object, code: int, message: str) -> None:
    write_json({"jsonrpc": "2.0", "id": request_id, "error": {"code": code, "message": message}})


def append_marker(path: str, value: str) -> None:
    if path:
        with open(path, "a", encoding="utf-8") as marker:
            marker.write(value + "\n")


def new_tool(mode: str) -> dict[str, object]:
    name = "fixture_other" if mode == "wrong-tool" else "fixture_ping"
    description = (
        "Provider-controlled description changed."
        if mode == "changed-description"
        else "Echo one message for deterministic provider binding tests."
    )
    properties: dict[str, object] = {"message": {"type": "string"}}
    required = ["message"]
    if mode in RUN_MODES:
        properties["path"] = {"type": "string"}
        required = ["message", "path"]
    if mode == "input-schema-mismatch":
        required = ["different"]
    output_required = ["different"] if mode == "output-schema-mismatch" else ["echo"]
    return {
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": properties,
            "required": required,
            "additionalProperties": False,
        },
        "outputSchema": {
            "type": "object",
            "properties": {"echo": {"type": "string"}},
            "required": output_required,
            "additionalProperties": False,
        },
    }


def write_tools(request_id: object, mode: str) -> None:
    if mode == "missing-tool":
        tools: list[object] = []
    else:
        tool = new_tool(mode)
        tools = [tool, tool.copy()] if mode == "duplicate-tool" else [tool]
    write_json({"jsonrpc": "2.0", "id": request_id, "result": {"tools": tools}})


def barrier_call(request: dict[str, object], directory: str) -> str:
    if not directory:
        return "success"
    barrier = Path(directory)
    barrier.mkdir(parents=True, exist_ok=True)
    params = request.get("params") or {}
    arguments = params.get("arguments") if isinstance(params, dict) else {}
    message = arguments.get("message", "") if isinstance(arguments, dict) else ""
    match = re.search(r"(?:hello-from-|member[-/])([a-z0-9]+)$", str(message))
    token = f"member-{match.group(1)}" if match else f"{os.getpid()}-{uuid.uuid4().hex}"
    (barrier / f"entered-{token}").touch()
    peer_count_path = barrier / "peer-count"
    peer_count = int(peer_count_path.read_text(encoding="utf-8").strip()) if peer_count_path.exists() else 2
    deadline = time.monotonic() + 10
    while len(list(barrier.glob("entered-*"))) < peer_count:
        if time.monotonic() > deadline:
            return "peer-timeout"
        time.sleep(0.01)
    (barrier / f"active-{token}").touch()
    while not (barrier / f"release-{token}").exists() and not (barrier / "release").exists():
        if time.monotonic() > deadline:
            return "release-timeout"
        time.sleep(0.01)
    outcome_path = barrier / f"outcome-{token}"
    return outcome_path.read_text(encoding="utf-8").strip() if outcome_path.exists() else "success"


def main() -> int:
    options = parse_args(sys.argv[1:])
    mode = options["Mode"]
    marker_file = options["MarkerFile"]
    barrier_directory = options["BarrierDirectory"]

    if mode == "record-cwd" and options["CwdMarkerFile"]:
        Path(options["CwdMarkerFile"]).write_text(os.getcwd(), encoding="utf-8")
        print(f"fixture: recorded CWD to {options['CwdMarkerFile']}", file=sys.stderr, flush=True)
    if mode in {"hang-initialize", "run-hang-initialize"}:
        if mode == "run-hang-initialize":
            append_marker(marker_file, "provider_started")
        print("fixture: hanging during initialize", file=sys.stderr, flush=True)
        while True:
            time.sleep(60)
    if mode == "stdout-log-text":
        print("fixture: emitting log text on stdout", file=sys.stderr, flush=True)
        print("LOG: this is not JSON", flush=True)
    if mode == "oversized-line":
        print("fixture: emitting oversized protocol line", file=sys.stderr, flush=True)
        print("x" * (9 * 1024 * 1024), flush=True)
    if mode == "retained-stderr":
        print("fixture: writing diagnostic to stderr", file=sys.stderr, flush=True)
        print("fixture: additional diagnostic data", file=sys.stderr, flush=True)
    if mode == "descendant-alive":
        child = subprocess.Popen(["sleep", "300"])
        print("fixture: spawning descendant process", file=sys.stderr, flush=True)
        print(f"fixture: descendant PID={child.pid}", file=sys.stderr, flush=True)
    if mode == "exit-early":
        print("fixture: exiting before initialization", file=sys.stderr, flush=True)
        time.sleep(0.05)
        return 0

    initialized = False
    client_initialized = False
    tools_list_count = 0
    for line in sys.stdin:
        if not line.strip():
            continue
        try:
            request = json.loads(line)
        except json.JSONDecodeError:
            print("fixture: malformed JSON input", file=sys.stderr, flush=True)
            continue
        request_id = request.get("id")
        method = request.get("method")
        if request.get("jsonrpc") != "2.0" or not isinstance(method, str):
            write_error(request_id, -32600, "Invalid Request")
            continue

        if method == "initialize":
            if mode in {"record-methods", *RUN_MODES, "missing-tool"}:
                append_marker(marker_file, "initialize")
            if mode == "malformed-json":
                print("{not-json", flush=True)
                continue
            if mode == "initialization-error":
                write_error(request_id, -32602, "Initialization rejected by fixture")
                continue
            version = "1900-01-01" if mode == "incompatible-version" else PROTOCOL_VERSION
            name = "unexpected-provider" if mode == "server-name-mismatch" else "tethers-stdio-fixture"
            initialized = True
            write_json({
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "protocolVersion": version,
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": name, "version": "0.1.0"},
                },
            })
        elif method == "notifications/initialized":
            if initialized:
                client_initialized = True
        elif method == "tools/list":
            tools_list_count += 1
            if mode in {"record-methods", *RUN_MODES, "missing-tool"}:
                append_marker(marker_file, "tools/list")
            if mode == "hang-tools-list":
                time.sleep(3600)
            if not client_initialized:
                write_error(request_id, -32002, "Server not initialized")
                continue
            if mode in {"paginated-tools", "cursor-loop", "paged-duplicate"}:
                params = request.get("params") or {}
                has_cursor = isinstance(params, dict) and "cursor" in params
                if not has_cursor:
                    write_json({"jsonrpc": "2.0", "id": request_id, "result": {"tools": [new_tool(mode)], "nextCursor": "opaque::+/="}})
                    continue
                if params.get("cursor") != "opaque::+/=":
                    write_error(request_id, -32602, "opaque cursor was changed")
                    continue
                page = [new_tool(mode)] if mode == "paged-duplicate" else [{
                    "name": "fixture_unapproved_addition",
                    "description": "Untrusted additional operation.",
                    "inputSchema": {"type": "object"},
                    "outputSchema": {"type": "object"},
                    "annotations": {"readOnlyHint": True},
                }]
                result: dict[str, object] = {"tools": page}
                if mode == "cursor-loop":
                    result["nextCursor"] = "opaque::+/="
                write_json({"jsonrpc": "2.0", "id": request_id, "result": result})
                continue
            effective_mode = "input-schema-mismatch" if mode == "catalogue-change-drift" and tools_list_count > 1 else mode
            if mode in {"catalogue-change-unchanged", "catalogue-change-drift"} and tools_list_count == 1:
                write_json({"jsonrpc": "2.0", "method": "notifications/tools/list_changed", "params": {}})
            write_tools(request_id, effective_mode)
        elif method == "tools/call":
            append_marker(marker_file, "tools/call")
            if mode == "c2-overlap-barrier":
                outcome = barrier_call(request, barrier_directory)
                if outcome in {"peer-timeout", "release-timeout"}:
                    write_error(request_id, -32000, "overlap peer did not enter" if outcome == "peer-timeout" else "overlap release timed out")
                elif outcome == "failed":
                    write_error(request_id, -32000, "controlled provider failure")
                elif outcome == "uncertain":
                    write_json({"jsonrpc": "2.0", "id": request_id})
                else:
                    params = request.get("params") or {}
                    args = params.get("arguments", {}) if isinstance(params, dict) else {}
                    write_json({"jsonrpc": "2.0", "id": request_id, "result": {"echo": args.get("message", "")}})
                continue
            if mode == "run-success":
                params = request.get("params") or {}
                args = params.get("arguments", {}) if isinstance(params, dict) else {}
                message = args.get("message") if isinstance(args, dict) else None
                if not isinstance(message, str):
                    write_error(request_id, -32602, "fixture_ping requires string message")
                else:
                    write_json({"jsonrpc": "2.0", "id": request_id, "result": {"echo": message}})
            elif mode == "run-explicit-error":
                write_error(request_id, -32600, "fixture explicit error for negative matrix")
            elif mode == "run-invalid-output":
                write_json({"jsonrpc": "2.0", "id": request_id, "result": {"wrong_field": "not echo"}})
            elif mode == "run-hang-call":
                print("fixture: hanging during tools/call", file=sys.stderr, flush=True)
                while True:
                    time.sleep(60)
            elif mode == "record-methods":
                append_marker(marker_file, "tools/call")
                write_error(request_id, -32601, "Method not found")
            else:
                write_error(request_id, -32601, "Method not found")
        elif method == "ping":
            if mode == "catalogue-change-on-probe":
                write_json({"jsonrpc": "2.0", "method": "notifications/tools/list_changed", "params": {}})
            write_json({"jsonrpc": "2.0", "id": request_id, "result": {}})
        else:
            if mode == "record-methods":
                append_marker(marker_file, method)
            write_error(request_id, -32601, "Method not found")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
