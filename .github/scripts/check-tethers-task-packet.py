#!/usr/bin/env python3
"""Native task-packet consistency checker used by all platforms.

This is the single semantic authority for task-packet validation. The PowerShell
entrypoint is intentionally only a compatibility launcher.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path


VALID_STATUSES = {
    "PROPOSED",
    "READY",
    "IN_PROGRESS",
    "BLOCKED",
    "COMPLETE",
    "ACCEPTED",
    "REJECTED",
}
VALID_COLOURS = {"Green", "Amber", "Red"}
TERMINAL_STATUSES = {"BLOCKED", "COMPLETE", "ACCEPTED", "REJECTED"}
COMPLETED_STATUSES = {"COMPLETE", "ACCEPTED", "REJECTED"}
REQUIRED_PACKET_SECTIONS = (
    "Objective",
    "Relevant background and existing behaviour",
    "Required behaviour",
    "Relevant components",
    "Frozen decisions and invariants",
    "Acceptance criteria",
    "Required verification",
    "Forbidden changes",
    "Stop conditions",
    "Expected pre-existing changes",
)
REQUIRED_NOTE_FIELDS = (
    "Task",
    "Task packet",
    "Owner",
    "Status",
    "Base commit",
    "Implementation checkpoint",
)
REQUIRED_NOTE_SECTIONS = (
    "Requested outcome",
    "Changes made",
    "Decisions and assumptions",
    "Evidence",
    "Discoveries",
    "Remaining risks",
    "Smallest next action",
    "References",
)


def field(text: str, name: str, *, required: bool = True) -> str | None:
    match = re.search(rf"(?mi)^{re.escape(name)}:\s*`([^`]+)`\s*$", text)
    if not match and required:
        raise ValueError(f"Task packet must contain '{name}: `value`'.")
    return match.group(1).strip() if match else None


def section(text: str, name: str, *, required: bool = True) -> str | None:
    match = re.search(rf"(?ims)^## {re.escape(name)}\s*(.*?)(?=^## |\Z)", text)
    if not match and required:
        raise ValueError(f"Task packet is missing section: {name}")
    return match.group(1).strip() if match else None


def run_git(root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", "-C", str(root), *args],
        text=True,
        capture_output=True,
        check=False,
    )


def git(root: Path, *args: str) -> str:
    result = run_git(root, *args)
    if result.returncode:
        detail = result.stderr.strip() or result.stdout.strip()
        raise ValueError(f"git {' '.join(args)} failed: {detail}")
    return result.stdout.strip()


def git_lines(root: Path, *args: str) -> list[str]:
    output = git(root, *args)
    return output.splitlines() if output else []


def assert_checkpoint(root: Path, checkpoint: str, base: str, head: str, label: str) -> str:
    if not re.fullmatch(r"[0-9a-fA-F]{40}", checkpoint or ""):
        raise ValueError(f"{label} must be one full 40-character commit SHA.")
    normalized = checkpoint.lower()
    exists = run_git(root, "cat-file", "-e", f"{normalized}^{{commit}}")
    if exists.returncode:
        raise ValueError(f"{label} does not identify a local commit: {normalized}")
    if run_git(root, "merge-base", "--is-ancestor", base, normalized).returncode:
        raise ValueError(f"Base commit {base} is not an ancestor of {label} {normalized}.")
    if run_git(root, "merge-base", "--is-ancestor", normalized, head).returncode:
        raise ValueError(f"{label} {normalized} is not an ancestor of HEAD {head}.")
    return normalized


def assert_worker_note(
    root: Path,
    relative_path: str,
    expected_task_status: str,
    expected_owner: str,
    expected_base_commit: str,
    expected_packet_path: str,
) -> tuple[str, str]:
    if not re.fullmatch(r"docs/worker-notes/[a-z0-9][a-z0-9._-]*\.md", relative_path, re.IGNORECASE):
        raise ValueError(f"Worker note must be a safe Markdown path under docs/worker-notes/: {relative_path}")

    full_path = root / relative_path
    if not full_path.is_file():
        raise ValueError(f"Required worker note does not exist: {relative_path}")
    note = full_path.read_text(encoding="utf-8")

    for name in REQUIRED_NOTE_FIELDS:
        value = field(note, name)
        if not value or "<" in value:
            raise ValueError(f"Worker note contains an unresolved placeholder in: {name}")

    for name in REQUIRED_NOTE_SECTIONS:
        body = section(note, name)
        if not body:
            raise ValueError(f"Worker note section is empty: {name}")
        if re.search(r"(?i)^(state|list|record|give|include|say)\s+(what|exact|only|one|unexpected)", body):
            raise ValueError(f"Worker note still contains template instructions: {name}")

    if field(note, "Owner") != expected_owner:
        raise ValueError("Worker note Owner does not match task packet Owner.")
    if (field(note, "Base commit") or "").lower() != expected_base_commit:
        raise ValueError("Worker note Base commit does not match the task packet.")
    if field(note, "Task packet") != expected_packet_path:
        raise ValueError("Worker note Task packet does not match the checked packet path.")

    checkpoint = field(note, "Implementation checkpoint") or ""
    if checkpoint != "WORKTREE" and not re.fullmatch(r"[0-9a-fA-F]{40}", checkpoint):
        raise ValueError("Implementation checkpoint must be WORKTREE or one full 40-character commit SHA.")

    note_status = field(note, "Status") or ""
    if expected_task_status == "BLOCKED" and note_status != "BLOCKED":
        raise ValueError("A BLOCKED task requires a BLOCKED worker note.")
    if expected_task_status in COMPLETED_STATUSES and note_status != "COMPLETE":
        raise ValueError(f"{expected_task_status} requires a COMPLETE worker note.")

    return note, checkpoint


def normalize_scope_path(value: str) -> str:
    return value.replace("\\", "/")


def porcelain_path(line: str) -> str | None:
    if len(line) < 4:
        return None
    value = line[3:]
    if " -> " in value:
        value = value.split(" -> ")[-1]
    return normalize_scope_path(value.strip('"'))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--packet", default="docs/CURRENT_CLINE_TASK.md")
    parser.add_argument("--skip-worktree-check", action="store_true")
    args = parser.parse_args()

    try:
        root = Path(git(Path.cwd(), "rev-parse", "--show-toplevel"))
        packet_path = root / args.packet
        if not packet_path.is_file():
            raise ValueError(f"Task packet not found: {args.packet}")
        text = packet_path.read_text(encoding="utf-8")

        base_match = re.search(r"(?mi)^Base commit:\s*`([0-9a-fA-F]{40})`\s*$", text)
        if not base_match:
            raise ValueError("Task packet must contain one full 40-character Base commit SHA.")
        base = base_match.group(1).lower()
        head = git(root, "rev-parse", "HEAD").lower()

        if run_git(root, "cat-file", "-e", f"{base}^{{commit}}").returncode:
            raise ValueError(f"Base commit does not identify a local commit: {base}")
        if run_git(root, "merge-base", "--is-ancestor", base, head).returncode:
            raise ValueError(f"Base commit {base} is not an ancestor of HEAD {head}.")

        control_v1 = bool(re.search(r"(?mi)^Control contract:\s*`1`\s*$", text))
        task_status: str | None = None
        worker_note_path: str | None = None

        if control_v1:
            task_status = field(text, "Status")
            task_colour = field(text, "Task colour")
            owner = field(text, "Owner")
            route = field(text, "Route")
            worker_note_path = field(text, "Worker note")

            if task_status not in VALID_STATUSES:
                raise ValueError(f"Invalid task Status: {task_status}")
            if task_colour not in VALID_COLOURS:
                raise ValueError(f"Invalid Task colour: {task_colour}")
            if not owner or "<" in owner or "," in owner or re.search(r"(?i)\s+and\s+", owner):
                raise ValueError("Owner must name exactly one implementation owner.")
            if not route:
                raise ValueError("Route must name the current worker/tool route.")

            for name in REQUIRED_PACKET_SECTIONS:
                body = section(text, name)
                if not body:
                    raise ValueError(f"Task packet section is empty: {name}")

            required_count = len(re.findall(r"(?m)^\d+\.\s+", section(text, "Required behaviour") or ""))
            acceptance_count = len(re.findall(r"(?m)^\d+\.\s+", section(text, "Acceptance criteria") or ""))
            if required_count == 0 or acceptance_count < required_count:
                raise ValueError(
                    "Control-v1 packets require at least one numbered acceptance criterion for every numbered "
                    f"required behaviour. Required: {required_count}; acceptance: {acceptance_count}."
                )

            note_text: str | None = None
            checkpoint: str | None = None
            if task_status in TERMINAL_STATUSES:
                if not worker_note_path:
                    raise ValueError("Terminal task packets require a worker note.")
                note_text, checkpoint = assert_worker_note(
                    root,
                    worker_note_path,
                    task_status,
                    owner,
                    base,
                    args.packet,
                )

            if task_status in COMPLETED_STATUSES:
                assert checkpoint is not None
                if checkpoint == "WORKTREE":
                    raise ValueError(
                        "COMPLETE/ACCEPTED/REJECTED tasks must record a committed implementation checkpoint SHA, "
                        "not WORKTREE."
                    )
                assert_checkpoint(root, checkpoint, base, head, "Implementation checkpoint")
                # Completed packets are historical evidence. Later repository work does not invalidate them.

            if task_status == "IN_PROGRESS":
                evidence_checkpoint = field(text, "Evidence checkpoint", required=False)
                scope_body = section(text, "Implementation scope", required=False)
                if evidence_checkpoint is not None:
                    if not scope_body:
                        raise ValueError(
                            "IN_PROGRESS evidence requires both Evidence checkpoint and a non-empty Implementation scope."
                        )
                    scope_paths = sorted(
                        {
                            normalize_scope_path(match)
                            for match in re.findall(r"(?m)^-\s+`([^`]+)`\s*$", scope_body)
                        }
                    )
                    if not scope_paths:
                        raise ValueError("Implementation scope must list at least one repository-relative path.")
                    evidence_commit = assert_checkpoint(root, evidence_checkpoint, base, head, "Evidence checkpoint")
                    changed_paths = {
                        normalize_scope_path(path)
                        for path in git_lines(root, "diff", "--name-only", f"{evidence_commit}..{head}", "--")
                    }
                    changed_scoped = sorted(path for path in changed_paths if path in scope_paths)
                    if changed_scoped:
                        raise ValueError(
                            "STALE CURRENT TASK EVIDENCE: implementation scope changed after Evidence checkpoint "
                            f"{evidence_commit}: {', '.join(changed_scoped)}"
                        )
        else:
            legacy_match = re.search(r"(?mi)^Status:\s*`([^`]+)`\s*$", text)
            if legacy_match:
                task_status = legacy_match.group(1).strip()

        pre_work_state = task_status is None or task_status in {"PROPOSED", "READY"}
        planning_paths = {
            ".github/scripts/check-tethers-task-packet.ps1",
            ".github/scripts/check-tethers-task-packet.py",
            "docs/CURRENT_CLINE_TASK.md",
            "docs/COPILOT_TRIAL.md",
            "docs/PROJECT_DASHBOARD.md",
        }
        if worker_note_path:
            planning_paths.add(worker_note_path)

        if base != head and pre_work_state:
            descendant_paths = {
                normalize_scope_path(path)
                for path in git_lines(root, "diff", "--name-only", f"{base}..{head}", "--")
                if path
            }
            unexpected_descendants = sorted(path for path in descendant_paths if path not in planning_paths)
            if unexpected_descendants:
                raise ValueError(
                    "Pre-work commits after Base commit change non-planning paths: "
                    + ", ".join(unexpected_descendants)
                )

        if not args.skip_worktree_check and pre_work_state:
            expected_section = section(text, "Expected pre-existing changes") or ""
            if re.search(r"(?im)^\s*None\b", expected_section):
                expected_paths: set[str] = set()
            else:
                expected_paths = {
                    normalize_scope_path(path)
                    for path in re.findall(r"(?m)^-\s+`([^`]+)`\s*$", expected_section)
                }

            actual_paths: set[str] = set()
            for line in git_lines(root, "status", "--porcelain=v1", "--untracked-files=all"):
                path = porcelain_path(line)
                if path and path not in planning_paths:
                    actual_paths.add(path)

            missing = sorted(expected_paths - actual_paths)
            unexpected = sorted(actual_paths - expected_paths)
            if missing or unexpected:
                raise ValueError(
                    "Expected dirty paths do not match live pre-work Git state. "
                    f"Missing: [{', '.join(missing)}]. Unexpected: [{', '.join(unexpected)}]."
                )

        contract_label = (
            f"control-v1/{task_status}"
            if control_v1
            else f"legacy/{task_status}" if task_status is not None else "legacy/unknown"
        )
        print(f"PASS task packet consistency ({contract_label}): base {base[:7]}, HEAD {head[:7]}")
        return 0
    except (OSError, ValueError, IndexError) as error:
        print(f"FAIL task packet checker: {error}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
