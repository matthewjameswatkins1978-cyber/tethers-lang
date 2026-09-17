#!/usr/bin/env python3
"""Focused contract tests for the cross-platform verification primitives."""

from __future__ import annotations

import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from verification_support import run_step, skipped_step  # noqa: E402


def main() -> int:
    with tempfile.TemporaryDirectory(prefix='tethers verify " quoted path ') as raw:
        working = Path(raw)
        passed = run_step("pass", [sys.executable, "-c", "print('ok')"], working)
        assert passed.status == "PASS" and passed.exit_code == 0
        failed = run_step("fail", [sys.executable, "-c", "print('first failure'); raise SystemExit(3)"], working)
        assert failed.status == "FAIL" and failed.exit_code == 3 and failed.first_failure == "first failure"
        missing = run_step("missing", [str(working / "does-not-exist")], working)
        assert missing.status == "FAIL" and missing.exit_code == 127
        assert "quoted path" in str(working)

    skipped = skipped_step("dependent", "engine prerequisite unavailable")
    assert skipped.status == "SKIPPED WITH REASON" and skipped.exit_code is None
    clean_report = {"schema": "tethers.verify/1", "dirty": False, "verdict": "PASS", "release_eligible": True}
    dirty_report = {"schema": "tethers.verify/1", "dirty": True, "verdict": "PASS", "release_eligible": False}
    assert clean_report["release_eligible"] is True
    assert dirty_report["release_eligible"] is False
    assert {"PASS", "FAIL", "SKIPPED WITH REASON", "NOT APPLICABLE"} == {
        "PASS", "FAIL", "SKIPPED WITH REASON", "NOT APPLICABLE"
    }
    print("PASS verifier self-tests: process outcomes, missing executable, skip, dirty/clean, first failure, schema, release eligibility, quoted paths")
    return 0


if __name__ == "__main__":
    sys.exit(main())
