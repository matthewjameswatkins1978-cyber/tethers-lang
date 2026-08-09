# F10 — Foundation Clean-Checkout Completion Proof

Status: `COMPLETE`

Task: `F10 — Foundation Clean-Checkout Completion Proof`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Base commit: `f1fcf6c1af380bb8a787d725ac83d7faae5bc17c`

Implementation checkpoint: `6abde58cdd51b602ecdc221d3703a99cbcc80f60`

## Control start and packet correction

The original control-start preparation commit
`ca63d98895f11f50b0e726d4165a2ce01162fd6f` was not pushed. Its F10 packet
contained six numbered required behaviours but five acceptance criteria. The
packet checker correctly rejected that structural mismatch.

Only `docs/CURRENT_CLINE_TASK.md` was amended to add the sixth corresponding
acceptance criterion. No required behaviour was changed. The corrected commit
was captured directly from Git as
`CONTROL_START_SHA=6abde58cdd51b602ecdc221d3703a99cbcc80f60`; the packet
checker reported `control-v1/IN_PROGRESS PASS`. The control-only comparison to
the accepted target contained only `docs/CURRENT_CLINE_TASK.md` and
`docs/PROJECT_DASHBOARD.md`.

## Clean Windows proof checkout

Proof worktree: `D:\The Next Thing\Tethers Lang - F10 Clean Checkout`

The first newly-created detached checkout was at the exact control-start SHA,
with no tracked changes, untracked files, ignored outputs, `target`, `_build`,
`_opam`, or `.tethers` directory. The repository environment probe has a
metadata-only detached-HEAD edge case: it indexes the empty output of
`git branch --show-current` before reporting its probes. This was not a product
or test failure.

Under the bounded local-environment authority, the exact disposable worktree
was removed only after proving it had no tracked or non-ignored files, then
recreated cold at the same SHA attached to the already-existing
`foundation/f10-clean-checkout-proof` branch. No branch was created and no
tracked source/configuration was changed. The recreated checkout again had no
pre-build tracked, untracked, or ignored artefacts. All later generated files
were ignored `target`, `_build`, and `.tethers` outputs in that proof checkout.

## Programme and fixture review

`git merge-base 24428139807cac0adeb0b62264547e61ca809d16
6abde58cdd51b602ecdc221d3703a99cbcc80f60` returned the required Foundation
baseline. The programme range contains 161 commits, 132 changed files, and
10,290 insertions / 1,208 deletions. The complete stat, name-status, and
whitespace diff review passed. Rust production/test, OCaml production/interface
and Dune graph, PowerShell/Just/Nextest/task-checker tooling, authoritative
docs, and fixture material were reviewed against the Foundation plan.

`tethers-0.1/SPEC.md` is unchanged. Rust manifests/lockfile and OCaml opam
manifests/lockfile are unchanged. The review found no unauthorised dependency
migration, new product capability, or knowingly unauthorised public JSON, CLI,
Trail, replay, or recovery migration.

The byte comparison from accepted F1 fixture checkpoint
`f295daa288f4d3dc48181888d6655df798675033` to the control-start SHA had zero
diff for `docs/foundation-pass/fixtures/` and
`docs/foundation-pass/FIXTURE_MANIFEST.md`.

## Environment and verification results

- Existing OCaml switch used:
  `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`
- OCaml: `5.5.0`; Dune: `3.24.0`.
- Rust toolchain: `1.97.1-x86_64-pc-windows-msvc`; Cargo `1.97.1`.
- Cross-language probe: PASS. Its Rust metadata was offline; Dune build and
  runtest both passed; installation remained denied.
- Fixture validator: PASS — 46 JSON and 30 JSONL fixture files.
- Engine fixtures: PASS — 29 reported behavioural/determinism/line-ending
  checks. The harness's initial pre-test launch had no selected OPAM switch;
  setting `OPAMSWITCH` in that one process session restored the already-proven
  existing switch, and the actual test run passed. No test case failed.
- MCP transcripts: PASS — 15 reported cases.
- Clippy: PASS (exit 0). It emitted 63 advisory warnings, including 29
  duplicates; these are distinct from the enforced compiler-warning gate and
  were not changed in F10.
- `verify-agent`: PASS — 1,592 Nextest tests passed and 2 were skipped.

## Command table

| Required command or capture | Result | Actual evidence |
| --- | --- | --- |
| `git fetch origin` | PASS | Phase-1 target fetch completed before control start. |
| `git rev-parse origin/foundation/pre-f10-gate-consistency` | PASS | Exactly `f1fcf6c1af380bb8a787d725ac83d7faae5bc17c`. |
| `git rev-parse HEAD` (control start) | PASS | Exactly `6abde58cdd51b602ecdc221d3703a99cbcc80f60`. |
| `check-tethers-task-packet.ps1` (`IN_PROGRESS`) | PASS | `control-v1/IN_PROGRESS`; six behaviours map to six criteria. |
| Control-only `git diff` versus accepted target | PASS | Only current task packet and dashboard differ. |
| Pre-build proof-worktree path/HEAD/branch/status/diff/cache/untracked/ignored captures | PASS | Cold exact-SHA checkout; no tracked, untracked, or ignored artefacts. |
| `git merge-base`, `rev-list --count`, `diff --stat --name-status --check` over Foundation range | PASS | Required merge base; 161 commits; 132 files; 10,290 insertions / 1,208 deletions; no whitespace errors. |
| SPEC and dependency manifest/lock comparisons | PASS | No Foundation change. |
| F1 fixture/manifest byte-level `git diff` | PASS | Zero diff. |
| `pwsh -NoProfile -File tethers-0.1/scripts/check-fixtures.ps1` | PASS | 46 JSON, 30 JSONL. |
| Existing-switch discovery and OCaml/Dune version probes | PASS | Existing switch, OCaml 5.5.0, Dune 3.24.0. |
| `pwsh -NoProfile -File scripts/check-tethers-environment.ps1 -Profile cross-language -OcamlSwitchPath <existing absolute switch>` | PASS | All seven repository probes passed. |
| `pwsh -NoProfile -File tethers-0.1/scripts/test-engine.ps1` | PASS | 29 reported checks after temporary process-local `OPAMSWITCH` activation. |
| `pwsh -NoProfile -File tethers-0.1/scripts/test-mcp-transcripts.ps1` | PASS | 15 reported cases. |
| `rustup show` | PASS | Repository override selected Rust 1.97.1. |
| `cargo --version` | PASS | `cargo 1.97.1 (c980f4866 2026-06-30)`. |
| `pwsh -NoProfile -File scripts/check-dev-tools.ps1` | PASS | All listed developer tools resolved. |
| `cargo clippy --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked -- -W clippy::all` | PASS | Exit 0; 63 advisory warnings, 29 duplicates. |
| `just verify-agent` | PASS | All eight timed stages passed; 1,592 Nextest passed, 2 skipped. |
| Final proof `git status`, unstaged/cached diff, and untracked-file captures | PASS | No tracked or non-ignored untracked files. |

### `verify-agent` emitted timings

| Stage | Result | Elapsed |
| --- | --- | --- |
| `task-packet` | PASS | 783 ms (0.8 s) |
| `cargo-fmt` | PASS | 1,075 ms (1.1 s) |
| `cargo-check` | PASS | 16,265 ms (16.3 s) |
| `cargo-test` | PASS | 47,397 ms (47.4 s) |
| `agent-tools` | PASS | 2,873 ms (2.9 s) |
| `deps-policy` | PASS | 593 ms (0.6 s) |
| `deps-advisories` | PASS | 1,451 ms (1.5 s) |
| `nextest` | PASS | 37,752 ms (37.8 s) |

## Final proof state and risks

After verification, the proof checkout still resolved to the implementation
checkpoint with empty status, unstaged diff, cached diff, and non-ignored
untracked-file captures. Its only artefacts were ignored Cargo `target`, OCaml
`_build`, and `.tethers/timings.jsonl` files generated by this proof.

No product/test/tooling/fixture/dependency/specification change was made by
F10. The remaining acceptance decision is Lucy's independent review of the
pushed evidence. The known Clippy advisories remain intentionally advisory and
are not a zero-warning compiler-gate failure.

## Requested outcome

Independently prove the accepted Foundation programme from a genuinely cold
Windows checkout, without changing Foundation implementation or declaring
Foundation accepted.

## Changes made

Only F10 control/evidence documentation changed: the task packet, dashboard,
and this worker note. The proof checkout regenerated ignored outputs only.

## Decisions and assumptions

The corrected control-start checkpoint is the sole verified implementation
checkpoint. The existing 5.5.0 switch was used without installation or switch
creation. A process-local `OPAMSWITCH` setting was the bounded repair for the
engine harness's missing session selection.

## Evidence

The command table records the exact clean-checkout, programme-review, fixture,
environment, protocol, Rust, advisory, and complete-gate results. The ignored
timing record contains all eight emitted `verify-agent` timings.

## Discoveries

The environment probe assumes a non-empty current-branch string when building
its informational JSON, so a detached clean worktree trips an array-index
exception. Recreating the purpose-built worktree at the same commit on the
already-existing control branch resolved that local metadata precondition;
verification itself passed without source changes.

## Remaining risks

Clippy continues to emit advisory diagnostics, but its command succeeds and
the separately enforced Rust compiler-warning gate passed. Lucy must still
independently accept or reject the pushed F10 evidence.

## Smallest next action

Lucy independently reviews the pushed F10 evidence; no Foundation merge or
further implementation action is authorised by this packet.

## References

- `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/PROJECT_DASHBOARD.md`
- `docs/foundation-pass/FIXTURE_MANIFEST.md`
