# Road to Tethers 0.3 — Public Plug Authoring

> A Plug gives Tethers a capability. A Tether decides when and why to use it.

> Deep Plug, narrow subject. Wide workflow, Tether.

## Status

P1 COMPLETE / ACCEPTED. P2 FINAL / ACCEPTED. P3 FINAL / ACCEPTED. P4 — Plug Author
Manual — FINAL ACCEPTED at `1e1f9b8738a48f727187316dd0078b7f9435f1c6`. P5 —
Fresh-Agent Authoring Proof — complete, awaiting Lucy review. P6 remains next
and has NOT started.

## Sequence

### P0 — Publish Tethers 0.2.2

Complete. Published from `c0fd57780156bee023d8dcff884737ea470d096c`.

### P1 — Generic Operational Scope Evidence ✓

Replace Plug-specific operational-scope types with a generic **Operational Scope Evidence** model.

Accepted at `270a5913a93d64256113cca3450619c484b7ddc7`.

End state:

```text
Tethers Core
├── package
├── manifest
├── capability
├── provider
├── generic scope evidence
├── MCP stdio
├── permission
├── execution
├── outcome
└── Trail
```

Core must not contain knowledge such as FileTools, Pdf, Image, Audio, Git, Email.

### P2 — Public pack / inspect / conform authoring surface ✓

Expose the generic author workflow: `plug pack`, `plug inspect`, `plug conform`.

- **P2A — Public deterministic plug pack** ✓ accepted at `3d7fd7e580d274de0a422fb78c5741a6bd1405f1`
- **P2B — Public supervised conform** ✓ accepted at `061a57d4bd48e59cae2d496b889834df7fe54418`
- **P2C — End-to-end author proof + final P2 verification** ✓ FINAL ACCEPTED
  - canonical P2C implementation checkpoint: `4c32b96446e7ae3e20d2994056d0fd435dcc32f3`
  - final independently reviewed P2 evidence head: `84f1002904dd54929fa8002d1634c42c85112f54`
  - public author journey proven: `plug pack → plug inspect → plug conform`
  - full Cargo gate: 1714 passed, 0 failed, 2 ignored
  - Nextest: 1714 passed, 2 skipped

### P3 — PDF Tools Reference Plug Crucible ✓

Move PDF Tools across the public boundary into `reference-plugs/pdf-tools/`.

P3 proved that a real existing capability can live outside the generic host and
use only the public Plug boundary established by P1–P2.

FINAL ACCEPTED at `e23030ad5e9820373133b25222680194af967c39`; final P3
implementation correction checkpoint `fcf22bff911393869d8dd560efeee1442a50b119`.

### P4 — Plug author manual ✓

Write the complete Plug-authoring manual using only interfaces proven by P1–P3.
Canonical public manual: `docs/PLUG_AUTHORING.md`.

FINAL ACCEPTED at `1e1f9b8738a48f727187316dd0078b7f9435f1c6`.

### P5 — Fresh-agent authoring proof ✓ (awaiting Lucy review)

A clean agent with only released Tethers and the manual builds a new Plug.

A fresh DeepSeek V4 Flash / High-thinking author, guided only by
`docs/PLUG_AUTHORING.md`, built a new non-PDF Plug (`tethers.text-stats`) under
`reference-plugs/text-stats-proof/` and completed the full public journey:
build → pack → inspect → conform-refusal → approved conform. Provider semantic
tests pass; digest continuity and source immutability proven; one genuine narrow
manual gap surfaced (advertise both `inputSchema` and `outputSchema` in
`tools/list`) and was fixed in the manual. Experiment log:
`docs/p5-fresh-agent-proof.md`.

### P6 — Adversarial-provider proof

Hostile providers must not compromise host protocol correctness.

## Future

```text
0.3 Plug extensibility
→ 0.4 concurrency
→ 0.5 HQ foundations
```
