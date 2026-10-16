//! Recorded local declarations and direct member use sites from one native Project pair.
//! These data projections confer neither canonical API names nor runtime target authority.
use super::*;
use wow_core::{SourceHandle, SourceSpan};

mod federation;
mod query;
pub use federation::BoundObservationQueryResult;
pub use query::{ObservationQueryRequest, ObservationQueryResult};
const PROFILE: &str = "wow-search/project-source-observation-profile/1";
const MANIFEST: &str = "wow-search/project-source-observation-shard/1";

#[derive(Debug, Clone)]
pub struct ObservationSearchProfile {
    bounds: SearchBounds,
    digest: Box<str>,
    declarations: bool,
}
impl ObservationSearchProfile {
    pub fn new(bounds: SearchBounds) -> SearchResult<Self> {
        bounds.physical()?;
        Ok(Self {
            bounds,
            declarations: false,
            digest: identity(
                "",
                &(
                    PROFILE,
                    MANIFEST,
                    "ProjectView.local_flow_report.bindings/member_call_report.references",
                    "recorded-case-sensitive-name-and-receiver-syntax-only",
                    FTS_CANDIDATE_PROFILE,
                    "local_metadata_only",
                    bounds,
                ),
            )?,
        })
    }
    pub fn digest(&self) -> &str {
        &self.digest
    }
    pub fn with_function_declarations(bounds: SearchBounds) -> SearchResult<Self> {
        let mut profile = Self::new(bounds)?;
        profile.digest = identity(
            "",
            &(
                "wow-search/project-source-observation-declarations-profile/1",
                &profile.digest,
                "native-named-function-binding-syntax/1;qualified-short-name-and-member",
                "immutable-native-custody-only",
            ),
        )?;
        profile.declarations = true;
        Ok(profile)
    }
    pub fn includes_function_declarations(&self) -> bool {
        self.declarations
    }
    pub fn bounds(&self) -> SearchBounds {
        self.bounds
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    LocalBinding,
    MemberReference,
    FunctionDeclaration,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationDocument {
    fact_id: Box<str>,
    kind: Kind,
    file_id: ProjectFileId,
    path: Box<str>,
    content_sha256: Box<str>,
    recorded_name: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recorded_short_name: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recorded_receiver: Option<Box<str>>,
    source_handle: SourceHandle,
    // Original serialized owner evidence, never deserialized into a native fact.
    owner_fact: serde_json::Value,
}
impl ObservationDocument {
    pub fn fact_id(&self) -> &str {
        &self.fact_id
    }
    pub fn recorded_name(&self) -> &str {
        &self.recorded_name
    }
    pub fn kind(&self) -> &'static str {
        match self.kind {
            Kind::LocalBinding => "local_binding",
            Kind::MemberReference => "member_reference",
            Kind::FunctionDeclaration => "function_declaration",
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ObservationManifest {
    schema: Box<str>,
    profile_digest: Box<str>,
    owner: OwnerManifestEvidence,
    project_kind: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform_source_profile: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform_package_binding: Option<serde_json::Value>,
    local_analysis_id: Box<str>,
    member_analysis_id: Box<str>,
    main_snapshot_id: Box<str>,
    local_file_coverage: Vec<serde_json::Value>,
    member_file_coverage: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_names: Option<FunctionNameCoverage>,
    documents: Vec<ObservationDocument>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FunctionNameCoverage {
    producer_profile: Box<str>,
    function_analysis_id: Box<str>,
    source_health_complete: bool,
    states: std::collections::BTreeMap<String, u32>,
    files: Vec<serde_json::Value>,
}
pub struct PreparedObservationShard {
    manifest: ObservationManifest,
    physical: PreparedSearchShard,
}
impl PreparedObservationShard {
    pub fn logical_id(&self) -> &str {
        self.physical.search_shard_id()
    }
    pub fn object_id(&self) -> &ObjectId {
        self.physical.object_id()
    }
    pub fn source_generation(&self) -> &str {
        &self.manifest.owner.source.store_generation_id
    }
    pub fn profile_digest(&self) -> &str {
        &self.manifest.profile_digest
    }
    pub fn documents(&self) -> &[ObservationDocument] {
        &self.manifest.documents
    }
    pub fn into_object(self) -> PendingObject {
        self.physical.into_object()
    }
}

pub fn prepare_source_observation_shard(
    read: &ReadSnapshot,
    generation: &StoreGenerationId,
    profile: &ObservationSearchProfile,
    permission: LocalMetadataPermission,
    object_limits: wow_store::StoreLimits,
    stop: &AtomicBool,
) -> SearchResult<PreparedObservationShard> {
    check(stop)?;
    if !permission.authorized {
        return Err(SearchError::LocalMetadataPermissionRequired);
    }
    if &read.manifest().generation_id != generation {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let pair = AcquiredProjectPair::read(read, stop)?;
    let (manifest, bytes) = capture_manifest(read, &pair, profile, stop)?;
    let logical_id = format!("search-shard:sha256:{}", hex(&Sha256::digest(&bytes)));
    let rows = manifest
        .documents
        .iter()
        .map(|d| SearchStoreDocument::new(d.fact_id(), d.recorded_name()))
        .collect::<Vec<_>>();
    let physical = SearchStore::prepare(
        SearchStoreProfile::new(profile.digest.to_string(), profile.bounds.physical()?)?,
        logical_id,
        &rows,
        &bytes,
        object_limits,
        stop,
    )?;
    Ok(PreparedObservationShard { manifest, physical })
}

fn capture_manifest(
    read: &ReadSnapshot,
    pair: &AcquiredProjectPair,
    profile: &ObservationSearchProfile,
    stop: &AtomicBool,
) -> SearchResult<(ObservationManifest, Vec<u8>)> {
    check(stop)?;
    let project = pair.project();
    let local = project.local_flow_report();
    let member = project.member_call_report();
    let report = if profile.declarations {
        Some(
            project
                .function_call_report()
                .ok_or(SearchError::OwnerEvidenceInvalid)?,
        )
    } else {
        None
    };
    let mut function_names = report.map(|r| FunctionNameCoverage {
        producer_profile: "wow-emmy/named-function-binding-syntax/1".into(),
        function_analysis_id: r.analysis_id().into(),
        source_health_complete: r.source_health_complete(),
        states: std::collections::BTreeMap::new(),
        files: Vec::new(),
    });
    let mut named_count = 0usize;
    if let (Some(report), Some(coverage)) = (report, &mut function_names) {
        if report.main_snapshot_id() != local.main_snapshot_id() {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        for fact in report.functions() {
            check(stop)?;
            // Census genuine owner facts before allocating per-document custody
            // handles. The source join is mandatory during capture below.
            let state = value(&fact.binding_name_state())?
                .as_str()
                .ok_or(SearchError::OwnerEvidenceInvalid)?
                .to_owned();
            let count = coverage.states.entry(state).or_default();
            *count = count
                .checked_add(1)
                .ok_or(SearchError::FileSetOverCapacity)?;
            named_count = named_count
                .checked_add(usize::from(fact.binding_name().is_some()))
                .ok_or(SearchError::FileSetOverCapacity)?;
        }
    }
    // Census the whole selected corpus before copying facts or preparing physical storage.
    let count = local
        .bindings()
        .len()
        .checked_add(member.references().len())
        .and_then(|n| n.checked_add(named_count))
        .ok_or(SearchError::FileSetOverCapacity)?;
    if count > profile.bounds.max_documents.get() as usize {
        return Err(SearchError::FileSetOverCapacity);
    }
    if count == 0 {
        return Err(SearchError::FileSetEmpty);
    }
    if local
        .files()
        .len()
        .checked_add(member.files().len())
        .is_none_or(|n| n > profile.bounds.max_owner_evidence_records.get() as usize)
    {
        return Err(SearchError::FileSetBudgetExceeded);
    }
    if local.main_snapshot_id() != member.main_snapshot_id()
        || local.member_call_analysis_id() != member.analysis_id()
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let mut manifest = ObservationManifest {
        schema: MANIFEST.into(),
        profile_digest: profile.digest.clone(),
        owner: owner_manifest_evidence(read, pair, profile.bounds, stop)?,
        project_kind: value(&project.configuration().project_kind())?,
        platform_source_profile: project
            .configuration()
            .platform_packages()
            .map(|p| value(p.source().profile()))
            .transpose()?,
        platform_package_binding: project
            .configuration()
            .platform_package_binding()
            .map(value)
            .transpose()?,
        local_analysis_id: local.analysis_id().into(),
        member_analysis_id: member.analysis_id().into(),
        main_snapshot_id: local.main_snapshot_id().into(),
        local_file_coverage: Vec::new(),
        member_file_coverage: Vec::new(),
        function_names,
        documents: Vec::new(),
    };
    let evidence_count = manifest.owner.graph_partitions.iter().try_fold(
        manifest
            .owner
            .project_coverage_and_conflicts
            .len()
            .checked_add(manifest.owner.graph_coverage.len())
            .and_then(|n| n.checked_add(local.files().len()))
            .and_then(|n| n.checked_add(member.files().len()))
            .and_then(|n| n.checked_add(report.map_or(0, |r| r.files().len())))
            .ok_or(SearchError::FileSetBudgetExceeded)?,
        |n, partition| {
            n.checked_add(1)
                .and_then(|n| n.checked_add(partition.coverage.len()))
                .ok_or(SearchError::FileSetBudgetExceeded)
        },
    )?;
    if evidence_count > profile.bounds.max_owner_evidence_records.get() as usize {
        return Err(SearchError::FileSetBudgetExceeded);
    }
    let mut charged = size(&manifest)?;
    charge(&mut charged, 0, profile.bounds)?;
    if let (Some(report), Some(coverage)) = (report, &mut manifest.function_names) {
        for file in report.files() {
            check(stop)?;
            let file = value(file)?;
            charge(
                &mut charged,
                size(&file)? + u64::from(!coverage.files.is_empty()),
                profile.bounds,
            )?;
            coverage.files.push(file);
        }
    }
    for report in local.files() {
        check(stop)?;
        let report = value(report)?;
        charge(
            &mut charged,
            size(&report)? + u64::from(!manifest.local_file_coverage.is_empty()),
            profile.bounds,
        )?;
        manifest.local_file_coverage.push(report);
    }
    for report in member.files() {
        check(stop)?;
        let report = value(report)?;
        charge(
            &mut charged,
            size(&report)? + u64::from(!manifest.member_file_coverage.is_empty()),
            profile.bounds,
        )?;
        manifest.member_file_coverage.push(report);
    }
    let mut capture = |fact_id: &str,
                       kind: Kind,
                       path: &str,
                       sha: &str,
                       name: &str,
                       short_name: Option<&str>,
                       receiver: Option<&str>,
                       span: SourceSpan,
                       fact: serde_json::Value|
     -> SearchResult<()> {
        check(stop)?;
        if fact_id.len() > profile.bounds.max_document_id_bytes.get() as usize
            || name.len() > profile.bounds.max_owner_metadata_label_bytes.get() as usize
            || short_name.is_some_and(|n| {
                n.len() > profile.bounds.max_owner_metadata_label_bytes.get() as usize
            })
            || receiver.is_some_and(|r| {
                r.len() > profile.bounds.max_owner_metadata_label_bytes.get() as usize
            })
        {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        let file = project
            .file_by_path(path)?
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        if file.content_digest().to_string() != sha
            || span
                .byte_start()
                .zip(span.byte_end())
                .is_none_or(|(start, end)| start > end || end > file.byte_length())
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let doc = ObservationDocument {
            fact_id: fact_id.into(),
            kind,
            file_id: file.file_id().clone(),
            path: path.into(),
            content_sha256: sha.into(),
            recorded_name: name.into(),
            recorded_short_name: short_name.map(Into::into),
            recorded_receiver: receiver.map(Into::into),
            source_handle: project.source_handle(path, span, None)?,
            owner_fact: fact,
        };
        charge(
            &mut charged,
            size(&doc)? + u64::from(!manifest.documents.is_empty()),
            profile.bounds,
        )?;
        manifest.documents.push(doc);
        Ok(())
    };
    for fact in local.bindings() {
        capture(
            fact.fact_id(),
            Kind::LocalBinding,
            fact.path(),
            fact.content_sha256(),
            fact.name(),
            None,
            None,
            fact.declaration_span(),
            value(fact)?,
        )?;
    }
    for fact in member.references() {
        capture(
            fact.fact_id(),
            Kind::MemberReference,
            fact.path(),
            fact.content_sha256(),
            fact.member(),
            None,
            Some(fact.receiver()),
            fact.reference_span(),
            value(fact)?,
        )?;
    }
    if let Some(report) = report {
        for fact in report.functions() {
            check(stop)?;
            let metadata = project
                .function_binding_metadata(fact.fact_id())?
                .ok_or(SearchError::OwnerEvidenceInvalid)?;
            if let Some(name) = metadata.binding_name() {
                let evidence = serde_json::json!({"function_id": metadata.fact().fact_id(), "binding_name":name,
                    "function_fact_span":fact.span(), "function_content_digest":fact.content_digest(),
                    "function_source_handle":metadata.source_handle(),
                    "scope":metadata.scope(),
                    "function_analysis_id":metadata.function_analysis_id()});
                capture(
                    fact.fact_id(),
                    Kind::FunctionDeclaration,
                    fact.path(),
                    fact.content_digest(),
                    name.qualified_name(),
                    Some(name.short_name()),
                    name.receiver(),
                    name.name_span(),
                    evidence,
                )?;
            }
        }
    }
    manifest.documents.sort_by(|a, b| a.fact_id.cmp(&b.fact_id));
    if manifest
        .documents
        .windows(2)
        .any(|p| p[0].fact_id == p[1].fact_id)
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let bytes =
        wow_core::canonical_json_bytes(&manifest).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    if bytes.len() as u64 != charged {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    Ok((manifest, bytes))
}

pub struct ObservationSearchShard {
    manifest: ObservationManifest,
    physical: PublishedSearchShard,
    bounds: SearchBounds,
}
impl ObservationSearchShard {
    pub fn includes_function_declarations(&self) -> bool {
        self.manifest.function_names.is_some()
    }
    pub fn logical_id(&self) -> &str {
        self.physical.search_shard_id()
    }
    pub fn object_id(&self) -> &ObjectId {
        self.physical.object_id()
    }
    pub fn source_generation(&self) -> &str {
        &self.manifest.owner.source.store_generation_id
    }
    pub fn profile_digest(&self) -> &str {
        &self.manifest.profile_digest
    }
    pub fn release(self, store: &mut Store) -> SearchResult<()> {
        self.physical.release(store).map_err(SearchError::from)
    }
}
#[allow(clippy::too_many_arguments)]
pub fn open_source_observation_shard(
    store: &mut Store,
    logical_id: &str,
    object_id: &ObjectId,
    generation: &StoreGenerationId,
    profile: &ObservationSearchProfile,
    permission: LocalMetadataPermission,
    lease: SearchStoreLeaseRequest,
    stop: &AtomicBool,
) -> SearchResult<ObservationSearchShard> {
    check(stop)?;
    if !permission.authorized {
        return Err(SearchError::LocalMetadataPermissionRequired);
    }
    let physical = SearchStore::open_exact(
        store,
        SearchStoreProfile::new(profile.digest.to_string(), profile.bounds.physical()?)?,
        logical_id.to_owned(),
        object_id,
        lease,
        stop,
    )?;
    let result = (|| {
        let bytes = physical.reader().owner_manifest_bytes();
        let manifest: ObservationManifest =
            serde_json::from_slice(bytes).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        if manifest.schema.as_ref() != MANIFEST
            || manifest.profile_digest != profile.digest
            || manifest.function_names.is_some() != profile.declarations
            || manifest.owner.source.store_generation_id.as_ref() != generation.as_str()
            || wow_core::canonical_json_bytes(&manifest)
                .map_err(|_| SearchError::OwnerEvidenceInvalid)?
                != bytes
            || format!("search-shard:sha256:{}", hex(&Sha256::digest(bytes))) != logical_id
            || manifest.documents.len() != physical.reader().document_count() as usize
            || manifest.documents.is_empty()
            || manifest.documents.len() > profile.bounds.max_documents.get() as usize
            || !manifest
                .documents
                .windows(2)
                .all(|p| p[0].fact_id < p[1].fact_id)
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        for doc in &manifest.documents {
            check(stop)?;
            doc.source_handle
                .validate()
                .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
            if doc.source_handle.path().as_str() != doc.path.as_ref()
                || doc.source_handle.content_digest().to_string() != doc.content_sha256.as_ref()
            {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
            let row = physical
                .reader()
                .exact_id(doc.fact_id())?
                .ok_or(SearchError::OwnerEvidenceInvalid)?;
            if row.owner_metadata_label() != doc.recorded_name() {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
        }
        Ok(manifest)
    })();
    match result {
        Ok(manifest) => Ok(ObservationSearchShard {
            manifest,
            physical,
            bounds: profile.bounds,
        }),
        Err(error) => Err(release_after_owner_error(physical, store, error)),
    }
}
fn value<T: Serialize + ?Sized>(value: &T) -> SearchResult<serde_json::Value> {
    serde_json::to_value(value).map_err(|_| SearchError::OwnerEvidenceInvalid)
}
fn size<T: Serialize + ?Sized>(value: &T) -> SearchResult<u64> {
    wow_core::canonical_json_bytes(value)
        .map(|b| b.len() as u64)
        .map_err(|_| SearchError::OwnerEvidenceInvalid)
}
fn identity<T: Serialize + ?Sized>(prefix: &str, value: &T) -> SearchResult<Box<str>> {
    let bytes =
        wow_core::canonical_json_bytes(value).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    Ok(format!("{prefix}sha256:{}", hex(&Sha256::digest(bytes))).into())
}
fn charge(charged: &mut u64, amount: u64, bounds: SearchBounds) -> SearchResult<()> {
    *charged = charged
        .checked_add(amount)
        .ok_or(SearchError::FileSetBudgetExceeded)?;
    if *charged > bounds.max_owner_manifest_bytes.get() {
        return Err(SearchError::FileSetBudgetExceeded);
    }
    Ok(())
}
fn check(stop: &AtomicBool) -> SearchResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(SearchError::Cancelled)
    } else {
        Ok(())
    }
}
