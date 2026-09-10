# Tethers Agent Workspace Plug

This directory is the reviewed author source for the Phase B workspace
capabilities. It contains the descriptor and trusted manifests. The provider
executable is deliberately built from the repository source into an ephemeral
staging directory; a machine-specific release binary is not checked into the
author source.

On Windows, from the repository root, run:

```powershell
pwsh -NoProfile -File .\scripts\build-agent-workspace-plug.ps1
```

The script builds `agent_workspace_provider`, stages this descriptor and its
manifests, then invokes the normal `tethers plug pack` command. It refuses to
replace an existing output package. The current Plug pack contract is
Windows/x86_64; Linux package publication remains a later packaging slice and
is not implied by this Phase B material.

The package is not trusted or enabled by being built. Installation,
conformance, trust, and an explicit scope file are still required.
