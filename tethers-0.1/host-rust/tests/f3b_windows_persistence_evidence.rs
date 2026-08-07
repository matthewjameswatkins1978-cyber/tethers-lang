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
// Evidence labels separated into three properties:
//
//   a) flush operation accepted/succeeded           — PROVEN (F3b)
//   b) exact bytes survive close/reopen              — PROVEN (F3b)
//   c) file data survives sudden power loss          — UNVERIFIED (F3b)
//
// Atomic-visibility labels:
//
//   d) final path absent before rename               — PROVEN (F3b)
//   e) final path complete bytes after rename        — PROVEN (F3b)
//   f) atomic visibility during rename (concurrent)  — UNVERIFIED (F3b)

#[test]
fn sync_all_rename_flush_accepted() {
    let dir = temp_test_dir("sync-flush-accepted");
    let tmp = dir.join(".record.tmp");

    {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .expect("create temp file");
        f.write_all(CANARY).expect("write canary bytes");
        f.sync_all().expect("sync_all on temp file");
    }
    // Property (a): flush operation accepted/succeeded — PROVEN (F3b)
    // sync_all() returned Ok(()).

    remove_test_dir(&dir);
}

#[test]
fn sync_all_rename_bytes_survive_close_and_reopen() {
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
    // Property (b): exact bytes survive close/reopen — PROVEN (F3b)
    let reopened = fs::read(&final_path).expect("reopen and read");
    assert_eq!(reopened, CANARY);

    remove_test_dir(&dir);
}

#[test]
fn sync_all_rename_final_absent_before_rename() {
    let dir = temp_test_dir("absent-before");
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

        // Property (d): final path absent before rename — PROVEN (F3b)
        assert!(!final_path.exists(), "final must not exist before rename");
    }
    fs::rename(&tmp, &final_path).expect("rename");

    // Property (e): final path complete bytes after rename — PROVEN (F3b)
    assert!(final_path.exists());
    assert_eq!(fs::read(&final_path).expect("read"), CANARY);

    // Property (f): atomic visibility during rename (concurrent observer)
    // UNVERIFIED (F3b). No concurrent observer is present in this test.

    remove_test_dir(&dir);
}

#[test]
fn sync_all_rename_temporary_disappears_after_rename() {
    let dir = temp_test_dir("tmp-disappears");
    let tmp = dir.join(".temp.tmp");
    let final_path = dir.join("temp.json");

    {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .expect("create");
        f.write_all(CANARY).expect("write");
        f.sync_all().expect("sync");
    }
    fs::rename(&tmp, &final_path).expect("rename");

    assert!(
        !tmp.exists(),
        "temporary path no longer exists after rename"
    );

    remove_test_dir(&dir);
}

#[test]
fn sync_all_rename_multiple_records_independent() {
    let dir = temp_test_dir("multi-record");

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
    let dir = temp_test_dir("stale-tmp");
    let tmp = dir.join(".stale.tmp");
    let dst = dir.join("stale.json");

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
    if rename_result.is_err() {
        assert!(tmp.exists(), "tmp remains after failed rename");
        assert_eq!(fs::read(&tmp).expect("read tmp"), CANARY);
    }
    if rename_result.is_ok() {
        assert!(!tmp.exists());
    }

    remove_test_dir(&dir);
}

// ---------------------------------------------------------------------------
// F3b-2: Parent-directory durability feasibility investigation
// ---------------------------------------------------------------------------
//
// Directly assert the two operations we claim are feasible.
// If either fails, the test FAILS — the finding is UNVERIFIED for this target.

#[test]
fn parent_directory_flush_feasibility() {
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
        use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
        use windows_sys::Win32::Storage::FileSystem::{
            CreateFileW, FlushFileBuffers, FILE_FLAG_BACKUP_SEMANTICS, FILE_GENERIC_WRITE,
            FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
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

        // Must directly assert: directory handle open succeeded on this target.
        // If this fails, the finding is UNVERIFIED.
        assert!(
            handle != INVALID_HANDLE_VALUE && handle != std::ptr::null_mut(),
            "F3b-2: CreateFileW(dir, FILE_GENERIC_WRITE | FILE_FLAG_BACKUP_SEMANTICS) \
             opened the directory handle. \
             If this assertion fails, directory flush access is not available \
             on this Windows target and the entire F3b-2 route is UNVERIFIED."
        );

        let flush_ok = FlushFileBuffers(handle);
        CloseHandle(handle);

        // Must directly assert: FlushFileBuffers succeeded.
        // If this fails, directory flush is not supported on this volume/handle.
        assert_ne!(
            flush_ok, 0,
            "F3b-2: FlushFileBuffers(directory_handle) succeeded on this primary target. \
             Windows accepted the flush on this opened directory handle. \
             This proves: FlushFileBuffers is technically feasible. \
             This does NOT prove directory-entry durability after power loss — \
             that depends on volume write-cache behaviour and is UNVERIFIED (F3b)."
        );
    }

    remove_test_dir(&dir);
}

// ---------------------------------------------------------------------------
// F3b-5: Local Anchor root reparse-point safety characterization
// ---------------------------------------------------------------------------

#[test]
fn local_anchor_reparse_point_can_redirect_writes() {
    use std::os::windows::process::CommandExt;

    let parent = temp_test_dir("reparse-parent");
    let real_root = parent.join("real-admission-store");
    let junction_root = parent.join("junction-admission-store");

    fs::create_dir(&real_root).expect("create real root");

    let create_result = std::process::Command::new("cmd")
        .args(&[
            "/C",
            "mklink",
            "/J",
            junction_root.to_str().expect("valid path"),
            real_root.to_str().expect("valid path"),
        ])
        .creation_flags(0x08000000)
        .output();

    match create_result {
        Ok(out) if out.status.success() => {
            let via_junction = junction_root.join("a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0.json");
            fs::write(&via_junction, b"admitted-through-junction").expect("write through junction");

            let expected_in_real = real_root.join("a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0.json");
            assert!(
                expected_in_real.exists(),
                "F3b-5 DISPROVEN: a reparse point on the store root directory \
                 can redirect admission writes. File written through junction \
                 landed at {:?} despite SHA-256 hashed filenames. \
                 The Local Anchor root has no verify_chain()/reject_reparse().",
                expected_in_real
            );
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            eprintln!(
                "F3b-5: could not create junction ({}): {}. \
                 Root reparse-point safety UNVERIFIED (F3b) — \
                 mklink /J requires administrator or developer-mode privileges. \
                 Re-test with elevated privileges.",
                out.status,
                stderr.trim()
            );
            // Do not fail — the test tooling limitation prevents
            // characterization; record UNVERIFIED, not DISPROVEN.
        }
        Err(e) => {
            eprintln!("F3b-5: could not spawn mklink: {}. UNVERIFIED (F3b).", e);
        }
    }

    remove_test_dir(&parent);
}
