#!/usr/bin/env python3
"""Check that Cargo warnings remain within the accepted repository baseline."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

from verification_support import run_step


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    manifest = root / "tethers-0.1/host-rust/Cargo.toml"
    result = run_step(
        "Cargo warning-ratchet probe",
        ["cargo", "check", "--manifest-path", str(manifest), "--all-targets", "--all-features", "--locked"],
        root,
    )
    if result.status != "PASS":
        print(result.output, end="")
        return result.exit_code or 1
    warnings = [line for line in result.output.splitlines() if "warning:" in line.lower()]
    unaccepted = [
        line
        for line in warnings
        if "present in multiple build targets" not in line.lower()
        and "duplicate target" not in line.lower()
        and "associated function `from_discovered` is never used" not in line.lower()
        and "`tethers-reference-host` (lib test) generated 1 warning" not in line.lower()
    ]
    if unaccepted:
        print("New warning(s) are outside the accepted baseline:", " | ".join(unaccepted))
        return 1
    if warnings:
        print("PASS warning ratchet: existing duplicate-target warning remains within baseline")
    else:
        print("PASS warning ratchet: no warnings emitted")
    return 0


if __name__ == "__main__":
    sys.exit(main())
