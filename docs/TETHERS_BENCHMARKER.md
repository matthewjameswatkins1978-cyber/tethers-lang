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
tethers-bench --quick --json
tethers-bench --warmups 2 --samples 5 --iterations 3 --json-out bench.json
tethers-bench --json-out after.json --compare before.json
```

Human output gives the tool version, parity result, selected backend for each
case, and timing summary. JSON uses schema `tethers.benchmarker/1` and records
the workload, case results, comparison rows, tool/runtime/OS context, backend
type, sample/iteration counts, operations, min/median/p95/max, operations per
second, and a stable case digest.

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
