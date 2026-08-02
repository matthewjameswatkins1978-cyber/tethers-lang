use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;
use sha2::Digest;
use tethers_reference_host::local_anchor::{
    AdmissionBinding, AdmissionResult, LocalAnchorCoordinator, EVENT_FORMAT_VERSION, EVENT_NAME,
};

fn temp_root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "tethers-m5-integration-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after epoch")
            .as_nanos()
    ))
}

fn event_json(event_id: &str, payload: serde_json::Value) -> String {
    let payload_bytes = serde_json_canonicalizer::to_vec(&payload).unwrap();
    let digest = format!(
        "sha256:{:x}",
        sha2::Sha256::digest(payload_bytes.as_slice())
    );
    json!({
        "event_format_version": EVENT_FORMAT_VERSION,
        "event_id": event_id,
        "event_name": EVENT_NAME,
        "provider_identity": "file-tools",
        "installed_plug_id": "plug-m5",
        "session_id": "session-m5",
        "occurred_at_unix_ms": 42,
        "payload": payload,
        "payload_digest": digest,
        "source_relative_path": "in/event.txt",
        "generation": 0
    })
    .to_string()
}

#[test]
fn durable_local_anchor_restart_duplicate_conflict_and_scope() {
    let root = temp_root();
    let source = root.join("source");
    fs::create_dir_all(source.join("in")).unwrap();
    fs::write(source.join("in/event.txt"), b"fixture").unwrap();
    let binding = AdmissionBinding {
        installed_plug_id: "plug-m5".into(),
        provider_identity: "file-tools".into(),
        session_id: "session-m5".into(),
        event_name: EVENT_NAME.into(),
        source_root: source.clone(),
    };
    let first = event_json("provider-event-1", json!({"path":"in/event.txt"}));
    let mut coordinator =
        LocalAnchorCoordinator::open(root.join("admission"), binding.clone()).unwrap();
    let (admitted, anchor) = coordinator
        .admit_notification(&first, 100, |_| Ok(()))
        .unwrap();
    assert!(matches!(admitted, AdmissionResult::Admitted { .. }));
    assert_eq!(anchor.generation, 0);

    drop(coordinator);
    let mut restarted = LocalAnchorCoordinator::open(root.join("admission"), binding).unwrap();
    let mut acknowledgements = 0;
    let (duplicate, duplicate_anchor) = restarted
        .admit_notification(&first, 101, |_| {
            acknowledgements += 1;
            Ok(())
        })
        .unwrap();
    assert!(matches!(duplicate, AdmissionResult::Duplicate { .. }));
    assert_eq!(duplicate_anchor.event_id, anchor.event_id);
    assert_eq!(acknowledgements, 1);

    let conflict = event_json("provider-event-1", json!({"path":"in/other.txt"}));
    assert!(restarted
        .admit_notification(&conflict, 102, |_| panic!("conflict must not acknowledge"))
        .is_err());
    assert!(root.join("admission").read_dir().unwrap().count() >= 2);

    let _ = fs::remove_dir_all(root);
}
