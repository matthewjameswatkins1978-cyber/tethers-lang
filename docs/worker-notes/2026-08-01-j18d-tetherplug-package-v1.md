# J18D Worker Note

## Task

J18D - `.tetherplug` Package Format v1.

## Changes

Added the candidate package-format contract. Marked J18C accepted, prepended
the package decision, and aligned current-state documents with J18D active and
J18E next after Lucy acceptance.

## Decisions and assumptions

The package is a narrowly profiled ZIP archive, not installed state or
authority. `plug.json` is strict and singular. One package declares one
provider and multiple capabilities. Payloads are indexed and hashed; semantic,
raw archive, and manifest digests remain separate. Inspection never executes;
the host owns quarantine extraction, policy, bindings, credentials, and runtime
configuration.

## Existing package-related structures inspected

Inspected J18B, accepted J18C documents, `manifest.rs`, `runtime_config.rs`,
representative capability-manifest and runtime-configuration concepts. Existing
duplicate-key rejection, unknown-field rejection, RFC 8785/JCS hashing,
manifest pins, stdio provider bindings, scope bindings, and policy remain
authoritative. No implementation files were modified.

## Tool bootstrap

Resolved existing WinGet executables process-locally:

- `C:\Users\Matmus\AppData\Local\Microsoft\WinGet\Packages\BurntSushi.ripgrep.MSVC_Microsoft.Winget.Source_8wekyb3d8bbwe\ripgrep-15.2.0-x86_64-pc-windows-msvc\rg.exe` - 15.2.0
- `C:\Users\Matmus\AppData\Local\Microsoft\WinGet\Packages\sharkdp.fd_Microsoft.Winget.Source_8wekyb3d8bbwe\fd-v10.4.2-x86_64-pc-windows-msvc\fd.exe` - 10.4.2
- `C:\Users\Matmus\AppData\Local\Microsoft\WinGet\Packages\jqlang.jq_Microsoft.Winget.Source_8wekyb3d8bbwe\jq.exe` - 1.8.2
- `C:\Program Files\GitHub CLI\gh.exe` - 2.97.0
- `C:\Users\Matmus\AppData\Local\Microsoft\WinGet\Packages\MikeFarah.yq_Microsoft.Winget.Source_8wekyb3d8bbwe\yq.exe` - 4.53.3

## Evidence

Base J18C commit: `202abbb79d0095d2e9b4e07cd2d1d67f335f2302`. Released tag
peels to `b5546411661dcbcb53e1cf2538eaec594c6f76f2`. The package contract is
documentation only and creates no archive, schema, provider, installer, or
runtime configuration.

## Discoveries

The existing host runtime configuration already owns provider commands,
manifest paths and digests, scope bindings, and policy. The package therefore
references candidate manifests and payloads without replacing installed host
authority.

Lucy found an empty-directory contradiction in the illustrative package tree.
The bare `signatures/` entry was removed. No normative package rule changed;
no implementation or schema was created.

## Remaining risks

J18F must define conformance material and J18G must define signature trust,
sandbox, and rollback details. Exact finite implementation limits remain for
implementation planning.

## Next action

Lucy reviews J18D. Do not begin J18E or package implementation before acceptance.

## References

- `docs/architecture/TETHERPLUG_PACKAGE_V1.md`
- `docs/architecture/TETHERS_SOCKET_V1.md`
- `docs/architecture/TETHERS_SOCKET_V1_MCP_STDIO_BINDING.md`
- `tethers-0.1/host-rust/src/manifest.rs`
- `tethers-0.1/host-rust/src/runtime_config.rs`
