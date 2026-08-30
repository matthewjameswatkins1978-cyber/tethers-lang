# Tethers Portable 0.1.0 release record

This release is the deterministic Windows x64 host-policy decision façade. It
is additive to Tethers Core: it does not package or replace the OCaml parser,
planner, or evaluator, and it never executes an Action.

## Immutable artifact

- Version: `0.1.0`
- Platform: Windows x64 (`windows-x64`)
- ZIP: `dist/tethers-portable-0.1.0-windows-x64.zip`
- ZIP SHA-256:
  `C5BB5520E316474FB26AE0946BD1506AAD63CC08D9389FB1E52E58CFF4E6A7F9`
- Executable inside the ZIP: `tethers-portable-0.1.0-windows-x64/tethers.exe`
- Extracted executable SHA-256:
  `73C62FB69A4C8F66A249B0C15E66797A53B4C779D42CA5E6EC7D762555531946`
- Release tag: `tethers-portable-v0.1.0`

The ZIP is retained as evidence and must not be rebuilt or replaced under this
release tag. Embedding the application icon in a future executable requires a
new release and checksum.

## Invocation contract

One request is supplied on stdin (or with `--input PATH`) and one JSON response
is written to stdout:

```powershell
Get-Content .\examples\allow.json -Raw |
  .\tethers.exe evaluate --policy .\policies\default.json
```

The request contains `action`, object-valued `context`, and either an embedded
`policy` or an external `--policy` file. Exact action name/version rules take
precedence over the policy default. Valid decisions are `ALLOW`, `ASK`, and
`DENY`; malformed requests, invalid policies, and evaluator failures return
`DENY` with an `error` field. A valid explicit `DENY` is a normal policy result.

The full machine-facing contract is documented in `README.md` and exercised by
the Rust library and CLI tests.
