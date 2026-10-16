//! Ordered, borrowed native owners. Serialized metadata cannot construct this proof.
use super::AcquiredProjectPair;
use crate::{ProjectError, ProjectErrorCode, ProjectKind, ProjectPhase, ProjectResult};
use serde::Serialize;
use std::sync::atomic::AtomicBool;
use wow_core::{CanonicalResult, ContentDigest, CoverageRecord, ProfileIdentity};

pub const RETAINED_GENERATION_PAIR_PROFILE: &str = "wow-project/retained-generation-pair/1";

pub struct RetainedGenerationPair<'a> {
    _before_custody: wow_store::project::ReadSnapshotCustody<'a>,
    _after_custody: wow_store::project::ReadSnapshotCustody<'a>,
    before: &'a AcquiredProjectPair,
    after: &'a AcquiredProjectPair,
    id: Box<str>,
    configuration_equal: bool,
    graph_registry_equal: bool,
}

impl<'a> RetainedGenerationPair<'a> {
    pub fn bind(
        before: &'a AcquiredProjectPair,
        after: &'a AcquiredProjectPair,
        before_custody: wow_store::project::ReadSnapshotCustody<'a>,
        after_custody: wow_store::project::ReadSnapshotCustody<'a>,
        stop: &AtomicBool,
    ) -> ProjectResult<Self> {
        crate::analyzer::checkpoint(stop)?;
        let left = before.project().configuration();
        let right = after.project().configuration();
        if !before.read_origin.matches(&before_custody)
            || !after.read_origin.matches(&after_custody)
            || !before_custody.same_open_owner(&after_custody)
            || before.read_epoch_id != after.read_epoch_id
            || left.project_id() != right.project_id()
            || left.project_kind() != right.project_kind()
            || left.workspace_id() != right.workspace_id()
            || left.source_origin_id() != right.source_origin_id()
            || left.logical_root() != right.logical_root()
            || before.graph().snapshot().universe() != after.graph().snapshot().universe()
        {
            return Err(ProjectError::new(
                ProjectErrorCode::IncompatibleGenerationPair,
                ProjectPhase::View,
                "retained generations have different logical project scope",
            ));
        }
        if left.project_kind() == ProjectKind::BlizzardUiPlatformSource {
            return Err(ProjectError::new(
                ProjectErrorCode::DeferredCapability,
                ProjectPhase::View,
                "this pair profile admits ordinary project publications only",
            ));
        }
        let digest = wow_core::domain_separated_digest(
            RETAINED_GENERATION_PAIR_PROFILE,
            &(
                (
                    &before.read_epoch_id,
                    &before.read_generation_id,
                    before.publication_set_id(),
                    before.project().snapshot_id(),
                    before.graph().snapshot().snapshot_id(),
                ),
                (
                    &after.read_epoch_id,
                    &after.read_generation_id,
                    after.publication_set_id(),
                    after.project().snapshot_id(),
                    after.graph().snapshot().snapshot_id(),
                ),
            ),
        )
        .map_err(|_| {
            ProjectError::new(
                ProjectErrorCode::CanonicalizationFailed,
                ProjectPhase::View,
                "generation pair identity could not be encoded",
            )
        })?;
        crate::analyzer::checkpoint(stop)?;
        Ok(Self {
            _before_custody: before_custody,
            _after_custody: after_custody,
            before,
            after,
            id: format!(
                "project-generation-pair:{}",
                ContentDigest::<CanonicalResult>::from_bytes(digest)
            )
            .into(),
            // These validated owner commitments compare configurations, never entities.
            configuration_equal: left.configuration_digest() == right.configuration_digest(),
            graph_registry_equal: before.graph().registry().bundle_id()
                == after.graph().registry().bundle_id()
                && before.graph().registry().registry_digest()
                    == after.graph().registry().registry_digest(),
        })
    }

    pub fn before(&self) -> &AcquiredProjectPair {
        self.before
    }
    pub fn after(&self) -> &AcquiredProjectPair {
        self.after
    }
    pub fn pair_id(&self) -> &str {
        &self.id
    }
    pub const fn exact_configuration_and_registry(&self) -> bool {
        self.configuration_equal && self.graph_registry_equal
    }
    pub const fn status(&self) -> &'static str {
        if self.configuration_equal && self.graph_registry_equal {
            "valid_partial"
        } else {
            "not_evaluated"
        }
    }
    pub fn descriptor(&self) -> RetainedGenerationPairDescriptor<'_> {
        RetainedGenerationPairDescriptor {
            schema: "wow-project/retained-generation-pair/1",
            profile_id: RETAINED_GENERATION_PAIR_PROFILE,
            pair_id: &self.id,
            status: self.status(),
            before: Binding::new(self.before),
            after: Binding::new(self.after),
            configuration_equal: self.configuration_equal,
            graph_registry_equal: self.graph_registry_equal,
            same_generation: self.before.read_generation_id == self.after.read_generation_id,
            lineage_evaluation: "not_evaluated",
            negative_authority: false,
        }
    }
}

#[derive(Serialize)]
pub struct RetainedGenerationPairDescriptor<'a> {
    schema: &'static str,
    profile_id: &'static str,
    pair_id: &'a str,
    status: &'static str,
    before: Binding<'a>,
    after: Binding<'a>,
    configuration_equal: bool,
    graph_registry_equal: bool,
    same_generation: bool,
    lineage_evaluation: &'static str,
    negative_authority: bool,
}

#[derive(Serialize)]
struct Binding<'a> {
    store_epoch_id: String,
    store_generation_id: String,
    publication_set_id: &'a str,
    project_id: &'a str,
    project_kind: crate::ProjectKind,
    project_snapshot_id: &'a str,
    project_generation_id: String,
    project_generation_schema_version: u64,
    analyzer_snapshot_id: &'a str,
    analyzer_pin_id: &'a str,
    analyzer_configuration_digest: String,
    reference_generation_id: String,
    configuration_digest: String,
    profile: &'a ProfileIdentity,
    graph_universe_id: String,
    graph_generation_id: String,
    graph_snapshot_id: String,
    graph_registry_id: String,
    graph_registry_digest: String,
    source_context_id: String,
    project_coverage: &'a [CoverageRecord],
    #[serde(serialize_with = "serialize_deferred")]
    deferred_capabilities: &'a [crate::ProjectDeferredCapability],
    graph_coverage: &'a [wow_graph::GraphCoverageRecord],
}

fn serialize_deferred<S: serde::Serializer>(
    records: &[crate::ProjectDeferredCapability],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    #[derive(Serialize)]
    struct Deferred<'a> {
        capability_id: &'a wow_core::CapabilityId,
        record: &'a wow_core::NotEvaluatedRecord,
    }
    serializer.collect_seq(records.iter().map(|record| Deferred {
        capability_id: record.capability_id(),
        record: record.record(),
    }))
}
impl<'a> Binding<'a> {
    fn new(pair: &'a AcquiredProjectPair) -> Self {
        let project = pair.project();
        let configuration = project.configuration();
        let graph = pair.graph();
        Self {
            store_epoch_id: pair.read_epoch_id.as_str().to_owned(),
            store_generation_id: pair.read_generation_id.as_str().to_owned(),
            publication_set_id: pair.publication_set_id(),
            project_id: configuration.project_id().as_str(),
            project_kind: configuration.project_kind(),
            project_snapshot_id: project.snapshot_id(),
            project_generation_id: project.project_generation().to_string(),
            project_generation_schema_version: project
                .snapshot()
                .generation_candidate()
                .project_generation_schema_version(),
            analyzer_snapshot_id: project.analyzer_snapshot_id(),
            analyzer_pin_id: configuration.analyzer_binding().accepted_pin_id(),
            analyzer_configuration_digest: configuration
                .analyzer_binding()
                .analyzer_configuration_digest()
                .to_string(),
            reference_generation_id: configuration.reference_generation().to_string(),
            configuration_digest: configuration.configuration_digest().to_string(),
            profile: configuration.selected_profile(),
            graph_universe_id: graph.snapshot().universe().to_string(),
            graph_generation_id: graph.snapshot().generation().to_string(),
            graph_snapshot_id: graph.snapshot().snapshot_id().to_string(),
            graph_registry_id: graph.registry().bundle_id().to_string(),
            graph_registry_digest: graph.registry().registry_digest().to_string(),
            source_context_id: graph.source_context_id().to_string(),
            project_coverage: project.snapshot().project_coverage_records(),
            deferred_capabilities: project.snapshot().deferred_capabilities(),
            graph_coverage: graph.snapshot().coverage(),
        }
    }
}
