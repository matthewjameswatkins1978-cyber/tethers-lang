//! P6 The Evil Bunny Test — adversarial provider proof.
//!
//! Every test drives the REAL public CLI and the REAL production conformance
//! seam (`run_host_conformance`) against the committed Evil Bunny fixture
//! under `reference-plugs/evil-bunny-proof/`.  The fixture is a safe,
//! deterministic protocol test provider; it is not a malware test and never
//! claims operating-system isolation.

#![cfg(windows)]

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tethers_reference_host::candidate::{extract_to_quarantine, CandidateRegistry};
use tethers_reference_host::conformance::{
    run_host_conformance, CaseDisposition, ConformanceDisposition,
};
use tethers_reference_host::launch_profile::PreparedSupervisedLaunch;
use tethers_reference_host::trust::{
    DeveloperApprovalStore, PackageTrustEvidence, PublisherTrustStore,
};

fn host_binary() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_tethers-reference-host")
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_tethers_reference_host"))
        .map(PathBuf::from)
        .or_else(|| {
            std::env::current_exe().ok().and_then(|path| {
                path.parent()?
                    .parent()
                    .map(|dir| dir.join("tethers-reference-host.exe"))
            })
        })
        .expect("compiled reference host binary")
}

fn provider_binary() -> PathBuf {
    PathBuf::from(
        std::env::var_os("TETHERS_EVIL_BUNNY_PROVIDER_EXE")
            .expect("Evil Bunny provider path is required"),
    )
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("reference-plugs/evil-bunny-proof")
}

fn author_root() -> PathBuf {
    fixture_root().join("author")
}

fn parse(output: &std::process::Output) -> Value {
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    let text = String::from_utf8(output.stdout.clone()).unwrap();
    assert_eq!(text.lines().count(), 1);
    serde_json::from_str(text.trim()).unwrap()
}

fn make_source(root: &Path, case_id: &str) -> PathBuf {
    let source = root.join("source").join(case_id);
    fs::create_dir_all(source.join("provider")).unwrap();
    fs::create_dir_all(source.join("manifests")).unwrap();
    fs::copy(
        author_root().join("cases").join(case_id).join("plug.json"),
        source.join("plug.json"),
    )
    .unwrap();
    fs::copy(
        author_root().join("manifests/evil-probe-v1.json"),
        source.join("manifests/evil-probe-v1.json"),
    )
    .unwrap();
    fs::copy(
        provider_binary(),
        source.join("provider/tethers_evil_bunny_provider.exe"),
    )
    .unwrap();
    source
}

fn pack(root: &Path, case_id: &str) -> PathBuf {
    let source = make_source(root, case_id);
    let package = root.join(format!("{case_id}.tetherplug"));
    let output = Command::new(host_binary())
        .args(["plug", "pack", "--source"])
        .arg(source)
        .args(["--output"])
        .arg(&package)
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "pack must succeed for {case_id}"
    );
    package
}

fn assert_default_conform_refusal(package: &Path) {
    let denied = Command::new(host_binary())
        .args(["plug", "conform", "--package"])
        .arg(package)
        .output()
        .unwrap();
    assert_eq!(denied.status.code(), Some(5), "default conform must exit 5");
    let denied = parse(&denied);
    assert_eq!(denied["status"], "approval_required");
    assert_eq!(
        denied["error"]["code"],
        "conformance_execution_approval_required"
    );
}

#[test]
#[ignore = "requires TETHERS_EVIL_BUNNY_PROVIDER_EXE from the standalone fixture build"]
fn p6_evil_bunny_good_control_public_journey() {
    let root = std::env::temp_dir().join(format!("tethers-p6-eb00-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();

    let package = pack(&root, "EB-00-good");

    let inspected = Command::new(host_binary())
        .args(["plug", "inspect", "--package"])
        .arg(&package)
        .output()
        .unwrap();
    assert_eq!(inspected.status.code(), Some(0));
    let inspected = parse(&inspected);
    let inspection = &inspected["data"]["inspection"];
    assert_eq!(
        inspection["package"]["package_id"],
        "tethers.evil-bunny-proof"
    );
    assert_eq!(inspection["capabilities"][0]["name"], "evil.probe");

    assert_default_conform_refusal(&package);

    let conformed = Command::new(host_binary())
        .args(["plug", "conform", "--package"])
        .arg(&package)
        .arg("--allow-non-isolated-supervised-execution")
        .output()
        .unwrap();
    assert_eq!(conformed.status.code(), Some(0), "Good Bunny must pass");
    let conformed = parse(&conformed);
    assert_eq!(conformed["data"]["conformance"]["disposition"], "passed");
    assert_eq!(conformed["data"]["launch_profile"]["isolated"], false);
    assert!(conformed["data"]["launch_profile"]["limitation"]
        .as_str()
        .unwrap()
        .contains("not isolated"));

    let _ = fs::remove_dir_all(root);
}

#[test]
#[ignore = "requires TETHERS_EVIL_BUNNY_PROVIDER_EXE from the standalone fixture build"]
fn p6_evil_bunny_hostile_cases_refused_public_journey() {
    let root = std::env::temp_dir().join(format!("tethers-p6-hostile-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();

    let cases: &[(&str, Option<&str>)] = &[
        ("EB-01-identity-liar", Some("provider_identity")),
        ("EB-02-protocol-version-liar", Some("protocol_pin")),
        ("EB-03-missing-operation", Some("catalogue_drift")),
        ("EB-04-surprise-operation", Some("catalogue_drift")),
        ("EB-05-wrong-operation-name", Some("catalogue_drift")),
        ("EB-06-input-schema-liar", Some("catalogue_drift")),
        ("EB-07a-output-schema-mismatch", Some("catalogue_drift")),
        ("EB-07b-output-schema-omitted", Some("catalogue_drift")),
        ("EB-08-malformed-stdout", Some("conformance_protocol")),
        ("EB-09-wrong-response-id", Some("protocol_correlation")),
        ("EB-10-early-death", Some("conformance_protocol")),
        ("EB-11-silent-hang", Some("conformance_protocol")),
        ("EB-12-shutdown-refusal", None),
    ];

    for (case_id, session_code) in cases {
        let package = pack(&root, case_id);
        assert_default_conform_refusal(&package);

        let conformed = Command::new(host_binary())
            .args(["plug", "conform", "--package"])
            .arg(&package)
            .arg("--allow-non-isolated-supervised-execution")
            .output()
            .unwrap();
        let env = parse(&conformed);
        assert_eq!(
            conformed.status.code(),
            Some(6),
            "{case_id} approved conform must fail, envelope: {env}"
        );
        assert_eq!(env["status"], "failed");
        assert_eq!(env["error"]["code"], "plug_conformance_failed");
        let conformance = &env["data"]["conformance"];
        assert_ne!(
            conformance["disposition"], "passed",
            "{case_id} must not pass"
        );

        let session = conformance["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["case_id"] == "conformance_session");
        if let Some(expected) = session_code {
            let code = session
                .and_then(|c| c["safe_diagnostic_code"].as_str())
                .unwrap_or_else(|| panic!("{case_id} missing conformance_session code"));
            assert_eq!(
                code, *expected,
                "{case_id} violated contract must be identified by {expected}"
            );
        } else {
            assert!(
                session.is_none(),
                "{case_id} must reach shutdown with no protocol session failure"
            );
            let shutdown = conformance["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["case_id"] == "bounded_shutdown_process_cleanup")
                .expect("{case_id} must record the shutdown case");
            assert_eq!(shutdown["disposition"], "failed");
            assert_eq!(
                shutdown["safe_diagnostic_code"],
                "provider_did_not_exit_gracefully"
            );
        }
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
#[ignore = "requires TETHERS_EVIL_BUNNY_PROVIDER_EXE from the standalone fixture build"]
fn p6_evil_bunny_fixed_gaps_rejected_at_real_conformance_seam() {
    // The real generic conformance seam (`run_host_conformance`, the exact
    // function behind public `plug conform`) must reject every previously
    // accepted hostile case.  Each case gets its own root because every Evil
    // Bunny package shares one package_id/version while carrying a distinct
    // semantic digest (mode argument), and candidate admission is exact.
    let cases: &[(&str, Option<&str>)] = &[
        ("EB-07a-output-schema-mismatch", Some("catalogue_drift")),
        ("EB-07b-output-schema-omitted", Some("catalogue_drift")),
        ("EB-09-wrong-response-id", Some("protocol_correlation")),
        ("EB-12-shutdown-refusal", None),
    ];

    for (case_id, session_code) in cases {
        let root = std::env::temp_dir().join(format!(
            "tethers-p6-seam-{}-{}",
            case_id.replace('.', "-"),
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).unwrap();
        let package = pack(&root, case_id);
        let report = tethers_reference_host::package::inspect(&package).unwrap();
        let quarantine = root.join("quarantine");
        let candidate = CandidateRegistry::open(&root.join("candidates"), &quarantine)
            .unwrap()
            .create(&extract_to_quarantine(&report, &quarantine).unwrap())
            .unwrap();
        let developers = DeveloperApprovalStore::open(&root.join("developers")).unwrap();
        let developer = developers
            .approve_exact_digest(&candidate.semantic_package_digest, "Matthew")
            .unwrap();
        let trust = PackageTrustEvidence::unsigned(&developer).unwrap();
        let publishers = PublisherTrustStore::open(&root.join("publishers")).unwrap();
        let prepared = PreparedSupervisedLaunch::prepare(
            &candidate,
            &quarantine,
            &root.join("scratch"),
            Duration::from_secs(5),
        )
        .unwrap();
        let conformance = run_host_conformance(
            &prepared,
            &candidate,
            &quarantine,
            &trust,
            &publishers,
            &developers,
            "tethers-reference-host@0.3-p6",
        )
        .unwrap();

        assert_ne!(
            conformance.disposition,
            ConformanceDisposition::Passed,
            "{case_id} must not reach passed at the real conformance seam"
        );
        let session = conformance
            .cases
            .iter()
            .find(|c| c.case_id == "conformance_session");
        if let Some(expected) = session_code {
            assert_eq!(
                session
                    .expect("{case_id} must record a session failure")
                    .safe_diagnostic_code
                    .as_deref(),
                Some(*expected)
            );
        } else {
            assert!(
                session.is_none(),
                "{case_id} must reach shutdown with no protocol session failure"
            );
            let shutdown = conformance
                .cases
                .iter()
                .find(|c| c.case_id == "bounded_shutdown_process_cleanup")
                .expect("{case_id} must record the shutdown case");
            assert_eq!(shutdown.disposition, CaseDisposition::Failed);
            assert_eq!(
                shutdown.safe_diagnostic_code.as_deref(),
                Some("provider_did_not_exit_gracefully")
            );
        }
        prepared.cleanup_scratch().unwrap();
        let _ = fs::remove_dir_all(root);
    }
}
