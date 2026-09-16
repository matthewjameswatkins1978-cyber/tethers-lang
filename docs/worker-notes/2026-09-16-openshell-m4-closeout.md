# Tethers × Lantern Warden × NVIDIA OpenShell — M4 closeout

Task: `Lantern Warden × Tethers × NVIDIA OpenShell / Milestone 4 closeout`

Owner: `Codex`

Status: `READY FOR REVIEW`

## Scope

This closeout exercises the already-published M4 OpenShell executor and the
existing Lantern authority bridge. It adds no new provider, policy, planner,
transport, scheduler, or Resolve mechanism.

The live proof path is:

```text
Tethers policy and binding
 -> authenticated Lantern Warden authority check
 -> Lantern decision receipt
 -> durable Tethers intent/replay admission
 -> Resolve guard admission seam
 -> one OpenShell effect
 -> Tethers Trail outcome
 -> Lantern outcome receipt
```

The gated test is
`host_execution::tests::m4_live_lantern_authority_openshell_chain`. It uses
the real `LanternHttpAuthorityProvider`, the existing host authority
precheck/post-outcome methods, the existing guarded shared boundary, and the
published `OpenShellExecutor`. The Lantern fixture seeds its grant only
through trusted in-process bootstrap and exposes no session or grant creation
route.

## Verified local evidence

- Windows build `26220`, WSL2 Ubuntu, Docker Desktop `4.91.0`, Docker Engine
  `29.8.0`, OpenShell `0.0.116`.
- OpenShell gateway health passed on `https://127.0.0.1:17670`.
- Sandbox `lantern-m4-demo` was `Ready`, with hard Landlock and default-deny
  networking.
- Live Lantern Warden authority check returned `ALLOW` for the exact scoped
  capability and principal.
- The live Tethers test passed with one OpenShell effect and recorded both the
  bounded authority Trail evidence and the successful provider outcome.
- Lantern decision and outcome receipt routes both completed successfully;
  the bearer token was never passed to OpenShell.
- Existing OpenShell-only rejection, uncertainty, filesystem, network, and
  credential-isolation probes remain unchanged and pass.

## Pinned semantic references

- Tethers baseline: `07870c356e034103573c5499347c61fce0700218`.
- Published M4 implementation: `0418d42f512f5f7a6937750cf187ed16fb1a7fd6`.
- Resolve R0 semantic authority: `8d42e5b061f86b2b2a2c1949c629654968a550ff`.

## Exclusions

This is local evidence only. It does not claim Nebius deployment, hosted
OpenShell, live Tavily, a trust console, public demo, Resolve transport, or
protection against a compromised host or gateway.
