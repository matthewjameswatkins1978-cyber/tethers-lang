# Road to Tethers 0.3 — Public Plug Authoring

> A Plug gives Tethers a capability. A Tether decides when and why to use it.

> Deep Plug, narrow subject. Wide workflow, Tether.

## Status

P1 COMPLETE / ACCEPTED. P2 ACTIVE (P2A pack complete).

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

### P2 — Public pack / inspect / conform authoring surface

Expose the generic author workflow: `plug pack`, `plug inspect`, `plug conform`.

- **P2A — Public deterministic plug pack** ✓ (this packet)
- **P2B — Public conform** — next
- **P2C — End-to-end author proof + final P2 verification** — after P2B

### P3 — PDF Tools Reference Plug Crucible

Move PDF Tools across the public boundary into `reference-plugs/pdf-tools/`.

### P4 — Plug author manual

Write the complete Plug-authoring manual using only interfaces proven by P1–P3.

### P5 — Fresh-agent authoring proof

A clean agent with only released Tethers and the manual builds a new Plug.

### P6 — Adversarial-provider proof

Hostile providers must not compromise host protocol correctness.

## Future

```text
0.3 Plug extensibility
→ 0.4 concurrency
→ 0.5 HQ foundations
```
