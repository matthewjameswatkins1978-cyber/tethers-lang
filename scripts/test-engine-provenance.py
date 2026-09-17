#!/usr/bin/env python3
"""Exercise the native runner's fail-closed provenance boundary."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    manifest = root / "verification/current-engine-provenance.json"
    runner = root / "scripts/run-rust-tests.sh"
    original = manifest.read_bytes()
    native_env = os.environ.copy()
    native_env["PATH"] = os.pathsep.join([
        "/usr/local/sbin", "/usr/local/bin", "/usr/sbin", "/usr/bin", "/sbin", "/bin",
        "/home/matmus/.cargo/bin", "/home/matmus/.local/bin",
    ])

    def run() -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["bash", str(runner), "--verify-only"],
            cwd=root,
            env=native_env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
        )

    try:
        data = json.loads(original)
        mutations = [
            ("stale source commit", {**data, "source_commit": "0" * 40}),
            ("tampered binary hash", {**data, "binary_sha256": "0" * 64}),
            ("missing engine", {**data, "binary_relative_path": "missing-engine"}),
        ]
        for label, mutated in mutations:
            manifest.write_text(json.dumps(mutated) + "\n", encoding="utf-8")
            result = run()
            if result.returncode == 0 or "VERIFICATION PREREQUISITE FAILED" not in result.stdout:
                raise AssertionError(f"runner accepted {label}: {result.stdout}")
            print(f"PASS {label} rejected")
        manifest.unlink()
        result = run()
        if result.returncode == 0 or "provenance manifest was not produced" not in result.stdout:
            raise AssertionError(f"runner accepted missing provenance: {result.stdout}")
        print("PASS missing provenance rejected")
    finally:
        manifest.write_bytes(original)

    result = run()
    if result.returncode:
        print(result.stdout, end="")
        return result.returncode
    print("PASS valid provenance accepted")
    return 0


if __name__ == "__main__":
    sys.exit(main())
