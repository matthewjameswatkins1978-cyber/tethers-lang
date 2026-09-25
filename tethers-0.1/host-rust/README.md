# Tethers reference Host crate

`tethers-reference-host` packages the Rust reference Host, executable, and
Authority Gate. The public Rust integration entry point is
`tethers_reference_host::authority_v1`; its messages use the independently
versioned `tethers.authority/1` protocol. The process boundary is documented in
the repository's `docs/INTEGRATING_TETHERS.md` and
`docs/tethers.authority.1.md`.

```rust
use std::path::PathBuf;
use tethers_reference_host::authority_v1::{
    AuthorityGate, GateConfig, ResponseStatus, AUTHORITY_PROTOCOL,
};

let mut gate = AuthorityGate::new(GateConfig {
    config_path: PathBuf::from("/absolute/consumer/runtime.json"),
    engine_path: PathBuf::from("/absolute/consumer/tethers-engine"),
    trail_path: PathBuf::from("/absolute/consumer/trail.jsonl"),
    host_data_root: PathBuf::from("/absolute/consumer/host-data"),
});
let response = gate.handle("hello", "consumer-hello-1", &serde_json::Map::new());
assert_eq!(response.schema, AUTHORITY_PROTOCOL);
assert!(matches!(response.status, ResponseStatus::Ok));
```

Pin the exact 0.8.0 crate from the repository tag until a crates.io release is
made:

```toml
[dependencies]
tethers-reference-host = { git = "https://github.com/matthewjameswatkins1978-cyber/tethers-lang", tag = "v0.8.0" }
serde_json = "1"
```

`authority_v1` is the intended Rust entry point; other public modules remain
available for existing in-repository integration suites and are not promoted
as a stable application API. Until product 1.0, pin an exact release and
revalidate when upgrading. For language-independent use, prefer the
`tethers.authority/1` process protocol. Neither API lets the Gate perform
provider effects: the Host owns physical execution and must report outcomes.
