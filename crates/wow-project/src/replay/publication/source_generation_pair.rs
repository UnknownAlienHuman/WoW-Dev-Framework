//! Source comparisons borrow both original read custodies and genuine native
//! parts. Different revision-scoped graph universes are preserved, never merged.
use super::{AcquiredProjectPair, validate_read_native_parts};
use crate::{
    ProjectError, ProjectErrorCode, ProjectKind, ProjectPhase, ProjectResult,
    graph::PlatformIndexSourceParts,
};
use std::sync::atomic::AtomicBool;
use wow_store::project::{ProjectStoreNamespace, ReadSnapshotCustody};
pub const RETAINED_SOURCE_GENERATION_PAIR_PROFILE: &str =
    "wow-project/retained-blizzard-ui-source-pair/1";

pub struct SourceGenerationSide<'a> {
    pub pair: &'a AcquiredProjectPair,
    pub native: &'a PlatformIndexSourceParts,
    pub custody: ReadSnapshotCustody<'a>,
}
pub struct RetainedSourceGenerationPair<'a> {
    before: SourceGenerationSide<'a>,
    after: SourceGenerationSide<'a>,
    namespace: &'a ProjectStoreNamespace,
    id: Box<str>,
}
impl<'a> RetainedSourceGenerationPair<'a> {
    pub fn bind(
        before: SourceGenerationSide<'a>,
        after: SourceGenerationSide<'a>,
        namespace: &'a ProjectStoreNamespace,
        stop: &AtomicBool,
    ) -> ProjectResult<Self> {
        crate::analyzer::checkpoint(stop)?;
        namespace.validate().map_err(|_| invalid())?;
        if !before.pair.read_origin.matches(&before.custody)
            || !after.pair.read_origin.matches(&after.custody)
            || !before.custody.same_open_owner(&after.custody)
            || before.custody.epoch().namespace() != Some(namespace)
            || after.custody.epoch().namespace() != Some(namespace)
            || before.pair.read_epoch_id != after.pair.read_epoch_id
        {
            return Err(invalid());
        }
        for side in [&before, &after] {
            let configuration = side.pair.project.configuration();
            if configuration.project_kind() != ProjectKind::BlizzardUiPlatformSource
                || configuration.project_id().as_str() != namespace.owner_project_id()
            {
                return Err(invalid());
            }
            validate_read_native_parts(&side.pair.project, &side.pair.graph, side.native, stop)?;
            let packages = configuration.platform_packages().ok_or_else(invalid)?;
            let source = packages.source();
            packages
                .binding()
                .validate(source, packages.load_plan(), packages.main_plan())?;
            if source.profile().profile_id().as_str() != namespace.logical_namespace()
                || source.profile().target().reference_profile != *configuration.selected_profile()
                || source.profile().target().reference_generation
                    != configuration.reference_generation()
                || side.pair.graph().snapshot().universe().as_str()
                    != packages.binding().universe_id()
            {
                return Err(invalid());
            }
        }
        let left = before.pair.project.configuration();
        let right = after.pair.project.configuration();
        let a = left
            .platform_packages()
            .ok_or_else(invalid)?
            .source()
            .profile();
        let b = right
            .platform_packages()
            .ok_or_else(invalid)?
            .source()
            .profile();
        if left.project_id() != right.project_id()
            || a.profile_id() != b.profile_id()
            || a.source_class() != b.source_class()
            || a.target().product != b.target().product
            || a.target().channel != b.target().channel
            || a.target().reference_profile.flavor_id() != b.target().reference_profile.flavor_id()
            || a.target().reference_profile.edition_id()
                != b.target().reference_profile.edition_id()
            || left.analyzer_binding().accepted_pin_id()
                != right.analyzer_binding().accepted_pin_id()
            || before.native.profile() != after.native.profile()
            || before.native.capabilities() != after.native.capabilities()
            || before.native.registry_digest() != after.native.registry_digest()
            || before
                .pair
                .project
                .snapshot()
                .generation_candidate()
                .project_generation_schema_version()
                != after
                    .pair
                    .project
                    .snapshot()
                    .generation_candidate()
                    .project_generation_schema_version()
        {
            return Err(invalid());
        }
        let identity = |side: &SourceGenerationSide<'_>| {
            let source = side
                .pair
                .project
                .configuration()
                .platform_packages()
                .map(|p| p.source())
                .ok_or_else(invalid)?;
            Ok::<_, ProjectError>((
                side.pair.read_epoch_id.clone(),
                side.pair.read_generation_id.clone(),
                side.pair.publication_set_id().to_owned(),
                side.pair.project.snapshot_id().to_owned(),
                side.pair.graph().snapshot().snapshot_id().clone(),
                source.profile().digest(),
                source.receipt().source_snapshot_id().to_owned(),
                source.receipt().content_manifest_digest(),
                source.receipt().admission_digest(),
                side.native.profile(),
                side.native.native_prefix_snapshot_id().clone(),
                side.native.native_prefix_generation().clone(),
            ))
        };
        let hash = wow_core::domain_separated_digest(
            RETAINED_SOURCE_GENERATION_PAIR_PROFILE,
            &(namespace, identity(&before)?, identity(&after)?),
        )
        .map_err(|_| {
            ProjectError::new(
                ProjectErrorCode::CanonicalizationFailed,
                ProjectPhase::View,
                "source pair identity encoding failed",
            )
        })?;
        crate::analyzer::checkpoint(stop)?;
        let id = format!(
            "source-generation-pair:{}",
            wow_core::ContentDigest::<wow_core::CanonicalResult>::from_bytes(hash)
        )
        .into();
        Ok(Self {
            before,
            after,
            namespace,
            id,
        })
    }
    pub fn before(&self) -> &AcquiredProjectPair {
        self.before.pair
    }
    pub fn after(&self) -> &AcquiredProjectPair {
        self.after.pair
    }
    pub fn before_native(&self) -> &PlatformIndexSourceParts {
        self.before.native
    }
    pub fn after_native(&self) -> &PlatformIndexSourceParts {
        self.after.native
    }
    pub fn namespace(&self) -> &ProjectStoreNamespace {
        self.namespace
    }
    pub fn pair_id(&self) -> &str {
        &self.id
    }
}
fn invalid() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::IncompatibleGenerationPair,
        ProjectPhase::View,
        "native Source generation comparison is incompatible or unbound",
    )
}
