# Decisions

## 2026-07-20: Preserve The Prototype Archive

Decision: Keep `Tethers-0.1-Prototype.tar.gz` in the workspace.

Reason: The tarball is the original imported artifact and provides a recovery
point for the extracted prototype.

## 2026-07-20: Extract Without Flattening

Decision: Extract the archive as `tethers-0.1/` instead of moving its contents
into the workspace root.

Reason: The archive already contains a clean top-level directory. Preserving it
avoids accidental collisions and keeps the prototype boundary clear.

## 2026-07-20: Use `tethers-0.1/` As The Active 0.1 Tree

Decision: `tethers-0.1/` is the active development tree for the entire 0.1
cycle, not a frozen snapshot. Historical baselines will be preserved through Git
commits and later Git tags, not by copying complete source trees into new
version-numbered folders.

Reason: The verified native Windows opam switch is path-bound to
`tethers-0.1/engine-ocaml`. Moving or renaming the tree would disturb the
working environment, and version history belongs in Git.

## 2026-07-20: Keep The Prototype Source Intact

Decision: Do not edit imported source files during the first integration pass.

Reason: The request is to inspect, extract, integrate, and document. Changing
semantics before verification would mix preservation with implementation.

## 2026-07-20: Document Before Expanding Scope

Decision: Add project-control documents under `docs/`.

Reason: The workspace needs a clear overview, active goal, decision log, and
task queue before compilation or further design work begins.

## Open Decisions

- Whether future documentation should live at the workspace root, inside
  `tethers-0.1/`, or both.
- What the first post-baseline implementation milestone should be.
