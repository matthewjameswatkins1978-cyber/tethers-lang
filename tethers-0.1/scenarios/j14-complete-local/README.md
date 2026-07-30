# J14A Complete Local Scenario

This scenario proves the public Tethers operational route:

```
configured provider
-> verified manifest admission
-> live capability availability
-> deterministic Plan
-> effective Allow
-> durable intent
-> exactly one provider tools/call
-> output validation
-> durable succeeded outcome
-> standard capability.succeeded Result Anchor
-> public Trail inspection
-> replay blocks a second external effect
```

## Public commands used

1. `check`  - validates Tether source, engine, and provider availability
2. `run`    - submits one explicit Anchor and Facts through the real slice
3. `trail`  - reads and filters the Trail by execution identity

## Files

- `tethers/complete.tether` - the scenario Tether
- `input.json` - public run input
- `runtime.template.json` - runtime configuration template with path placeholders
- `README.md` - this file

## Harness

The PowerShell acceptance script materialises paths in a unique system temporary
directory. It copies the committed scenario files, replaces path placeholders,
and invokes the public operational commands. No repository file is mutated.

The existing reviewed fixture manifest (`fixture-ping-standing-allow.json`) and
the existing stdio fixture provider are used unchanged.

## Running

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass `
  -File .\tethers-0.1\scripts\test-j14a-complete-scenario.ps1
```

J14A is one part of the J14 milestone. J14B will implement the negative scenarios.