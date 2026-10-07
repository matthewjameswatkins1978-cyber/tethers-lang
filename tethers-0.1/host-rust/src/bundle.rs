//! Atomic host-execution bundle authority (`tethers.authority/2`).
//!
//! A bundle is a bounded ordered set of PREPAREd ordinary Actions admitted
//! atomically: either every member becomes dispatchable together, or none
//! does. The bundle ledger below is the durable transaction record. Files
//! are immutable once published (`*.committing.json`, then
//! `*.committed.json`); a member is dispatchable if and only if the ledger
//! holds the committed marker for its bundle.
//!
//! Crash model: process crash between any two durable phases leaves at most
//! a committing file plus a prefix of member claims/intents/armed states.
//! Nothing armed is dispatchable without the committed marker, and an
//! identical retry resumes the same bundle (same bundle id, same recovered
//! execution identities) instead of forking a second one.
//!
//! Byte truth: bundle records carry identities and digests only —
//! `composition_digest` as an opaque handle, argument/manifest digests, never
//! raw stream bytes, secrets, or argv content.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Canonical bundle record format identity.
pub const BUNDLE_FORMAT: &str = "tethers-bundle-v1";
/// Directory under the host-data root holding bundle records.
pub const BUNDLE_DIR_NAME: &str = "bundles";
/// A bundle composes at least two stages; single Actions use `commit`.
pub const MIN_BUNDLE_MEMBERS: usize = 2;
/// Upper bound on members per bundle. Keeps one `commit_bundle` call
/// bounded in replay, Trail, and response size.
pub const MAX_BUNDLE_MEMBERS: usize = 8;
/// Bound on the opaque composition identity (handle, never parsed).
pub const MAX_COMPOSITION_DIGEST_BYTES: usize = 512;
/// Bound on one bundle record file.
pub const MAX_BUNDLE_FILE_BYTES: u64 = 65_536;

/// Machine error codes for the `commit_bundle` operation.
pub mod code {
    pub const UNKNOWN_PREPARED: &str = "commit_bundle.unknown_prepared";
    pub const ALREADY_COMMITTED: &str = "commit_bundle.already_committed";
    pub const BUNDLE_MEMBER: &str = "commit_bundle.bundle_member";
    pub const DUPLICATE_MEMBER: &str = "commit_bundle.duplicate_member";
    pub const MEMBER_COUNT: &str = "commit_bundle.member_count";
    pub const COMPOSITION_MISSING: &str = "commit_bundle.composition_missing";
    pub const COMPOSITION_INVALID: &str = "commit_bundle.composition_invalid";
    pub const COMPOSITION_MISMATCH: &str = "commit_bundle.composition_mismatch";
    pub const DENY: &str = "commit_bundle.deny";
    pub const UNAVAILABLE: &str = "commit_bundle.unavailable";
    pub const MISSING_PIN: &str = "commit_bundle.missing_pin";
    pub const IDENTITY_MISMATCH: &str = "commit_bundle.identity_mismatch";
    pub const APPROVAL_REQUIRED: &str = "commit_bundle.approval_required";
    pub const APPROVAL_NOT_READY: &str = "commit_bundle.approval_not_ready";
    pub const APPROVAL_FAILED: &str = "commit_bundle.approval_failed";
    pub const APPROVAL_CONSUME_FAILED: &str = "commit_bundle.approval_consume_failed";
    pub const REPLAY_UNAVAILABLE: &str = "commit_bundle.replay_unavailable";
    pub const REPLAY_BLOCKED: &str = "commit_bundle.replay_blocked";
    pub const INTENT_FAILED: &str = "commit_bundle.intent_failed";
    pub const ARMED_FAILED: &str = "commit_bundle.armed_failed";
    pub const LEDGER_UNAVAILABLE: &str = "commit_bundle.ledger_unavailable";
    pub const CRASH_SIMULATED: &str = "commit_bundle.crash_simulated";
}

/// Test-only crash injection between durable bundle phases. Set per gate
/// instance by tests; the production stdio path never sets one. When hit,
/// the commit aborts with [`code::CRASH_SIMULATED`] while durable state
/// stays exactly as a real process crash would leave it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleFailPoint {
    AfterBundleIntent,
    AfterMemberIntent(usize),
    AfterMemberArmed(usize),
    AfterApprovalConsume,
    BeforeCommittedMarker,
}

/// One bundle member's durable identity material. Digests and opaque
/// handles only — never raw arguments or stream bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleMember {
    pub prepared_id: String,
    pub action_id: String,
    pub evaluation_id: String,
    pub event_id: String,
    pub capability_name: String,
    pub capability_version: u32,
    pub manifest_digest: String,
    pub provider_identity: String,
    pub argument_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<String>,
    pub approval_consumed: bool,
}

/// Durable bundle record. Written once as `committing`, once as `committed`;
/// never mutated in place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleRecord {
    pub format: String,
    pub bundle_id: String,
    pub composition_digest: String,
    pub config_digest: String,
    pub members: Vec<BundleMember>,
}

/// Bounded summary for the `/2` status surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BundleSummary {
    pub bundle_id: String,
    pub state: &'static str,
    pub composition_digest: String,
    pub members: Vec<BundleMemberSummary>,
}

/// Bounded per-member summary (identities and outcome presence only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BundleMemberSummary {
    pub prepared_id: String,
    pub action_id: String,
    pub execution_id: Option<String>,
}

/// Extract the opaque composition identity from Core-planned action
/// arguments. The value is an exact bounded handle: non-empty, no control
/// characters. Tethers compares it for equality only and never parses it.
pub fn extract_composition_digest(
    arguments: &serde_json::Value,
) -> Result<String, (String, String)> {
    let object = arguments.as_object().ok_or_else(|| {
        (
            code::COMPOSITION_MISSING.to_owned(),
            "bundle members must carry a composition_digest argument".to_owned(),
        )
    })?;
    let value = object.get("composition_digest").ok_or_else(|| {
        (
            code::COMPOSITION_MISSING.to_owned(),
            "bundle members must carry a composition_digest argument".to_owned(),
        )
    })?;
    let digest = value.as_str().ok_or_else(|| {
        (
            code::COMPOSITION_INVALID.to_owned(),
            "composition_digest must be an opaque string identity".to_owned(),
        )
    })?;
    if digest.is_empty()
        || digest.len() > MAX_COMPOSITION_DIGEST_BYTES
        || digest.chars().any(|c| c.is_control())
    {
        return Err((
            code::COMPOSITION_INVALID.to_owned(),
            "composition_digest must be a bounded opaque identity without control characters"
                .to_owned(),
        ));
    }
    Ok(digest.to_owned())
}

/// Derive the deterministic bundle identity over the ordered member set,
/// the common composition identity, and the authority config digest. Any
/// reorder, add, remove, argument change, composition change, or
/// re-authorization under a new config yields a different bundle id, so a
/// retry collides only with its own unfinished attempt.
pub fn derive_bundle_id(
    members: &[BundleMember],
    composition_digest: &str,
    config_digest: &str,
) -> String {
    let material = serde_json::json!({
        "format": BUNDLE_FORMAT,
        "members": members.iter().map(|member| serde_json::json!({
            "prepared_id": member.prepared_id,
            "action_id": member.action_id,
            "evaluation_id": member.evaluation_id,
            "capability_name": member.capability_name,
            "capability_version": member.capability_version,
            "manifest_digest": member.manifest_digest,
            "provider_identity": member.provider_identity,
            "argument_digest": member.argument_digest,
        })).collect::<Vec<_>>(),
        "composition_digest": composition_digest,
        "config_digest": config_digest,
    });
    let bytes = serde_json_canonicalizer::to_vec(&material).expect("canonical bundle material");
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("bundle_{:x}", hasher.finalize())
}

fn is_bundle_file_name(name: &str, suffix: &str) -> Option<String> {
    let stem = name.strip_suffix(suffix)?;
    let hex = stem.strip_prefix("bundle_")?;
    if hex.len() == 64
        && hex
            .as_bytes()
            .iter()
            .all(|b| b.is_ascii_hexdigit() && (*b < b'A' || *b > b'F'))
    {
        Some(format!("bundle_{hex}"))
    } else {
        None
    }
}

/// Durable bundle ledger rooted at `<host-data-root>/bundles`.
#[derive(Debug, Clone)]
pub struct BundleLedger {
    dir: PathBuf,
}

impl BundleLedger {
    pub fn new(host_data_root: &Path) -> Self {
        Self {
            dir: host_data_root.join(BUNDLE_DIR_NAME),
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Ensure the ledger directory exists. Idempotent; safe to call on
    /// every bundle operation.
    pub fn provision(&self) -> Result<(), String> {
        std::fs::create_dir_all(&self.dir)
            .map_err(|e| format!("bundle ledger unavailable: {e}"))?;
        Ok(())
    }

    fn committing_name(bundle_id: &str) -> String {
        format!("{bundle_id}.committing.json")
    }

    fn committed_name(bundle_id: &str) -> String {
        format!("{bundle_id}.committed.json")
    }

    /// Atomically publish immutable bytes under a create-new name: write a
    /// unique temp file, sync it, rename over the final name, then read the
    /// final file back and verify byte identity. A crash leaves at most an
    /// ignored temp file; readers only ever see absent or complete records.
    fn publish_new(&self, name: &str, bytes: &[u8]) -> Result<(), String> {
        self.provision()?;
        if bytes.len() as u64 > MAX_BUNDLE_FILE_BYTES {
            return Err("bundle record exceeds size bound".to_owned());
        }
        let tmp_name = format!("{name}.{}.tmp", uuid::Uuid::new_v4().simple());
        let tmp_path = self.dir.join(&tmp_name);
        let final_path = self.dir.join(name);
        let result = (|| -> Result<(), String> {
            // Write + sync through one write handle: on Windows a
            // read-only reopen cannot FlushFileBuffers (os error 5), so
            // the sync must happen on the writing handle before close.
            // `create_new` keeps publication create-new atomic.
            {
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&tmp_path)
                    .map_err(|e| format!("bundle ledger write failed: {e}"))?;
                file.write_all(bytes)
                    .map_err(|e| format!("bundle ledger write failed: {e}"))?;
                file.sync_all()
                    .map_err(|e| format!("bundle ledger sync failed: {e}"))?;
            }
            std::fs::rename(&tmp_path, &final_path)
                .map_err(|e| format!("bundle ledger publish failed: {e}"))?;
            let back = std::fs::read(&final_path)
                .map_err(|e| format!("bundle ledger verify failed: {e}"))?;
            if back != bytes {
                return Err("bundle ledger published bytes do not verify".to_owned());
            }
            Ok(())
        })();
        let _ = std::fs::remove_file(&tmp_path);
        result
    }

    fn read_record(&self, name: &str) -> Result<Option<BundleRecord>, String> {
        let path = self.dir.join(name);
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("bundle ledger read failed: {e}")),
        };
        if bytes.len() as u64 > MAX_BUNDLE_FILE_BYTES {
            return Err("bundle record exceeds size bound".to_owned());
        }
        let record: BundleRecord =
            serde_json::from_slice(&bytes).map_err(|e| format!("bundle record malformed: {e}"))?;
        if record.format != BUNDLE_FORMAT {
            return Err("bundle record format mismatch".to_owned());
        }
        Ok(Some(record))
    }

    pub fn write_committing(&self, record: &BundleRecord) -> Result<(), String> {
        let bytes =
            serde_json::to_vec(record).map_err(|e| format!("bundle record encode failed: {e}"))?;
        self.publish_new(&Self::committing_name(&record.bundle_id), &bytes)
    }

    pub fn write_committed(&self, record: &BundleRecord) -> Result<(), String> {
        let bytes =
            serde_json::to_vec(record).map_err(|e| format!("bundle record encode failed: {e}"))?;
        self.publish_new(&Self::committed_name(&record.bundle_id), &bytes)
    }

    pub fn read_committing(&self, bundle_id: &str) -> Result<Option<BundleRecord>, String> {
        self.read_record(&Self::committing_name(bundle_id))
    }

    pub fn is_committed(&self, bundle_id: &str) -> Result<bool, String> {
        Ok(self
            .read_record(&Self::committed_name(bundle_id))?
            .is_some())
    }

    pub fn read_committed(&self, bundle_id: &str) -> Result<Option<BundleRecord>, String> {
        self.read_record(&Self::committed_name(bundle_id))
    }

    /// Find any bundle (committing or committed) that already lists this
    /// prepared identity as a member. Used to refuse individual commits of
    /// bundled members with an explicit code.
    pub fn find_member_bundle(&self, prepared_id: &str) -> Result<Option<String>, String> {
        let entries = match std::fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("bundle ledger scan failed: {e}")),
        };
        for entry in entries {
            let entry = entry.map_err(|e| format!("bundle ledger scan failed: {e}"))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let is_bundle = is_bundle_file_name(&name, ".committing.json")
                .or_else(|| is_bundle_file_name(&name, ".committed.json"));
            if is_bundle.is_none() {
                continue;
            }
            let Some(record) = self.read_record(&name)? else {
                continue;
            };
            if record.members.iter().any(|m| m.prepared_id == prepared_id) {
                return Ok(Some(record.bundle_id));
            }
        }
        Ok(None)
    }

    /// Bounded ledger summary for the `/2` status surface. Malformed files
    /// surface as ledger-unavailable rather than guessed state.
    pub fn summaries(&self) -> Result<Vec<BundleSummary>, String> {
        let mut out = Vec::new();
        let entries = match std::fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
            Err(e) => return Err(format!("bundle ledger scan failed: {e}")),
        };
        for entry in entries {
            let entry = entry.map_err(|e| format!("bundle ledger scan failed: {e}"))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let (bundle_id, state) = if let Some(id) = is_bundle_file_name(&name, ".committed.json")
            {
                (id, "committed")
            } else if let Some(id) = is_bundle_file_name(&name, ".committing.json") {
                // A committing file shadowed by its committed marker is
                // history; report the committed state once.
                let committed_path = self.dir.join(Self::committed_name(&id));
                if committed_path.exists() {
                    continue;
                }
                (id, "committing")
            } else {
                continue;
            };
            let Some(record) = self.read_record(&name)? else {
                continue;
            };
            if record.bundle_id != bundle_id {
                return Err("bundle record identity mismatch".to_owned());
            }
            out.push(BundleSummary {
                bundle_id,
                state,
                composition_digest: record.composition_digest,
                members: record
                    .members
                    .into_iter()
                    .map(|m| BundleMemberSummary {
                        prepared_id: m.prepared_id,
                        action_id: m.action_id,
                        execution_id: m.execution_id,
                    })
                    .collect(),
            });
            if out.len() >= crate::gate_protocol::MAX_STATUS_ENTRIES {
                break;
            }
        }
        out.sort_by(|a, b| a.bundle_id.cmp(&b.bundle_id));
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(prepared: &str, action: &str) -> BundleMember {
        BundleMember {
            prepared_id: prepared.to_owned(),
            action_id: action.to_owned(),
            evaluation_id: "eval-1".to_owned(),
            event_id: "evt-1".to_owned(),
            capability_name: "process.execute".to_owned(),
            capability_version: 2,
            manifest_digest:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            provider_identity: "tethers-agent-coding".to_owned(),
            argument_digest: format!("sha256:{:0<64}", prepared.trim_start_matches("prep_")),
            execution_id: None,
            approval_consumed: false,
        }
    }

    #[test]
    fn bundle_id_is_stable_and_order_sensitive() {
        let members = vec![member("prep_a", "a1"), member("prep_b", "a2")];
        let first = derive_bundle_id(&members, "comp", "cfg");
        let again = derive_bundle_id(&members, "comp", "cfg");
        assert_eq!(first, again);
        assert!(first.starts_with("bundle_"));
        let mut swapped = members.clone();
        swapped.reverse();
        assert_ne!(first, derive_bundle_id(&swapped, "comp", "cfg"));
        assert_ne!(first, derive_bundle_id(&members, "other", "cfg"));
        assert_ne!(first, derive_bundle_id(&members, "comp", "other-cfg"));
    }

    #[test]
    fn composition_digest_is_opaque_but_bounded() {
        assert!(extract_composition_digest(&serde_json::json!({})).is_err());
        assert!(extract_composition_digest(&serde_json::json!({"composition_digest": 7})).is_err());
        assert!(
            extract_composition_digest(&serde_json::json!({"composition_digest": "a\nb"})).is_err()
        );
        assert_eq!(
            extract_composition_digest(&serde_json::json!({"composition_digest": "opaque-handle"}))
                .unwrap(),
            "opaque-handle"
        );
    }

    #[test]
    fn ledger_round_trip_and_member_lookup() {
        let dir =
            std::env::temp_dir().join(format!("tethers-bundle-{}", uuid::Uuid::new_v4().simple()));
        let ledger = BundleLedger::new(&dir);
        let members = vec![member("prep_a", "a1"), member("prep_b", "a2")];
        let bundle_id = derive_bundle_id(&members, "comp", "cfg");
        let record = BundleRecord {
            format: BUNDLE_FORMAT.to_owned(),
            bundle_id: bundle_id.clone(),
            composition_digest: "comp".to_owned(),
            config_digest: "cfg".to_owned(),
            members,
        };
        ledger.write_committing(&record).unwrap();
        assert!(!ledger.is_committed(&bundle_id).unwrap());
        assert_eq!(
            ledger
                .read_committing(&bundle_id)
                .unwrap()
                .unwrap()
                .bundle_id,
            bundle_id
        );
        assert_eq!(
            ledger.find_member_bundle("prep_a").unwrap().as_deref(),
            Some(bundle_id.as_str())
        );
        assert_eq!(ledger.find_member_bundle("prep_other").unwrap(), None);
        ledger.write_committed(&record).unwrap();
        assert!(ledger.is_committed(&bundle_id).unwrap());
        let summaries = ledger.summaries().unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].state, "committed");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
