# Rocket V3 R3-3A Exact Enc_V2 Label Certificate

Task: `Rocket V3 R3-3A — Exact Enc_V2 Label Certificate`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `21bb7442fa9f8442db98e193eb4954096f356678`

Implementation checkpoint: `ef5e4b7e21aad37eb8cd9bb76c6cf37b0bfff290`

## Requested outcome

Implement the exact bridge from a discrete stable Rocket V3 semantic leaf to
the legal frozen V2 label assignment whose existing Enc_V2 payload is the
unsigned-byte lexicographic minimum. Do not implement general R3-3 Stage-B
individualisation/refinement search.

## Changes made

The leaf encoder now treats stable partition cells as a precondition only. It
enumerates every legal bijection for Origin, Fact, Batch, Branch and
ItemTemplate, and every legal permutation inside the frozen Program-role and
template-role blocks. Each complete assignment is passed to
`Tethers_core_canonical_v2_format.encode_program`; the minimum is selected by
`compare_bytes_lex_unsigned`. No candidate is skipped by a heuristic or
partial-prefix rule.

The existing model lookup bridge remains construction/encoding-boundary only.
The focused Stage-A test fixture covers chains, entry/complete control flow,
facts, branches and outcomes, Batch sites, templates, Action bindings,
program roles, multiple template role blocks, and storage/ID permutations.

## Decisions and assumptions

The original chain-3 mismatch was the first byte of the frozen
`entry_origin` integer label: V3 emitted `0x32` (`2`) and the oracle emitted
`0x31` (`1`) at payload offset 13. The former cell-order bridge was therefore
not an Enc_V2 label authority. The corrected certificate makes no assumption
such as “entry Action gets 1”; it obtains that result because the complete
legal candidate set is compared by the frozen byte law.

Decimal integer ordering is treated as encoded-byte ordering. The proof tests
cover 8/9, 9/10, 10/11 and 11/12, including the fact that encoded `10;` sorts
before encoded `9;`.

## Evidence

Exact switch: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`.

Focused certificate test: `rocket-v3-stage-a: 39/39 checks passed`.

`dune build @all`: passed.

`dune runtest --force`: passed. This included R3-1 model `214/214`, R3-2
refinement `4807/4807`, the V2 reference/production/IR suites, the existing
5,000-case generated differential corpus, and the focused Stage-A suite.

`git diff --check`: passed before the implementation checkpoint.

The packet checker passed `control-v1/READY` before implementation and will
be rerun for `control-v1/COMPLETE` after this closeout note and packet status
are written.

## Discoveries

A discrete semantic partition proves that the anonymous entities are
distinguishable, but it does not prove a numeric V2 label order. The legal V2
label domain remains unresolved even at a discrete leaf. Exact frozen emission
is consequently the certificate authority.

The prior oversized rich fixture was not a valid tractable slow-oracle case;
the focused all-features fixture was reduced to a valid 576-candidate oracle
domain while retaining the required semantic coverage.

## Remaining risks

This Stage-A certificate is deliberately complete enumeration of the legal
residual label domain. It is not a scalable replacement for the later R3-3
search architecture. No Stage-B I/R code, pruning, budget, production
integration or cutover was added.

## Smallest next action

Independent review may accept this Stage-A checkpoint and authorise a separate
R3-3 Stage-B task. Codex must stop here.

## References

- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_encode.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_encode.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_search_test.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_format.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_reference.ml`
