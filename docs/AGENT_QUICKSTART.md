# Tethers agent quickstart

This is the machine-oriented entry path for Tethers. The CLI and trusted Plug
manifests are the discovery mechanism; this document only shows the sequence.

## 1. Identify the host

```text
tethers describe --json
```

This is read-only. It reports the CLI/protocol versions, supported feature
families, and whether a host-data root was supplied. With a configured root:

```text
tethers describe --host-data-root C:\\tethers-data --json
```

`host_health.status` is `configured_state_only`: discovery does not start a
provider and does not claim that a provider is healthy.

## 2. Discover capabilities

```text
tethers capability list --host-data-root C:\\tethers-data --json
tethers capability list --host-data-root C:\\tethers-data --all --effect metadata.read --json
```

The default list contains enabled, integrity-checked capability bindings. Add
`--all` to include disabled installed capabilities. Results are sorted by
canonical capability name and version.

## 3. Inspect one trusted contract

```text
tethers capability inspect file.metadata --host-data-root C:\\tethers-data --version 1 --json
```

Inspect returns the trusted input/output schemas, Effects, scope, reversibility,
determinism, idempotency, confirmation and retry rules, provider binding, Plug
identity, manifest digest, availability, and conformance evidence. If more than
one version exists, `--version` is required; Tethers never picks arbitrarily.

## 4. Inspect an installed Plug

```text
tethers plug show --host-data-root C:\\tethers-data --installed-id <installed-id> --json
```

This uses the installed lifecycle record. The source `.tetherplug` archive is
not needed, and private provider state and secrets are not returned.

## 5. Preview and run

The existing configured execution path remains:

```text
tethers check --config <config.json> --engine <engine.exe>
tethers run --config <config.json> --engine <engine.exe> --input <input.json> --trail <trail.jsonl> --host-data-root C:\\tethers-data
```

For a side-effect-free public preview, use the configured host command. It
parses and validates the input, evaluates the selected Tether, and returns the
proposed Plan without requesting authority, starting a provider, or writing a
Trail:

```text
tethers preview --config <config.json> --engine <engine.exe> --input <input.json>
```

The preview is an observation, not an execution. Use `run` when an authorised
execution and durable Trail evidence are intended.

For repeatable Rocket measurements, use the first-class `tethers-bench`
executable. It emits human output by default and the stable
`tethers.benchmarker/1` JSON schema with `--json` or `--json-out`; add
`--compare <baseline.json>` for a machine-readable before/after comparison.
The benchmark workload and exactness checks are deterministic; timing values
remain environment-dependent. AI toolbelt configurations should expose it as
an explicit named verification check rather than allowing arbitrary process
execution.

The Phase B workspace provider's reviewed operation set is:

```text
filesystem_read       filesystem_list       filesystem_stat
text_search            text_read_range      text_replace_exact
text_compare           patch_apply          hash_sha256
hash_verify            hash_directory_manifest
```

These operations are scoped by host-delivered roots and bounded output. Text
search requires an explicit `literal` or `regex` mode; replacement requires an
expected match count; patching is exact-context, one-file unified patching with
an optional base digest. The provider is separate from the frozen M4 provider;
it is not trusted or enabled merely because its executable exists.

The reviewed author source is under
`reference-plugs/tethers-agent-workspace`. On Windows it can be packed with:

```powershell
pwsh -NoProfile -File .\scripts\build-agent-workspace-plug.ps1
```

Building or packing does not trust or enable the Plug. Use the existing
install/conformance/trust/enable flow and provide explicit operational roots.
The current packer emits Windows/x86_64 packages; this is a package-material
checkpoint, not a claim of Linux package publication.

The Phase C coding provider adds a separate trusted Plug for structured Git,
bounded argv-only process execution, and named verification checks:

```text
git_status            git_diff               git_log
git_show              git_branch_list        git_branch_current
git_add               git_branch_create      git_checkout
git_commit            process_execute        verification_run
```

Build its reference package with:

```powershell
pwsh -NoProfile -File .\scripts\build-agent-coding-plug.ps1
```

Its scope requires canonical `repository_root` and `process_cwd_root`
directories, an explicit executable allow-list, runtime/output limits, and
allow-listed environment keys. `process_execute` receives an argv array and
never invokes a shell. `verification_run` receives only a configured check
name; the executable, arguments, cwd, and environment are not caller input.
Git operations are rooted to the configured repository and reject Git
pathspec glob syntax, destructive convenience flags, and ambiguous revisions.

## 6. Query the Trail

The current compatibility lookup is:

```text
tethers trail --trail <trail.jsonl> --execution-id <execution-id>
```

Trail records are evidence of admitted execution and are distinct from policy
authority. A permission decision answers whether an operation may happen;
execution outcome answers what happened:

| Authority | Execution outcome |
| --- | --- |
| `ALLOW` | `SUCCESS`, `FAILURE`, `UNCERTAIN`, or `CANCELLED` |
| `ASK` | waits for explicit approval before execution |
| `DENY` | no execution is authorised |

The core distinction is:

* Capability schema — what may be requested.
* Scope — where and how it may operate.
* Policy — whether this request may proceed.
* Provider — the component that performs it.
* Trail — the redacted record of what was admitted and observed.

Discovery is intentionally side-effect-free: it does not trust a provider's
live advertising, enable a Plug, grant a scope, or perform an operation.
