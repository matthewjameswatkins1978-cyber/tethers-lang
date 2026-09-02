# Tethers Benchmarker

`tethers-bench` is the first-class Rocket performance and exactness tool for
Tethers 0.5. It is deterministic and scriptable: the workload, case order,
input digests, backend labels, and parity checks are fixed by the executable.
Wall-clock measurements are naturally dependent on the host environment.

## Build

From the repository root, use the packet-authorised OCaml switch:

```powershell
opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune build bin/tethers_benchmarker.exe
```

The executable is available through the public name `tethers-bench` when run
from the Dune build context.

## Run

```text
tethers-bench
tethers-bench --quick
tethers-bench --hardcore
tethers-bench --profile sane --json
tethers-bench --warmups 2 --samples 5 --iterations 3 --json-out bench.json
tethers-bench --json-out after.json --compare before.json
```

With no options, `tethers-bench` uses the sane profile: a balanced fixed corpus
and sampling level for ordinary before/after work. `--quick` is a small smoke
run. `--hardcore` is a larger but still bounded crucible that adds path-6 and
independent-6 cases and takes deeper samples. `--profile NAME` is the explicit
form when an AI tool wants to record the selected profile in its own plan.

Profile flags select their profile defaults. Put an individual override after
the profile flag when needed, for example:

```text
tethers-bench --hardcore --samples 5 --json-out hardcore.json
```

Human output is intentionally readable at a glance: it shows the profile,
parity status, median and p95 portfolio timings, reference median, speedup,
operations per second, route counts, elapsed time, and any before/after deltas.
ANSI styling is used only for an interactive terminal; use `--no-color` for
plain logs. JSON uses schema `tethers.benchmarker/1` and records the selected
profile, exact settings, fixed case names, bounded-workload marker, case
results, comparison rows, tool/runtime/OS context, backend type, sample and
iteration counts, operations, min/median/p95/max, operations per second,
resource counters, and a stable case digest.

The portfolio is checked against the exhaustive Rocket reference before timing
is reported. A parity failure is a failed benchmark, never a performance
result. Routing and thresholds can change runtime only; they cannot change
the frozen V2 identity or accepted canonical result.

## Agent-toolbelt use

Expose `tethers-bench` only as an explicit named verification check in the
coding Plug scope. Keep its arguments fixed or bounded in the verification
configuration, write JSON to a caller-selected workspace path, and compare
against a checked-in or previously captured baseline when a before/after claim
matters. Do not grant general shell access just to run the benchmarker.

The benchmarker never invokes providers, requests authority, writes Trail
records, or changes canonicalisation semantics. It is an observation and
verification tool for Rocket.

## AI-toolbelt contract

The canonical tool entrypoint is `tethers-bench`; the profile is part of the
run contract, not an implicit environment setting. An AI should normally use:

```text
tethers-bench --json-out bench.json
```

Use `--quick` for a pre-commit smoke check and `--hardcore` for the performance
crucible. Use `--compare before.json` only when the baseline was produced from
the same profile and workload. A non-zero exit, malformed JSON, or parity
failure means there is no benchmark result to interpret.

The published native host bundle remains separate from the OCaml Rocket
benchmarker runtime. Repository and build-tool installations should expose
the `tethers-bench` executable explicitly; the native `tethers` host must not
silently substitute a different workload or claim Rocket timings without that
benchmark engine present.
