#[cfg(target_os = "macos")]
use std::path::Path;

/// Accepts only Darwin's standard root aliases, and only while they still
/// resolve to their documented private targets. Callers must continue checking
/// the rest of the path chain.
#[cfg(target_os = "macos")]
pub(crate) fn is_macos_system_path_alias(path: &Path) -> bool {
    let expected = match path {
        path if path == Path::new("/var") => Path::new("/private/var"),
        path if path == Path::new("/tmp") => Path::new("/private/tmp"),
        path if path == Path::new("/etc") => Path::new("/private/etc"),
        _ => return false,
    };

    std::fs::canonicalize(path).is_ok_and(|canonical| canonical == expected)
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn only_standard_darwin_root_aliases_with_expected_targets_are_accepted() {
        for (alias, target) in [
            (Path::new("/var"), Path::new("/private/var")),
            (Path::new("/tmp"), Path::new("/private/tmp")),
            (Path::new("/etc"), Path::new("/private/etc")),
        ] {
            if std::fs::symlink_metadata(alias)
                .is_ok_and(|metadata| metadata.file_type().is_symlink())
            {
                assert!(is_macos_system_path_alias(alias));
                assert_eq!(std::fs::canonicalize(alias).unwrap(), target);
            }
        }

        let root =
            std::env::temp_dir().join(format!("tethers-alias-test-{}", uuid::Uuid::new_v4()));
        let target = root.join("target");
        let link = root.join("link");
        std::fs::create_dir_all(&target).unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(!is_macos_system_path_alias(&link));
        std::fs::remove_dir_all(root).unwrap();
    }
}
