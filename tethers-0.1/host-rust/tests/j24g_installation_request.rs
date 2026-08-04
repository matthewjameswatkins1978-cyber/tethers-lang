use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tethers_reference_host::installation_request::{
    load_installation_request, parse_installation_request_bytes, InstallationRequest,
    InstallationTargetState, InstallationTrustScope, INSTALLATION_REQUEST_MAX_BYTES,
    INSTALLATION_REQUEST_SCHEMA,
};
use uuid::Uuid;

const CANDIDATE_ID: &str = "3d846d40-01fc-4e1e-b77d-83944dbed76f";
const VALID_SCHEMA: &str = "tethers.plug-install/1";

fn valid_value() -> Value {
    json!({
        "schema": VALID_SCHEMA,
        "candidate_id": CANDIDATE_ID,
        "trust": {"scope": "exact_candidate"},
        "conformance": {"allow_non_isolated_supervised_execution": true},
        "installation": {"target_state": "disabled"}
    })
}

fn valid_bytes() -> Vec<u8> {
    serde_json::to_vec(&valid_value()).unwrap()
}

fn temp_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("tethers-j24g-{name}-{}", Uuid::new_v4()))
}

fn write_request(root: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = root.join(name);
    fs::write(&path, bytes).unwrap();
    path
}

fn assert_error(
    result: Result<
        InstallationRequest,
        tethers_reference_host::installation_request::InstallationRequestError,
    >,
    code: &str,
    message: &str,
    field: Option<&str>,
) {
    let error = result.unwrap_err();
    assert_eq!(error.code, code);
    assert_eq!(error.message, message);
    assert_eq!(error.field.as_deref(), field);
}

fn assert_invalid(
    result: Result<
        InstallationRequest,
        tethers_reference_host::installation_request::InstallationRequestError,
    >,
    message: &str,
    field: Option<&str>,
) {
    assert_error(result, "installation_request_invalid", message, field);
}

fn snapshot(root: &Path) -> BTreeMap<String, String> {
    fn visit(root: &Path, path: &Path, output: &mut BTreeMap<String, String>) {
        let mut entries = fs::read_dir(path)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        entries.sort();
        for entry in entries {
            let relative = entry
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let metadata = fs::symlink_metadata(&entry).unwrap();
            if metadata.is_dir() {
                output.insert(format!("{relative}/"), "<directory>".into());
                visit(root, &entry, output);
            } else if metadata.is_file() {
                output.insert(relative, sha256(&fs::read(&entry).unwrap()));
            } else if metadata.file_type().is_symlink() {
                output.insert(relative, "<symlink>".into());
            }
        }
    }

    let mut output = BTreeMap::new();
    visit(root, root, &mut output);
    output
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn insert_unknown(value: &mut Value, object: &str, key: &str) {
    value
        .get_mut(object)
        .and_then(Value::as_object_mut)
        .unwrap()
        .insert(key.to_owned(), Value::Null);
}

#[test]
fn exact_request_parses_into_typed_values() {
    let request = parse_installation_request_bytes(&valid_bytes()).unwrap();

    assert_eq!(request.schema, INSTALLATION_REQUEST_SCHEMA);
    assert_eq!(request.candidate_id, CANDIDATE_ID);
    assert_eq!(request.trust.scope, InstallationTrustScope::ExactCandidate);
    assert!(request.conformance.allow_non_isolated_supervised_execution);
    assert_eq!(
        request.installation.target_state,
        InstallationTargetState::Disabled
    );
}

#[test]
fn exact_absolute_file_loads_and_changes_no_filesystem_state() {
    let root = temp_dir("load");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("sentinel"), b"unchanged").unwrap();
    let path = write_request(&root, "request.json", &valid_bytes());
    let before = snapshot(&root);

    let request = load_installation_request(&path).unwrap();

    assert_eq!(request.candidate_id, CANDIDATE_ID);
    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sixteen_kibibytes_is_allowed_and_one_more_byte_is_rejected() {
    let mut exact = valid_bytes();
    exact.resize(INSTALLATION_REQUEST_MAX_BYTES, b' ');
    assert_eq!(exact.len(), INSTALLATION_REQUEST_MAX_BYTES);
    assert!(parse_installation_request_bytes(&exact).is_ok());

    let mut oversized = exact.clone();
    oversized.push(b' ');
    assert_invalid(
        parse_installation_request_bytes(&oversized),
        "installation request exceeds 16 KiB limit",
        None,
    );
}

#[test]
fn bounded_loader_accepts_limit_and_rejects_one_byte_over_limit() {
    let root = temp_dir("bounds");
    fs::create_dir_all(&root).unwrap();
    let mut exact = valid_bytes();
    exact.resize(INSTALLATION_REQUEST_MAX_BYTES, b' ');
    let exact_path = write_request(&root, "exact.json", &exact);
    assert!(load_installation_request(&exact_path).is_ok());

    let mut oversized = exact;
    oversized.push(b' ');
    let oversized_path = write_request(&root, "oversized.json", &oversized);
    let before = snapshot(&root);
    assert_invalid(
        load_installation_request(&oversized_path),
        "installation request exceeds 16 KiB limit",
        None,
    );
    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn byte_boundary_rejections_use_stable_errors() {
    let mut bom = vec![0xef, 0xbb, 0xbf];
    bom.extend(valid_bytes());
    assert_invalid(
        parse_installation_request_bytes(&bom),
        "installation request contains UTF-8 BOM",
        None,
    );
    assert_invalid(
        parse_installation_request_bytes(b"{\xff"),
        "installation request is not valid UTF-8",
        None,
    );
    for bytes in [
        br#"{"# .as_slice(),
        br#"{"schema":"tethers.plug-install/1"} {}"#.as_slice(),
        br#"{"schema":"tethers.plug-install/1","schema":"tethers.plug-install/1"}"#.as_slice(),
        br#"{"schema":"tethers.plug-install/1","candidate_id":"3d846d40-01fc-4e1e-b77d-83944dbed76f","trust":{"scope":"exact_candidate","scope":"exact_candidate"},"conformance":{"allow_non_isolated_supervised_execution":true},"installation":{"target_state":"disabled"}}"#.as_slice(),
        br#"{"schema":"tethers.plug-install/1","candidate_id":"3d846d40-01fc-4e1e-b77d-83944dbed76f","trust":{"scope":"exact_candidate"},"conformance":{"allow_non_isolated_supervised_execution":true,"allow_non_isolated_supervised_execution":true},"installation":{"target_state":"disabled"}}"#.as_slice(),
        br#"{"schema":"tethers.plug-install/1","candidate_id":"3d846d40-01fc-4e1e-b77d-83944dbed76f","trust":{"scope":"exact_candidate"},"conformance":{"allow_non_isolated_supervised_execution":true},"installation":{"target_state":"disabled","target_state":"disabled"}}"#.as_slice(),
    ] {
        assert_invalid(
            parse_installation_request_bytes(bytes),
            "installation request must be valid JSON with no duplicate keys or trailing content",
            None,
        );
    }
}

#[test]
fn every_root_field_is_required() {
    for field in [
        "schema",
        "candidate_id",
        "trust",
        "conformance",
        "installation",
    ] {
        let mut value = valid_value();
        value.as_object_mut().unwrap().remove(field);
        assert_invalid(
            parse_installation_request_bytes(&serde_json::to_vec(&value).unwrap()),
            "required field is missing",
            Some(&format!("/{field}")),
        );
    }
}

#[test]
fn every_nested_field_is_required() {
    for (object, field, pointer) in [
        ("trust", "scope", "/trust/scope"),
        (
            "conformance",
            "allow_non_isolated_supervised_execution",
            "/conformance/allow_non_isolated_supervised_execution",
        ),
        ("installation", "target_state", "/installation/target_state"),
    ] {
        let mut value = valid_value();
        value
            .get_mut(object)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert_invalid(
            parse_installation_request_bytes(&serde_json::to_vec(&value).unwrap()),
            "required field is missing",
            Some(pointer),
        );
    }
}

#[test]
fn unknown_fields_are_rejected_with_rfc6901_pointers_at_every_depth() {
    let mut root = valid_value();
    root.as_object_mut()
        .unwrap()
        .insert("a/b~c".into(), Value::Null);
    assert_invalid(
        parse_installation_request_bytes(&serde_json::to_vec(&root).unwrap()),
        "field is not permitted in installation request",
        Some("/a~1b~0c"),
    );

    for (object, pointer) in [
        ("trust", "/trust/a~1b~0c"),
        ("conformance", "/conformance/a~1b~0c"),
        ("installation", "/installation/a~1b~0c"),
    ] {
        let mut value = valid_value();
        insert_unknown(&mut value, object, "a/b~c");
        assert_invalid(
            parse_installation_request_bytes(&serde_json::to_vec(&value).unwrap()),
            "field is not permitted in installation request",
            Some(pointer),
        );
    }
}

#[test]
fn wrong_root_and_nested_object_types_are_rejected() {
    for value in [
        json!(null),
        json!(true),
        json!(7),
        json!("request"),
        json!([]),
    ] {
        assert_invalid(
            parse_installation_request_bytes(&serde_json::to_vec(&value).unwrap()),
            "value must be an object",
            Some(""),
        );
    }

    for field in ["trust", "conformance", "installation"] {
        let mut value = valid_value();
        value
            .as_object_mut()
            .unwrap()
            .insert(field.into(), json!("object"));
        assert_invalid(
            parse_installation_request_bytes(&serde_json::to_vec(&value).unwrap()),
            "value must be an object",
            Some(&format!("/{field}")),
        );
    }
}

#[test]
fn wrong_scalar_types_are_rejected_at_their_pointers() {
    for field in ["schema", "candidate_id"] {
        let mut value = valid_value();
        value
            .as_object_mut()
            .unwrap()
            .insert(field.into(), json!(false));
        assert_invalid(
            parse_installation_request_bytes(&serde_json::to_vec(&value).unwrap()),
            "value must be a string",
            Some(&format!("/{field}")),
        );
    }

    let mut trust = valid_value();
    trust["trust"]["scope"] = json!(false);
    assert_invalid(
        parse_installation_request_bytes(&serde_json::to_vec(&trust).unwrap()),
        "value must be a string",
        Some("/trust/scope"),
    );

    let mut conformance = valid_value();
    conformance["conformance"]["allow_non_isolated_supervised_execution"] = json!("true");
    assert_invalid(
        parse_installation_request_bytes(&serde_json::to_vec(&conformance).unwrap()),
        "value must be a boolean",
        Some("/conformance/allow_non_isolated_supervised_execution"),
    );

    let mut installation = valid_value();
    installation["installation"]["target_state"] = json!(false);
    assert_invalid(
        parse_installation_request_bytes(&serde_json::to_vec(&installation).unwrap()),
        "value must be a string",
        Some("/installation/target_state"),
    );
}

#[test]
fn unsupported_semantic_values_are_rejected_at_their_pointers() {
    let mut schema = valid_value();
    schema["schema"] = json!("tethers.plug-install/2");
    assert_invalid(
        parse_installation_request_bytes(&serde_json::to_vec(&schema).unwrap()),
        "schema must be exactly \"tethers.plug-install/1\"",
        Some("/schema"),
    );

    let mut trust = valid_value();
    trust["trust"]["scope"] = json!("publisher");
    assert_invalid(
        parse_installation_request_bytes(&serde_json::to_vec(&trust).unwrap()),
        "trust scope must be exactly \"exact_candidate\"",
        Some("/trust/scope"),
    );

    let mut conformance = valid_value();
    conformance["conformance"]["allow_non_isolated_supervised_execution"] = json!(false);
    assert_invalid(
        parse_installation_request_bytes(&serde_json::to_vec(&conformance).unwrap()),
        "non-isolated supervised execution must be explicitly approved",
        Some("/conformance/allow_non_isolated_supervised_execution"),
    );

    let mut installation = valid_value();
    installation["installation"]["target_state"] = json!("enabled");
    assert_invalid(
        parse_installation_request_bytes(&serde_json::to_vec(&installation).unwrap()),
        "installation target_state must be exactly \"disabled\"",
        Some("/installation/target_state"),
    );
}

#[test]
fn candidate_id_must_be_canonical_lowercase_hyphenated_uuid() {
    for candidate_id in [
        "not-a-uuid",
        "3d846d4001fc4e1eb77d83944dbed76f",
        "{3d846d40-01fc-4e1e-b77d-83944dbed76f}",
        "3D846D40-01FC-4E1E-B77D-83944DBED76F",
        "3d846d40-01fc-4e1e-b77d-83944dbed76g",
    ] {
        let mut value = valid_value();
        value["candidate_id"] = json!(candidate_id);
        assert_invalid(
            parse_installation_request_bytes(&serde_json::to_vec(&value).unwrap()),
            "candidate_id must be a canonical lowercase hyphenated UUID",
            Some("/candidate_id"),
        );
    }
}

#[test]
fn paths_fail_closed_without_exposing_platform_errors_or_mutating_files() {
    let root = temp_dir("paths");
    fs::create_dir_all(&root).unwrap();
    let directory = root.join("directory");
    fs::create_dir(&directory).unwrap();
    let missing = root.join("secret-missing-request.json");
    let before = snapshot(&root);

    assert_invalid(
        load_installation_request(Path::new("relative-request.json")),
        "installation request path must be absolute",
        None,
    );
    assert_error(
        load_installation_request(&missing),
        "installation_request_io",
        "cannot read installation request",
        None,
    );
    assert_invalid(
        load_installation_request(&directory),
        "installation request path must name an ordinary file",
        None,
    );
    assert_eq!(before, snapshot(&root));

    let error = load_installation_request(&missing).unwrap_err();
    assert!(!error.to_string().contains("secret-missing-request"));
    assert!(!error.to_string().contains(root.to_string_lossy().as_ref()));
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn final_symlink_is_rejected_without_following_it() {
    use std::os::windows::fs::symlink_file;

    let root = temp_dir("symlink");
    fs::create_dir_all(&root).unwrap();
    let target = write_request(&root, "target.json", &valid_bytes());
    let link = root.join("request-link.json");
    if let Err(error) = symlink_file(&target, &link) {
        if error.raw_os_error() == Some(1314) {
            fs::remove_dir_all(root).unwrap();
            return;
        }
        panic!("could not create symlink fixture: {error}");
    }
    let before = snapshot(&root);

    assert_invalid(
        load_installation_request(&link),
        "installation request path must name an ordinary file",
        None,
    );
    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn parsing_does_not_create_or_remove_any_path() {
    let root = temp_dir("parse-no-mutation");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("sentinel"), b"unchanged").unwrap();
    let before = snapshot(&root);

    assert!(parse_installation_request_bytes(&valid_bytes()).is_ok());
    let mut invalid = valid_value();
    invalid
        .as_object_mut()
        .unwrap()
        .insert("unexpected".into(), Value::Null);
    assert!(parse_installation_request_bytes(&serde_json::to_vec(&invalid).unwrap()).is_err());

    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn public_error_display_is_stable_and_does_not_include_field_contents() {
    let mut value = valid_value();
    value["schema"] = json!("wrong");
    let error = parse_installation_request_bytes(&serde_json::to_vec(&value).unwrap()).unwrap_err();
    assert_eq!(
        error.to_string(),
        "installation_request_invalid: schema must be exactly \"tethers.plug-install/1\""
    );
    assert_eq!(error.field.as_deref(), Some("/schema"));
}
