# Tethers 0.5 cold-agent evidence

This is a bounded transcript from a fresh checkout of the 0.5 release
worktree. It records only deterministic discovery and harmless inspection;
the fresh data root contained no installed capabilities or Plugs.

## Fresh-client discovery

Command:

```text
tethers.exe describe --json
```

Result: exit `0`; schema `tethers.cli/1`; command `describe`; host version
`0.2.2`; capabilities, planning, Plugs, Together, and Trails reported as
supported; discovery commands were `describe`, `capability list`,
`capability inspect`, and `plug show`; host state was `not_configured`.

## Empty-state truthfulness

With a newly-created `work/cold-agent-root`:

```text
tethers.exe capability list --host-data-root work/cold-agent-root --json
tethers.exe plug list --host-data-root work/cold-agent-root --json
```

Both commands exited `0` and returned count `0`. No provider was started and
no capability or Plug was invented by discovery.

## Trusted Plug inspection and conformance

The two reference packages were packed and inspected before this transcript:

| package | capabilities | semantic manifest digest |
| --- | ---: | --- |
| `tethers-agent-workspace-0.1.0.tetherplug` | 11 | `sha256:e5c20ed9465ea5bd406ea3c4f1d28f6c44fe6c512848af4949af4d4fe4376a10` |
| `tethers-agent-coding-0.1.0.tetherplug` | 12 | `sha256:8dc7ae3fb97b7e390c22ee0bd76b4854534f1214a91458187fba0fbfb5cc9641` |

`plug inspect` exited `0` for both packages. `plug conform` exited `0` for
both, with 6/6 checks passing in suite `m3-generic-1`.

Conformance is evidence about package shape and provider behaviour; it is not
trust, enablement, permission, or an isolated hostile-code sandbox claim.
