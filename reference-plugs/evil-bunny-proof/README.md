# The Evil Bunny Test 🐇😈

Adversarial provider proof for Tethers 0.3 P6.

**This is a protocol test fixture, not a malware test, and it never claims
operating-system isolation.** `plug conform` executes provider code under
process supervision and reports `isolated: false`. This fixture deliberately
lies, hangs, crashes, emits malformed protocol, and advertises false contracts
at the Tethers/MCP boundary. It must never damage files, escape the filesystem,
touch credentials, attack the network, spawn uncontrolled processes, persist
itself, or perform destructive actions.

## Identity

- Package: `tethers.evil-bunny-proof` `1.0.0`
- Provider: `tethers-evil-bunny-provider` `1.0.0`
- Capability: `evil.probe@1`, operation `evil_probe`

## Layout

```text
reference-plugs/evil-bunny-proof/
├── provider-rust/            # one provider binary, deterministic --mode
├── author/
│   ├── plug.json             # EB-00 Good Bunny descriptor
│   ├── manifests/evil-probe-v1.json
│   └── cases/<EB-XX>/plug.json   # per-case pack descriptor (mode argument)
├── scripts/
│   └── run-evil-bunny-proof.ps1  # public CLI evidence driver
└── README.md
```

## Modes

| Mode | EB case | Violated contract |
| --- | --- | --- |
| `good` | EB-00 | none (control) |
| `identity-liar` | EB-01 | provider identity |
| `protocol-liar` | EB-02 | MCP protocol version |
| `missing-operation` | EB-03 | declared operation not advertised |
| `surprise-operation` | EB-04 | undeclared operation advertised |
| `wrong-name` | EB-05 | operation name differs from binding |
| `input-schema-liar` | EB-06 | advertised `inputSchema` differs from manifest |
| `output-schema-liar` | EB-07a | advertised `outputSchema` differs from manifest |
| `output-schema-omitted` | EB-07b | `outputSchema` omitted from `tools/list` |
| `malformed-stdout` | EB-08 | malformed protocol line on stdout |
| `wrong-response-id` | EB-09 | JSON-RPC response id mismatch |
| `early-death` | EB-10 | provider exits during conformance |
| `silent` | EB-11 | provider never responds (bounded hang) |
| `shutdown-refusal` | EB-12 | provider refuses graceful shutdown |

## Building the provider

```powershell
cargo build --manifest-path reference-plugs/evil-bunny-proof/provider-rust/Cargo.toml --locked
```

Produces `reference-plugs/evil-bunny-proof/provider-rust/target/debug/tethers_evil_bunny_provider.exe`.

## Running the public CLI evidence

```powershell
pwsh -NoProfile -File reference-plugs/evil-bunny-proof/scripts/run-evil-bunny-proof.ps1
```

The driver builds the provider, assembles a temporary pack source for every case
(pack descriptor + shared manifest + provider executable), runs the public
`plug pack` → `plug inspect` → `plug conform` (refusal) → `plug conform`
(approved) journey, and writes per-case evidence under
`reference-plugs/evil-bunny-proof/evidence/<case>/`.

## Safety notes

- Every mode is deterministic and bounded by the host's existing conformance
  policies (`PUBLIC_CONFORM_WALL_TIME_SECS` = 30 s public wall time).
- The `silent` and `shutdown-refusal` modes are terminated by the host's
  existing Job Object / `SupervisedChild` cleanup; no process is left running.
- The fixture performs no filesystem writes, network access, credential access,
  or child-process spawning.
