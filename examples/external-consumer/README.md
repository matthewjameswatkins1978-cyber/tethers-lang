# External Tethers consumer

This directory is a small host-side integration example. It uses only the
`tethers` executable and the versioned `tethers.authority/1` NDJSON contract;
it imports no Tethers implementation modules and uses only Python's standard
library.

The consumer owns the physical effect. `prepare` always has
`authorizes_dispatch: false`; the host may perform an effect only after an
`allow_prepared` decision or an explicit human approval followed by a
successful `commit`. It then reports the observed result with `outcome`.
`commit` records intent and never proves that the effect happened.

The host data root must be provisioned before first Gate use. The supported
current administrative operation is `tethers provision-replay ABSOLUTE_ROOT`
from the installed distribution. The command is intentionally hidden from
ordinary help, so packaging and provisioning remain release-managed tasks.

Run the real-process smoke from the repository root with paths to the current
product executable and Core engine:

```powershell
python examples/external-consumer/smoke.py --tethers C:\path\to\tethers.exe --engine C:\path\to\tethers_mcp_main.exe
```

The smoke creates fresh temporary consumer data, exercises ALLOW, ASK before
and after approval, DENY, malformed protocol, and a forged authority field.
Its harmless “effect” is a marker file written by this external Python host;
it asserts that the marker is absent before approval and on denied/error paths.
