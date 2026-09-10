use std::sync::Arc;

use serde::Serialize;
use wow_core::{
    CanonicalResult, CapabilityId, ContentDigest, CoverageId, CoveragePartitionId, CoverageRecord,
    CoverageStatus, GenerationContext, GenerationContextBuilder, MessageCode, NotEvaluatedId,
    NotEvaluatedRecord, ProducerId, ProducerVersionEntry, ProjectGenerationId, ToolVersion,
};
use wow_emmy::{
    EmmyLocalBindingFact, EmmyLocalFlowReport, EmmyMemberCallReport, EmmyMemberReferenceFact,
    EmmySyntaxDiagnostic, EmmySyntaxReport,
};

use crate::configuration::PROJECT_SNAPSHOT_SCHEMA_VERSION;
use crate::identity::{canonical_digest, canonical_id};
use crate::{
    ProjectAnalyzerBinding, ProjectConfiguration, ProjectError, ProjectErrorCode, ProjectFileId,
    ProjectFileRecord, ProjectGenerationCandidate, ProjectPhase, ProjectResult,
    ProjectSourceRegistry,
};

/// Published project snapshot state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectPublicationStatus {
    Published,
}

/// Typed deferred capability record retaining the requested capability identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDeferredCapability {
    capability_id: CapabilityId,
    record: NotEvaluatedRecord,
}

impl ProjectDeferredCapability {
    #[must_use]
    pub const fn capability_id(&self) -> &CapabilityId {
        &self.capability_id
    }

    #[must_use]
    pub const fn record(&self) -> &NotEvaluatedRecord {
        &self.record
    }
}

/// Immutable, content-addressed project publication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSnapshot {
    snapshot_id: Box<str>,
    canonical_snapshot_digest: ContentDigest<CanonicalResult>,
    project_generation: ProjectGenerationId,
    configuration: ProjectConfiguration,
    generation_candidate: ProjectGenerationCandidate,
    input_inventory: crate::ProjectInputInventory,
    generation_context: GenerationContext,
    source_registry: ProjectSourceRegistry,
    analyzer_binding: ProjectAnalyzerBinding,
    project_coverage_records: Vec<CoverageRecord>,
    deferred_capabilities: Vec<ProjectDeferredCapability>,
    publication_status: ProjectPublicationStatus,
}

impl ProjectSnapshot {
    pub(crate) fn build(
        configuration: ProjectConfiguration,
        generation_candidate: ProjectGenerationCandidate,
        input_inventory: crate::ProjectInputInventory,
        source_registry: ProjectSourceRegistry,
        analyzer_binding: ProjectAnalyzerBinding,
    ) -> ProjectResult<Self> {
        configuration.validate()?;
        generation_candidate.validate(&configuration, &input_inventory)?;
        if generation_candidate.project_generation() != source_registry.project_generation()
            || generation_candidate.project_generation() != analyzer_binding.project_generation()
            || configuration.configuration_digest()
                != generation_candidate.project_configuration_digest()
        {
            return Err(ProjectError::new(
                ProjectErrorCode::SnapshotInvalid,
                ProjectPhase::Snapshot,
                "project generation, configuration, registry, and analyzer binding disagree",
            )
            .with_candidate_generation(generation_candidate.project_generation()));
        }
        source_registry.validate()?;
        let producer_id = project_producer_id()?;
        let producer_version = project_producer_version()?;
        let generation_context = GenerationContextBuilder::new(
            configuration.selected_profile().clone(),
            configuration.reference_generation(),
        )
        .project_generation(generation_candidate.project_generation())
        .producer_versions(vec![ProducerVersionEntry::new(
            producer_id.clone(),
            producer_version.clone(),
        )])
        .build()
        .map_err(|source| {
            ProjectError::new(
                ProjectErrorCode::SnapshotInvalid,
                ProjectPhase::Snapshot,
                format!("project generation context cannot be built: {source}"),
            )
            .with_candidate_generation(generation_candidate.project_generation())
        })?;
        let project_coverage_records = build_project_coverage(
            &configuration,
            &generation_context,
            &source_registry,
            &analyzer_binding,
            &producer_id,
            &producer_version,
        )?;
        let deferred_capabilities = build_deferred_capabilities(
            &configuration,
            &generation_context,
            &producer_id,
            &producer_version,
        )?;
        let (canonical_snapshot_digest, snapshot_id) = derive_snapshot_identity(
            &configuration,
            &generation_candidate,
            &generation_context,
            &source_registry,
            &analyzer_binding,
            &project_coverage_records,
            &deferred_capabilities,
        )?;
        let snapshot = Self {
            snapshot_id,
            canonical_snapshot_digest,
            project_generation: generation_candidate.project_generation(),
            configuration,
            generation_candidate,
            input_inventory,
            generation_context,
            source_registry,
            analyzer_binding,
            project_coverage_records,
            deferred_capabilities,
            publication_status: ProjectPublicationStatus::Published,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    pub fn validate(&self) -> ProjectResult<()> {
        self.configuration.validate()?;
        self.generation_candidate
            .validate(&self.configuration, &self.input_inventory)?;
        self.generation_context.validate().map_err(|source| {
            ProjectError::new(
                ProjectErrorCode::SnapshotInvalid,
                ProjectPhase::Snapshot,
                format!("project generation context is invalid: {source}"),
            )
        })?;
        if self.generation_context.project_generation() != Some(self.project_generation)
            || self.source_registry.project_generation() != self.project_generation
            || self.analyzer_binding.project_generation() != self.project_generation
            || self.publication_status != ProjectPublicationStatus::Published
        {
            return Err(ProjectError::new(
                ProjectErrorCode::SnapshotInvalid,
                ProjectPhase::Snapshot,
                "published project snapshot contains mixed generation state",
            ));
        }
        self.source_registry.validate()?;
        if self.source_registry.file_records().len() != self.input_inventory.files().len() {
            return Err(ProjectError::new(
                ProjectErrorCode::SnapshotInvalid,
                ProjectPhase::Snapshot,
                "published source registry differs from retained project inventory",
            ));
        }
        for input in self.input_inventory.files() {
            let Some(record) = self.source_registry.file_by_id(input.file_id()) else {
                return Err(ProjectError::new(
                    ProjectErrorCode::SnapshotInvalid,
                    ProjectPhase::Snapshot,
                    "published source registry is missing a retained project input",
                )
                .with_file_id(input.file_id().as_str()));
            };
            if record.content_digest() != input.content_digest()
                || record.byte_length() != input.byte_length()
                || record.relative_path() != input.relative_path()
            {
                return Err(ProjectError::new(
                    ProjectErrorCode::SnapshotInvalid,
                    ProjectPhase::Snapshot,
                    "published file identity differs from retained project input",
                )
                .with_file_id(input.file_id().as_str()));
            }
        }
        for record in &self.project_coverage_records {
            record.validate().map_err(core_snapshot_error)?;
            if record.context_id() != self.generation_context.context_id() {
                return Err(ProjectError::new(
                    ProjectErrorCode::SnapshotInvalid,
                    ProjectPhase::Snapshot,
                    "project coverage record belongs to another generation context",
                ));
            }
        }
        for deferred in &self.deferred_capabilities {
            deferred.record.validate().map_err(core_snapshot_error)?;
            if deferred.record.context_id() != self.generation_context.context_id() {
                return Err(ProjectError::new(
                    ProjectErrorCode::SnapshotInvalid,
                    ProjectPhase::Snapshot,
                    "deferred capability record belongs to another generation context",
                ));
            }
        }
        let (digest, snapshot_id) = derive_snapshot_identity(
            &self.configuration,
            &self.generation_candidate,
            &self.generation_context,
            &self.source_registry,
            &self.analyzer_binding,
            &self.project_coverage_records,
            &self.deferred_capabilities,
        )?;
        if digest != self.canonical_snapshot_digest
            || snapshot_id.as_ref() != self.snapshot_id.as_ref()
        {
            return Err(ProjectError::new(
                ProjectErrorCode::SnapshotDigestMismatch,
                ProjectPhase::Snapshot,
                "published project snapshot identity does not match its canonical fields",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }

    #[must_use]
    pub const fn canonical_snapshot_digest(&self) -> ContentDigest<CanonicalResult> {
        self.canonical_snapshot_digest
    }

    #[must_use]
    pub const fn project_generation(&self) -> ProjectGenerationId {
        self.project_generation
    }

    #[must_use]
    pub const fn configuration(&self) -> &ProjectConfiguration {
        &self.configuration
    }

    #[must_use]
    pub const fn generation_candidate(&self) -> &ProjectGenerationCandidate {
        &self.generation_candidate
    }

    #[must_use]
    pub const fn generation_context(&self) -> &GenerationContext {
        &self.generation_context
    }

    #[must_use]
    pub const fn source_registry(&self) -> &ProjectSourceRegistry {
        &self.source_registry
    }

    #[must_use]
    pub const fn analyzer_binding(&self) -> &ProjectAnalyzerBinding {
        &self.analyzer_binding
    }

    #[must_use]
    pub fn file_manifest(&self) -> &[ProjectFileRecord] {
        self.source_registry.file_records()
    }

    #[must_use]
    pub fn project_coverage_records(&self) -> &[CoverageRecord] {
        &self.project_coverage_records
    }

    #[must_use]
    pub fn deferred_capabilities(&self) -> &[ProjectDeferredCapability] {
        &self.deferred_capabilities
    }

    #[must_use]
    pub const fn publication_status(&self) -> ProjectPublicationStatus {
        self.publication_status
    }

    #[must_use]
    pub fn open_view(self: &Arc<Self>) -> ProjectView {
        ProjectView {
            snapshot: Arc::clone(self),
        }
    }
}

/// Immutable read surface over one exact snapshot.
#[derive(Debug, Clone)]
pub struct ProjectView {
    snapshot: Arc<ProjectSnapshot>,
}

impl ProjectView {
    #[must_use]
    pub fn snapshot(&self) -> &Arc<ProjectSnapshot> {
        &self.snapshot
    }

    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        self.snapshot.snapshot_id()
    }

    #[must_use]
    pub fn project_generation(&self) -> ProjectGenerationId {
        self.snapshot.project_generation()
    }

    #[must_use]
    pub fn configuration(&self) -> &ProjectConfiguration {
        self.snapshot.configuration()
    }

    #[must_use]
    pub fn file_manifest(&self) -> &[ProjectFileRecord] {
        self.snapshot.file_manifest()
    }

    #[must_use]
    pub fn file_by_id(&self, file_id: &ProjectFileId) -> Option<&ProjectFileRecord> {
        self.snapshot.source_registry().file_by_id(file_id)
    }

    pub fn file_by_path(&self, path: &str) -> ProjectResult<Option<&ProjectFileRecord>> {
        self.snapshot.source_registry().file_by_path(path)
    }

    #[must_use]
    pub fn analyzer_snapshot_id(&self) -> &str {
        self.snapshot.analyzer_binding().analyzer_snapshot_id()
    }

    #[must_use]
    pub fn syntax_report(&self) -> &EmmySyntaxReport {
        self.snapshot.analyzer_binding().syntax_report()
    }

    #[must_use]
    pub fn member_call_report(&self) -> &EmmyMemberCallReport {
        self.snapshot.analyzer_binding().member_call_report()
    }

    #[must_use]
    pub fn local_flow_report(&self) -> &EmmyLocalFlowReport {
        self.snapshot.analyzer_binding().local_flow_report()
    }

    #[must_use]
    pub fn generic_diagnostics_for_file(
        &self,
        file_id: &ProjectFileId,
    ) -> Vec<&EmmySyntaxDiagnostic> {
        let Some(record) = self.file_by_id(file_id) else {
            return Vec::new();
        };
        self.syntax_report()
            .diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.path() == record.relative_path().as_str())
            .collect()
    }

    #[must_use]
    pub fn member_references_for_file(
        &self,
        file_id: &ProjectFileId,
    ) -> Vec<&EmmyMemberReferenceFact> {
        let Some(record) = self.file_by_id(file_id) else {
            return Vec::new();
        };
        self.member_call_report()
            .references()
            .iter()
            .filter(|fact| fact.path() == record.relative_path().as_str())
            .collect()
    }

    #[must_use]
    pub fn local_bindings_for_file(&self, file_id: &ProjectFileId) -> Vec<&EmmyLocalBindingFact> {
        let Some(record) = self.file_by_id(file_id) else {
            return Vec::new();
        };
        self.local_flow_report()
            .bindings()
            .iter()
            .filter(|fact| fact.path() == record.relative_path().as_str())
            .collect()
    }

    #[must_use]
    pub fn project_coverage_records(&self) -> &[CoverageRecord] {
        self.snapshot.project_coverage_records()
    }

    #[must_use]
    pub fn deferred_capabilities(&self) -> &[ProjectDeferredCapability] {
        self.snapshot.deferred_capabilities()
    }
}

fn build_project_coverage(
    configuration: &ProjectConfiguration,
    context: &GenerationContext,
    registry: &ProjectSourceRegistry,
    analyzer: &ProjectAnalyzerBinding,
    producer_id: &ProducerId,
    producer_version: &ToolVersion,
) -> ProjectResult<Vec<CoverageRecord>> {
    let mut records = vec![
        complete_coverage(
            context,
            "project.fixture.configuration.valid",
            CoveragePartitionId::new("project.configuration", None).map_err(core_snapshot_error)?,
            producer_id,
            producer_version,
        )?,
        complete_coverage(
            context,
            "project.fixture.files.complete",
            CoveragePartitionId::new(
                "project.workspace",
                Some(configuration.workspace_id().as_str()),
            )
            .map_err(core_snapshot_error)?,
            producer_id,
            producer_version,
        )?,
        complete_coverage(
            context,
            "project.source.registry.complete",
            CoveragePartitionId::new(
                "project.source_origin",
                Some(configuration.source_origin_id().as_str()),
            )
            .map_err(core_snapshot_error)?,
            producer_id,
            producer_version,
        )?,
        complete_coverage(
            context,
            "project.source.handle.resolve",
            CoveragePartitionId::new("project.source_registry", Some(registry.registry_id()))
                .map_err(core_snapshot_error)?,
            producer_id,
            producer_version,
        )?,
    ];
    let project_generation_key = context
        .project_generation()
        .ok_or_else(|| {
            ProjectError::new(
                ProjectErrorCode::SnapshotInvalid,
                ProjectPhase::Snapshot,
                "project coverage requires an exact project generation",
            )
        })?
        .canonical();
    records.push(complete_coverage(
        context,
        "project.generation.coherent",
        CoveragePartitionId::new("project.generation", Some(&project_generation_key))
            .map_err(core_snapshot_error)?,
        producer_id,
        producer_version,
    )?);
    records.push(complete_coverage(
        context,
        "project.analyzer.snapshot.available",
        CoveragePartitionId::new("project.analyzer", Some(analyzer.analyzer_snapshot_id()))
            .map_err(core_snapshot_error)?,
        producer_id,
        producer_version,
    )?);
    for file in registry.file_records() {
        let facts_complete = analyzer.file_facts_complete(file.file_id());
        records.push(coverage_for_state(
            context,
            configuration,
            "project.analyzer.facts.available",
            CoveragePartitionId::new("project.file", Some(file.file_id().as_str()))
                .map_err(core_snapshot_error)?,
            facts_complete,
            producer_id,
            producer_version,
        )?);
        records.push(complete_coverage(
            context,
            "project.analyzer.generic_diagnostics.available",
            CoveragePartitionId::new("project.file.diagnostics", Some(file.file_id().as_str()))
                .map_err(core_snapshot_error)?,
            producer_id,
            producer_version,
        )?);
    }
    records.sort_by(|left, right| {
        left.capability_id()
            .cmp(right.capability_id())
            .then(
                left.partition_id()
                    .canonical()
                    .cmp(&right.partition_id().canonical()),
            )
            .then(left.coverage_id().cmp(&right.coverage_id()))
    });
    Ok(records)
}

fn complete_coverage(
    context: &GenerationContext,
    capability: &str,
    partition: CoveragePartitionId,
    producer_id: &ProducerId,
    producer_version: &ToolVersion,
) -> ProjectResult<CoverageRecord> {
    CoverageRecord::new(
        context.context_id(),
        parse_capability(capability)?,
        partition,
        CoverageStatus::Complete,
        producer_id.clone(),
        producer_version.clone(),
        Vec::new(),
        None,
        Vec::new(),
        Vec::new(),
    )
    .map_err(core_snapshot_error)
}

#[allow(clippy::too_many_arguments)]
fn coverage_for_state(
    context: &GenerationContext,
    configuration: &ProjectConfiguration,
    capability: &str,
    partition: CoveragePartitionId,
    complete: bool,
    producer_id: &ProducerId,
    producer_version: &ToolVersion,
) -> ProjectResult<CoverageRecord> {
    let capability_id = parse_capability(capability)?;
    if complete {
        return CoverageRecord::new(
            context.context_id(),
            capability_id,
            partition,
            CoverageStatus::Complete,
            producer_id.clone(),
            producer_version.clone(),
            Vec::new(),
            None,
            Vec::new(),
            Vec::new(),
        )
        .map_err(core_snapshot_error);
    }
    if !configuration
        .capability_policy()
        .is_degradable(&capability_id)
    {
        return Err(ProjectError::new(
            ProjectErrorCode::MandatoryCapabilityUnavailable,
            ProjectPhase::Snapshot,
            format!("project capability {capability_id} is unavailable and not degradable"),
        ));
    }
    CoverageRecord::new(
        context.context_id(),
        capability_id,
        partition,
        CoverageStatus::Failed,
        producer_id.clone(),
        producer_version.clone(),
        Vec::new(),
        Some(parse_message_code("analyzer_file_facts_unavailable")?),
        Vec::new(),
        Vec::new(),
    )
    .map_err(core_snapshot_error)
}

fn build_deferred_capabilities(
    configuration: &ProjectConfiguration,
    context: &GenerationContext,
    producer_id: &ProducerId,
    producer_version: &ToolVersion,
) -> ProjectResult<Vec<ProjectDeferredCapability>> {
    let reason = parse_message_code("operation_not_implemented_for_milestone")?;
    let mut deferred = Vec::new();
    for capability in configuration.capability_policy().explicitly_deferred() {
        let record = NotEvaluatedRecord::new(
            context.context_id(),
            producer_id.clone(),
            producer_version.clone(),
            "capability",
            capability.as_str(),
            reason.clone(),
            vec![capability.clone()],
            Vec::new(),
            Vec::new(),
        )
        .map_err(core_snapshot_error)?;
        deferred.push(ProjectDeferredCapability {
            capability_id: capability.clone(),
            record,
        });
    }
    deferred.sort_by(|left, right| left.capability_id.cmp(&right.capability_id));
    Ok(deferred)
}

#[allow(clippy::too_many_arguments)]
fn derive_snapshot_identity(
    configuration: &ProjectConfiguration,
    generation_candidate: &ProjectGenerationCandidate,
    generation_context: &GenerationContext,
    registry: &ProjectSourceRegistry,
    analyzer: &ProjectAnalyzerBinding,
    coverage: &[CoverageRecord],
    deferred: &[ProjectDeferredCapability],
) -> ProjectResult<(ContentDigest<CanonicalResult>, Box<str>)> {
    #[derive(Serialize)]
    struct AnalyzerIdentity<'a> {
        analyzer_snapshot_id: &'a str,
        analyzer_configuration_digest: ContentDigest<CanonicalResult>,
        main_workspace_id: &'a str,
        library_snapshot_ids: Vec<&'a str>,
        syntax_analysis_id: &'a str,
        member_call_analysis_id: &'a str,
        local_flow_analysis_id: &'a str,
    }
    let coverage_ids = coverage
        .iter()
        .map(CoverageRecord::coverage_id)
        .collect::<Vec<CoverageId>>();
    let deferred_ids = deferred
        .iter()
        .map(|entry| entry.record.not_evaluated_id())
        .collect::<Vec<NotEvaluatedId>>();
    #[derive(Serialize)]
    struct Identity<'a> {
        snapshot_schema_version: u64,
        project_generation: ProjectGenerationId,
        selected_profile: &'a wow_core::ProfileIdentity,
        reference_generation: wow_core::ReferenceGenerationId,
        project_configuration_digest: ContentDigest<CanonicalResult>,
        generation_context_id: wow_core::GenerationContextId,
        source_origin: &'a crate::ProjectSourceOrigin,
        source_registry_id: &'a str,
        source_registry_digest: ContentDigest<CanonicalResult>,
        file_manifest: &'a [ProjectFileRecord],
        analyzer: AnalyzerIdentity<'a>,
        coverage_ids: &'a [CoverageId],
        deferred_ids: &'a [NotEvaluatedId],
        publication_status: ProjectPublicationStatus,
    }
    let analyzer_identity = AnalyzerIdentity {
        analyzer_snapshot_id: analyzer.analyzer_snapshot_id(),
        analyzer_configuration_digest: analyzer.analyzer_configuration_digest(),
        main_workspace_id: analyzer.main_workspace().snapshot_id(),
        library_snapshot_ids: analyzer.library_snapshot_ids().collect(),
        syntax_analysis_id: analyzer.syntax_report().analysis_id(),
        member_call_analysis_id: analyzer.member_call_report().analysis_id(),
        local_flow_analysis_id: analyzer.local_flow_report().analysis_id(),
    };
    let identity = Identity {
        snapshot_schema_version: PROJECT_SNAPSHOT_SCHEMA_VERSION,
        project_generation: generation_candidate.project_generation(),
        selected_profile: configuration.selected_profile(),
        reference_generation: configuration.reference_generation(),
        project_configuration_digest: configuration.configuration_digest(),
        generation_context_id: generation_context.context_id(),
        source_origin: registry.source_origin(),
        source_registry_id: registry.registry_id(),
        source_registry_digest: registry.canonical_digest(),
        file_manifest: registry.file_records(),
        analyzer: analyzer_identity,
        coverage_ids: &coverage_ids,
        deferred_ids: &deferred_ids,
        publication_status: ProjectPublicationStatus::Published,
    };
    let digest = canonical_digest(
        "wow-project/snapshot/e0-d/1",
        &identity,
        ProjectPhase::Snapshot,
    )?;
    let snapshot_id = canonical_id(
        "project-snapshot:sha256:",
        "wow-project/snapshot-id/e0-d/1",
        &identity,
        ProjectPhase::Snapshot,
    )?;
    Ok((digest, snapshot_id))
}

fn project_producer_id() -> ProjectResult<ProducerId> {
    "wow.project".parse().map_err(core_snapshot_error)
}

fn project_producer_version() -> ProjectResult<ToolVersion> {
    "0.1.0".parse().map_err(core_snapshot_error)
}

fn parse_capability(value: &str) -> ProjectResult<CapabilityId> {
    value.parse().map_err(core_snapshot_error)
}

fn parse_message_code(value: &str) -> ProjectResult<MessageCode> {
    value.parse().map_err(core_snapshot_error)
}

fn core_snapshot_error(source: wow_core::CoreError) -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SnapshotInvalid,
        ProjectPhase::Snapshot,
        format!("project snapshot core invariant failed: {source}"),
    )
}
