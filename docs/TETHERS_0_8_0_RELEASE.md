# Tethers 0.8.0

Status: Windows x64 runtime release

Tethers 0.8.0 makes the versioned `tethers.authority/1` Gate available to
external consumers. Tethers Core remains a deterministic planning layer. The
Gate checks explicit host authority and records durable intent; the consuming
Host remains responsible for physical effects and truthful outcome reporting.

## Integration routes

- **Rust source/crate:** `tethers-reference-host` 0.8.0 is available as a
  locked Cargo package under MIT OR Apache-2.0. Crates.io publication is
  deferred. Use the documented `authority_v1` module for the stable Rust entry
  point.
- **Runtime/binary:** the Windows x64 bundle contains the `tethers` host and
  matching OCaml Core engine. A consumer starts the host process and uses the
  versioned JSON protocol. The OCaml engine remains executable-only.

See [integration guide](INTEGRATING_TETHERS.md) and [crate README](../tethers-0.1/host-rust/README.md).

## Compatibility and platform

The release preserves the 0.7.1 runtime compatibility surface where reasonable.
The new Gate fails closed on malformed or unsupported requests and does not
accept caller-supplied authority. The official full-runtime target for this
release is Windows x86-64. Linux is not an official full-runtime release target.

## Provenance

The GitHub release assets include a provenance manifest, SHA-256 files, and a
verification report. They identify the exact tag commit, source tree, runtime
hashes, installation path, and proof results. The report is produced only after
the release artifacts have been built from the tag and tested.
