#![cfg(windows)]

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const CANARY: &[u8] = b"f3b-sync-rename-canary-v1";

fn temp_test_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("f3b-{}-{}", label, uuid::Uuid::new_v4()));
    match fs::create_dir(&dir) {
        Ok(()) => dir,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            fs::remove_dir_all(&dir).expect("cleanup stale dir");
            fs::create_dir(&dir).expect("create fresh dir");
            dir
        }
        Err(e) => panic!("cannot create test dir {}: {}", dir.display(), e),
    }
}

fn remove_test_dir(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

// ---------------------------------------------------------------------------
// F3b-1: sync_all() + fs::rename  characterization
// ---------------------------------------------------------------------------
//
// These tests characterize the write-sync-rename primitive used by
// StoreRoot (m3_store.rs), candidate.rs write_new(), and local_anchor.rs
// atomic_create().  They answer what can be directly observed on the
// primary Windows target, separating ordinary-execution observable
// behaviour from power-loss / crash durability.
//
// Properties tested:
//   1. temporary file is fully written
//   2. sync_all() succeeds
//   3. rename succeeds
//   4. final path contains the complete expected bytes
//   5. temporary path disappears
//   6. no partial final file is exposed during ordinary execution
//   7. restart/reopen reads the exact expected bytes

#[test]
fn sync_all_rename_full_cycle_observed() {
    // --- arranged: isolated temporary directory ---
    let dir = temp_test_dir("sync-rename-cycle");
    let tmp = dir.join(".record.tmp");
    let final_path = dir.join("record.json");

    // --- act: write, sync, rename ---
    {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .expect("create temp file");
        f.write_all(CANARY).expect("write canary bytes");
        f.sync_all().expect("sync_all on temp file");
    }
    fs::rename(&tmp, &final_path).expect("rename temp to final");

    // --- assert ---

    // (1) temporary file was fully written — proved by the absence of
    //     write_all error and because the bytes we later read match
    // (4) final path contains the complete expected bytes
    let bytes = fs::read(&final_path).expect("read final file");
    assert_eq!(bytes, CANARY, "final file bytes match");

    // (5) temporary path disappears
    assert!(!tmp.exists(), "temporary path no longer exists");

    // (6) no partial final file — the only operations between open and
    //     rename are write_all + sync_all; the file was not readable
    //     under the final name before rename succeeded.
    // (7) reopen reads exact bytes — verified above via fs::read

    // cleanup
    remove_test_dir(&dir);
}

#[test]
fn sync_all_rename_survives_close_and_reopen() {
    let dir = temp_test_dir("survives-reopen");
    let tmp = dir.join(".entry.tmp");
    let final_path = dir.join("entry.json");

    {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .expect("create temp");
        f.write_all(CANARY).expect("write");
        f.sync_all().expect("sync");
    }
    fs::rename(&tmp, &final_path).expect("rename");

    // Simulate "restart": drop all handles, reopen from scratch.
    // This corresponds to action (7): restart/reopen reads exact bytes.
    let reopened = fs::read(&final_path).expect("reopen and read");
    assert_eq!(reopened, CANARY);

    remove_test_dir(&dir);
}

#[test]
fn sync_all_rename_no_partial_file_visible() {
    // During ordinary execution the temporary file is written with
    // create_new, so it cannot be opened by another handle before
    // write+sync completes.  The rename of a complete file is the
    // first moment the final name becomes visible.
    //
    // We prove the negative: before rename the final name does not
    // exist, and after rename the bytes are complete.
    let dir = temp_test_dir("no-partial");
    let tmp = dir.join(".staging.tmp");
    let final_path = dir.join("staging.json");

    {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .expect("create");
        f.write_all(CANARY).expect("write");
        f.sync_all().expect("sync");

        // before rename: final must not exist
        assert!(!final_path.exists(), "final must not exist before rename");
    }
    fs::rename(&tmp, &final_path).expect("rename");

    // after rename: final must be present and complete
    assert!(final_path.exists());
    assert_eq!(fs::read(&final_path).expect("read"), CANARY);

    remove_test_dir(&dir);
}

#[test]
fn sync_all_rename_multiple_records_independent() {
    // Each record is independent — one failed sync or rename should
    // not affect a previously published record.
    let dir = temp_test_dir("multi-record");

    // First record
    let t1 = dir.join(".a.tmp");
    let d1 = dir.join("a.json");
    {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&t1)
            .expect("create");
        f.write_all(b"first").expect("write");
        f.sync_all().expect("sync");
    }
    fs::rename(&t1, &d1).expect("rename 1");

    // Second record
    let t2 = dir.join(".b.tmp");
    let d2 = dir.join("b.json");
    {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&t2)
            .expect("create");
        f.write_all(b"second").expect("write");
        f.sync_all().expect("sync");
    }
    fs::rename(&t2, &d2).expect("rename 2");

    assert_eq!(fs::read(&d1).expect("read1"), b"first");
    assert_eq!(fs::read(&d2).expect("read2"), b"second");
    assert!(!t1.exists());
    assert!(!t2.exists());

    remove_test_dir(&dir);
}

#[test]
fn sync_all_stale_tmp_visible_after_failure() {
    // When rename fails (e.g. destination exists), the temporary
    // file remains.  This is normal: a stale .tmp is evidence
    // of an incomplete publish, and the recovery reader must
    // handle it.  We are not testing recovery behaviour here —
    // only that the tmp is observable.
    let dir = temp_test_dir("stale-tmp");
    let tmp = dir.join(".stale.tmp");
    let dst = dir.join("stale.json");

    // Create the destination first (simulating a pre-existing record)
    fs::write(&dst, b"pre-existing").expect("pre-write");

    {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .expect("create");
        f.write_all(CANARY).expect("write");
        f.sync_all().expect("sync");
    }
    let rename_result = fs::rename(&tmp, &dst);
    // On Windows, rename typically replaces unless restricted.
    // We record the actual behaviour rather than asserting.
    if rename_result.is_err() {
        // tmp should still exist with its bytes intact
        assert!(tmp.exists(), "tmp remains after failed rename");
        assert_eq!(fs::read(&tmp).expect("read tmp"), CANARY);
    }
    // If rename succeeded, tmp should not exist.
    if rename_result.is_ok() {
        assert!(!tmp.exists());
    }

    remove_test_dir(&dir);
}

// ---------------------------------------------------------------------------
// F3b-2: Parent-directory durability feasibility investigation
// ---------------------------------------------------------------------------

#[test]
fn parent_directory_flush_feasibility() {
    // Investigate whether we can open a directory with sufficient
    // access to call FlushFileBuffers on it, then verify the result.
    //
    // Windows CreateFileW with FILE_FLAG_BACKUP_SEMANTICS permits
    // opening directories.  GENERIC_WRITE + FILE_FLAG_BACKUP_SEMANTICS
    // should let us flush the directory handle metadata.
    //
    // We test: open dir -> call FlushFileBuffers -> observe result.
    // This is a feasibility probe, not a durability proof.

    use std::os::windows::ffi::OsStrExt;

    let dir = temp_test_dir("dir-flush");
    let file = dir.join("test.txt");
    fs::write(&file, b"test").expect("write test file");

    let dir_wide: Vec<u16> = dir
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();

    unsafe {
        use windows_sys::Win32::Foundation::HANDLE;
        use windows_sys::Win32::Storage::FileSystem::{
            CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_GENERIC_WRITE, FILE_SHARE_READ,
            FILE_SHARE_WRITE, OPEN_EXISTING,
        };

        let handle: HANDLE = CreateFileW(
            dir_wide.as_ptr(),
            FILE_GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            std::ptr::null_mut(),
        );

        use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
        if handle == INVALID_HANDLE_VALUE || handle == std::ptr::null_mut() {
            // Windows refused to open the directory for write.
            // This is expected on some configurations.
            // Record: directory flush access NOT available.
            eprintln!(
                "F3b-2: CreateFileW(dir, FILE_GENERIC_WRITE) returned {:?} — \
                 directory flush access not available via this path",
                if handle == INVALID_HANDLE_VALUE {
                    "INVALID_HANDLE_VALUE"
                } else {
                    "NULL"
                }
            );
            remove_test_dir(&dir);
            return; // expected; this is a characterization, not a requirement
        }

        // SAFETY: handle is a valid directory handle.
        use windows_sys::Win32::Storage::FileSystem::FlushFileBuffers;
        let flush_ok = FlushFileBuffers(handle);

        // SAFETY: handle must be closed regardless of flush outcome.
        use windows_sys::Win32::Foundation::CloseHandle;
        CloseHandle(handle);

        if flush_ok != 0 {
            // FlushFileBuffers on the directory handle succeeded.
            // This proves the OS accepted the flush request.
            // It does NOT prove the directory entry is durable —
            // only that the OS accepted the operation.
            eprintln!(
                "F3b-2: FlushFileBuffers(directory_handle) succeeded — \
                 Windows accepted the flush. Full durability depends on \
                 volume configuration and write-cache behaviour."
            );
        } else {
            eprintln!(
                "F3b-2: FlushFileBuffers(directory_handle) failed — \
                 directory flush not supported on this volume or handle."
            );
        }
    }

    remove_test_dir(&dir);
}

// ---------------------------------------------------------------------------
// F3b-5: Local Anchor root reparse-point safety characterization
// ---------------------------------------------------------------------------

#[test]
fn local_anchor_reparse_point_can_redirect_writes() {
    // Characterize whether a reparse point (junction) on the
    // admission store root path can redirect writes despite
    // SHA-256 hashed safe filenames.
    //
    // The Local Anchor Admission Store uses safe_filename()
    // (SHA-256 hash) to prevent traversal in individual filenames.
    // However, if the store root itself is a reparse point, new
    // files written under it will land at the junction target.
    //
    // This test proves whether that exposure exists.

    use std::os::windows::process::CommandExt;

    let parent = temp_test_dir("reparse-parent");
    let real_root = parent.join("real-admission-store");
    let junction_root = parent.join("junction-admission-store");

    fs::create_dir(&real_root).expect("create real root");

    // Create a directory junction at junction_root pointing to real_root.
    let create_result = std::process::Command::new("cmd")
        .args(&[
            "/C",
            "mklink",
            "/J",
            junction_root.to_str().expect("valid path"),
            real_root.to_str().expect("valid path"),
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output();

    match create_result {
        Ok(out) if out.status.success() => {
            // junction created — now write through it
            let via_junction = junction_root.join("a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0.json");
            fs::write(&via_junction, b"admitted-through-junction").expect("write through junction");

            // The file should appear in real_root, not junction_root directly
            let expected_in_real = real_root.join("a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0.json");
            assert!(
                expected_in_real.exists(),
                "F3b-5 CONFIRMED DEFECT: reparse point on store root \
                 redirects admission writes. File written through junction \
                 landed at {:?}",
                expected_in_real
            );
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            // mklink requires admin or developer mode on some Windows
            // versions.  Record the limitation but do not fail the
            // characterization — the inability to test is itself data.
            eprintln!(
                "F3b-5: could not create junction ({}): {}. \
                 Re-test with administrator privileges.",
                out.status,
                stderr.trim()
            );
        }
        Err(e) => {
            eprintln!("F3b-5: could not spawn mklink: {}", e);
        }
    }

    remove_test_dir(&parent);
}
