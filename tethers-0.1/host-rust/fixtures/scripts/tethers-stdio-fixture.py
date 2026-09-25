#!/usr/bin/env python3
"""MCP stdio fixture provider for Tethers host tests (POSIX counterpart).

Implements the same modes and request semantics as
tethers-stdio-fixture.ps1 using only the Python standard library, so Linux
(and other non-Windows) test runs exercise the identical fixture contract.
Reads JSON-RPC requests line by line from stdin, writes one compact JSON
response per line to stdout, diagnostics to stderr.

Arguments (PowerShell-style flags, as passed by the Rust test harness):
  -Mode <mode>                 Fixture mode (default: valid).
  -MarkerFile <path>           Append method markers for run/record modes.
  -CwdMarkerFile <path>        Record the working directory (record-cwd mode).
  -BarrierDirectory <path>     Overlap barrier root (c2-overlap-barrier mode).
"""

import json
import os
import re
import subprocess
import sys
import time

PROTOCOL_VERSION = "2025-11-25"
SERVER_NAME = "tethers-stdio-fixture"
SERVER_VERSION = "0.1.0"


def parse_args(argv):
    args = {"Mode": "valid", "MarkerFile": "", "CwdMarkerFile": "", "BarrierDirectory": ""}
    i = 0
    while i < len(argv):
        name = argv[i]
        if name.startswith("-") and i + 1 < len(argv):
            key = name[1:]
            if key in args:
                args[key] = argv[i + 1]
                i += 2
                continue
        i += 1
    return args


ARGS = parse_args(sys.argv[1:])
MODE = ARGS["Mode"]
MARKER_FILE = ARGS["MarkerFile"]
CWD_MARKER_FILE = ARGS["CwdMarkerFile"]
BARRIER_DIRECTORY = ARGS["BarrierDirectory"]

TOOLS_LIST_COUNT = 0
INITIALIZED = False
CLIENT_INITIALIZED = False


def log(message):
    sys.stderr.write("fixture: %s\n" % message)
    sys.stderr.flush()


def emit(obj):
    sys.stdout.write(json.dumps(obj, separators=(",", ":")) + "\n")
    sys.stdout.flush()


def emit_error(request_id, code, message):
    emit({"jsonrpc": "2.0", "id": request_id, "error": {"code": code, "message": message}})


def mark(text):
    if MARKER_FILE:
        with open(MARKER_FILE, "a", encoding="utf-8") as handle:
            handle.write(text + "\n")


def new_tool():
    name = "fixture_other" if MODE == "wrong-tool" else "fixture_ping"
    description = (
        "Provider-controlled description changed."
        if MODE == "changed-description"
        else "Echo one message for deterministic provider binding tests."
    )
    input_schema = {
        "type": "object",
        "properties": {"message": {"type": "string"}},
        "required": ["message"],
        "additionalProperties": False,
    }
    if MODE in ("run-success", "run-explicit-error", "run-invalid-output", "run-hang-call"):
        input_schema["properties"]["path"] = {"type": "string"}
        input_schema["required"] = ["message", "path"]
    if MODE == "input-schema-mismatch":
        input_schema["required"] = ["different"]
    output_schema = {
        "type": "object",
        "properties": {"echo": {"type": "string"}},
        "required": ["echo"],
        "additionalProperties": False,
    }
    if MODE == "output-schema-mismatch":
        output_schema["required"] = ["different"]
    return {
        "name": name,
        "description": description,
        "inputSchema": input_schema,
        "outputSchema": output_schema,
    }


def hang():
    while True:
        time.sleep(60)


# --- Special modes that bypass the main loop ---

if MODE == "hang-initialize":
    log("hanging during initialize")
    hang()
if MODE == "run-hang-initialize":
    if MARKER_FILE:
        mark("provider_started")
    log("hanging during run initialization")
    hang()
if MODE == "stdout-log-text":
    log("emitting log text on stdout")
    sys.stdout.write("LOG: this is not JSON\n")
    sys.stdout.flush()
if MODE == "oversized-line":
    log("emitting oversized protocol line")
    sys.stdout.write("x" * (9 * 1024 * 1024) + "\n")
    sys.stdout.flush()
if MODE == "retained-stderr":
    log("writing diagnostic to stderr")
    log("additional diagnostic data")
if MODE == "descendant-alive":
    log("spawning descendant process")
    try:
        child = subprocess.Popen(["sleep", "300"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        log("descendant PID=%d" % child.pid)
    except OSError as error:
        log("could not spawn descendant: %s" % error)
if MODE == "record-cwd" and CWD_MARKER_FILE:
    with open(CWD_MARKER_FILE, "w", encoding="utf-8") as handle:
        handle.write(os.getcwd())
    log("recorded CWD to %s" % CWD_MARKER_FILE)
if MODE == "exit-early":
    log("exiting before initialization")
    sys.stderr.flush()
    time.sleep(0.05)
    sys.exit(0)


def handle_initialize(request):
    global INITIALIZED
    if MODE in ("record-methods", "run-success", "run-explicit-error",
                "run-invalid-output", "run-hang-call", "missing-tool") and MARKER_FILE:
        mark("initialize")
    if MODE == "malformed-json":
        sys.stdout.write("{not-json\n")
        sys.stdout.flush()
        return
    if MODE == "initialization-error":
        emit_error(request.get("id"), -32602, "Initialization rejected by fixture")
        return
    selected_version = "1900-01-01" if MODE == "incompatible-version" else PROTOCOL_VERSION
    server_name = "unexpected-provider" if MODE == "server-name-mismatch" else SERVER_NAME
    INITIALIZED = True
    emit({
        "jsonrpc": "2.0",
        "id": request.get("id"),
        "result": {
            "protocolVersion": selected_version,
            "capabilities": {"tools": {}},
            "serverInfo": {"name": server_name, "version": SERVER_VERSION},
        },
    })


def handle_initialized_notification():
    global CLIENT_INITIALIZED
    if not INITIALIZED:
        log("initialized notification before initialize")
        return
    CLIENT_INITIALIZED = True


def handle_tools_list(request):
    global TOOLS_LIST_COUNT
    TOOLS_LIST_COUNT += 1
    if MODE in ("record-methods", "run-success", "run-explicit-error",
                "run-invalid-output", "run-hang-call", "missing-tool") and MARKER_FILE:
        mark("tools/list")
    if MODE == "hang-tools-list":
        time.sleep(3600)
    if not CLIENT_INITIALIZED:
        emit_error(request.get("id"), -32002, "Server not initialized")
        return
    params = request.get("params") or {}
    if MODE in ("paginated-tools", "cursor-loop", "paged-duplicate"):
        if "cursor" not in params:
            emit({
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "result": {"tools": [new_tool()], "nextCursor": "opaque::+/="},
            })
            return
        if params.get("cursor") != "opaque::+/=":
            emit_error(request.get("id"), -32602, "opaque cursor was changed")
            return
        if MODE == "paged-duplicate":
            page_tools = [new_tool()]
        else:
            page_tools = [{
                "name": "fixture_unapproved_addition",
                "description": "Untrusted additional operation.",
                "inputSchema": {"type": "object"},
                "outputSchema": {"type": "object"},
                "annotations": {"readOnlyHint": True},
            }]
        result = {"tools": page_tools}
        if MODE == "cursor-loop":
            result["nextCursor"] = "opaque::+/="
        emit({"jsonrpc": "2.0", "id": request.get("id"), "result": result})
        return
    tools = []
    if MODE != "missing-tool":
        tools.append(new_tool())
    if MODE == "duplicate-tool":
        tools.append(new_tool())
    if MODE == "catalogue-change-drift" and TOOLS_LIST_COUNT > 1:
        tools[0]["inputSchema"]["required"] = ["different"]
    if MODE in ("catalogue-change-unchanged", "catalogue-change-drift") and TOOLS_LIST_COUNT == 1:
        emit({"jsonrpc": "2.0", "method": "notifications/tools/list_changed", "params": {}})
    emit({"jsonrpc": "2.0", "id": request.get("id"), "result": {"tools": tools}})


def member_token(message):
    match = re.search(r"(?:hello-from-|member[-/])([a-z0-9]+)$", message or "")
    if match:
        return "member-%s" % match.group(1)
    return "%d-%s" % (os.getpid(), os.urandom(8).hex())


def wait_for(predicate, timeout_seconds=10):
    deadline = time.monotonic() + timeout_seconds
    while not predicate():
        if time.monotonic() > deadline:
            return False
        time.sleep(0.01)
    return True


def handle_barrier_call(request):
    if not BARRIER_DIRECTORY:
        emit_error(request.get("id"), -32602, "BarrierDirectory is required")
        return
    os.makedirs(BARRIER_DIRECTORY, exist_ok=True)
    arguments = (request.get("params") or {}).get("arguments") or {}
    token = member_token(arguments.get("message"))
    entered = os.path.join(BARRIER_DIRECTORY, "entered-%s" % token)
    active = os.path.join(BARRIER_DIRECTORY, "active-%s" % token)
    with open(entered, "w", encoding="utf-8") as handle:
        handle.write("entered")
    peer_count = 2
    peer_count_file = os.path.join(BARRIER_DIRECTORY, "peer-count")
    if os.path.isfile(peer_count_file):
        with open(peer_count_file, "r", encoding="utf-8") as handle:
            peer_count = int(handle.read().strip())

    def peers_entered():
        try:
            return sum(1 for name in os.listdir(BARRIER_DIRECTORY) if name.startswith("entered-")) >= peer_count
        except OSError:
            return False

    if not wait_for(peers_entered):
        emit_error(request.get("id"), -32000, "overlap peer did not enter")
        return
    with open(active, "w", encoding="utf-8") as handle:
        handle.write("active")
    release_member = os.path.join(BARRIER_DIRECTORY, "release-%s" % token)
    release_shared = os.path.join(BARRIER_DIRECTORY, "release")

    def released():
        return os.path.exists(release_member) or os.path.exists(release_shared)

    if not wait_for(released):
        emit_error(request.get("id"), -32000, "overlap release timed out")
        return
    outcome_file = os.path.join(BARRIER_DIRECTORY, "outcome-%s" % token)
    outcome = "success"
    if os.path.isfile(outcome_file):
        with open(outcome_file, "r", encoding="utf-8") as handle:
            outcome = handle.read().strip()
    if outcome == "uncertain":
        # No result and no error: the host classifies this as
        # NoFinalResponse -> Uncertain.
        emit({"jsonrpc": "2.0", "id": request.get("id")})
        return
    if outcome == "failed":
        emit_error(request.get("id"), -32000, "controlled provider failure")
        return
    emit({"jsonrpc": "2.0", "id": request.get("id"), "result": {"echo": arguments.get("message")}})


def handle_tools_call(request):
    if MODE == "c2-overlap-barrier":
        handle_barrier_call(request)
        return
    if MODE == "run-success":
        if MARKER_FILE:
            mark("tools/call")
        message = (request.get("params") or {}).get("arguments", {}).get("message")
        if not isinstance(message, str):
            emit_error(request.get("id"), -32602, "fixture_ping requires string message")
            return
        emit({"jsonrpc": "2.0", "id": request.get("id"), "result": {"echo": message}})
        return
    if MODE == "run-explicit-error":
        if MARKER_FILE:
            mark("tools/call")
        emit_error(request.get("id"), -32600, "fixture explicit error for negative matrix")
        return
    if MODE == "run-invalid-output":
        if MARKER_FILE:
            mark("tools/call")
        emit({"jsonrpc": "2.0", "id": request.get("id"), "result": {"wrong_field": "not echo"}})
        return
    if MODE == "run-hang-call":
        if MARKER_FILE:
            mark("tools/call")
        log("hanging during tools/call")
        hang()
    if MODE == "record-methods" and MARKER_FILE:
        mark("tools/call")
    emit_error(request.get("id"), -32601, "Method not found")


def handle_ping(request):
    if MODE == "catalogue-change-on-probe":
        emit({"jsonrpc": "2.0", "method": "notifications/tools/list_changed", "params": {}})
    emit({"jsonrpc": "2.0", "id": request.get("id"), "result": {}})


def main():
    for line in sys.stdin:
        if not line.strip():
            continue
        try:
            request = json.loads(line)
        except ValueError:
            log("malformed JSON input")
            continue
        if not isinstance(request, dict) or request.get("jsonrpc") != "2.0" or not isinstance(request.get("method"), str):
            request_id = request.get("id") if isinstance(request, dict) else None
            emit_error(request_id, -32600, "Invalid Request")
            continue
        method = request["method"]
        if method == "initialize":
            handle_initialize(request)
        elif method == "notifications/initialized":
            handle_initialized_notification()
        elif method == "tools/list":
            handle_tools_list(request)
        elif method == "tools/call":
            handle_tools_call(request)
        elif method == "ping":
            handle_ping(request)
        else:
            if MODE == "record-methods" and MARKER_FILE:
                mark(method)
            emit_error(request.get("id"), -32601, "Method not found")


main()
