//! Plug-neutral operational-scope evidence.
//!
//! Wraps either a File Tools or PDF operational scope so enablement evidence
//! remains generic and does not depend on one Plug's specific root layout.
//! The untagged Serde representation preserves backward compatibility: an
//! existing File Tools `OperationalScopeBinding` serialises as the identical
//! JSON object, without a variant tag or wrapper field.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OperationalScope {
    FileTools(crate::file_tools::OperationalScopeBinding),
    Pdf(crate::pdf_tools::PdfOperationalScopeBinding),
}

impl OperationalScope {
    pub fn installed_id(&self) -> &str {
        match self {
            OperationalScope::FileTools(s) => &s.installed_id,
            OperationalScope::Pdf(s) => &s.installed_id,
        }
    }

    pub fn capability_name(&self) -> &str {
        match self {
            OperationalScope::FileTools(s) => &s.capability_name,
            OperationalScope::Pdf(s) => &s.capability_name,
        }
    }

    pub fn capability_version(&self) -> u32 {
        match self {
            OperationalScope::FileTools(s) => s.capability_version,
            OperationalScope::Pdf(s) => s.capability_version,
        }
    }

    pub fn integrity_digest(&self) -> &str {
        match self {
            OperationalScope::FileTools(s) => &s.integrity_digest,
            OperationalScope::Pdf(s) => &s.integrity_digest,
        }
    }

    pub fn validate(&self) -> crate::m3_store::Result<()> {
        match self {
            OperationalScope::FileTools(s) => s
                .validate()
                .map_err(|e| crate::m3_store::M3Error::new(e.code, e.message)),
            OperationalScope::Pdf(s) => s
                .validate()
                .map_err(|e| crate::m3_store::M3Error::new(e.code, e.message)),
        }
    }
}

impl From<crate::file_tools::OperationalScopeBinding> for OperationalScope {
    fn from(binding: crate::file_tools::OperationalScopeBinding) -> Self {
        OperationalScope::FileTools(binding)
    }
}

impl From<crate::pdf_tools::PdfOperationalScopeBinding> for OperationalScope {
    fn from(binding: crate::pdf_tools::PdfOperationalScopeBinding) -> Self {
        OperationalScope::Pdf(binding)
    }
}
