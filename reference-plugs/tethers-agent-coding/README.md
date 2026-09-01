# Tethers Agent Coding Plug

This reference Plug provides the Phase C coding essentials: structured local
Git operations, bounded argv-only process execution, and named verification
checks. It is intentionally separate from the workspace Plug because process
execution and repository mutation have a different trust and scope boundary.

The provider requires `TETHERS_OPERATIONAL_SCOPE_JSON`. The scope must name
canonical absolute `repository_root` and `process_cwd_root` directories, an
allow-list of executable names, runtime/output limits, allowed environment
keys, and explicitly configured verification checks. The provider never
interprets a shell command string.

The current repository packer emits Windows/x86_64 `.tetherplug` packages.
Linux packaging remains a later distribution slice; the provider code itself
uses the portable Rust process/Git seam and is not tied to a daemon or shell.

Do not auto-enable this Plug. Install, inspect, conformance-test, and grant an
explicit operational scope before enabling it.
