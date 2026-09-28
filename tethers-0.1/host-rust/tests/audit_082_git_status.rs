// Tethers 0.8.2 adversarial-hardening regression suite (Lane B evidence).
//
// DEF-05 proof: `tethers git status` parses `git status --porcelain=v1 -z
// --branch` per the official Git specification against REAL temporary
// repositories, and its machine-readable result agrees with native Git for
// renames, conflicts, untracked files, spaces, Unicode, detached HEAD, and
// clean repositories. The pre-0.8.2 parser misparsed rename source tokens as
// independent entries, panicked on multibyte token slices, and hard-coded
// `conflict: false`; these tests fail against that implementation.

use std::path::{Path, PathBuf};
use std::process::Command;

fn host_binary() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_tethers-reference-host")
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_tethers_reference_host"))
        .map(PathBuf::from)
        .or_else(|| {
            std::env::current_exe().ok().and_then(|path| {
                path.parent()?.parent().map(|dir| {
                    dir.join(if cfg!(windows) {
                        "tethers-reference-host.exe"
                    } else {
                        "tethers-reference-host"
                    })
                })
            })
        })
        .expect("compiled reference host binary")
}

fn git(repo: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("git must be available");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_allow_failure(repo: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("git must be available");
    let _ = output;
}

fn git_porcelain_z(repo: &Path) -> Vec<u8> {
    Command::new("git")
        .args(["status", "--porcelain=v1", "-z", "--branch"])
        .current_dir(repo)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("git must be available")
        .stdout
}

fn new_repo(label: &str) -> (PathBuf, PathBuf) {
    let base = std::env::temp_dir().join(format!("audit-082-git-{label}-{}", uuid::Uuid::new_v4()));
    let repo = base.join("repo");
    let state = base.join("state");
    std::fs::create_dir_all(&repo).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    git(&repo, &["init", "-b", "main"]);
    git(&repo, &["config", "core.autocrlf", "false"]);
    git(&repo, &["config", "user.email", "audit@example.invalid"]);
    git(&repo, &["config", "user.name", "Audit 082"]);
    std::fs::write(repo.join("base.txt"), "base\n").unwrap();
    git(&repo, &["add", "base.txt"]);
    git(&repo, &["commit", "-m", "base"]);
    (repo, state)
}

fn tethers_git_status(repo: &Path, state: &Path) -> serde_json::Value {
    let output = Command::new(host_binary())
        .args(["git", "status"])
        .current_dir(repo)
        .env("TETHERS_HOST_DATA_ROOT", state)
        .output()
        .expect("run host binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout must be a JSON envelope");
    assert_eq!(envelope["schema"], "tethers.cli/1");
    assert_eq!(
        envelope["status"], "ok",
        "git status must succeed: {stdout}"
    );
    envelope["data"].clone()
}

fn strings(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .expect("array")
        .iter()
        .map(|item| item.as_str().unwrap().to_owned())
        .collect()
}

/// Independent native-Git oracle: collect (code, path) pairs and rename
/// source/destination pairs straight from the NUL-separated porcelain stream.
fn native_pairs(repo: &Path) -> (Vec<(String, String)>, Vec<(String, String)>) {
    let stdout = git_porcelain_z(repo);
    let mut tokens: Vec<&[u8]> = stdout.split(|byte| *byte == 0).collect();
    if tokens.last().is_some_and(|token| token.is_empty()) {
        tokens.pop();
    }
    let mut entries = Vec::new();
    let mut renames = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let token = tokens[index];
        index += 1;
        if token.starts_with(b"## ") {
            continue;
        }
        let code = String::from_utf8_lossy(&token[..2]).into_owned();
        let path = String::from_utf8_lossy(&token[3..]).into_owned();
        if code.starts_with('R') || code.starts_with('C') {
            let source = String::from_utf8_lossy(tokens[index]).into_owned();
            index += 1;
            renames.push((source, path.clone()));
        }
        entries.push((code, path));
    }
    (entries, renames)
}

#[test]
fn audit_082_clean_repository_reports_clean() {
    let (repo, state) = new_repo("clean");
    let data = tethers_git_status(&repo, &state);
    assert_eq!(data["clean"], true);
    assert_eq!(data["conflict"], false);
    assert!(strings(&data["staged_paths"]).is_empty());
    assert!(strings(&data["unstaged_paths"]).is_empty());
    assert!(strings(&data["untracked_paths"]).is_empty());
    assert!(data["renamed_paths"].as_array().unwrap().is_empty());
    assert_eq!(data["branch"].as_str().unwrap(), "main");
    let _ = std::fs::remove_dir_all(repo.parent().unwrap());
}

#[test]
fn audit_082_modified_staged_untracked_classification_matches_native_git() {
    let (repo, state) = new_repo("classify");
    std::fs::write(repo.join("base.txt"), "modified\n").unwrap();
    std::fs::write(repo.join("staged.txt"), "staged\n").unwrap();
    git(&repo, &["add", "staged.txt"]);
    std::fs::write(repo.join("untracked file.txt"), "untracked\n").unwrap();

    let data = tethers_git_status(&repo, &state);
    assert_eq!(data["clean"], false);
    assert_eq!(data["conflict"], false);
    assert_eq!(strings(&data["staged_paths"]), vec!["staged.txt"]);
    assert_eq!(strings(&data["unstaged_paths"]), vec!["base.txt"]);
    assert_eq!(
        strings(&data["untracked_paths"]),
        vec!["untracked file.txt"]
    );

    let (entries, _) = native_pairs(&repo);
    assert!(entries.contains(&("A ".to_owned(), "staged.txt".to_owned())));
    assert!(entries.contains(&(" M".to_owned(), "base.txt".to_owned())));
    assert!(entries.contains(&("??".to_owned(), "untracked file.txt".to_owned())));
    let _ = std::fs::remove_dir_all(repo.parent().unwrap());
}

#[test]
fn audit_082_staged_rename_reports_source_and_destination() {
    let (repo, state) = new_repo("rename");
    std::fs::write(repo.join("old name.txt"), "content\n").unwrap();
    git(&repo, &["add", "old name.txt"]);
    git(&repo, &["commit", "-m", "add"]);
    git(&repo, &["mv", "old name.txt", "new name.txt"]);

    let data = tethers_git_status(&repo, &state);
    let renamed = data["renamed_paths"].as_array().unwrap();
    assert_eq!(renamed.len(), 1, "renamed_paths: {renamed:?}");
    assert_eq!(renamed[0]["from"], "old name.txt");
    assert_eq!(renamed[0]["to"], "new name.txt");
    assert_eq!(strings(&data["staged_paths"]), vec!["new name.txt"]);
    assert!(strings(&data["unstaged_paths"]).is_empty());
    assert_eq!(data["conflict"], false);

    // Agreement with native Git.
    let (entries, renames) = native_pairs(&repo);
    assert_eq!(
        renames,
        vec![("old name.txt".to_owned(), "new name.txt".to_owned())]
    );
    assert!(entries.iter().any(|(code, _)| code.starts_with('R')));
    // The pre-0.8.2 parser pushed a corrupted fragment of the SOURCE token
    // into staged/unstaged; exactly one staged path may exist.
    assert_eq!(strings(&data["staged_paths"]).len(), 1);
    let _ = std::fs::remove_dir_all(repo.parent().unwrap());
}

#[test]
fn audit_082_rename_then_worktree_modification_is_both_staged_and_unstaged() {
    let (repo, state) = new_repo("rename-modified");
    std::fs::write(repo.join("a.txt"), "content\n").unwrap();
    git(&repo, &["add", "a.txt"]);
    git(&repo, &["commit", "-m", "add"]);
    git(&repo, &["mv", "a.txt", "b.txt"]);
    std::fs::write(repo.join("b.txt"), "changed after rename\n").unwrap();

    let data = tethers_git_status(&repo, &state);
    assert_eq!(strings(&data["staged_paths"]), vec!["b.txt"]);
    assert_eq!(strings(&data["unstaged_paths"]), vec!["b.txt"]);
    let renamed = data["renamed_paths"].as_array().unwrap();
    assert_eq!(renamed.len(), 1);
    assert_eq!(renamed[0]["from"], "a.txt");
    assert_eq!(renamed[0]["to"], "b.txt");
    let _ = std::fs::remove_dir_all(repo.parent().unwrap());
}

#[test]
fn audit_082_merge_conflict_is_reported_as_conflict() {
    let (repo, state) = new_repo("conflict");
    std::fs::write(repo.join("file.txt"), "line1\nline2\n").unwrap();
    git(&repo, &["add", "file.txt"]);
    git(&repo, &["commit", "-m", "base line"]);
    git(&repo, &["checkout", "-b", "conflict-branch"]);
    std::fs::write(repo.join("file.txt"), "branch\nline2\n").unwrap();
    git(&repo, &["commit", "-am", "branch edit"]);
    git(&repo, &["checkout", "main"]);
    std::fs::write(repo.join("file.txt"), "main\nline2\n").unwrap();
    git(&repo, &["commit", "-am", "main edit"]);
    git_allow_failure(&repo, &["merge", "conflict-branch"]);

    let data = tethers_git_status(&repo, &state);
    assert_eq!(
        data["conflict"], true,
        "conflict must never be hard-coded false"
    );
    assert_eq!(strings(&data["conflicted_paths"]), vec!["file.txt"]);
    // Unmerged entries must not leak into staged/unstaged.
    assert!(!strings(&data["staged_paths"]).contains(&"file.txt".to_owned()));
    assert!(!strings(&data["unstaged_paths"]).contains(&"file.txt".to_owned()));

    let (entries, _) = native_pairs(&repo);
    assert!(
        entries
            .iter()
            .any(|(code, path)| code == "UU" && path == "file.txt"),
        "native git must agree the path is unmerged: {entries:?}"
    );
    let _ = std::fs::remove_dir_all(repo.parent().unwrap());
}

#[test]
fn audit_082_unicode_paths_round_trip_without_corruption() {
    let (repo, state) = new_repo("unicode");
    let name = "\u{65e5}\u{672c}\u{8a9e} \u{1f389}.txt";
    std::fs::write(repo.join(name), "unicode\n").unwrap();
    git(&repo, &["add", name]);

    let data = tethers_git_status(&repo, &state);
    assert_eq!(strings(&data["staged_paths"]), vec![name]);

    git(&repo, &["commit", "-m", "unicode"]);
    git(&repo, &["mv", name, "renamed \u{65e5}.txt"]);
    let data = tethers_git_status(&repo, &state);
    let renamed = data["renamed_paths"].as_array().unwrap();
    assert_eq!(renamed.len(), 1);
    assert_eq!(renamed[0]["from"], name);
    assert_eq!(renamed[0]["to"], "renamed \u{65e5}.txt");
    let _ = std::fs::remove_dir_all(repo.parent().unwrap());
}

#[test]
fn audit_082_detached_head_reports_native_header() {
    let (repo, state) = new_repo("detached");
    git(&repo, &["checkout", "--detach"]);
    let data = tethers_git_status(&repo, &state);
    let branch = data["branch"].as_str().expect("branch header must exist");
    assert!(
        branch.contains("HEAD"),
        "detached HEAD header must be reported verbatim: {branch}"
    );
    assert_eq!(data["clean"], true);
    let _ = std::fs::remove_dir_all(repo.parent().unwrap());
}
