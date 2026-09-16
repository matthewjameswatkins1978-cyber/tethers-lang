# OpenShell demo lane

This fixture is the opt-in M4 effect lane for `demo.export_summary@1`. Tethers
must perform policy, capability resolution, authority, and durable intent
admission before constructing `OpenShellExecutor`. OpenShell supplies only the
sandboxed effect boundary.

The image is Debian-based because the OpenShell supervisor is dynamically
linked and requires `iproute2` for network-namespace setup. Build from the WSL
filesystem when using Docker Desktop; Docker Desktop's Windows-mounted build
context can reject the tar header produced for this fixture.

## Local setup

1. Copy this directory to a WSL-local build directory and build it:

   ```text
   /home/<user>/lantern-m4-openshell
   docker build --tag lantern-m4-sandbox:debian /home/<user>/lantern-m4-openshell
   ```

2. Copy `gateway.toml.example` to a WSL-local `gateway.toml`, replace the
   marked home-directory placeholder, and start an OpenShell gateway with the
   generated local TLS/JWT
   material, and the Docker compute driver. The checked-in TOML is a shape
   example; its absolute paths are local-machine configuration.

3. Create the named sandbox with `policy.yaml` and a long-running command:

   ```text
   openshell sandbox create --name lantern-m4-demo \
     --from lantern-m4-sandbox:debian --policy policy.yaml --detach --no-tty \
     -- /bin/sh -c "while true; do sleep 3600; done"
   ```

## Acceptance probes

The approved probe writes and reads
`/sandbox/outbox/approved/summary.txt`. A write to
`/sandbox/outbox/forbidden/summary.txt` must fail with permission denied. A
request to `https://example.com` must fail through the default-deny network
proxy. The sandbox environment must not contain Lantern control credentials or
an audit token.

The real Tethers boundary test is opt-in and requires a ready gateway and
sandbox:

```powershell
$env:LANTERN_OPENSHELL_E2E = '1'
cargo test openshell_executor::tests::valid_grant_executes_once_in_openshell -- --exact --nocapture
```

The test uses Tethers' real policy/resolution/intent/replay/shared-boundary
path and the supervised child-process owner. OpenShell failure is surfaced as
uncertain; there is no unsandboxed fallback or retry.
