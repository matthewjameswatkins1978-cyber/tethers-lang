#!/usr/bin/env python3
"""Authoritative cross-platform Tethers verification coordinator."""

from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path

SCRIPT_ROOT = Path(__file__).resolve().parent
REPOSITORY_ROOT = SCRIPT_ROOT.parent
sys.path.insert(0, str(SCRIPT_ROOT))

from verification_support import (  # noqa: E402
    StepResult,
    command_path,
    discover_ocaml_switch,
    run_step,
    skipped_step,
)


def git_value(*args: str) -> str:
    return subprocess.run(["git", "-C", str(REPOSITORY_ROOT), *args], text=True, capture_output=True, check=True).stdout.strip()


def python_command() -> str:
    return sys.executable


def configure_native_path() -> None:
    if os.name != "nt":
        native = [
            "/usr/local/sbin", "/usr/local/bin", "/usr/sbin", "/usr/bin", "/sbin", "/bin",
            "/home/matmus/.cargo/bin", "/home/matmus/.local/bin",
        ]
        os.environ["PATH"] = os.pathsep.join(native)


def run_python(name: str, script: Path, *args: str, category: str = "required") -> StepResult:
    return run_step(name, [python_command(), str(script), *args], REPOSITORY_ROOT, category=category)


def toolchain_status(switch: str) -> dict[str, object]:
    names = ["git", "cargo", "rustc", "rustfmt", "opam", "jq"]
    found = {name: command_path(name) is not None for name in names}
    opam_dune = subprocess.run(
        ["opam", "exec", f"--switch={switch}", "--", "dune", "--version"],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    opam_ocaml = subprocess.run(
        ["opam", "exec", f"--switch={switch}", "--", "ocamlc", "-version"],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    found["dune"] = opam_dune.returncode == 0
    return {
        "status": "PASS" if all(found.values()) else "FAIL",
        "platform": platform.system().lower(),
        "expected_rust": "1.97.1",
        "expected_ocaml_switch": switch,
        "tools": found,
        "ocaml_version": opam_ocaml.stdout.strip() if opam_ocaml.returncode == 0 else None,
        "dune_version": opam_dune.stdout.strip() if opam_dune.returncode == 0 else None,
    }


def engine_commands(switch: str, release: bool) -> tuple[list[str], dict[str, str]]:
    environment = {"TETHERS_OCAML_SWITCH": switch}
    if os.name == "nt":
        command = ["pwsh.exe", "-NoProfile", "-File", str(REPOSITORY_ROOT / "scripts/prepare-current-engine.ps1"), "-OcamlSwitchPath", switch]
        if release:
            command.append("-ReleaseMode")
        return command, environment
    command = ["bash", str(REPOSITORY_ROOT / "scripts/prepare-current-engine.sh")]
    if release:
        environment["TETHERS_RELEASE_MODE"] = "1"
    return command, environment


def test_commands(switch: str, release: bool) -> tuple[list[str], dict[str, str]]:
    environment = {"TETHERS_OCAML_SWITCH": switch}
    if os.name == "nt":
        command = ["pwsh.exe", "-NoProfile", "-File", str(REPOSITORY_ROOT / "scripts/run-rust-tests.ps1"), "-OcamlSwitchPath", switch]
        if release:
            command.append("-Release")
        return command, environment
    return ["bash", str(REPOSITORY_ROOT / "scripts/run-rust-tests.sh")], environment


def main() -> int:
    configure_native_path()
    parser = argparse.ArgumentParser()
    parser.add_argument("--ocaml-switch")
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--report-path", default="verification/tethers-verification.json")
    args = parser.parse_args()
    switch = None
    switch_error = None
    try:
        switch = discover_ocaml_switch(REPOSITORY_ROOT, requested=args.ocaml_switch)
    except RuntimeError as error:
        switch_error = str(error)
    results: list[StepResult] = []
    head = git_value("rev-parse", "HEAD").lower()
    tree = git_value("rev-parse", "HEAD^{tree}").lower()
    dirty = bool(git_value("status", "--porcelain=v1", "--untracked-files=all"))

    def add(result: StepResult) -> StepResult:
        results.append(result)
        print(f"{result.status}: {result.name}")
        if result.status == "FAIL" and result.output:
            print(result.output, end="")
        return result

    add(run_python("task packet checker", REPOSITORY_ROOT / ".github/scripts/check-tethers-task-packet.py", category="external"))
    add(run_step("Rust formatting", ["cargo", "fmt", "--manifest-path", str(REPOSITORY_ROOT / "tethers-0.1/host-rust/Cargo.toml"), "--all", "--", "--check"], REPOSITORY_ROOT))
    if switch is None:
        engine_result = StepResult(
            name="current OCaml engine build and provenance",
            category="required",
            status="FAIL",
            exit_code=1,
            duration_ms=0,
            first_failure=switch_error,
            output=switch_error or "No compatible OCaml switch was discovered.",
        )
        add(engine_result)
    else:
        engine_command, engine_env = engine_commands(switch, args.release)
        engine_result = add(run_step("current OCaml engine build and provenance", engine_command, REPOSITORY_ROOT, env=engine_env))
    provenance_path = REPOSITORY_ROOT / "verification/current-engine-provenance.json"
    provenance = None
    if engine_result.status == "PASS" and provenance_path.is_file():
        try:
            provenance = json.loads(provenance_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            engine_result.status = "FAIL"
            engine_result.first_failure = "current engine provenance is not valid JSON"
    if provenance is None:
        engine_result.status = "FAIL"

    if engine_result.status == "PASS":
        opam_args = ["opam", "exec", f"--switch={switch}", "--", "dune", "runtest", "--force"]
        add(run_step("OCaml tests", opam_args, REPOSITORY_ROOT / "tethers-0.1/engine-ocaml"))
        cargo = ["cargo", "check", "--manifest-path", str(REPOSITORY_ROOT / "tethers-0.1/host-rust/Cargo.toml"), "--all-targets", "--all-features", "--locked"]
        add(run_step("Rust static checks", cargo, REPOSITORY_ROOT))
        add(run_python("warning ratchet", SCRIPT_ROOT / "check-warning-ratchet.py"))
        test_command, test_env = test_commands(switch, args.release)
        add(run_step("Rust and cross-language tests", test_command, REPOSITORY_ROOT, env=test_env))
        add(
            run_step(
                "R2 authority gate suite",
                [
                    "cargo",
                    "test",
                    "--manifest-path",
                    str(REPOSITORY_ROOT / "tethers-0.1/host-rust/Cargo.toml"),
                    "--test",
                    "r2_authority_gate",
                    "--locked",
                    "--",
                    "--test-threads=1",
                ],
                REPOSITORY_ROOT,
            )
        )
        consumer_build = add(
            run_step(
                "external consumer CLI build",
                [
                    "cargo",
                    "build",
                    "--manifest-path",
                    str(REPOSITORY_ROOT / "tethers-0.1/host-rust/Cargo.toml"),
                    "--bin",
                    "tethers",
                    "--locked",
                ],
                REPOSITORY_ROOT,
            )
        )
        if consumer_build.status == "PASS":
            executable_name = "tethers.exe" if os.name == "nt" else "tethers"
            engine_binary = REPOSITORY_ROOT / str(provenance["binary_relative_path"])
            add(
                run_python(
                    "external consumer integration",
                    REPOSITORY_ROOT / "examples/external-consumer/smoke.py",
                    "--tethers",
                    str(REPOSITORY_ROOT / "tethers-0.1/host-rust/target/debug" / executable_name),
                    "--engine",
                    str(engine_binary),
                )
            )
        else:
            add(skipped_step("external consumer integration", "external consumer CLI build failed"))
        add(run_python("protocol fixture sanity", REPOSITORY_ROOT / "tethers-0.1/scripts/check-fixtures.py"))
        add(run_python("MCP transcript suite", REPOSITORY_ROOT / "tethers-0.1/scripts/test-mcp-transcripts.py"))
        add(run_python("compatibility corpus", SCRIPT_ROOT / "check-compatibility-corpus.py"))
    else:
        reason = "current engine prerequisite failed; dependent suites were not run"
        for name in ("OCaml tests", "Rust static checks", "warning ratchet", "Rust and cross-language tests", "R2 authority gate suite", "external consumer CLI build", "external consumer integration", "protocol fixture sanity", "MCP transcript suite", "compatibility corpus"):
            add(skipped_step(name, reason))

    counts = {status: sum(result.status == status for result in results) for status in ("PASS", "FAIL", "SKIPPED WITH REASON", "NOT APPLICABLE")}
    first_failure = next((result.report() for result in results if result.status == "FAIL"), None)
    verdict = "PASS" if counts["FAIL"] == 0 else "FAIL"
    report = {
        "schema": "tethers.verify/1",
        "source_commit": head,
        "source_tree": tree,
        "dirty": dirty,
        "platform": platform.system().lower(),
        "toolchain": toolchain_status(switch) if switch else {"status": "FAIL", "platform": platform.system().lower(), "expected_rust": "1.97.1", "expected_ocaml": "5.5.0", "expected_dune": "3.24.0", "discovery_failure": switch_error},
        "engine": provenance,
        "suites": [result.report() for result in results],
        "outcome_counts": counts,
        "first_meaningful_failure": first_failure,
        "verdict": verdict,
        "release_eligible": not dirty and verdict == "PASS",
    }
    report_path = REPOSITORY_ROOT / args.report_path
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"Machine verification report: {report_path.relative_to(REPOSITORY_ROOT)}")
    print(f"Verdict: {verdict}")
    return 0 if verdict == "PASS" else 1


if __name__ == "__main__":
    sys.exit(main())
