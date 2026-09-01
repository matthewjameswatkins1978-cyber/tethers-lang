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

## 5. Plan and run

The existing configured execution path remains:

```text
tethers check --config <config.json> --engine <engine.exe>
tethers run --config <config.json> --engine <engine.exe> --input <input.json> --trail <trail.jsonl> --host-data-root C:\\tethers-data
```

The Agent Essentials side-effect-free `plan` surface is being added as a
separate implementation phase. It will share the real pre-execution path and
will never invoke a provider or write a fake receipt.

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
