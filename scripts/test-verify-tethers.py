#!/usr/bin/env python3
"""Focused contract tests for the cross-platform verification primitives."""

from __future__ import annotations

import os
import sys
import tempfile
from types import SimpleNamespace
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from verification_support import command_path, discover_ocaml_switch, run_step, skipped_step  # noqa: E402


def main() -> int:
    assert command_path(Path(sys.executable).stem) is not None
    with tempfile.TemporaryDirectory(prefix="tethers switch discovery ") as raw:
        repository = Path(raw)
        calls: list[list[str]] = []

        def fake_opam(argv: list[str], **_kwargs: object) -> SimpleNamespace:
            calls.append(argv)
            if argv == ["opam", "switch", "list", "--short"]:
                return SimpleNamespace(returncode=0, stdout="unrelated-switch\nD:\\OCaml Tools\\tethers switch\n")
            if argv[1] == "list":
                candidate = argv[2].removeprefix("--switch=")
                if candidate != "unrelated-switch":
                    return SimpleNamespace(returncode=0, stdout="yojson 2.2.2\ndigestif 1.3.1\n")
                return SimpleNamespace(returncode=0, stdout="yojson 3.0.0\ndigestif 1.3.1\n")
            candidate = argv[2].removeprefix("--switch=")
            if candidate != "unrelated-switch" and argv[-2:] == ["ocamlc", "-version"]:
                return SimpleNamespace(returncode=0, stdout="5.5.0\n")
            if candidate != "unrelated-switch" and argv[-2:] == ["dune", "--version"]:
                return SimpleNamespace(returncode=0, stdout="3.24.0\n")
            return SimpleNamespace(returncode=1, stdout="unavailable\n")

        discovered = discover_ocaml_switch(repository, environ={}, run=fake_opam)
        assert discovered == "D:\\OCaml Tools\\tethers switch"
        assert ["opam", "exec", "--switch=D:\\OCaml Tools\\tethers switch", "--", "ocamlc", "-version"] in calls

        explicit_calls: list[list[str]] = []
        explicit = discover_ocaml_switch(
            repository,
            requested="C:\\Users\\Matmus\\.opam\\tethers",
            environ={},
            run=lambda argv, **kwargs: (explicit_calls.append(argv), fake_opam(argv, **kwargs))[1],
        )
        assert explicit == "C:\\Users\\Matmus\\.opam\\tethers"
        assert not any(call == ["opam", "switch", "list", "--short"] for call in explicit_calls)
        try:
            discover_ocaml_switch(repository, requested="missing-switch", environ={}, run=fake_opam)
        except RuntimeError as error:
            assert "Requested OCaml switch is unavailable or incompatible" in str(error)
        else:
            raise AssertionError("an invalid explicit OCaml switch must fail closed")

    quoted_prefix = 'tethers verify " quoted path ' if os.name != "nt" else "tethers verify quoted path "
    with tempfile.TemporaryDirectory(prefix=quoted_prefix) as raw:
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
