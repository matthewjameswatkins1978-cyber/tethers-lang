use crate::candidate::CandidateRecord;
use crate::installation_trust::ExactCandidateTrustStore;
use crate::m3_store::{M3Error, Result};
use crate::trust::{
    DeveloperApprovalStore, PackageTrustEvidence, PublisherTrustStore, TrustModeEvidence,
};

pub(crate) trait CurrentTrustAuthority {
    fn revalidate_current(
        &self,
        candidate: &CandidateRecord,
        evidence: &PackageTrustEvidence,
        now_unix_ms: u64,
    ) -> Result<()>;
}

pub(crate) struct PublisherDeveloperTrustAuthority<'a> {
    publisher_trust: &'a PublisherTrustStore,
    developer_approvals: &'a DeveloperApprovalStore,
}

impl<'a> PublisherDeveloperTrustAuthority<'a> {
    pub(crate) fn new(
        publisher_trust: &'a PublisherTrustStore,
        developer_approvals: &'a DeveloperApprovalStore,
    ) -> Self {
        Self {
            publisher_trust,
            developer_approvals,
        }
    }
}

impl CurrentTrustAuthority for PublisherDeveloperTrustAuthority<'_> {
    fn revalidate_current(
        &self,
        candidate: &CandidateRecord,
        evidence: &PackageTrustEvidence,
        now_unix_ms: u64,
    ) -> Result<()> {
        evidence.revalidate_current(
            &candidate.package_id,
            self.publisher_trust,
            self.developer_approvals,
            now_unix_ms,
        )
    }
}

pub(crate) struct ExactCandidateTrustAuthority<'a> {
    exact_trust: &'a ExactCandidateTrustStore,
}

impl<'a> ExactCandidateTrustAuthority<'a> {
    pub(crate) fn new(exact_trust: &'a ExactCandidateTrustStore) -> Self {
        Self { exact_trust }
    }
}

impl CurrentTrustAuthority for ExactCandidateTrustAuthority<'_> {
    fn revalidate_current(
        &self,
        candidate: &CandidateRecord,
        evidence: &PackageTrustEvidence,
        _now_unix_ms: u64,
    ) -> Result<()> {
        candidate
            .validate()
            .map_err(|error| M3Error::new("candidate_invalid", error.message))?;
        evidence.require_for_candidate(candidate)?;
        let (
            candidate_id,
            candidate_record_digest,
            installation_trust_record_digest,
            approving_authority,
        ) = match &evidence.mode {
            TrustModeEvidence::ExactCandidate {
                candidate_id,
                candidate_record_digest,
                installation_trust_record_digest,
                approving_authority,
            } => (
                candidate_id,
                candidate_record_digest,
                installation_trust_record_digest,
                approving_authority,
            ),
            _ => {
                return Err(M3Error::new(
                    "trust_exact_candidate_authority_required",
                    "exact-candidate trust requires current installation-trust authority",
                ));
            }
        };
        let record = self.exact_trust.find(candidate_id)?.ok_or_else(|| {
            M3Error::new(
                "trust_drift",
                "exact-candidate installation trust is absent",
            )
        })?;
        record.require_for_candidate(candidate)?;
        if candidate_id != &record.candidate_id
            || candidate_record_digest != &record.candidate_record_digest
            || installation_trust_record_digest != &record.record_digest
            || approving_authority != &record.approving_authority
        {
            return Err(M3Error::new(
                "trust_drift",
                "exact-candidate installation trust changed",
            ));
        }
        let current = PackageTrustEvidence::exact_candidate(&record)?;
        if current != *evidence {
            return Err(M3Error::new(
                "trust_drift",
                "exact-candidate installation trust changed",
            ));
        }
        Ok(())
    }
}
