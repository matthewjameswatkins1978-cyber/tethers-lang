# F1 Persistence Inventory

Each store classified by durability contract. Source: Rust host at `tethers-0.1/host-rust/src/`.

## Persistence Classes

### 1. Immutable Atomic Record

Once written, never modified. Corrupt or missing = invalid state.

| Store | Module | Write Primitive | Atomic Visibility | Recovery Reader |
|---|---|---|---|---|
| Candidate Evidence | `candidate.rs`, `candidate_preparation.rs` | Write-then-rename (tmp -> final) | Atomic via filesystem rename | `load_all()` with torn `.tmp` rejection |
| Package Trust Evidence | `trust.rs` | Write-then-rename | Atomic rename | `revalidate_current()` with digest recompute |
| Launch Profile Evidence | `launch_profile.rs` | Write-then-rename | Atomic rename | `load_all()` with filename-as-digest convention |
| Conformance Evidence | `conformance.rs` | Write-then-rename | Atomic rename | By evidence identity |
| Installation Approval Record | `approval.rs` | Write-then-rename | Atomic rename | By record identity |
| Installed Plug Record | `installed.rs` | Write-then-rename | Atomic rename | By candidate identity |
| Trusted Manifests | `trusted_store.rs` | In-memory store (not file-backed) | N/A (memory) | Identity + digest dual-index |
| Enablement Records | `enablement.rs` | Write-then-rename | Atomic rename | Predecessor-linked chain |
| Disabled Binding Records | `enablement.rs` | Write-then-rename | Atomic rename | By installed identity |

### 2. Replaceable Current-State Record

May be overwritten with a newer value. Previous state is discarded.

| Store | Module | Write Primitive | Atomic Visibility |
|---|---|---|---|
| Replay Claim Ledger | `replay_windows.rs` | Locked exclusive write via Win32 | File lock + atomic rename per generation |
| Candidate Registry | `candidate.rs` | Directory of immutable records | Individual record atomic; registry is aggregate |

### 3. Append-Only Causal Log

New entries appended; existing entries never modified.

| Store | Module | Write Primitive | Recovery |
|---|---|---|---|
| Trail | `dispatch.rs` (`RecordingTrail`) | Line-delimited JSON append | Replay by execution identity; deduplication on replay |
| Replay Windows Ledger (generations 0-2) | `replay_windows.rs` | Sequential file-per-generation | Full chain validation; predecessor-mismatch fails closed |

### 4. Multi-Step Intent/Recovery Journal

Records intent, then steps toward completion. Intermediate states are valid and recoverable.

| Store | Module | Write Primitive | Recovery Contract |
|---|---|---|---|
| Installation Publication Intent | `installation_publication_intent.rs` | Write-then-rename | Post-intent failure hook for test; intent removal on completion |
| Installation Recovery Plan | `installation_recovery_plan.rs` | Read-only planning (no mutation) | Staleness detection on every evidence pin |
| Installation Execution State | `installation_execution.rs` | File lock + sequential steps | Resumable after postplan failure; lock releases on error |
| Local Anchor Coordinator | `local_anchor.rs` | Directory-based event store | Restart + duplicate detection + scope enforcement |
| M3 Trust Lifecycle | `m3_store.rs`, `trust.rs` | Multi-step (trust -> launch -> conformance -> approval -> installed) | Each step validates predecessor; cross-candidate drift fails |

### 5. Unclassified / Transient

| Store | Module | Notes |
|---|---|---|
| Execution Environment Contract | `execution_environment.rs` | In-memory; constructed per-request |
| Socket Catalogue | `socket.rs` | In-memory; discovered from MCP providers |
| Event Queue | `event_queue.rs` | In-memory queue per session |
| Scope Bindings | `operational_scope.rs` | Serialized from enablement records; per-call construction |

## Write Primitive Analysis

All file-backed stores use **write-then-rename** (`tmp` -> `final`) as the atomicity guarantee. This is correct on NTFS but does NOT provide directory-level durability — only individual file atomicity.

The replay Windows ledger adds an additional **exclusive file lock** layer via Win32 for multi-process safety.

## Known Unknowns

1. **Directory durability**: Tests close files but do not test `FlushFileBuffers` on parent directories. NTFS metadata may not be durable without explicit directory handle flush.
2. **Temporary-file sync**: Not all stores explicitly sync temp files before rename. The rename itself is atomic on NTFS, but unsynced temp content could be lost on power failure before rename.
3. **Recovery from partial writes**: The `.tmp` suffix convention provides torn-write detection, but only if readers check for it. The `load_all()` functions in evidence stores do check. Not all stores independently verified.
