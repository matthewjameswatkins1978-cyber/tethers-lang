use std::path::PathBuf;
use tethers_reference_host::authority_v1::{
    AuthorityGate, GateConfig, ResponseStatus, AUTHORITY_PROTOCOL,
};

fn main() {
    let mut gate = AuthorityGate::new(GateConfig {
        config_path: PathBuf::from("/consumer/runtime.json"),
        engine_path: PathBuf::from("/consumer/tethers-engine"),
        trail_path: PathBuf::from("/consumer/trail.jsonl"),
        host_data_root: PathBuf::from("/consumer/host-data"),
    });
    let response = gate.handle("hello", "external-rust-consumer-1", &serde_json::Map::new());
    assert_eq!(response.schema, AUTHORITY_PROTOCOL);
    assert!(matches!(response.status, ResponseStatus::Ok));
    assert_eq!(response.result.as_ref().unwrap()["authority_granted"], false);
    assert_eq!(gate.provider_invocations(), 0);
    println!("PASS pinned Git-tag crate consumer: {AUTHORITY_PROTOCOL}");
}
