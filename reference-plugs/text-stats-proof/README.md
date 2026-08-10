# Tethers Text Stats Plug

A non-PDF reference Plug for Tethers. It reads bounded basic statistics from one
UTF-8 text file inside a host-approved `query_root` and returns them as a
capability result.

> Apps provide the sockets. Tethers provides the cables.

- Package: `tethers.text-stats` / `1.0.0`
- Provider: `tethers-text-stats-provider` / `1.0.0`
- Capability: `text.stats@1`
- Provider operation: `text_stats`

## Layout

```text
reference-plugs/text-stats-proof/
├── provider-rust/                 # the provider's build project (source + Cargo)
├── author/                        # the curated pack source
│   ├── plug.json                  # the package descriptor
│   └── manifests/
│       └── text-stats-v1.json     # the text.stats@1 capability manifest
└── README.md
```

## Semantics

The capability takes one argument:

```json
{ "path": "example.txt" }
```

and returns:

```json
{
  "path": "example.txt",
  "size_bytes": 46,
  "sha256": "sha256:9493b7309fb9317eac89826ea6a626c135779433ddaedcd1dfe123fe5bc21891",
  "line_count": 3,
  "word_count": 8,
  "character_count": 43
}
```

- `path` — the original relative path supplied.
- `size_bytes` — exact file byte length.
- `sha256` — `sha256:<64 lowercase hex digits>` of the file bytes.
- `line_count` — logical text lines (`\n` separated; a single trailing `\n` adds
  no phantom final line; an empty file has zero lines).
- `word_count` — whitespace-separated words.
- `character_count` — Unicode character (scalar value) count after valid UTF-8
  decoding.

Constraints:

- the path must remain inside the operational scope's `query_root`;
- the input must be a regular file;
- malformed UTF-8 fails cleanly;
- `max_bytes` is honoured;
- `max_bytes` has a hard maximum of 8 MiB;
- read-only capability: no network, no writes beyond ordinary diagnostics, no
  hidden or test-only behaviour.

The provider reads its operational scope from
`TETHERS_OPERATIONAL_SCOPE_JSON`. During host conformance
(`TETHERS_CONFORMANCE=1`) with no installed scope present, it falls back to
`TEMP` as a safe root with the 8 MiB hard maximum.

## Build the provider

```powershell
cargo build --manifest-path reference-plugs/text-stats-proof/provider-rust/Cargo.toml --locked
```

The provider executable is produced at
`reference-plugs/text-stats-proof/provider-rust/target/debug/tethers_text_stats_provider.exe`.

## Run the provider semantic tests

```powershell
cargo test --manifest-path reference-plugs/text-stats-proof/provider-rust/Cargo.toml --locked
```

## Pack, inspect, and conform

Assemble a temporary pack source (outside the repository) containing:

```text
plug.json
manifests/
    text-stats-v1.json
provider/
    tethers_text_stats_provider.exe
```

then:

```powershell
tethers-reference-host.exe plug pack --source <pack-source> --output text-stats.tetherplug
tethers-reference-host.exe plug inspect --package text-stats.tetherplug
tethers-reference-host.exe plug conform --package text-stats.tetherplug
tethers-reference-host.exe plug conform --package text-stats.tetherplug --allow-non-isolated-supervised-execution
```

`plug conform` is supervised but non-isolated, non-installing, and
non-trust-creating. The default path refuses execution until explicitly approved
with `--allow-non-isolated-supervised-execution`.
