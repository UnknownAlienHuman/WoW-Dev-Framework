//! Native selected Source producer. The before/after owners and native address
//! maps are borrowed; no source parser, receipt hydration or second hash exists.
use super::*;
use crate::platform_source::{
    PlatformEntryDisposition, PlatformFileKind, PlatformInventoryScope, PlatformLicenseState,
    PlatformSourceClass, PlatformSourceRevision, PlatformUnevaluatedCapability,
};
use crate::replay::publication::RetainedSourceGenerationPair;

pub fn build_source_fingerprint_partition(
    pair: &RetainedSourceGenerationPair<'_>,
    limits: FunctionFingerprintLimits,
    graph_limits: LineageLimits,
    stop: &AtomicBool,
) -> ProjectResult<ValidatedLineageInput> {
    crate::analyzer::checkpoint(stop)?;
    let (before, after) = (pair.before(), pair.after());
    let (a, b) = (pair.before_native(), pair.after_native());
    let mut retained = ProducerBudget::new(0, limits.max_metadata_bytes())?;
    retained.charge_serialized(&(before.graph().snapshot(), after.graph().snapshot()), stop)?;
    let count = a
        .source()
        .functions()
        .len()
        .checked_add(b.source().functions().len())
        .ok_or_else(invalid)?;
    if count > limits.max_records() || count > graph_limits.max_entities {
        return Err(exhausted());
    }
    let inventory_count = |native: &PlatformIndexSourceParts| {
        native
            .project()
            .configuration()
            .platform_packages()
            .map(|p| p.source().receipt().inventory().entries.len())
            .ok_or_else(invalid)
    };
    let work = inventory_count(a)?
        .checked_add(inventory_count(b)?)
        .ok_or_else(invalid)?;
    if u64::try_from(work).map_err(|_| exhausted())? > graph_limits.max_work {
        return Err(exhausted());
    }
    let left = before.graph_read(stop).map_err(lineage_error)?;
    let right = after.graph_read(stop).map_err(lineage_error)?;
    let l = left.owner().producer_lookup(stop).map_err(lineage_error)?;
    let r = right.owner().producer_lookup(stop).map_err(lineage_error)?;
    for owner in [before, after] {
        retained.charge_serialized(owner.project().configuration().selected_profile(), stop)?;
        retained.charge_serialized(&owner.project().snapshot().project_coverage_records(), stop)?;
        #[derive(Serialize)]
        struct Deferred<'a> {
            capability_id: &'a wow_core::CapabilityId,
            record: &'a wow_core::NotEvaluatedRecord,
        }
        for item in owner.project().snapshot().deferred_capabilities() {
            retained.charge_serialized(
                &Deferred {
                    capability_id: item.capability_id(),
                    record: item.record(),
                },
                stop,
            )?;
        }
    }
    let source = LineageSourceComparisonBinding {
        namespace_id: pair.namespace().id().as_str().into(),
        logical_namespace: pair.namespace().logical_namespace().into(),
        owner_project_id: pair.namespace().owner_project_id().into(),
        compatibility_profile_id: SOURCE_SPAN_COMPARISON_POLICY.into(),
        before: metadata(a, &mut retained, stop)?,
        after: metadata(b, &mut retained, stop)?,
    };
    let binding = LineageComparisonBinding {
        universe_class: LineageUniverseClass::BlizzardUiSource,
        profile_id: SOURCE_SPAN_LINEAGE_PROFILE.into(),
        pair_id: pair.pair_id().into(),
        before: binding(before, l.scope()),
        after: binding(after, r.scope()),
        source: Some(source),
        reference: None,
    };
    retained.charge_serialized(&binding, stop)?;
    let mut builder = Builder {
        retained,
        source_bytes: 0,
        limits,
        stop,
        data: ProjectFingerprintPartition {
            schema: SOURCE_SPAN_INPUT_SCHEMA.into(),
            producer_id: SOURCE_SPAN_PRODUCER.into(),
            binding,
            records: Vec::new(),
            reference_records: Vec::new(),
            reference_omissions: Vec::new(),
            partitions: Vec::new(),
            coverage: "partial".into(),
            source_bodies: "excluded".into(),
            proof_ceiling: GraphConfidence::Candidate,
        },
    };
    builder.role_native(before, a.source(), Some(a), LineageRole::Before, &left, &l)?;
    builder.role_native(after, b.source(), Some(b), LineageRole::After, &right, &r)?;
    wow_graph::lineage::validate_source_fingerprint_partition(
        builder.data,
        &left,
        &right,
        graph_limits,
        stop,
    )
    .map_err(lineage_error)
}

fn metadata(
    native: &PlatformIndexSourceParts,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<LineageSourceGenerationMetadata> {
    let packages = native
        .project()
        .configuration()
        .platform_packages()
        .ok_or_else(invalid)?;
    let source = packages.source();
    let profile = source.profile();
    let receipt = source.receipt();
    let inventory = receipt.inventory();
    let coverage = receipt.coverage();
    let target = profile.target();
    // Conservative reserved serialization of borrowed immutable payloads plus
    // fixed field scaffolding before creating the compact owner-neutral copy.
    let borrowed = (
        profile,
        &inventory.origin,
        &inventory.materializer,
        &inventory.license,
        &inventory.roots,
        coverage,
        receipt.source_snapshot_id(),
        native.scope(),
        native.native_prefix_snapshot_id(),
        native.native_prefix_generation(),
        native.foundation_snapshot_id(),
    );
    budget.charge_serialized(&borrowed, stop)?;
    budget.charge_serialized(&borrowed, stop)?;
    let scaffolding: &[u8] = &[0u8; 2048];
    budget.charge_serialized(&scaffolding, stop)?;
    let mut omissions = LineageSourceOmissions::default();
    for (index, entry) in inventory.entries.iter().enumerate() {
        if index % 256 == 0 {
            crate::analyzer::checkpoint(stop)?;
        }
        match entry.disposition {
            PlatformEntryDisposition::Excluded { .. } => omissions.excluded += 1,
            PlatformEntryDisposition::Unsupported { .. } => omissions.unsupported += 1,
            PlatformEntryDisposition::External { .. } => omissions.external += 1,
            PlatformEntryDisposition::Conflict { .. } => omissions.conflict += 1,
            PlatformEntryDisposition::Failed { .. } => omissions.failed += 1,
            _ => {}
        }
        if entry.kind == PlatformFileKind::Unknown {
            omissions.unknown_kind += 1;
        }
    }
    let origin = &inventory.origin;
    let materializer = &inventory.materializer;
    let license = &inventory.license;
    Ok(LineageSourceGenerationMetadata {
        source_profile_id: profile.profile_id().clone(),
        source_profile_digest: profile.digest(),
        source_snapshot_id: receipt.source_snapshot_id().into(),
        content_manifest_digest: receipt.content_manifest_digest(),
        admission_digest: receipt.admission_digest(),
        source_class: match profile.source_class() {
            PlatformSourceClass::SyntheticFixture => LineageSourceClass::SyntheticFixture,
            PlatformSourceClass::VendorUiSourceMirror => LineageSourceClass::VendorUiSourceMirror,
        },
        product: target.product.as_str().into(),
        channel: target.channel.as_str().into(),
        target_reference_profile: target.reference_profile.clone(),
        target_reference_generation: target.reference_generation,
        origin: LineageSourceOrigin {
            provider: origin.provider.as_str().into(),
            repository: origin.repository.as_str().into(),
            revision: match &origin.revision {
                PlatformSourceRevision::Git {
                    object_format,
                    commit,
                    tree,
                } => LineageSourceRevision::Git {
                    object_format: object_format.as_str().into(),
                    commit: commit.as_str().into(),
                    tree: tree.as_str().into(),
                },
                PlatformSourceRevision::Archive { digest } => {
                    LineageSourceRevision::Archive { digest: *digest }
                }
                PlatformSourceRevision::Fixture { digest } => {
                    LineageSourceRevision::Fixture { digest: *digest }
                }
            },
        },
        materializer: LineageSourceMaterializer {
            producer: materializer.producer.clone(),
            version: materializer.version.clone(),
            configuration_digest: materializer.configuration_digest,
            report_digest: materializer.report_digest,
        },
        license: LineageSourceLicense {
            state: match license.state {
                PlatformLicenseState::Known => LineageSourceLicenseState::Known,
                PlatformLicenseState::Unknown => LineageSourceLicenseState::Unknown,
                PlatformLicenseState::Conflict => LineageSourceLicenseState::Conflict,
            },
            attribution: license.attribution.as_str().into(),
            evidence_digest: license.evidence_digest,
        },
        admission_coverage: LineageSourceAdmissionCoverage {
            inventory: coverage.inventory(),
            declared_included_bytes: coverage.declared_included_bytes(),
            verified_files: coverage.verified_files(),
            verified_bytes: coverage.verified_bytes(),
            unevaluated: coverage
                .unevaluated()
                .iter()
                .map(|c| match c {
                    PlatformUnevaluatedCapability::RootCompleteness => {
                        LineageSourceUnevaluated::RootCompleteness
                    }
                    PlatformUnevaluatedCapability::GitMembership => {
                        LineageSourceUnevaluated::GitMembership
                    }
                    PlatformUnevaluatedCapability::MaterializerSecurity => {
                        LineageSourceUnevaluated::MaterializerSecurity
                    }
                    PlatformUnevaluatedCapability::ClientCompatibility => {
                        LineageSourceUnevaluated::ClientCompatibility
                    }
                    PlatformUnevaluatedCapability::LicensePermission => {
                        LineageSourceUnevaluated::LicensePermission
                    }
                    PlatformUnevaluatedCapability::Decoding => LineageSourceUnevaluated::Decoding,
                    PlatformUnevaluatedCapability::PackageLoad => {
                        LineageSourceUnevaluated::PackageLoad
                    }
                    PlatformUnevaluatedCapability::Analyzer => LineageSourceUnevaluated::Analyzer,
                    PlatformUnevaluatedCapability::Graph => LineageSourceUnevaluated::Graph,
                    PlatformUnevaluatedCapability::ApiContract => {
                        LineageSourceUnevaluated::ApiContract
                    }
                    PlatformUnevaluatedCapability::Runtime => LineageSourceUnevaluated::Runtime,
                })
                .collect(),
        },
        roots: profile
            .roots()
            .iter()
            .map(|r| LineageSourceRoot {
                root: r.root.as_str().into(),
                selected_tocs: r.selected_tocs.iter().map(|s| s.as_str().into()).collect(),
            })
            .collect(),
        exclusions: profile
            .exclusions()
            .iter()
            .map(|e| e.path.as_str().into())
            .collect(),
        root_inventory: inventory
            .roots
            .iter()
            .map(|r| LineageSourceRootInventory {
                root: r.root.as_str().into(),
                declared_entries: r.declared_entries,
                scope: match r.scope {
                    PlatformInventoryScope::DeclaredComplete => {
                        LineageSourceInventoryScope::DeclaredComplete
                    }
                    PlatformInventoryScope::DeclaredPartial => {
                        LineageSourceInventoryScope::DeclaredPartial
                    }
                },
                evidence_digest: r.evidence_digest,
            })
            .collect(),
        inventory_entries: inventory.entries.len(),
        omissions,
        compatibility_evidence: inventory.compatibility_evidence,
        native_profile_id: native.profile().into(),
        native_prefix_snapshot_id: native.native_prefix_snapshot_id().clone(),
        native_prefix_generation: native.native_prefix_generation().clone(),
        foundation_snapshot_id: native.foundation_snapshot_id().clone(),
        native_registry_digest: native.registry_digest().into(),
        source_universe: native.scope().universe.clone(),
    })
}
