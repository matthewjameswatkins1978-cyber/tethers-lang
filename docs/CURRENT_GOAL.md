# Current Goal

Updated: 2026-08-04

## Goal

Pause the Plug-installation sequence after accepted J24I and complete a bounded
repository spring clean before J24J begins.

The first maintenance increment, M01A, refreshes the repository-owned Rust
compiler from 1.89.0 to exact Rust 1.97.1 and removes stale live version
duplication from build commands, toolchain checks, task templates, and Rust
engineering guidance.

The second maintenance increment, M01B, will separately review inactive agent
configuration, duplicated guidance, optional developer utilities, warning debt,
one-off scripts, and files that may be safely removed.

## Accepted product baseline

Tethers 0.2.0 remains the accepted and published baseline. The annotated
`v0.2.0` tag remains at
`b5546411661dcbcb53e1cf2538eaec594c6f76f2`; Tethers language semantics remain
0.1.

The accepted public Plug surface contains:

- `plug inspect`;
- `plug list`;
- `plug disable`;
- permission-file `plug enable`;
- `plug stage`.

The accepted package-intake sequence is:

```text
plug inspect
→ plug stage
→ immutable quarantine
→ reusable candidate identity
```

Staging grants no trust, approval, installation, permission, or operational
availability.

## Accepted installation foundations

J24G provides the strict installation-request contract for one canonical
candidate, exact-candidate trust, explicit non-isolated supervised execution,
and disabled installation.

J24H provides durable launch-profile evidence and non-creating read-only store
openings.

J24I is accepted at
`88d8ab2e5c65052401b3860d8a7d68f3ccb06265`. It adds exact-candidate
installation trust pinned to candidate ID, candidate-record digest, package and
provider identity, semantic and raw archive digests, and approving authority.

Exact-candidate `PackageTrustEvidence` validates its complete mode fields and
refuses current-authority execution revalidation until the later locked executor
supplies the exact installation-trust authority.

After maintenance, the reviewed installation sequence resumes as:

```text
J24J  read-only installation reconciliation planner
J24K  host installation lock and gate executor
J24L  thin public plug install CLI
```

## Active maintenance increment

M01A is:

```text
Rust 1.89.0
→ exact Rust 1.97.1
→ live build commands follow root pin
→ toolchain checker derives repository truth
→ Just recipes fail fast
→ active Rust guidance is current
```

Frozen M01A boundaries:

- exact Rust point release is pinned, never floating `stable`;
- Rust edition remains 2021;
- declared `rust-version` becomes 1.97;
- compiler refresh is separate from dependency updates and edition migration;
- Cargo.lock remains byte-identical;
- OCaml 5.5.0, Dune 3.24.0, and Yojson 2.2.2 remain unchanged;
- no production source, production test, Tethers behaviour, Plug lifecycle, or
  runtime change is allowed;
- historical worker notes and release evidence retain their original versions;
- M01A records deletion candidates but removes nothing;
- M01B performs the evidence-backed pruning pass.

## Toolchain maintenance posture

- Exact compiler pins are repository authority.
- Stable toolchains are reviewed after meaningful milestones and at least
  monthly during active development.
- Security, soundness, and miscompilation point releases are prioritised.
- Compiler, dependency, warning-cleanup, and edition changes remain separate.
- Ordinary Cargo verification uses the committed lock with `--locked`.
- No floating toolchain channel or automatic background upgrade is permitted.

## Frozen installation shape

The installation design remains unchanged during maintenance:

```text
validated installation request
→ exact candidate validation
→ exact-candidate trust
→ durable supervised launch profile
→ supervised conformance
→ installation approval
→ atomic installed publication
→ present_disabled
```

A failed installation may leave completed immutable gate evidence, but it must
never leave a Plug falsely or partially installed.

## Active development posture

Current operating mode: **Gorilla Coding**.

- Lucy: architecture, packet compilation, independent review, and routine safe
  merges.
- OpenCode: implementation programme.
- Luna: bounded Green and ordinary Amber implementation.
- HY3: mechanical, repetitive, low-risk implementation.
- DeepSeek Pro V4: thicker cross-file work under frozen contracts.
- Matthew: product authority, ideas, priorities, and human judgement.
- Cline and Goose are not used.
- Active prototype tree: `tethers-0.1/`.
- Current accepted Rust toolchain before M01A: 1.89.0.
- M01A target Rust toolchain: exact 1.97.1.
- Required automation shell where applicable: PowerShell 7.

DeepSeek editing rule: after an exact `oldString` replacement failure, reread
the current file and create a fresh smaller patch. Never repeat the identical
failed edit, and stop after two materially different failed attempts rather than
rewriting a file wholesale.

## Authoritative references

- Current maintenance task: `docs/CURRENT_CLINE_TASK.md`
- M01A blueprint: `docs/architecture/M01A_RUST_TOOLCHAIN_REFRESH.md`
- Current root Rust pin: `rust-toolchain.toml`
- Rust package metadata: `tethers-0.1/host-rust/Cargo.toml`
- Toolchain checker: `.github/scripts/check-tethers-toolchains.ps1`
- Enduring principles: `docs/CONSTITUTION.md`
- Language semantics: `tethers-0.1/SPEC.md`
- Universal Plug architecture:
  `docs/architecture/TETHERS_UNIVERSAL_PLUG_ARCHITECTURE.md`
- J24G request contract:
  `docs/architecture/J24G_INSTALLATION_REQUEST_CONTRACT.md`
- J24H evidence access:
  `docs/architecture/J24H_INSTALLATION_EVIDENCE_ACCESS_FOUNDATION.md`
- J24I exact-candidate trust:
  `docs/architecture/J24I_EXACT_CANDIDATE_INSTALLATION_TRUST.md`
- Accepted decisions: `docs/DECISIONS.md`
- Short project status: `docs/PROJECT_DASHBOARD.md`
- Detailed queue: `docs/TASK_QUEUE.md`
- Historical evidence: `docs/worker-notes/`
