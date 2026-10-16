//! Native initial binding and data-only validation for captured source results.
use super::*;

/// Only an exact native Project acquisition can create this initial binding.
/// Canonical bytes remain Search data and cannot recreate a Project owner.
pub struct BoundObservationQueryResult {
    bytes: Box<[u8]>,
    project_kind: wow_project::ProjectKind,
    platform_profile: Option<wow_project::platform_source::BlizzardUiSourceProfile>,
    platform_binding: Option<wow_project::platform_source::PlatformPackageBinding>,
}
impl BoundObservationQueryResult {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub(crate) const fn project_kind(&self) -> wow_project::ProjectKind {
        self.project_kind
    }
    pub(crate) fn platform_proof(
        &self,
    ) -> Option<(
        &wow_project::platform_source::BlizzardUiSourceProfile,
        &wow_project::platform_source::PlatformPackageBinding,
    )> {
        Some((
            self.platform_profile.as_ref()?,
            self.platform_binding.as_ref()?,
        ))
    }
}
impl ObservationSearchShard {
    pub fn retrieve_bound(
        &self,
        read: &ReadSnapshot,
        pair: &AcquiredProjectPair,
        request: &ObservationQueryRequest,
        stop: &AtomicBool,
    ) -> SearchResult<BoundObservationQueryResult> {
        check(stop)?;
        let profile = if self.includes_function_declarations() {
            ObservationSearchProfile::with_function_declarations(self.bounds)?
        } else {
            ObservationSearchProfile::new(self.bounds)?
        };
        let (_, expected) = capture_manifest(read, pair, &profile, stop)?;
        if expected != self.physical.reader().owner_manifest_bytes() {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let result = self.retrieve(request, stop)?;
        let mut payload: serde_json::Value = serde_json::from_slice(result.canonical_bytes())
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        let object = payload
            .as_object_mut()
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let id = object
            .remove("result_set_id")
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        object.insert(
            "universe_id".into(),
            self.federation_universe_id()?.to_string().into(),
        );
        object.insert(
            "result_query_id".into(),
            object
                .get("query_id")
                .cloned()
                .ok_or(SearchError::OwnerEvidenceInvalid)?,
        );
        let bytes = wow_core::canonical_json_bytes(
            &serde_json::json!({"result_set_id":id,"payload":payload}),
        )
        .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        check(stop)?;
        let configuration = pair.project().configuration();
        Ok(BoundObservationQueryResult {
            bytes: bytes.into_boxed_slice(),
            project_kind: configuration.project_kind(),
            platform_profile: configuration
                .platform_packages()
                .map(|p| p.source().profile().clone()),
            platform_binding: configuration.platform_package_binding().cloned(),
        })
    }
    fn federation_universe_id(&self) -> SearchResult<Box<str>> {
        identity(
            "source-observation-search-universe:",
            &(&self.manifest.owner.source, self.profile_digest()),
        )
    }
    pub(crate) fn validate_federation_summary(
        &self,
        summary: &serde_json::Value,
        stop: &AtomicBool,
    ) -> SearchResult<()> {
        check(stop)?;
        let field = |key| summary.get(key).ok_or(SearchError::OwnerEvidenceInvalid);
        let request = ObservationQueryRequest::from_json_slice(
            &wow_core::canonical_json_bytes(field("request")?)
                .map_err(|_| SearchError::OwnerEvidenceInvalid)?,
            stop,
        )?;
        let declarations = self.includes_function_declarations();
        let profile = query::profile(declarations);
        let query_id = identity(
            "source-observation-query:",
            &(profile, self.logical_id(), &request),
        )?;
        let expected = serde_json::json!({"schema":if declarations {"wow-search/source-observation-declaration-result-set/1"} else {"wow-search/source-observation-result-set/1"},
            "query_profile":profile,"query_id":query_id,"result_query_id":query_id,"universe_id":self.federation_universe_id()?,
            "shard_id":self.logical_id(),"profile_digest":self.profile_digest(),"request":request,
            "owner":self.manifest.owner,"project_kind":self.manifest.project_kind,
            "local_analysis_id":self.manifest.local_analysis_id,"member_analysis_id":self.manifest.member_analysis_id,"main_snapshot_id":self.manifest.main_snapshot_id,
            "local_file_coverage":self.manifest.local_file_coverage,"member_file_coverage":self.manifest.member_file_coverage,
            "absence_authority":"not_evaluated","continuation":"not_supported",
            "nonclaims":["canonical_name","alias","runtime_target","platform_availability","complete_source_fact_coverage"]});
        for (key, value) in expected
            .as_object()
            .ok_or(SearchError::OwnerEvidenceInvalid)?
        {
            if field(key)? != value {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
        }
        for (key, expected) in [
            (
                "platform_source_profile",
                self.manifest.platform_source_profile.as_ref(),
            ),
            (
                "platform_package_binding",
                self.manifest.platform_package_binding.as_ref(),
            ),
        ] {
            if summary.get(key) != expected {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
        }
        let coverage = self
            .manifest
            .function_names
            .as_ref()
            .map(value)
            .transpose()?;
        if summary.get("function_names") != coverage.as_ref() || summary.get("candidates").is_some()
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        request.validate_capture_summary(summary, self.manifest.documents.len(), declarations)?;
        Ok(())
    }
    pub(crate) fn validate_federation_candidate(
        &self,
        query_id: &str,
        candidate: &serde_json::Value,
        stop: &AtomicBool,
    ) -> SearchResult<()> {
        check(stop)?;
        let key = candidate
            .get("stable_tie_key")
            .and_then(serde_json::Value::as_str)
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let index = self
            .manifest
            .documents
            .binary_search_by(|d| d.fact_id().cmp(key))
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        let document = &self.manifest.documents[index];
        if candidate.get("document") != Some(&value(document)?)
            || candidate
                .get("candidate_id")
                .and_then(serde_json::Value::as_str)
                != Some(
                    identity(
                        "source-observation-candidate:",
                        &(query_id, document.fact_id()),
                    )?
                    .as_ref(),
                )
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        Ok(())
    }
}
