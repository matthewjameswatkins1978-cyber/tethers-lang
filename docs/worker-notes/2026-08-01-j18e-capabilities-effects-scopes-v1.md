# J18E Worker Note

## Task

J18E - Capability Classes, Effects and Scopes v1.

## Changes

Added the candidate capability class, effect, and scope contract. Marked J18D
accepted, added the decision entry, clarified the capability bridge authority,
and aligned current-state documents with J18E active and J18F next after Lucy
acceptance.

## Decisions and assumptions

Class, effect, scope, policy, and outcome are distinct. Every capability has one
reviewed class. Action, Query, and Anchor are first-slice targets; Job, Stream,
and Human Task remain reserved. Effective scope is the intersection of reviewed
supported scope, installation grant, policy, and explicitly resolved target.
Unknown effects and ambiguous mappings fail closed. No implementation is
authorised.

## Existing capability model inspected

Inspected `manifest.rs` fields for capability identity/version, schemas, effects,
permission scope, reversibility, determinism, idempotency, confirmation,
provider identity/binding, and digest. Existing manifest validation remains the
accepted 0.2 authority.

## Existing scope implementation inspected

Inspected `runtime_config.rs` and `configured_runtime.rs`. The current runtime
owns explicit provider bindings, manifest pins, policy, and scope bindings; the
implemented bounded scope is `path_prefix`. J18E does not claim broader scope
machinery is implemented.

## Tool bootstrap

Resolved existing WinGet executables process-locally:

- `C:\Users\Matmus\AppData\Local\Microsoft\WinGet\Packages\BurntSushi.ripgrep.MSVC_Microsoft.Winget.Source_8wekyb3d8bbwe\ripgrep-15.2.0-x86_64-pc-windows-msvc\rg.exe` - 15.2.0
- `C:\Users\Matmus\AppData\Local\Microsoft\WinGet\Packages\sharkdp.fd_Microsoft.Winget.Source_8wekyb3d8bbwe\fd-v10.4.2-x86_64-pc-windows-msvc\fd.exe` - 10.4.2
- `C:\Users\Matmus\AppData\Local\Microsoft\WinGet\Packages\jqlang.jq_Microsoft.Winget.Source_8wekyb3d8bbwe\jq.exe` - 1.8.2
- `C:\Program Files\GitHub CLI\gh.exe` - 2.97.0
- `C:\Users\Matmus\AppData\Local\Microsoft\WinGet\Packages\MikeFarah.yq_Microsoft.Winget.Source_8wekyb3d8bbwe\yq.exe` - 4.53.3

## Evidence

Base J18D commit: `70b95a38983ee270b908f47503be6350083b3e42`. Released tag peels
to `b5546411661dcbcb53e1cf2538eaec594c6f76f2`. No Rust, OCaml, manifest,
runtime configuration, schema, provider, package, or implementation file was
changed.

## Discoveries

The runtime already separates verified manifest claims, provider bindings,
manifest pins, explicit path scope bindings, and host policy. The broader J18E
model must remain a future contract rather than being presented as implemented.

## Remaining risks

J18F must define Anchor lifecycle and outcome evidence. J18G must define
credential secrecy and sandbox enforcement. Effect registry details and any
additional scope families require later accepted implementation planning.

## Next action

Lucy reviews J18E. Do not begin J18F or implementation before acceptance.

## References

- `docs/architecture/TETHERS_CAPABILITIES_EFFECTS_SCOPES_V1.md`
- `docs/architecture/TETHERPLUG_PACKAGE_V1.md`
- `docs/CAPABILITY_BRIDGE.md`
- `tethers-0.1/host-rust/src/manifest.rs`
- `tethers-0.1/host-rust/src/runtime_config.rs`
- `tethers-0.1/host-rust/src/configured_runtime.rs`
