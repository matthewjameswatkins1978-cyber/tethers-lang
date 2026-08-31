# Tethers Workbench 0.2.2 release record

This release extends the frozen Portable 0.2.1 Rust façade with a small,
script-friendly `check` command, stdin/file equivalence, structured version and
validation commands, deterministic doctor checks, human explanation output,
and frozen decision/error exit codes. It remains a local executable and never
executes an action.

Base release: `tethers-portable-v0.2.1`
Release tag: `tethers-portable-v0.2.2`

The original `tethers-portable-v0.1.0` tag, artifact, and checksum remain
immutable. New platform checksums belong in `SHA256SUMS` in the generated
release bundle.
The checked-in release record is `SHA256SUMS-0.2.2`; earlier checksum files and
artifacts remain unchanged.

Acceptance record: Windows Rust tests 20/20 and Linux/WSL Rust tests 20/20;
Python plug tests 4/4; release parity 6/6 plus adversarial parity 12/12.
Representative Windows test time changed from 0.520 s to 0.669 s and the
Windows release binary changed from 762,368 to 784,384 bytes. Direct runtime
dependencies remain serde, serde_json, and sha2. The 0.2.2 CLI adds no OCaml
runtime dependency; the portable façade remains the compatibility boundary.
