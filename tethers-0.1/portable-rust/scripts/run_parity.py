"""Run one request corpus through two Tethers binaries and compare results."""
import argparse
import json
import subprocess
import sys
from pathlib import Path


FIELDS = ("decision", "matched_rule", "reason", "policy_version", "policy_sha256", "error")


def run(binary: str, request: dict) -> dict:
    try:
        completed = subprocess.run([binary, "evaluate"], input=json.dumps(request), text=True,
                                   capture_output=True, timeout=10, check=False)
        return json.loads(completed.stdout)
    except (OSError, subprocess.TimeoutExpired, json.JSONDecodeError) as exc:
        return {"decision": "DENY", "error": f"parity invocation failed: {exc}"}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--windows", required=True)
    parser.add_argument("--linux", required=True)
    parser.add_argument("--corpus", default=str(Path(__file__).parents[1] / "tests" / "parity-corpus.json"))
    args = parser.parse_args()
    corpus = json.loads(Path(args.corpus).read_text(encoding="utf-8"))
    failures = []
    for case in corpus["cases"]:
        request = dict(case["request"])
        request["policy"] = case["policy"]
        windows = run(args.windows, request)
        linux = run(args.linux, request)
        left = {field: windows.get(field) for field in FIELDS}
        right = {field: linux.get(field) for field in FIELDS}
        if left != right or windows.get("decision") != case["expect"]:
            failures.append({"name": case["name"], "windows": left, "linux": right, "expected": case["expect"]})
    result = {"passed": not failures, "cases": len(corpus["cases"]), "failures": failures}
    print(json.dumps(result, sort_keys=True))
    return 0 if not failures else 1


if __name__ == "__main__":
    sys.exit(main())
