"""Small startup/evaluation benchmark for release reporting."""
import argparse
import json
import statistics
import subprocess
import time


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("binary")
    parser.add_argument("--iterations", type=int, default=25)
    args = parser.parse_args()
    request = json.dumps({"schema_version":"1","actor":"benchmark","action":"git.status","resource":"workspace","context":{},"policy":{"default":"allow","rules":[]}})
    samples = []
    for _ in range(args.iterations):
        started = time.perf_counter()
        result = subprocess.run([args.binary, "evaluate"], input=request, text=True, capture_output=True, check=False)
        samples.append((time.perf_counter() - started) * 1000)
        if result.returncode != 0 or json.loads(result.stdout).get("decision") != "ALLOW":
            raise SystemExit("benchmark invocation did not return ALLOW")
    print(json.dumps({"binary": args.binary, "iterations": args.iterations,
                      "startup_plus_evaluation_ms": {"min": min(samples), "median": statistics.median(samples), "max": max(samples)}}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
