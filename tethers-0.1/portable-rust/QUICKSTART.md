# Tethers Workbench 0.2.2 quickstart

Tethers is a local, fail-closed decision executable. It returns `ALLOW`,
`ASK`, or `DENY`; it never performs the requested action.

```powershell
cargo test --locked
cargo build --release --locked
.\target\release\tethers.exe doctor --json
.\target\release\tethers.exe init --profile coding-agent-default --output .tethers
Get-Content .tethers\request.json -Raw | .\target\release\tethers.exe check - --policy .tethers\policy.json --json
```

Useful commands are `lint POLICY`, `doctor`, `validate-manifest MANIFEST`, and
`test POLICY CORPUS`. Add `--audit PATH` to `evaluate` for append-only JSONL
decision records. Audit records contain decision metadata, not request context
or secret values.

On Linux, build the static x64 binary with:

```bash
cargo build --release --locked --target x86_64-unknown-linux-musl
```

Hosts must treat missing binaries, timeout, malformed JSON, schema mismatch,
and any operational uncertainty as `DENY`.
