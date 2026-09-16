# Tethers 0.7.1

Status: Windows x64 maintenance release

Tethers 0.7.1 is the proven 0.7 runtime with the corrected Host/execution
architecture and the current verification and release-control improvements.
It preserves the existing language, Plan, Plug, Trail and runtime
compatibility surfaces.

## What changed

- corrected Host/execution architecture and external Host Plan proof;
- current verification and release provenance checks;
- accepted Windows runtime improvements carried by the current `main` line.

## What did not change

- Tether language semantics remain `0.1`;
- Core/wire protocol remains `0.1`;
- Plan schema remains `tethers.plan/1`;
- Plug package format remains `1` and Plug Socket major remains `1`;
- the supported MCP revision remains `2025-11-25`;
- the official platform promise remains Windows x86-64.

## Supported target

The official 0.7.1 runtime release target is:

```text
Windows x86-64
```

Linux is targeted and under development. It is not an official 0.7.1 runtime
release.

Resolve and Lantern are not required product components for this release.
The 0.8 programme continues separately.

## Verification and provenance

The release archive, manifest and checksums are generated from one exact
release source commit. The archive contains the native Windows host and the
matching first-party OCaml engine. The release manifest records the source
commit, source tree and SHA-256 identity of the archive and packaged
executables.
