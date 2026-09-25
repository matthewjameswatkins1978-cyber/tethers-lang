"""Real-process smoke proving an unrelated host can use Tethers safely."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

from consumer import Gate, request

MANIFEST_TEXT = (Path(__file__).parent / "fixture-ping.json").read_text(encoding="utf-8")
MANIFEST_DIGEST = "sha256:eb61b62bde489e00a4d15c37c83e6cdb1e9e378b8f13b910d4b68bd6d68c19da"
TETHER = '''tether "External consumer smoke"

anchor
    coding.task_completed

when
    project.type is "software"
    and task.changed_files greater_than 0

do
    fixture.ping
        message: anchor.task
        path: anchor.path
'''


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def context(root: Path, decision: str) -> tuple[Path, Path, Path]:
    consumer = root / "consumer"
    (consumer / "tethers").mkdir(parents=True)
    (consumer / "manifests").mkdir()
    (consumer / "host-data").mkdir()
    (consumer / "tethers" / "complete.tether").write_text(TETHER, encoding="utf-8")
    manifest_path = consumer / "manifests" / "fixture-ping.json"
    manifest_path.write_text(MANIFEST_TEXT, encoding="utf-8")
    config = {
        "format_version": "0.1", "tether_set": {"id": "external.smoke", "version": "1",
            "tethers": [{"id": "external-complete", "version": "1",
                "source_path": "tethers/complete.tether", "core_environment": {
                    "program_id": "program.external.smoke", "core_version": "1",
                    "capabilities": [{"source_name": "fixture.ping", "capability_id": "cap.fixture.ping",
                        "contract_digest": "EXTERNAL-SMOKE-1", "runtime_name": "fixture.ping"}],
                    "input_facts": [
                        {"source_name": "project.type", "fact_id": "fact.project_type",
                         "host_snapshot_key": "project.type", "scalar_type": "string", "schema_description": "project type"},
                        {"source_name": "task.changed_files", "fact_id": "fact.changed_files",
                         "host_snapshot_key": "task.changed_files", "scalar_type": "integer", "schema_description": "changed files"}]}}],
            "capability_requirements": [{"name": "fixture.ping", "version": 1, "reason": "external smoke"}]},
        "providers": [{"id": "tethers-stdio-fixture", "display_name": "Tethers Stdio Fixture",
            "transport": {"kind": "stdio", "command": "provider-must-never-run", "args": [], "protocol_version": "2025-11-25"},
            "capabilities": [{"name": "fixture.ping", "version": 1, "manifest_path": "manifests/fixture-ping.json",
                "pinned_digest": MANIFEST_DIGEST, "scope_binding": {"kind": "path_prefix", "argument_json_pointer": "/path"}}]}],
        "policy": {"default": "deny", "rules": [{"name": "fixture.ping", "version": 1, "decision": decision}]}}
    config_path = consumer / "runtime.json"
    config_path.write_text(json.dumps(config, indent=2) + "\n", encoding="utf-8")
    host_data = consumer / "host-data"
    secure_host_data(host_data)
    provision = subprocess.run([str(args.tethers), "provision-replay", str(host_data)],
                               text=True, capture_output=True, timeout=20)
    require(provision.returncode == 0, f"replay provisioning failed: {provision.stdout}{provision.stderr}")
    return config_path, consumer / "trail.jsonl", host_data


def secure_host_data(root: Path) -> None:
    """Give replay state a private root before asking Tethers to provision it."""
    if os.name == "nt":
        identity = subprocess.run(["whoami.exe"], check=True, capture_output=True, text=True).stdout.strip()
        result = subprocess.run(["icacls.exe", str(root), "/inheritance:r", "/grant:r",
                                 f"{identity}:(OI)(CI)F", "NT AUTHORITY\\SYSTEM:(OI)(CI)F",
                                 "BUILTIN\\Administrators:(OI)(CI)F"], capture_output=True, text=True)
        require(result.returncode == 0, f"failed to protect host-data root: {result.stderr}")
    else:
        root.chmod(0o700)


def run() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--tethers", required=True, type=Path)
    parser.add_argument("--engine", required=True, type=Path)
    global args
    args = parser.parse_args()
    for executable in (args.tethers, args.engine):
        require(executable.is_file() and executable.is_absolute(), f"not an absolute executable file: {executable}")
    with tempfile.TemporaryDirectory(prefix="tethers-external-consumer-") as temp:
        root = Path(temp)
        cfg, trail, host_data = context(root / "allow", "allow")
        marker = root / "allow" / "consumer" / "effect.marker"
        with Gate(args.tethers, cfg, args.engine, trail, host_data) as gate:
            prepared = gate.prepare("projects/allow")
            require(prepared["decision"] == "allow_prepared" and not prepared["authorizes_dispatch"], "ALLOW prepare contract")
            dispatch = gate.call("commit", {"prepared_id": prepared["prepared_id"]})["result"]
            require(dispatch["schema"] == "tethers.dispatch/1" and not marker.exists(), "commit is intent, not execution")
            marker.write_text("external host performed harmless fixture\n", encoding="utf-8")
            outcome = gate.call("outcome", {"execution_id": dispatch["execution_id"], "classification": "succeeded",
                "attempted": True, "external_execution_identity": "external-marker-1", "result": {"echo": "ok"},
                "evidence": "external-consumer-smoke"})["result"]
            require(outcome["trail_outcome_recorded"] and outcome["provider_invocations"] == 0, "structured outcome evidence")
            records = [json.loads(line) for line in trail.read_text(encoding="utf-8").splitlines() if line.strip()]
            require(any(record.get("execution_id") == dispatch["execution_id"] for record in records), "Trail receipt binds exact execution")

        cfg, trail, host_data = context(root / "ask", "ask")
        marker = root / "ask" / "consumer" / "effect.marker"
        with Gate(args.tethers, cfg, args.engine, trail, host_data) as gate:
            prepared = gate.prepare("projects/ask")
            require(prepared["decision"] == "ask" and not marker.exists(), "ASK performs no effect")
            commit = gate.call("commit", {"prepared_id": prepared["prepared_id"]})
            require(commit.get("error", {}).get("code") == "commit.approval_not_ready" and not marker.exists(), "ASK blocks commit without approval")
            approval = gate.call("approval_decision", {"approval_id": prepared["approval"]["approval_id"], "decision": "approve"})["result"]
            require(approval["state"] == "approved" and not approval["authorizes_dispatch"], "explicit approval is recorded, not permission")
            dispatch = gate.call("commit", {"prepared_id": prepared["prepared_id"]})["result"]
            marker.write_text("approved external effect\n", encoding="utf-8")
            outcome = gate.call("outcome", {"execution_id": dispatch["execution_id"], "classification": "succeeded",
                "attempted": True, "external_execution_identity": "external-ask-1", "result": {"echo": "approved"}})["result"]
            require(outcome["trail_outcome_recorded"], "approved outcome recorded")

        cfg, trail, host_data = context(root / "deny", "deny")
        marker = root / "deny" / "consumer" / "effect.marker"
        with Gate(args.tethers, cfg, args.engine, trail, host_data) as gate:
            prepared = gate.prepare("projects/deny")
            commit = gate.call("commit", {"prepared_id": prepared["prepared_id"]})
            require(prepared["decision"] == "deny" and commit.get("error", {}).get("code") == "commit.deny", "DENY is final")
            require(not marker.exists(), "DENY performs no effect")

        cfg, trail, host_data = context(root / "bad", "allow")
        marker = root / "bad" / "consumer" / "effect.marker"
        malformed = request(args.tethers, cfg, args.engine, trail, host_data, "prepare", {"action_id": "action_1"})
        unsupported = request(args.tethers, cfg, args.engine, trail, host_data, "hello", {}, "tethers.authority/999")
        forged = request(args.tethers, cfg, args.engine, trail, host_data, "prepare", {
            "permission": True, "action_id": "action_1", "evaluation_id": "bad", "tether": {"id": "external-complete", "version": "1"},
            "event": {"id": "e", "name": "coding.task_completed", "data": {}}, "facts": {"project.type": "software", "task.changed_files": 1}})
        require(malformed["status"] == "error" and unsupported.get("error", {}).get("code") == "frame.unsupported_schema", "malformed and unsupported protocol fail closed")
        require(forged.get("error", {}).get("code") == "frame.forbidden_authority_key", "forged authority fails closed")
        require(not marker.exists(), "malformed/forged requests perform no effect")
    print("PASS external consumer: ALLOW, ASK/approval, DENY, Trail receipt, malformed and forged authority; zero unauthorized effects")


if __name__ == "__main__":
    run()
