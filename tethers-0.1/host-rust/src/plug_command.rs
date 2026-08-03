use crate::cli::{CliEnvelope, OutcomeStatus};
use crate::package;
use serde_json::json;
use std::path::Path;

pub struct PlugCommandResult {
    pub envelope: CliEnvelope,
    pub exit_code: i32,
}

pub fn run_inspect(package_path: &Path) -> PlugCommandResult {
    match package::inspect(package_path) {
        Ok(report) => {
            let envelope = CliEnvelope::ok("plug inspect", json!({ "inspection": report }));
            PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            }
        }
        Err(error) => {
            let status = if error.code == "archive_read" {
                OutcomeStatus::Unavailable
            } else {
                OutcomeStatus::InvalidData
            };
            let envelope =
                CliEnvelope::error("plug inspect", status, error.code, error.message, None);
            PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j24a_invalid_extension_maps_to_invalid_data() {
        let result = run_inspect(Path::new("not-a-package.zip"));
        assert_eq!(result.exit_code, 3);
        assert_eq!(result.envelope.status, OutcomeStatus::InvalidData);
        assert_eq!(
            result.envelope.error.as_ref().unwrap().code,
            "invalid_archive"
        );
    }

    #[test]
    fn j24a_missing_package_maps_to_unavailable() {
        let result = run_inspect(Path::new("missing.tetherplug"));
        assert_eq!(result.exit_code, 4);
        assert_eq!(result.envelope.status, OutcomeStatus::Unavailable);
        assert_eq!(result.envelope.error.as_ref().unwrap().code, "archive_read");
    }
}
