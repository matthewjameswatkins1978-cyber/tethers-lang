# J12 Packet 1 worker note - strict local runtime configuration foundation

Date: 2026-07-28

Task: J12 packet 1 strict minimal local runtime configuration foundation

Status: COMPLETE

Owner: Goose

## Exact base

- Base branch: `goose/j11-event-trail-final`
- Base commit: `f0a76ee3782f5b7d2d7120e1b36100f5fa465acb`
- Branch: `goose/j12-config-foundation`

## Implementation checkpoint

`INSERT_IMPLEMENTATION_SHA`

## Documentation checkpoint

`INSERT_DOCUMENTATION_SHA`

## Authorised files

1. `docs/CURRENT_CLINE_TASK.md` - replaced with J12 Packet 1 task packet
2. `docs/DECISIONS.md` - added J12 Runtime Configuration Foundation decision
3. `docs/worker-notes/2026-07-28-j12-config-foundation.md` - this file
4. `tethers-0.1/host-rust/src/main.rs` - registered `pub mod runtime_config`
5. `tethers-0.1/host-rust/src/manifest.rs` - exposed `parse_value_no_dupes` as `pub(crate)`
6. `tethers-0.1/host-rust/src/runtime_config.rs` - new module (parsing, validation, materialisation, tests)

## Frozen schema

The frozen J12 JSON configuration schema is:

```json
{
  "format_version": "0.1",
  "tether_set": {
    "id": "example.local",
    "version": "1",
    "tethers": [{"id": "...", "version": "...", "source_path": "..."}],
    "capability_requirements": [{"name": "...", "version": 1, "reason": "..."}]
  },
  "providers": [{
    "id": "...",
    "display_name": "...",
    "transport": {
      "kind": "stdio",
      "command": "...",
      "args": [...],
      "protocol_version": "..."
    },
    "capabilities": [{
      "name": "...",
      "version": 1,
      "manifest_path": "...",
      "pinned_digest": "sha256:...",
      "scope_binding": {
        "kind": "path_prefix",
        "argument_json_pointer": "/path"
      }
    }]
  }],
  "policy": {
    "default": "deny",
    "rules": [{"name": "...", "version": 1, "decision": "allow"}]
  }
}
```

## Validation matrix

| Rule | Enforced by |
|------|------------|
| Duplicate JSON keys at any depth | `parse_value_no_dupes` (shared) |
| Unknown fields at any level | `serde(deny_unknown_fields)` on every struct |
| format_version other than "0.1" | Semantic validation |
| Empty/whitespace-only identifiers/versions/commands/protocol_versions | Semantic validation |
| Zero capability versions | Semantic validation |
| Empty tethers/requirements/providers/capabilities | Semantic validation |
| Duplicate Tether id/version pairs | Semantic validation |
| Duplicate requirement name/version pairs | Semantic validation |
| Duplicate provider IDs | Semantic validation |
| Duplicate provider capability name/version pairs | Semantic validation |
| Duplicate policy name/version pairs | Semantic validation |
| Requirement without matching provider capability | Cross-reference validation |
| Provider capability not required by Tether Set | Cross-reference validation |
| Policy rule for undeclared requirement | Cross-reference validation |
| Policy default other than "deny" | Semantic validation |
| Policy decisions outside allow/ask/deny | Serde enum deserialization |
| Transport kind other than "stdio" | Serde enum deserialization |
| Source/manifest path absolute | Semantic validation (Path::is_absolute) |
| Empty source/manifest path | Semantic validation |
| Invalid pinned digest (not sha256: + 64 lowercase hex) | Semantic validation |
| Scope kind other than "path_prefix" | Serde enum deserialization |
| Empty JSON Pointer | Semantic validation |
| JSON Pointer not beginning with "/" | Semantic validation |
| Scope binding on duplicate capability identity | Semantic validation |

## Focused test names and count

32 tests with `j12_packet1_` prefix:

1. j12_packet1_valid_minimal_configuration_parses
2. j12_packet1_tether_order_preserved
3. j12_packet1_duplicate_key_rejected
4. j12_packet1_unknown_field_rejected
5. j12_packet1_wrong_format_version_rejected
6. j12_packet1_empty_tether_list_rejected
7. j12_packet1_duplicate_tether_identity_rejected
8. j12_packet1_duplicate_requirement_rejected
9. j12_packet1_duplicate_provider_id_rejected
10. j12_packet1_duplicate_provider_capability_rejected
11. j12_packet1_duplicate_policy_rule_rejected
12. j12_packet1_missing_provider_capability_rejected
13. j12_packet1_unused_provider_capability_rejected
14. j12_packet1_policy_rule_undeclared_rejected
15. j12_packet1_invalid_pinned_digest_rejected
16. j12_packet1_absolute_source_path_rejected
17. j12_packet1_absolute_manifest_path_rejected
18. j12_packet1_unsupported_transport_rejected
19. j12_packet1_non_deny_default_rejected
20. j12_packet1_malformed_scope_pointer_rejected
21. j12_packet1_loaded_config_resolves_relative_paths
22. j12_packet1_manifest_duplicate_key_unchanged
23. j12_packet1_missing_format_version_rejected
24. j12_packet1_empty_provider_id_rejected
25. j12_packet1_empty_source_path_rejected
26. j12_packet1_empty_manifest_path_rejected
27. j12_packet1_zero_capability_version_rejected
28. j12_packet1_empty_scope_pointer_rejected
29. j12_packet1_materialization_produces_correct_requirements
30. j12_packet1_materialization_produces_correct_policy
31. j12_packet1_provider_materialization_preserves_scope
32. j12_packet1_provider_config_from_materialization

Result: 32/32 PASS (all with j12_packet1_ prefix)

Full Rust total: 554/554 PASS

## Warning totals and deltas

| Command | Warnings | Delta |
|---------|----------|-------|
| cargo check | 9 | 0 (all pre-existing) |
| cargo check --tests | 4 | 0 (all pre-existing) |
| cargo clippy --all-targets --all-features | 16 (bin) + 20 (test) = 36 | 0 (all pre-existing) |

Zero new warnings introduced.

## Exact checks run

- `cargo fmt --check`: PASS (no diffs)
- `cargo check`: PASS (9 pre-existing warnings)
- `cargo check --tests`: PASS (4 pre-existing warnings)
- `cargo test j12_packet1_ -- --nocapture`: PASS (32/32)
- `cargo test`: PASS (554/554)
- `cargo clippy --all-targets --all-features`: PASS (0 new warnings)
- `cargo build`: PASS (9 pre-existing warnings)
- `cargo build --release`: PASS (9 pre-existing warnings)
- `check-tethers-task-packet.ps1`: PASS
- `check-fixtures.ps1`: PASS (46 JSON, 30 JSONL)
- `test-engine.ps1`: NOT RUN (pre-existing opam switch)
- `test-mcp-transcripts.ps1`: PASS (15/15)
- `test-host-denial.ps1`: NOT RUN (pre-existing opam switch)
- `test-host-execution-failure.ps1`: NOT RUN (pre-existing opam switch)
- `test-host-result-follow-up.ps1`: NOT RUN (pre-existing opam switch)
- `test-host-event-admission.ps1`: PASS (9/9)
- `test-host-event-admission-trail.ps1`: PASS (10/10)
- `demo.ps1`: NOT RUN (pre-existing opam switch)
- `opam exec -- dune build`: NOT RUN (pre-existing opam switch)
- `git diff --check`: PASS (LF/CRLF warnings only)
- `control-character scan`: PASS (all 5 authorized text files)
- `git status --porcelain`: 5 changed files (6th is this worker note)

## Direct evidence

- manifest.rs: `parse_value_no_dupes` changed from `fn` to `pub(crate) fn` (1 line)
- main.rs: added `pub mod runtime_config;` (1 line)
- runtime_config.rs: 1518 lines, 554 total tests pass including all 32 new
- DECISIONS.md: appended J12 decision freezing exact schema
- CURRENT_CLINE_TASK.md: replaced with J12 Packet 1 task packet
- Manifest behaviour unchanged: all existing tests pass; test j12_packet1_manifest_duplicate_key_unchanged verifies both valid parse and duplicate-key rejection

## Unimplemented Packet 2 seams

The following are explicitly deferred to Packet 2:

1. Launching a provider process (`StdioProviderConfig` is constructible from public fields but not launched).
2. Admitting a manifest into the trusted store.
3. Invoking the Tethers engine with a Tether Set from configuration.
4. Live scope assessment: the `scope_binding` field is parsed and validated but not cross-checked against live manifest `allowed_prefixes`.
5. Dispatching an Action to a provider.
6. Writing dispatch, execution, or result Trail entries.
7. Creating a J13 command, CLI, daemon, or GUI.

## Remaining risks

- The opam switch used by `test-engine.ps1`, `test-host-denial.ps1`, `test-host-execution-failure.ps1`, `test-host-result-follow-up.ps1`, and `demo.ps1` is configured for the original repository path and not this worktree. This is a pre-existing environment configuration issue unrelated to J12 changes.
- The `ProviderMaterialization` intermediate type carries scope bindings alongside `ProviderConfig`-compatible data. Packet 2 must wire scope bindings into the host-owned scope assessor.

## Smallest next action

Matthew reviews this worker note, verifies the implementation and documentation commits, and either accepts J12 Packet 1 or requests corrections.
