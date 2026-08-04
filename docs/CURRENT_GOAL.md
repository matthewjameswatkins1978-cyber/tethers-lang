# Current Goal

Updated: 2026-08-04

## Goal

Resume proper Tethers product development by completing the Plug-installation sequence from the accepted J24G, J24H, and J24I foundations.

```text
J24G   accepted strict installation request contract
J24H   accepted read-only evidence access foundation
J24I   accepted exact-candidate installation trust
J24J   active read-only installation reconciliation planner
J24K   host installation lock and gated executor
J24L   thin public plug install CLI
```

The bounded Rust maintenance programme is closed. M01A through M01C4 remain accepted history and are not prerequisites for further cosmetic cleanup.

## Active increment

J24J adds a pure read-only planner that reconciles one exact installation request against existing immutable evidence and returns one next legitimate action:

```text
create exact-candidate trust
run supervised conformance
create installation approval
publish disabled installation
complete
```

The planner validates the complete evidence chain but mutates nothing. It does not create trust, launch a provider, run conformance, approve, install, lock, enable, or add a CLI.

## Accepted product baseline

Tethers 0.2.0 remains the accepted and published baseline. The annotated `v0.2.0` tag remains at `b5546411661dcbcb53e1cf2538eaec594c6f76f2`; Tethers language semantics remain 0.1.

Accepted public Plug surface:

- `plug inspect`;
- `plug list`;
- `plug disable`;
- permission-file `plug enable`;
- `plug stage`.

Accepted installation foundations:

- J24G supplies typed request schema `tethers.plug-install/1`;
- J24H supplies non-creating evidence access and launch-profile persistence;
- J24I supplies exact-candidate trust pinned to candidate identity and record digest.

## Active engineering baseline

```text
Rust             1.97.1 exact root pin
Rust edition     2021
rust-version     1.97
cargo-nextest    0.9.140, retries 0
Cargo tests      926 passing minimum
Nextest tests    1133 passing historical complete-suite floor
OCaml            5.5.0
Dune             3.24.0
Yojson           2.2.2
Cargo.lock       D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

OpenCode LSP remains an optional experimental helper. Empty, null, unavailable, or hanging LSP output must be recorded honestly and replaced with `rg`, compiler, and test evidence. It never blocks a task merely because it failed to help.

## Active development posture

Current operating mode: **Gorilla Coding**.

- Lucy: architecture, packet compilation, independent review, and routine safe fast-forward merges.
- OpenCode: bounded implementation programme.
- DeepSeek Pro V4: semantic Amber work such as J24J.
- HY3: mechanical Green work.
- Matthew: product authority, ideas, priorities, and human judgement.
- Active prototype tree: `tethers-0.1/`.
- Required automation shell where applicable: PowerShell 7.

Tools begin as helpers, not gatekeepers. A tool may become a stop condition only after it is proven reliable for the repository and its failure genuinely prevents safe completion.

## J24J boundaries

- Add only the read-only installation planner module, export, focused tests, and task evidence.
- No store mutation, process launch, trust creation, conformance execution, approval, installation, lock, enablement, or CLI.
- No dependency, Cargo.lock, toolchain, protocol, package-schema, candidate-schema, evidence-schema, OCaml, or language-semantic change.
- Reconcile from the most advanced valid durable state backwards and return the earliest missing legitimate action.
- Invalid or corrupt evidence fails closed; historical failed or stale conformance may be ignored in favour of a new conformance run.

## Authoritative references

- Current task: `docs/CURRENT_CLINE_TASK.md`
- J24J blueprint: `docs/architecture/J24J_READ_ONLY_INSTALLATION_RECONCILIATION_PLANNER.md`
- J24G request contract: `docs/architecture/J24G_INSTALLATION_REQUEST_CONTRACT.md`
- J24H evidence access: `docs/architecture/J24H_INSTALLATION_EVIDENCE_ACCESS_FOUNDATION.md`
- J24I exact-candidate trust: `docs/architecture/J24I_EXACT_CANDIDATE_INSTALLATION_TRUST.md`
- Universal Plug architecture: `docs/architecture/TETHERS_UNIVERSAL_PLUG_ARCHITECTURE.md`
- Toolchain policy: `docs/TOOLCHAIN_POLICY.md`
- Enduring principles: `docs/CONSTITUTION.md`
- Language semantics: `tethers-0.1/SPEC.md`
- Historical evidence: `docs/worker-notes/`
