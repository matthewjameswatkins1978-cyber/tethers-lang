#!/usr/bin/env python3
"""Native task-packet consistency checker used by all platforms."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path


FIELD = re.compile(r"(?mi)^([^\n:]+):\s*`([^`]+)`\s*$")


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


def git(root: Path, *args: str) -> str:
    result = subprocess.run(["git", "-C", str(root), *args], text=True, capture_output=True, check=False)
    if result.returncode:
        raise ValueError(f"git {' '.join(args)} failed: {result.stderr.strip()}")
    return result.stdout.strip()


def assert_checkpoint(root: Path, checkpoint: str, base: str, head: str, label: str) -> None:
    if not re.fullmatch(r"[0-9a-fA-F]{40}", checkpoint or ""):
        raise ValueError(f"{label} must be one full 40-character commit SHA.")
    normalized = checkpoint.lower()
    git(root, "cat-file", "-e", f"{normalized}^{{commit}}")
    if subprocess.run(["git", "-C", str(root), "merge-base", "--is-ancestor", base, normalized]).returncode:
        raise ValueError(f"Base commit {base} is not an ancestor of {label} {normalized}.")
    if subprocess.run(["git", "-C", str(root), "merge-base", "--is-ancestor", normalized, head]).returncode:
        raise ValueError(f"{label} {normalized} is not an ancestor of HEAD {head}.")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--packet", default="docs/CURRENT_CLINE_TASK.md")
    parser.add_argument("--skip-worktree-check", action="store_true")
    args = parser.parse_args()
    try:
        root = Path(git(Path.cwd(), "rev-parse", "--show-toplevel"))
        packet_path = root / args.packet
        text = packet_path.read_text(encoding="utf-8")
        base_match = re.search(r"(?mi)^Base commit:\s*`([0-9a-fA-F]{40})`\s*$", text)
        base = base_match.group(1) if base_match else None
        if not base:
            raise ValueError("Task packet must contain one full 40-character Base commit SHA.")
        head = git(root, "rev-parse", "HEAD").lower()
        git(root, "cat-file", "-e", f"{base.lower()}^{{commit}}")
        if subprocess.run(["git", "-C", str(root), "merge-base", "--is-ancestor", base.lower(), head]).returncode:
            raise ValueError(f"Base commit {base} is not an ancestor of HEAD {head}.")

        if re.search(r"(?mi)^Control contract:\s*`1`\s*$", text):
            status = field(text, "Status")
            colour = field(text, "Task colour")
            owner = field(text, "Owner")
            route = field(text, "Route")
            note_path = field(text, "Worker note")
            if status not in {"PROPOSED", "READY", "IN_PROGRESS", "BLOCKED", "COMPLETE", "ACCEPTED", "REJECTED"}:
                raise ValueError(f"Invalid task Status: {status}")
            if colour not in {"Green", "Amber", "Red"}:
                raise ValueError(f"Invalid Task colour: {colour}")
            if not owner or "<" in owner or "," in owner or re.search(r"(?i)\s+and\s+", owner):
                raise ValueError("Owner must name exactly one implementation owner.")
            if not route:
                raise ValueError("Route must name the current worker/tool route.")
            required_sections = [
                "Objective", "Relevant background and existing behaviour", "Required behaviour",
                "Relevant components", "Frozen decisions and invariants", "Acceptance criteria",
                "Required verification", "Forbidden changes", "Stop conditions", "Expected pre-existing changes",
            ]
            for name in required_sections:
                body = section(text, name)
                if not body:
                    raise ValueError(f"Task packet section is empty: {name}")
            required_count = len(re.findall(r"(?m)^\d+\.\s+", section(text, "Required behaviour") or ""))
            acceptance_count = len(re.findall(r"(?m)^\d+\.\s+", section(text, "Acceptance criteria") or ""))
            if required_count == 0 or acceptance_count < required_count:
                raise ValueError("Control-v1 packet acceptance criteria do not cover required behaviour.")
            if status in {"IN_PROGRESS", "BLOCKED", "COMPLETE", "ACCEPTED", "REJECTED"}:
                scope_body = section(text, "Implementation scope", required=False)
                if status == "IN_PROGRESS" and scope_body is not None and not re.findall(r"(?m)^-\s+`([^`]+)`\s*$", scope_body):
                    raise ValueError("Implementation scope must list at least one repository-relative path.")
            if status in {"COMPLETE", "ACCEPTED", "REJECTED", "BLOCKED"}:
                if not note_path:
                    raise ValueError("Terminal task packets require a worker note.")
                note = (root / note_path).read_text(encoding="utf-8")
                for name in ("Task", "Task packet", "Owner", "Status", "Base commit", "Implementation checkpoint"):
                    value = field(note, name)
                    if not value or "<" in value:
                        raise ValueError(f"Worker note field is missing or unresolved: {name}")
                for name in ("Requested outcome", "Changes made", "Decisions and assumptions", "Evidence", "Discoveries", "Remaining risks", "Smallest next action", "References"):
                    if not section(note, name):
                        raise ValueError(f"Worker note section is empty: {name}")
                if field(note, "Owner") != owner or field(note, "Task packet") != args.packet or field(note, "Base commit").lower() != base.lower():
                    raise ValueError("Worker note does not match the task packet.")
                checkpoint = field(note, "Implementation checkpoint")
                if checkpoint == "WORKTREE" and status in {"COMPLETE", "ACCEPTED", "REJECTED"}:
                    raise ValueError("Completed task requires a committed implementation checkpoint.")
                if checkpoint != "WORKTREE":
                    assert_checkpoint(root, checkpoint, base.lower(), head, "Implementation checkpoint")
        print(f"PASS task packet checker: {args.packet}")
        return 0
    except (OSError, ValueError, IndexError) as error:
        print(f"FAIL task packet checker: {error}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
