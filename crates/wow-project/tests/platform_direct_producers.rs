//! Native direct stages over retained package input; no serialized owner admission.
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use sha2::{Digest, Sha256};
use wow_core::{
    CanonicalResult, ContentDigest, EvidenceId, ProfileIdentityBuilder, ProfileKind,
    ReferenceGenerationId, SchemaVersionEntry, SourceContent, SourceKind, SourceLogicalSnapshot,
    SourceSpan, SourceSpanKind, StableHandleId, domain_separated_digest,
};
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
};
use wow_graph::{
    GraphAssertionKind, GraphAssertionRef, GraphConfidence, GraphCoverageRecord,
    GraphCoverageState, GraphEntityProposal, GraphEvidenceCatalog, GraphLocalAssertion,
    GraphPartitionReplacement, GraphPartitionSnapshot, GraphProposalBatch, GraphProposalEndpoint,
    GraphProposalValue, GraphRelationKind, GraphSnapshot,
};
use wow_project::{
    AnalyzerBindingDeclaration, PackageXmlBindingProfile, PlatformGraphProfile,
    ProjectBudgetPolicy, ProjectCapabilityPolicy, ProjectConfigurationBuilder, ProjectErrorCode,
    ProjectId, ProjectInputBundle, ProjectKind, ProjectPublisher, ProjectSourceOriginId,
    ProjectView, ProjectWorkspaceId,
    disk::{ProjectDiskFile, ProjectInputDirectory},
    graph::{
        PLATFORM_DIRECT_GRAPH_PROFILE, PLATFORM_DIRECT_GRAPH_WITH_INVENTORY_SPANS_PROFILE,
        PLATFORM_DIRECT_GRAPH_WITH_STRUCTURAL_ROLES_PROFILE, PlatformGraphProducer,
        PlatformGraphProposalPlan, ProjectGraphPackageLoadOutcome, ProjectGraphProvenance,
        ProjectGraphXmlReferenceOutcome, ProjectTocFactKind, ProjectXmlFactKind,
        SOURCE_GRAPH_PARTITION, build_platform_graph_proposal_plan,
        build_platform_graph_proposal_plan_with_inventory_spans,
        build_platform_graph_proposal_plan_with_structural_roles, build_source_graph_proposals,
    },
    load::{
        LoadRecordKind, LoadSelection, ProjectPackageInput, ProjectPackageReachability,
        ProjectPackageVariantInput, XmlElementRole,
    },
    platform_source::{
        BlizzardUiSourceProfile, BlizzardUiSourceProfileRequest, PlatformEntryDisposition,
        PlatformFileKind, PlatformInventoryEntry, PlatformInventoryScope, PlatformLicenseRecord,
        PlatformLicenseState, PlatformMaterializer, PlatformPackageSpecialization,
        PlatformRootInventory, PlatformRootSpec, PlatformSourceClass, PlatformSourceInventory,
        PlatformSourceOrigin, PlatformSourceRevision, PlatformTarget, SourceAdmissionLimits,
    },
    replay::ProjectReplay,
};

type ResultOf<T = ()> = Result<T, Box<dyn Error>>;
static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);
const PACKAGES: [&str; 2] = ["Alpha", "Beta"];
const XML: &str = r#"<Ui xmlns="http://www.blizzard.com/wow/ui/">
  <Frame name="Template" virtual="true">
    <Scripts><OnShow function="UniqueHandler"/></Scripts>
  </Frame>
  <Frame name="Receiver">
    <Scripts>
      <OnLoad function="SharedHandler"/>
      <OnClick function="UniqueHandler"/>
      <OnEvent function="BadHandler()"/>
    </Scripts>
  </Frame>
  <Frame name="Inherited" inherits="Template,MissingTemplate"/>
</Ui>
"#;

struct FixtureRoot(PathBuf);
impl FixtureRoot {
    fn new() -> ResultOf<Self> {
        let root = Self(std::env::temp_dir().join(format!(
            "wow-platform-direct-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        )));
        std::fs::create_dir(&root.0)?;
        for package in PACKAGES {
            let directory = root.0.join("UI").join(package);
            std::fs::create_dir_all(&directory)?;
            std::fs::write(
                directory.join("Fixture.toc"),
                "## Interface: 120100\ndefs.lua\nframes.xml\n",
            )?;
            let lua = if package == "Alpha" {
                "function SharedHandler(self) return self end\nfunction UniqueHandler(self) return self end\n"
            } else {
                "function SharedHandler(self) return self end\n"
            };
            std::fs::write(directory.join("defs.lua"), lua)?;
            std::fs::write(directory.join("frames.xml"), XML)?;
        }
        Ok(root)
    }

    fn packages(&self, stop: &AtomicBool) -> ResultOf<Arc<PlatformPackageSpecialization>> {
        let profile = BlizzardUiSourceProfile::new(BlizzardUiSourceProfileRequest {
            profile_id: "profile:fixture:package-xml-bindings-v1".parse()?,
            source_class: PlatformSourceClass::SyntheticFixture,
            target: target()?,
            roots: vec![PlatformRootSpec {
                root: "UI".into(),
                selected_tocs: PACKAGES
                    .iter()
                    .map(|p| format!("UI/{p}/Fixture.toc"))
                    .collect(),
            }],
            exclusions: Vec::new(),
            limits: SourceAdmissionLimits {
                max_entries: 16,
                max_total_bytes: 1024 * 1024,
                max_file_bytes: 1024 * 1024,
                max_manifest_bytes: 32 * 1024,
            },
        })?;
        let mut entries = Vec::new();
        for package in PACKAGES {
            for (name, kind) in [
                ("Fixture.toc", PlatformFileKind::Toc),
                ("defs.lua", PlatformFileKind::Lua),
                ("frames.xml", PlatformFileKind::Xml),
            ] {
                let path = format!("UI/{package}/{name}");
                let bytes = std::fs::read(self.0.join(&path))?;
                entries.push(PlatformInventoryEntry {
                    path,
                    kind,
                    disposition: PlatformEntryDisposition::Included {
                        digest: raw_digest(&bytes),
                        byte_length: u64::try_from(bytes.len())?,
                        object_id: None,
                    },
                });
            }
        }
        let inventory = PlatformSourceInventory {
            schema: "wow-project/platform-source-inventory/1".into(),
            profile_digest: profile.digest(),
            target: profile.target().clone(),
            origin: PlatformSourceOrigin {
                provider: "handwritten-fixture".into(),
                repository: "native-package-xml-input".into(),
                revision: PlatformSourceRevision::Fixture {
                    digest: raw_digest(b"handwritten package XML fixture revision"),
                },
            },
            materializer: PlatformMaterializer {
                producer: "wow.fixture_materializer".parse()?,
                version: "1.0.0".parse()?,
                configuration_digest: ContentDigest::<CanonicalResult>::from_bytes([4; 32]),
                report_digest: raw_digest(b"fixture declaration, not source attestation"),
            },
            roots: vec![PlatformRootInventory {
                root: "UI".into(),
                declared_entries: u64::try_from(entries.len())?,
                scope: PlatformInventoryScope::DeclaredPartial,
                evidence_digest: raw_digest(b"fixture root accounting assertion"),
            }],
            entries,
            license: PlatformLicenseRecord {
                state: PlatformLicenseState::Unknown,
                attribution: "project-owned synthetic test input".into(),
                evidence_digest: raw_digest(b"local fixture notice"),
            },
            compatibility_evidence: raw_digest(b"caller fixture compatibility assertion"),
        };
        let source = Arc::new(
            ProjectInputDirectory::open(&self.0)?
                .admit_platform_source(&profile, inventory, stop)?,
        );
        assert_eq!(
            source.source_bytes("UI/Alpha/frames.xml")?,
            source.source_bytes("UI/Beta/frames.xml")?
        );
        std::fs::remove_dir_all(&self.0)?;
        let declarations = package_declarations();
        Ok(Arc::new(source.specialize_packages(
            &declarations,
            None,
            stop,
        )?))
    }
}
fn package_declarations() -> Vec<ProjectPackageInput> {
    PACKAGES
        .iter()
        .map(|p| {
            ProjectPackageInput::new(
                *p,
                format!("UI/{p}"),
                true,
                vec![ProjectPackageVariantInput::new(
                    ProjectDiskFile::new("Fixture.toc"),
                    true,
                )],
            )
        })
        .collect()
}
impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn raw_digest(bytes: &[u8]) -> ContentDigest<SourceContent> {
    ContentDigest::from_bytes(Sha256::digest(bytes).into())
}
fn target() -> ResultOf<PlatformTarget> {
    let reference_profile = ProfileIdentityBuilder::new(
        "profile:fixture:package-xml-reference-v1".parse()?,
        ProfileKind::Fixture,
        "retail",
        120_100,
        SourceKind::SyntheticFixture,
        "package-xml-handwritten-native-fixture-v1",
        ContentDigest::<SourceLogicalSnapshot>::from_bytes([2; 32]),
    )
    .schema_versions(vec![SchemaVersionEntry::new(
        "schema:wow:package-xml-fixture".parse()?,
        "1.0.0".parse()?,
    )])
    .fixture_scope("package-xml-bindings-local-fixture")
    .build()?;
    Ok(PlatformTarget {
        product: "fixture".into(),
        channel: "fixture".into(),
        reference_profile,
        reference_generation: ReferenceGenerationId::from_hash([3; 32]),
    })
}
fn publish(
    packages: &Arc<PlatformPackageSpecialization>,
    graph: bool,
    stop: &AtomicBool,
) -> ResultOf<ProjectPublisher> {
    publish_with_graph_profile(
        packages,
        graph.then_some(PlatformGraphProfile::PackageProjectionV1),
        stop,
    )
}
fn publish_with_graph_profile(
    packages: &Arc<PlatformPackageSpecialization>,
    graph_profile: Option<PlatformGraphProfile>,
    stop: &AtomicBool,
) -> ResultOf<ProjectPublisher> {
    let target = packages.source().profile().target();
    let backend = EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(EMMYLUA_CODE_ANALYSIS_VERSION),
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        format!("sha256:{}", "1".repeat(64)),
        format!("sha256:{}", "2".repeat(64)),
    )?;
    let analyzer = AnalyzerBindingDeclaration::new(
        "wow-emmy/e0-c/1",
        format!("emmy-pin:{EMMYLUA_REVISION}"),
        backend.compatibility_report_sha256(),
        ContentDigest::<CanonicalResult>::from_bytes([3; 32]),
        "wow-emmy/e0-c/1",
        "wow-emmy-e0-c-library-v1",
        backend.clone(),
    )?;
    let builder = ProjectConfigurationBuilder::new(
        ProjectId::new("package-xml-bindings-fixture")?,
        ProjectKind::BlizzardUiPlatformSource,
        target.reference_profile.clone(),
        target.reference_generation,
        analyzer,
    )
    .workspace_id(ProjectWorkspaceId::new(
        "workspace:main:package-xml-fixture",
    )?)
    .source_origin_id(ProjectSourceOriginId::new(
        "project-origin:package-xml-fixture",
    )?)
    .logical_root("fixtures/package-xml/UI")
    .capability_policy(ProjectCapabilityPolicy::strict_e0()?)
    .budget_policy(ProjectBudgetPolicy::fixture_e0()?)
    .platform_packages(Arc::clone(packages))?
    .with_package_xml_bindings(PackageXmlBindingProfile::SameSessionV1);
    let configuration = match graph_profile {
        Some(profile) => builder.with_platform_graph_profile(profile),
        None => builder,
    }
    .build()?;
    let library = LuaWorkspaceSnapshot::build(
        backend,
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "library/package-xml-fixture.lua",
            "---@meta _\nPackageXmlLibraryFixture = {}\n",
        )],
        LuaWorkspaceLimits::new(8, 16_384, 256 * 1024, 512 * 1024)?,
    )?;
    let mut publisher = ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(
        ProjectInputBundle::closed(configuration, packages.files().to_vec(), vec![library])?,
        stop,
    )?;
    Ok(publisher)
}

fn key(kind: GraphAssertionKind, id: &str) -> GraphLocalAssertion {
    GraphLocalAssertion {
        kind,
        proposal_id: id.into(),
    }
}
fn admit(
    owner: &GraphPartitionSnapshot,
    batch: GraphProposalBatch,
    version: &str,
    coverage: Vec<GraphCoverageRecord>,
    stop: &AtomicBool,
) -> ResultOf<GraphPartitionSnapshot> {
    Ok(owner
        .prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: owner.snapshot().snapshot_id().clone(),
                expected_partition_digest: owner
                    .partition(batch.producer_partition_id())
                    .map(|p| p.partition_digest().into()),
                producer_version: version.into(),
                batch,
                coverage,
            },
            stop,
        )?
        .candidate()
        .clone())
}
fn initial(
    plan: &PlatformGraphProposalPlan<'_>,
    stop: &AtomicBool,
) -> ResultOf<GraphPartitionSnapshot> {
    let mut owner = GraphPartitionSnapshot::new(
        plan.registry().clone(),
        plan.foundation().clone(),
        plan.scope().source_context_id,
        stop,
    )?;
    if let Some(batch) = plan.raw_inventory_batch() {
        owner = admit(
            &owner,
            batch.clone(),
            plan.raw_inventory_producer_version(),
            Vec::new(),
            stop,
        )?;
    }
    Ok(owner)
}
fn tombstone(
    owner: &GraphPartitionSnapshot,
    stop: &AtomicBool,
) -> ResultOf<GraphPartitionSnapshot> {
    let batch = GraphProposalBatch::build(
        owner.registry().bundle_id(),
        owner.registry().registry_digest(),
        owner.foundation().universe().clone(),
        owner.foundation().generation().clone(),
        owner.source_context_id(),
        "fixture:unrelated-empty",
        Vec::new(),
        Vec::new(),
    )?;
    admit(owner, batch, "1", Vec::new(), stop)
}
fn replan<'a>(
    view: &'a ProjectView,
    prefixes: &[GraphPartitionSnapshot],
    stop: &AtomicBool,
) -> ResultOf<PlatformGraphProposalPlan<'a>> {
    let mut plan = build_platform_graph_proposal_plan(view, stop)?;
    for (producer, owner) in plan.producer_order().iter().copied().zip(prefixes) {
        plan.build_stage(producer, owner, stop)?;
    }
    Ok(plan)
}
fn refused<T>(result: wow_project::ProjectResult<T>) -> ResultOf {
    assert_eq!(
        result.err().ok_or("unexpected direct-plan success")?.code(),
        ProjectErrorCode::SnapshotInvalid
    );
    Ok(())
}

fn inventory_spans(
    view: &ProjectView,
    original: &GraphPartitionSnapshot,
    source: &ProjectGraphProvenance,
    stop: &AtomicBool,
) -> ResultOf<GraphPartitionSnapshot> {
    assert_eq!(original.registry().version(), "16");
    assert_eq!(
        wow_project::graph::source_graph_profile(view.configuration()),
        "wow-project/source-load-proposals/21"
    );
    let known = source
        .source_handles()
        .iter()
        .filter(|(_, handle)| handle.span().kind() != SourceSpanKind::Unknown)
        .map(|(id, _)| *id)
        .collect::<BTreeSet<_>>();
    let omitted = source.source_handles().len() - known.len();
    let mut plan = build_platform_graph_proposal_plan_with_inventory_spans(view, stop)?;
    assert_eq!(
        plan.profile(),
        PLATFORM_DIRECT_GRAPH_WITH_INVENTORY_SPANS_PROFILE
    );
    assert_eq!(plan.registry().version(), "17");
    assert_eq!(plan.inventory_span_omissions(), Some(omitted));
    assert_eq!(plan.scope().universe, *original.foundation().universe());
    assert_ne!(plan.scope().generation, *original.foundation().generation());
    assert_eq!(plan.scope().source_context_id, original.source_context_id());
    let scope = plan.scope().clone();
    let raw = plan
        .raw_inventory_batch()
        .ok_or("spans raw prelude missing")?;
    assert_eq!(raw.universe(), &scope.universe);
    assert_eq!(raw.generation(), &scope.generation);
    assert_eq!(raw.source_context_id(), scope.source_context_id);
    assert_eq!(raw.registry_digest(), plan.registry().registry_digest());
    let old_raw = original
        .partition(wow_project::graph::PLATFORM_RAW_INVENTORY_PARTITION)
        .ok_or("original raw prelude missing")?
        .batch();
    assert_ne!(raw, old_raw);
    assert_eq!(raw.entity_proposals(), old_raw.entity_proposals());
    refused(plan.build_stage(PlatformGraphProducer::Inventory, original, stop))?;
    let mut owner = initial(&plan, stop)?;
    for &producer in plan.producer_order() {
        let stage = plan.build_stage(producer, &owner, stop)?;
        assert_eq!(stage.producer_version(), "2");
        let (batch, coverage) = stage.into_parts();
        if producer == PlatformGraphProducer::Inventory {
            let wrong_version = admit(&owner, batch.clone(), "1", coverage.clone(), stop)?;
            refused(plan.build_stage(PlatformGraphProducer::TocLoad, &wrong_version, stop))?;
        }
        owner = admit(&owner, batch, "2", coverage, stop)?;
    }
    owner.validate(stop)?;
    assert_eq!(
        owner
            .partitions()
            .iter()
            .map(|p| p.partition_id())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            wow_project::graph::PLATFORM_RAW_INVENTORY_PARTITION,
            PlatformGraphProducer::Inventory.partition_id(),
            PlatformGraphProducer::TocLoad.partition_id(),
            PlatformGraphProducer::AnalyzerStructure.partition_id(),
            PlatformGraphProducer::XmlStructure.partition_id(),
        ])
    );
    let finished = plan.finish(&owner, stop)?;
    assert_eq!(
        finished.profile(),
        PLATFORM_DIRECT_GRAPH_WITH_INVENTORY_SPANS_PROFILE
    );
    assert_eq!(finished.inventory_span_omissions(), Some(omitted));
    assert!(std::ptr::eq(finished.project(), view));
    assert!(std::ptr::eq(finished.graph(), &owner));
    assert_eq!(finished.source(), source);
    let inventory = owner
        .partition(PlatformGraphProducer::Inventory.partition_id())
        .ok_or("spans Inventory missing")?;
    let contains_coverage = inventory
        .coverage()
        .iter()
        .find(|record| record.relation() == GraphRelationKind::Contains)
        .ok_or("Inventory Contains coverage missing")?;
    assert_eq!(contains_coverage.state(), GraphCoverageState::Partial);
    assert!(!contains_coverage.negative_authority());

    let mut handles = source.source_handles().clone();
    let mut evidence = source.evidence().clone();
    for member in source
        .raw_inventory()
        .ok_or("original raw manifest missing")?
        .members()
    {
        if let Some(previous) = handles.insert(
            member.source_handle.handle_id(),
            member.source_handle.clone(),
        ) {
            assert_eq!(previous, member.source_handle);
        }
        if let Some(previous) =
            evidence.insert(member.evidence.evidence_id(), member.evidence.clone())
        {
            assert_eq!(previous, member.evidence);
        }
    }
    let expected_catalog = GraphEvidenceCatalog::new(
        source.context().clone(),
        evidence.clone(),
        handles.clone(),
        stop,
    )?;
    assert_eq!(
        finished.evidence_catalog().context(),
        expected_catalog.context()
    );
    assert_eq!(
        finished.evidence_catalog().digest(),
        expected_catalog.digest()
    );
    for (id, handle) in &handles {
        assert_eq!(finished.evidence_catalog().source_handle(id), Some(handle));
    }
    for (id, record) in &evidence {
        assert_eq!(finished.evidence_catalog().evidence(id), Some(record));
    }

    let lookup = owner.producer_lookup(stop)?;
    assert_eq!(lookup.scope(), &scope);
    let mut spans = BTreeMap::new();
    let mut contained = BTreeSet::new();
    for partition in owner.partitions() {
        assert!(partition.report().rejections().is_empty());
        assert_eq!(
            partition.report().accepted_entities().len(),
            partition.batch().entity_proposals().len()
        );
        for accepted in partition.report().accepted_entities() {
            let address = finished
                .assertion(&key(GraphAssertionKind::Entity, accepted.proposal_id()))
                .ok_or("spans entity address missing")?;
            let resolved = lookup.entity(finished.scope(), address, stop)?;
            assert_eq!(resolved.reference(), *address);
            assert_eq!(resolved.partition(), partition);
            assert_eq!(resolved.accepted(), accepted);
            let proposal = resolved.proposal();
            assert_eq!(
                partition.batch().entity_proposal(accepted.proposal_id()),
                Some(proposal)
            );
            for id in proposal.source_handle_ids() {
                assert_eq!(
                    finished.evidence_catalog().source_handle(id),
                    Some(handles.get(id).ok_or("entity handle missing")?)
                );
            }
            for id in proposal.evidence_ids() {
                assert_eq!(
                    finished.evidence_catalog().evidence(id),
                    Some(evidence.get(id).ok_or("entity evidence missing")?)
                );
            }
            if proposal.entity_kind_id() != "source_span" {
                continue;
            }
            assert_eq!(partition.partition_id(), inventory.partition_id());
            let [handle_id] = proposal.source_handle_ids() else {
                return Err("span must retain one original handle".into());
            };
            let [evidence_id] = proposal.evidence_ids() else {
                return Err("span must retain one original evidence record".into());
            };
            let handle = source
                .source_handles()
                .get(handle_id)
                .ok_or("original span handle missing")?;
            let record = source
                .evidence()
                .get(evidence_id)
                .ok_or("original span evidence missing")?;
            assert_eq!(record.source_handle_ids(), std::slice::from_ref(handle_id));
            assert_eq!(record.context_id(), scope.source_context_id);
            assert_eq!(proposal.confidence(), GraphConfidence::Proven);
            assert_eq!(
                proposal.semantic_key(),
                &BTreeMap::from([(
                    "source_handle".into(),
                    GraphProposalValue::String(handle_id.canonical().into())
                )])
            );
            assert_eq!(
                view.source_handle(
                    handle.path().as_str(),
                    handle.span(),
                    handle.entity_key().cloned()
                )?,
                *handle
            );
            assert!(known.contains(handle_id));
            if handle.span().kind() == SourceSpanKind::ByteRange {
                let file = source
                    .files()
                    .iter()
                    .find(|file| file.path == handle.path().as_str())
                    .ok_or("span file missing")?;
                let start = handle.span().byte_start().ok_or("span start missing")?;
                let end = handle.span().byte_end().ok_or("span end missing")?;
                assert!(start <= end && end <= file.byte_length);
            }
            assert!(
                spans
                    .insert(*handle_id, accepted.node().node_id())
                    .is_none()
            );
        }
        for accepted in partition.report().accepted_relations() {
            let address = finished
                .assertion(&key(GraphAssertionKind::Relation, accepted.proposal_id()))
                .ok_or("spans relation address missing")?;
            let resolved = lookup.relation(finished.scope(), address, stop)?;
            assert_eq!(resolved.reference(), *address);
            assert_eq!(resolved.partition(), partition);
            assert_eq!(resolved.accepted(), accepted);
            let proposal = resolved.proposal();
            assert_eq!(
                partition.batch().relation_proposal(accepted.proposal_id()),
                Some(proposal)
            );
            for id in proposal.source_handle_ids() {
                assert_eq!(
                    finished.evidence_catalog().source_handle(id),
                    Some(handles.get(id).ok_or("relation handle missing")?)
                );
            }
            for id in proposal.evidence_ids() {
                assert_eq!(
                    finished.evidence_catalog().evidence(id),
                    Some(evidence.get(id).ok_or("relation evidence missing")?)
                );
            }
            if proposal.relation_kind_id() != "source_file_contains_span" {
                continue;
            }
            assert_eq!(partition.partition_id(), inventory.partition_id());
            let [handle_id] = proposal.source_handle_ids() else {
                return Err("containment must retain one original handle".into());
            };
            let span = spans.get(handle_id).ok_or("contained span missing")?;
            let handle = source
                .source_handles()
                .get(handle_id)
                .ok_or("containment handle missing")?;
            let file = source
                .files()
                .iter()
                .find(|file| file.path == handle.path().as_str())
                .ok_or("containing file missing")?;
            let file_address = finished
                .assertion(&key(GraphAssertionKind::Entity, &file.proposal_id))
                .ok_or("file address missing")?;
            let file_entity = lookup.entity(finished.scope(), file_address, stop)?;
            assert_eq!(
                file_entity.partition().partition_id(),
                inventory.partition_id()
            );
            assert_eq!(file_entity.proposal().entity_kind_id(), "source_file");
            assert_eq!(
                accepted.edge().from(),
                file_entity.accepted().node().node_id()
            );
            assert_eq!(accepted.edge().to(), *span);
            assert_eq!(accepted.edge().relation(), GraphRelationKind::Contains);
            assert_eq!(accepted.edge().confidence(), GraphConfidence::Proven);
            let span_proposal = inventory
                .batch()
                .entity_proposals()
                .iter()
                .find(|p| {
                    p.source_handle_ids() == proposal.source_handle_ids()
                        && p.entity_kind_id() == "source_span"
                })
                .ok_or("span proposal missing")?;
            assert_eq!(proposal.evidence_ids(), span_proposal.evidence_ids());
            assert!(contained.insert(*handle_id));
        }
    }
    assert_eq!(spans.keys().copied().collect::<BTreeSet<_>>(), known);
    assert_eq!(contained, known);
    let ranged = source
        .source_handles()
        .values()
        .find(|handle| handle.span().kind() == SourceSpanKind::ByteRange)
        .ok_or("native ByteRange witness missing")?;
    let whole = source
        .source_handles()
        .values()
        .find(|handle| {
            handle.path() == ranged.path() && handle.span().kind() == SourceSpanKind::WholeFile
        })
        .ok_or("native WholeFile witness missing")?;
    assert_eq!(whole.content_digest(), ranged.content_digest());
    assert_ne!(whole.span(), ranged.span());
    assert_ne!(whole.handle_id(), ranged.handle_id());
    assert_ne!(
        spans.get(&whole.handle_id()),
        spans.get(&ranged.handle_id())
    );

    let mut legacy = build_platform_graph_proposal_plan(view, stop)?;
    assert_eq!(legacy.profile(), PLATFORM_DIRECT_GRAPH_PROFILE);
    assert_eq!(legacy.registry(), original.registry());
    assert_eq!(legacy.inventory_span_omissions(), None);
    let mut unchanged = initial(&legacy, stop)?;
    for &producer in legacy.producer_order() {
        let stage = legacy.build_stage(producer, &unchanged, stop)?;
        assert_eq!(stage.producer_version(), "1");
        let (batch, coverage) = stage.into_parts();
        unchanged = admit(&unchanged, batch, "1", coverage, stop)?;
    }
    assert_eq!(&unchanged, original);
    let legacy_finished = legacy.finish(&unchanged, stop)?;
    assert_eq!(legacy_finished.profile(), PLATFORM_DIRECT_GRAPH_PROFILE);
    assert_eq!(legacy_finished.inventory_span_omissions(), None);
    assert_eq!(legacy_finished.source(), source);
    Ok(owner)
}

fn role<'a>(
    batch: &'a GraphProposalBatch,
    kind: &str,
    fields: &[(&str, GraphProposalValue)],
) -> ResultOf<&'a GraphEntityProposal> {
    let mut matches = batch.entity_proposals().iter().filter(|proposal| {
        proposal.entity_kind_id() == kind
            && fields
                .iter()
                .all(|(field, value)| proposal.semantic_key().get(*field) == Some(value))
    });
    let proposal = matches.next().ok_or("structural role missing")?;
    assert!(matches.next().is_none(), "ambiguous structural role");
    Ok(proposal)
}

fn string(value: &str) -> GraphProposalValue {
    GraphProposalValue::String(value.into())
}

fn identifier(value: &impl serde::Serialize) -> ResultOf<GraphProposalValue> {
    let serde_json::Value::String(value) = serde_json::to_value(value)? else {
        return Err("native role identifier must be a string".into());
    };
    Ok(GraphProposalValue::Identifier(value.into()))
}

fn role_support(
    actual_handles: &[StableHandleId],
    actual_evidence: &[EvidenceId],
    handles: &[StableHandleId],
    evidence: &[EvidenceId],
) {
    let expected_handles = handles.iter().copied().collect::<BTreeSet<_>>();
    let expected_evidence = evidence.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(actual_handles.len(), expected_handles.len());
    assert_eq!(actual_evidence.len(), expected_evidence.len());
    assert_eq!(
        actual_handles.iter().copied().collect::<BTreeSet<_>>(),
        expected_handles
    );
    assert_eq!(
        actual_evidence.iter().copied().collect::<BTreeSet<_>>(),
        expected_evidence
    );
}

fn structural_roles(
    view: &ProjectView,
    original: &GraphPartitionSnapshot,
    spans: &GraphPartitionSnapshot,
    source: &ProjectGraphProvenance,
    stop: &AtomicBool,
) -> ResultOf {
    assert_eq!(original.registry().version(), "16");
    assert_eq!(spans.registry().version(), "17");
    let mut plan = build_platform_graph_proposal_plan_with_structural_roles(view, stop)?;
    assert_eq!(
        plan.profile(),
        PLATFORM_DIRECT_GRAPH_WITH_STRUCTURAL_ROLES_PROFILE
    );
    assert_eq!(plan.registry().version(), "18");
    let omitted = source
        .source_handles()
        .values()
        .filter(|handle| handle.span().kind() == SourceSpanKind::Unknown)
        .count();
    assert_eq!(plan.inventory_span_omissions(), Some(omitted));
    for prior in [original, spans] {
        assert_eq!(&plan.scope().universe, prior.foundation().universe());
        assert_ne!(&plan.scope().generation, prior.foundation().generation());
        assert_eq!(plan.scope().source_context_id, prior.source_context_id());
        refused(plan.build_stage(PlatformGraphProducer::Inventory, prior, stop))?;
    }
    let scope = plan.scope().clone();
    let raw = plan
        .raw_inventory_batch()
        .ok_or("structural raw prelude missing")?;
    assert_eq!(raw.universe(), &scope.universe);
    assert_eq!(raw.generation(), &scope.generation);
    assert_eq!(raw.source_context_id(), scope.source_context_id);
    assert_eq!(raw.registry_digest(), plan.registry().registry_digest());
    let raw_id = wow_project::graph::PLATFORM_RAW_INVENTORY_PARTITION;
    assert_eq!(
        raw.entity_proposals(),
        original
            .partition(raw_id)
            .ok_or("original raw partition missing")?
            .batch()
            .entity_proposals()
    );
    let mut owner = initial(&plan, stop)?;
    assert_eq!(owner.partitions().len(), 1);
    for &producer in plan.producer_order() {
        let stage = plan.build_stage(producer, &owner, stop)?;
        assert_eq!(stage.producer_version(), "3");
        let (batch, coverage) = stage.into_parts();
        if producer == PlatformGraphProducer::Inventory {
            let wrong_version = admit(&owner, batch.clone(), "2", coverage.clone(), stop)?;
            refused(plan.build_stage(PlatformGraphProducer::TocLoad, &wrong_version, stop))?;
        }
        owner = admit(&owner, batch, "3", coverage, stop)?;
    }
    owner.validate(stop)?;
    assert_eq!(
        owner
            .partitions()
            .iter()
            .map(|p| p.partition_id())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            raw_id,
            PlatformGraphProducer::Inventory.partition_id(),
            PlatformGraphProducer::TocLoad.partition_id(),
            PlatformGraphProducer::AnalyzerStructure.partition_id(),
            PlatformGraphProducer::XmlStructure.partition_id()
        ])
    );
    assert!(owner.partition(SOURCE_GRAPH_PARTITION).is_none());
    let finished = plan.finish(&owner, stop)?;
    assert_eq!(
        finished.profile(),
        PLATFORM_DIRECT_GRAPH_WITH_STRUCTURAL_ROLES_PROFILE
    );
    assert_eq!(finished.inventory_span_omissions(), Some(omitted));
    assert!(std::ptr::eq(finished.project(), view));
    assert!(std::ptr::eq(finished.graph(), &owner));
    assert_eq!(finished.scope(), &scope);
    // Includes exact load/TOC/XML outcomes and the original unresolved inheritance.
    assert_eq!(finished.source(), source);
    assert!(source.xml_facts().iter().any(|fact| matches!(&fact.kind,
        ProjectXmlFactKind::InheritanceUnresolved { name, .. } if name == "MissingTemplate")));
    assert!(
        source
            .xml_inheritance()
            .iter()
            .any(|row| row.outcome == ProjectGraphXmlReferenceOutcome::Unresolved)
    );

    let manifest = source
        .raw_inventory()
        .ok_or("structural raw manifest missing")?;
    let packages = view
        .configuration()
        .platform_packages()
        .ok_or("platform packages missing")?;
    let receipt = packages.source().receipt();
    assert_eq!(manifest.inventory(), receipt.inventory());
    assert_eq!(manifest.coverage(), receipt.coverage());
    assert_eq!(
        manifest.inventory().license.state,
        PlatformLicenseState::Unknown
    );
    assert!(
        manifest
            .inventory()
            .roots
            .iter()
            .all(|root| root.scope == PlatformInventoryScope::DeclaredPartial)
    );
    let mut handles = source.source_handles().clone();
    let mut evidence = source.evidence().clone();
    for member in manifest.members() {
        if let Some(prior) = handles.insert(
            member.source_handle.handle_id(),
            member.source_handle.clone(),
        ) {
            assert_eq!(prior, member.source_handle);
        }
        if let Some(prior) = evidence.insert(member.evidence.evidence_id(), member.evidence.clone())
        {
            assert_eq!(prior, member.evidence);
        }
        let native = packages.source().raw_member(&member.path, stop)?;
        assert_eq!(native.entry().kind, member.kind);
        assert_eq!(native.content_digest(), member.content_digest);
        assert_eq!(native.bytes().len() as u64, member.byte_length);
    }
    let catalog = GraphEvidenceCatalog::new(
        source.context().clone(),
        evidence.clone(),
        handles.clone(),
        stop,
    )?;
    assert_eq!(finished.evidence_catalog().digest(), catalog.digest());
    assert_eq!(finished.evidence_catalog().context(), source.context());
    for (id, handle) in &handles {
        assert_eq!(finished.evidence_catalog().source_handle(id), Some(handle));
    }
    for (id, record) in &evidence {
        assert_eq!(finished.evidence_catalog().evidence(id), Some(record));
        assert_eq!(record.context_id(), scope.source_context_id);
    }
    for handle in source.source_handles().values() {
        assert_eq!(
            view.source_handle(
                handle.path().as_str(),
                handle.span(),
                handle.entity_key().cloned()
            )?,
            *handle
        );
    }

    let lookup = owner.producer_lookup(stop)?;
    assert_eq!(lookup.scope(), &scope);
    let old_nodes = spans
        .partitions()
        .iter()
        .flat_map(|partition| {
            partition
                .report()
                .accepted_entities()
                .iter()
                .map(|accepted| (accepted.node().node_id().clone(), accepted.proposal_id()))
        })
        .collect::<BTreeMap<_, _>>();
    let mut nodes = BTreeMap::new();
    let mut preserved_entities = 0;
    let mut preserved_relations = 0;
    for partition in owner.partitions() {
        assert!(partition.report().rejections().is_empty());
        assert_eq!(
            partition.report().accepted_entities().len(),
            partition.batch().entity_proposals().len()
        );
        if partition.partition_id() != raw_id {
            assert_eq!(partition.producer_version(), "3");
        }
        for coverage in partition.coverage() {
            assert!(!coverage.negative_authority());
        }
        for accepted in partition.report().accepted_entities() {
            let address = finished
                .assertion(&key(GraphAssertionKind::Entity, accepted.proposal_id()))
                .ok_or("structural entity address missing")?;
            let resolved = lookup.entity(&scope, address, stop)?;
            assert_eq!(resolved.reference(), *address);
            assert_eq!(resolved.partition(), partition);
            assert_eq!(resolved.accepted(), accepted);
            let proposal = resolved.proposal();
            assert_eq!(
                partition.batch().entity_proposal(accepted.proposal_id()),
                Some(proposal)
            );
            assert_eq!(accepted.node().generation(), &scope.generation);
            assert_eq!(
                lookup.input_view().node(accepted.node().node_id()),
                Some(accepted.node())
            );
            for id in proposal.source_handle_ids() {
                assert_eq!(
                    finished.evidence_catalog().source_handle(id),
                    Some(handles.get(id).ok_or("entity support missing")?)
                );
            }
            for id in proposal.evidence_ids() {
                assert_eq!(
                    finished.evidence_catalog().evidence(id),
                    Some(evidence.get(id).ok_or("entity evidence missing")?)
                );
            }
            if let Some(old) = spans
                .partition(partition.partition_id())
                .and_then(|p| p.batch().entity_proposal(accepted.proposal_id()))
            {
                assert_eq!(proposal, old);
                preserved_entities += 1;
            }
            assert!(
                nodes
                    .insert(accepted.node().node_id().clone(), accepted.proposal_id())
                    .is_none()
            );
        }
    }
    let mut edges = BTreeMap::new();
    for partition in owner.partitions() {
        for accepted in partition.report().accepted_relations() {
            let address = finished
                .assertion(&key(GraphAssertionKind::Relation, accepted.proposal_id()))
                .ok_or("structural relation address missing")?;
            let resolved = lookup.relation(&scope, address, stop)?;
            assert_eq!(resolved.reference(), *address);
            assert_eq!(resolved.partition(), partition);
            assert_eq!(resolved.accepted(), accepted);
            let proposal = resolved.proposal();
            assert_eq!(
                partition.batch().relation_proposal(accepted.proposal_id()),
                Some(proposal)
            );
            for id in proposal.source_handle_ids() {
                assert_eq!(
                    finished.evidence_catalog().source_handle(id),
                    Some(handles.get(id).ok_or("relation support missing")?)
                );
            }
            for id in proposal.evidence_ids() {
                assert_eq!(
                    finished.evidence_catalog().evidence(id),
                    Some(evidence.get(id).ok_or("relation evidence missing")?)
                );
            }
            for (endpoint, node) in proposal
                .endpoints()
                .into_iter()
                .zip([accepted.edge().from(), accepted.edge().to()])
            {
                let id = *nodes.get(node).ok_or("structural endpoint missing")?;
                let native = lookup.entity(
                    &scope,
                    finished
                        .assertion(&key(GraphAssertionKind::Entity, id))
                        .ok_or("endpoint address missing")?,
                    stop,
                )?;
                assert_eq!(native.accepted().node().node_id(), node);
                match endpoint {
                    GraphProposalEndpoint::Proposed(local) => {
                        assert_eq!(local.as_ref(), id);
                        assert_eq!(native.partition(), partition);
                    }
                    GraphProposalEndpoint::Existing(input) => {
                        assert_eq!(input, node);
                        assert!(lookup.input_view().node(input).is_some());
                        assert_ne!(native.partition().partition_id(), partition.partition_id());
                    }
                }
            }
            if let Some(old_partition) = spans.partition(partition.partition_id())
                && let Some(old) = old_partition
                    .batch()
                    .relation_proposal(accepted.proposal_id())
            {
                assert_eq!(proposal.relation_kind_id(), old.relation_kind_id());
                assert_eq!(proposal.confidence(), old.confidence());
                assert_eq!(proposal.source_handle_ids(), old.source_handle_ids());
                assert_eq!(proposal.evidence_ids(), old.evidence_ids());
                let old_accepted = old_partition
                    .report()
                    .accepted_relations()
                    .iter()
                    .find(|row| row.proposal_id() == accepted.proposal_id())
                    .ok_or("old relation receipt missing")?;
                assert_eq!(
                    nodes.get(accepted.edge().from()),
                    old_nodes.get(old_accepted.edge().from())
                );
                assert_eq!(
                    nodes.get(accepted.edge().to()),
                    old_nodes.get(old_accepted.edge().to())
                );
                preserved_relations += 1;
            }
            let from = *nodes
                .get(accepted.edge().from())
                .ok_or("edge source missing")?;
            let to = *nodes
                .get(accepted.edge().to())
                .ok_or("edge target missing")?;
            assert!(
                edges
                    .insert((proposal.relation_kind_id(), from, to), proposal)
                    .is_none()
            );
        }
        let records = partition.batch().assertion_records();
        if let Some(records) = records {
            assert_eq!(&records.scope, &scope);
            for derivation in &records.derivations {
                assert!(finished.assertion(&derivation.output).is_some());
                for input in &derivation.inputs {
                    if let GraphAssertionRef::Producer { assertion, .. } = input {
                        assert_eq!(finished.assertion(assertion), Some(input));
                        lookup.entity(&scope, input, stop)?;
                    }
                }
            }
        }
    }
    assert_eq!(
        preserved_entities,
        spans
            .partitions()
            .iter()
            .map(|p| p.report().accepted_entities().len())
            .sum::<usize>()
    );
    assert_eq!(
        preserved_relations,
        spans
            .partitions()
            .iter()
            .map(|p| p.report().accepted_relations().len())
            .sum::<usize>()
    );

    let inventory = owner
        .partition(PlatformGraphProducer::Inventory.partition_id())
        .ok_or("Inventory missing")?;
    let toc = owner
        .partition(PlatformGraphProducer::TocLoad.partition_id())
        .ok_or("TocLoad missing")?;
    let xml = owner
        .partition(PlatformGraphProducer::XmlStructure.partition_id())
        .ok_or("XmlStructure missing")?;
    for (partition, relation) in [
        (inventory, GraphRelationKind::Contains),
        (toc, GraphRelationKind::Contains),
        (toc, GraphRelationKind::Defines),
        (toc, GraphRelationKind::LoadsBefore),
        (xml, GraphRelationKind::Contains),
        (xml, GraphRelationKind::Owns),
    ] {
        let coverage = partition
            .coverage()
            .iter()
            .find(|row| row.relation() == relation)
            .ok_or("structural coverage missing")?;
        assert_eq!(coverage.state(), GraphCoverageState::Partial);
        assert!(!coverage.negative_authority());
        assert!(!coverage.blocker_ids().is_empty());
    }

    let mut checked = BTreeSet::new();
    {
        let mut edge = |kind: &str,
                        from: &str,
                        to: &str,
                        handles: &[StableHandleId],
                        evidence: &[EvidenceId],
                        confidence: GraphConfidence|
         -> ResultOf {
            let proposal = edges
                .get(&(kind, from, to))
                .ok_or("structural role edge missing")?;
            assert_eq!(proposal.confidence(), confidence);
            role_support(
                proposal.source_handle_ids(),
                proposal.evidence_ids(),
                handles,
                evidence,
            );
            assert!(checked.insert(proposal.proposal_id()));
            Ok(())
        };
        let project = role(inventory.batch(), "source_project", &[])?;
        let declarations = package_declarations();
        // This JSON is expected digest material only; every authority above is native.
        let request = serde_json::json!({"packages": &declarations});
        let authority = ContentDigest::<CanonicalResult>::from_bytes(domain_separated_digest(
            "wow-project/platform-source-project-package-authority/1",
            &(packages.binding().binding_digest(), &request),
        )?);
        assert_ne!(authority, packages.binding().binding_digest());
        assert_eq!(
            project.semantic_key(),
            &BTreeMap::from([
                (
                    "project".into(),
                    string(view.configuration().project_id().as_str())
                ),
                ("project_snapshot".into(), string(view.snapshot_id())),
                (
                    "source_snapshot".into(),
                    string(receipt.source_snapshot_id())
                ),
                (
                    "profile_digest".into(),
                    string(&receipt.profile_digest().canonical())
                ),
                (
                    "content_manifest_digest".into(),
                    string(&receipt.content_manifest_digest().canonical())
                ),
                (
                    "admission_digest".into(),
                    string(&receipt.admission_digest().canonical())
                ),
                ("package_binding".into(), string(&authority.canonical())),
            ])
        );
        let witness = manifest
            .members()
            .iter()
            .min_by(|a, b| a.path.cmp(&b.path))
            .ok_or("project raw support missing")?;
        role_support(
            project.source_handle_ids(),
            project.evidence_ids(),
            &[witness.source_handle.handle_id()],
            &[witness.evidence.evidence_id()],
        );
        assert_eq!(project.confidence(), GraphConfidence::Proven);
        for package in source.packages() {
            edge(
                "source_project_contains_package",
                project.proposal_id(),
                &package.proposal_id,
                &[witness.source_handle.handle_id(), package.source_handle_id],
                &[witness.evidence.evidence_id(), package.evidence_id],
                GraphConfidence::Proven,
            )?;
        }
        for file in source.files() {
            edge(
                "source_project_contains_file",
                project.proposal_id(),
                &file.proposal_id,
                &[witness.source_handle.handle_id(), file.source_handle_id],
                &[witness.evidence.evidence_id(), file.evidence_id],
                GraphConfidence::Proven,
            )?;
        }
        for member in manifest.members() {
            edge(
                "source_project_contains_raw_member",
                project.proposal_id(),
                &member.proposal_id,
                &[
                    witness.source_handle.handle_id(),
                    member.source_handle.handle_id(),
                ],
                &[
                    witness.evidence.evidence_id(),
                    member.evidence.evidence_id(),
                ],
                GraphConfidence::Proven,
            )?;
            for declaration in &declarations {
                if member
                    .path
                    .strip_prefix(declaration.root())
                    .is_some_and(|tail| tail.starts_with('/'))
                {
                    let package = source
                        .packages()
                        .iter()
                        .find(|row| row.package == declaration.name())
                        .ok_or("original root package missing")?;
                    edge(
                        "source_package_contains_raw_member",
                        &package.proposal_id,
                        &member.proposal_id,
                        &[package.source_handle_id, member.source_handle.handle_id()],
                        &[package.evidence_id, member.evidence.evidence_id()],
                        GraphConfidence::Proven,
                    )?;
                }
            }
        }

        let load = packages.load_plan();
        for node in load.packages() {
            let package = source
                .packages()
                .iter()
                .find(|row| row.package == node.package)
                .ok_or("selected package missing")?;
            let plan = load
                .package_plan(&node.package)
                .ok_or("selected plan missing")?;
            let fact = source
                .toc_facts()
                .iter()
                .find(|row| {
                    row.package.as_deref() == Some(node.package.as_str())
                        && matches!(row.kind, ProjectTocFactKind::Package { .. })
                })
                .ok_or("TOC package fact missing")?;
            let manifest = role(
                toc.batch(),
                "source_toc_manifest",
                &[("document", string(&fact.selected_toc))],
            )?;
            assert_eq!(
                manifest.semantic_key(),
                &BTreeMap::from([
                    ("document".into(), string(&fact.selected_toc)),
                    ("plan_digest".into(), string(&plan.digest().canonical())),
                ])
            );
            let variant = role(
                toc.batch(),
                "source_toc_variant",
                &[("document", string(&fact.selected_toc))],
            )?;
            assert_eq!(
                variant.semantic_key(),
                &BTreeMap::from([
                    ("document".into(), string(&fact.selected_toc)),
                    (
                        "flavor".into(),
                        GraphProposalValue::Identifier(fact.flavor.clone().into())
                    ),
                    (
                        "selected_root".into(),
                        GraphProposalValue::Boolean(node.selected_root)
                    ),
                    ("load_on_demand".into(), identifier(&node.load_on_demand)?),
                    ("static_phase".into(), identifier(&node.phase)?),
                    (
                        "order_group".into(),
                        GraphProposalValue::Integer(i64::try_from(package.order_group)?)
                    ),
                    ("reachability".into(), identifier(&node.reachability)?),
                ])
            );
            for proposal in [manifest, variant] {
                role_support(
                    proposal.source_handle_ids(),
                    proposal.evidence_ids(),
                    &[fact.source_handle_id],
                    &[fact.evidence_id],
                );
                assert_eq!(proposal.confidence(), GraphConfidence::Derived);
            }
            edge(
                "source_package_selects_toc",
                &package.proposal_id,
                manifest.proposal_id(),
                &[fact.source_handle_id],
                &[fact.evidence_id],
                GraphConfidence::Derived,
            )?;
            edge(
                "source_toc_defines_variant",
                manifest.proposal_id(),
                variant.proposal_id(),
                &[fact.source_handle_id],
                &[fact.evidence_id],
                GraphConfidence::Derived,
            )?;
            let mut previous = None;
            for record in plan.records().iter().filter(|record| {
                record.document == plan.selected_toc()
                    && matches!(
                        record.kind,
                        LoadRecordKind::LuaFile | LoadRecordKind::XmlFile
                    )
            }) {
                let fact = source
                    .toc_facts()
                    .iter()
                    .find(|row| {
                        row.package.as_deref() == Some(node.package.as_str())
                            && row.ordinal == record.ordinal
                            && matches!(row.kind, ProjectTocFactKind::File { .. })
                    })
                    .ok_or("selected file fact missing")?;
                let ProjectTocFactKind::File {
                    path,
                    file_kind,
                    bootstrap,
                    conditions,
                    repeated,
                    ..
                } = &fact.kind
                else {
                    return Err("expected TOC file fact".into());
                };
                assert_eq!(
                    fact.span,
                    SourceSpan::byte_range(record.byte_start, record.byte_end)?
                );
                assert_eq!(fact.selection, record.selection);
                assert_eq!(fact.selection, LoadSelection::Included);
                assert_eq!(*file_kind, record.kind);
                assert_eq!(*conditions, record.conditions);
                assert_eq!(*bootstrap, record.bootstrap);
                let unit = load
                    .units()
                    .iter()
                    .find(|unit| {
                        unit.package == node.package
                            && unit.document == record.document
                            && unit.source_ordinal == record.ordinal
                    })
                    .ok_or("native TOC unit witness missing")?;
                assert_eq!(unit.reachability, ProjectPackageReachability::Reachable);
                let conditions_digest = ContentDigest::<CanonicalResult>::from_bytes(
                    domain_separated_digest("wow-project/platform-toc-conditions/1", conditions)?,
                );
                let unit_witness = serde_json::json!({"state": "admitted", "unit": unit});
                let unit_digest =
                    ContentDigest::<CanonicalResult>::from_bytes(domain_separated_digest(
                        "wow-project/platform-toc-load-unit-witness/1",
                        &unit_witness,
                    )?);
                let occurrence = role(
                    toc.batch(),
                    "source_toc_load_occurrence",
                    &[("fact_id", string(&fact.fact_id))],
                )?;
                assert_eq!(
                    occurrence.semantic_key(),
                    &BTreeMap::from([
                        ("fact_id".into(), string(&fact.fact_id)),
                        ("document".into(), string(&fact.selected_toc)),
                        (
                            "ordinal".into(),
                            GraphProposalValue::Integer(i64::try_from(fact.ordinal)?)
                        ),
                        ("file_kind".into(), identifier(file_kind)?),
                        ("selection".into(), identifier(&fact.selection)?),
                        (
                            "conditions_digest".into(),
                            string(&conditions_digest.canonical())
                        ),
                        ("repeated".into(), GraphProposalValue::Boolean(*repeated)),
                        ("bootstrap".into(), GraphProposalValue::Boolean(*bootstrap)),
                        ("load_unit_witness".into(), string(&unit_digest.canonical())),
                        ("static_phase".into(), identifier(&unit.phase)?),
                        (
                            "order_group".into(),
                            GraphProposalValue::Integer(i64::try_from(unit.order_group)?)
                        ),
                        ("reachability".into(), identifier(&node.reachability)?),
                    ])
                );
                role_support(
                    occurrence.source_handle_ids(),
                    occurrence.evidence_ids(),
                    &[fact.source_handle_id],
                    &[fact.evidence_id],
                );
                edge(
                    "source_toc_contains_occurrence",
                    variant.proposal_id(),
                    occurrence.proposal_id(),
                    &[fact.source_handle_id],
                    &[fact.evidence_id],
                    GraphConfidence::Derived,
                )?;
                let target = load
                    .source_path(&node.package, &unit.target)
                    .ok_or("qualified TOC target missing")?;
                assert_eq!(path.as_deref(), Some(target.as_str()));
                let file = source
                    .files()
                    .iter()
                    .find(|file| file.path == target)
                    .ok_or("TOC target file missing")?;
                edge(
                    "source_toc_occurrence_loads",
                    occurrence.proposal_id(),
                    &file.proposal_id,
                    &[fact.source_handle_id],
                    &[fact.evidence_id],
                    GraphConfidence::Derived,
                )?;
                if let Some((prior, prior_handle, prior_evidence)) = previous {
                    edge(
                        "source_toc_occurs_before",
                        prior,
                        occurrence.proposal_id(),
                        &[prior_handle, fact.source_handle_id],
                        &[prior_evidence, fact.evidence_id],
                        GraphConfidence::Derived,
                    )?;
                }
                previous = Some((
                    occurrence.proposal_id(),
                    fact.source_handle_id,
                    fact.evidence_id,
                ));
            }
        }
        assert_eq!(
            toc.batch()
                .entity_proposals()
                .iter()
                .filter(|p| p.entity_kind_id() == "source_toc_manifest")
                .count(),
            load.packages().len()
        );
        assert_eq!(
            toc.batch()
                .entity_proposals()
                .iter()
                .filter(|p| p.entity_kind_id() == "source_toc_variant")
                .count(),
            load.packages().len()
        );
        assert_eq!(
            toc.batch()
                .entity_proposals()
                .iter()
                .filter(|p| p.entity_kind_id() == "source_toc_load_occurrence")
                .count(),
            source
                .toc_facts()
                .iter()
                .filter(|f| matches!(f.kind, ProjectTocFactKind::File { .. }))
                .count()
        );
        // This fixture has no declared load-policy, parent-name or XML load site.
        assert!(
            !source
                .toc_facts()
                .iter()
                .any(|f| matches!(f.kind, ProjectTocFactKind::LoadOnDemand { .. }))
        );
        assert!(
            !toc.batch()
                .entity_proposals()
                .iter()
                .any(|p| p.entity_kind_id() == "source_toc_load_policy")
        );

        let mut occurrence_nodes = BTreeMap::<&str, Vec<_>>::new();
        let mut documents = BTreeSet::new();
        let mut scripts = BTreeSet::new();
        for row in source.xml_containment() {
            let package = row
                .scope
                .package
                .as_deref()
                .ok_or("XML package scope missing")?;
            let plan = load.package_plan(package).ok_or("XML plan missing")?;
            let (local, index) = plan
                .xml_documents()
                .iter()
                .find(|(local, _)| {
                    load.source_path(package, local).as_deref() == Some(row.document.as_str())
                })
                .ok_or("native XML document missing")?;
            let element = index
                .elements()
                .iter()
                .find(|element| element.occurrence_id == row.occurrence_id)
                .ok_or("native XML occurrence missing")?;
            assert_eq!(element.parent_occurrence_id, row.parent_occurrence_id);
            assert_eq!(element.qualified_name, row.element_name);
            let scope_text = wow_core::canonical_json_string(&row.scope)?;
            let fields = [
                ("scope", string(&scope_text)),
                ("document", string(&row.document)),
                ("document_digest", string(&index.digest().canonical())),
            ];
            let document = role(xml.batch(), "xml_source_document", &fields)?;
            let file = source
                .files()
                .iter()
                .find(|file| file.path == row.document)
                .ok_or("XML file missing")?;
            assert_eq!(
                document.semantic_key().get("content_digest"),
                Some(&string(&file.content_digest.canonical()))
            );
            role_support(
                document.source_handle_ids(),
                document.evidence_ids(),
                &[file.source_handle_id],
                &[file.evidence_id],
            );
            if documents.insert(document.proposal_id()) {
                edge(
                    "xml_file_contains_document",
                    &file.proposal_id,
                    document.proposal_id(),
                    &[file.source_handle_id],
                    &[file.evidence_id],
                    GraphConfidence::Proven,
                )?;
            }
            let mut occurrence_fields = fields.to_vec();
            occurrence_fields.push(("occurrence", string(&row.occurrence_id)));
            let occurrence = role(xml.batch(), "xml_source_occurrence", &occurrence_fields)?;
            let serde_json::Value::String(role_name) = serde_json::to_value(element.role)? else {
                return Err("native XML role must be a string".into());
            };
            assert_eq!(
                occurrence.semantic_key().get("role"),
                Some(&string(&role_name))
            );
            assert_eq!(
                occurrence.semantic_key().get("source_handle"),
                Some(&string(&row.source_handle_id.canonical()))
            );
            role_support(
                occurrence.source_handle_ids(),
                occurrence.evidence_ids(),
                &[row.source_handle_id],
                &[row.evidence_id],
            );
            assert_eq!(occurrence.confidence(), GraphConfidence::Proven);
            let handle = source
                .source_handles()
                .get(&row.source_handle_id)
                .ok_or("XML support missing")?;
            assert_eq!(handle.path().as_str(), row.document);
            assert_eq!(handle.span(), row.span);
            assert_eq!(handle.span().byte_start(), Some(element.span.byte_start));
            assert_eq!(handle.span().byte_end(), Some(element.span.byte_end));
            assert_eq!(*handle.content_digest(), row.content_digest);
            assert_eq!(row.document_digest, index.digest());
            assert_eq!(row.content_digest, index.source_digest());
            edge(
                "xml_document_contains_occurrence",
                document.proposal_id(),
                occurrence.proposal_id(),
                &[row.source_handle_id],
                &[row.evidence_id],
                GraphConfidence::Proven,
            )?;
            let span = role(
                inventory.batch(),
                "source_span",
                &[("source_handle", string(&row.source_handle_id.canonical()))],
            )?;
            edge(
                "xml_occurrence_source_span",
                occurrence.proposal_id(),
                span.proposal_id(),
                &[row.source_handle_id],
                &[row.evidence_id],
                GraphConfidence::Proven,
            )?;
            if let Some(parent_id) = &row.parent_occurrence_id {
                let parent = source
                    .xml_containment()
                    .iter()
                    .find(|parent| {
                        parent.scope == row.scope
                            && parent.document == row.document
                            && parent.occurrence_id == *parent_id
                    })
                    .ok_or("XML lexical parent missing")?;
                let mut parent_fields = fields.to_vec();
                parent_fields.push(("occurrence", string(parent_id)));
                let parent_role = role(xml.batch(), "xml_source_occurrence", &parent_fields)?;
                edge(
                    "xml_lexical_contains",
                    parent_role.proposal_id(),
                    occurrence.proposal_id(),
                    &[parent.source_handle_id, row.source_handle_id],
                    &[parent.evidence_id, row.evidence_id],
                    GraphConfidence::Proven,
                )?;
                assert!(
                    parent.span.byte_start() <= row.span.byte_start()
                        && parent.span.byte_end() >= row.span.byte_end()
                );
            }
            if element.script.is_some() {
                let fact = source
                    .xml_facts()
                    .iter()
                    .find(|fact| {
                        fact.scope == row.scope
                            && fact.document == row.document
                            && fact.occurrence_id == row.occurrence_id
                            && matches!(fact.kind, ProjectXmlFactKind::Script { .. })
                    })
                    .ok_or("native XML script fact missing")?;
                let script = role(xml.batch(), "xml_source_script_site", &occurrence_fields)?;
                assert_eq!(
                    script.semantic_key().get("state"),
                    Some(&string(&wow_core::canonical_json_string(&fact.kind)?))
                );
                role_support(
                    script.source_handle_ids(),
                    script.evidence_ids(),
                    &[fact.source_handle_id],
                    &[fact.evidence_id],
                );
                edge(
                    "xml_occurrence_owns_script_site",
                    occurrence.proposal_id(),
                    script.proposal_id(),
                    &[fact.source_handle_id],
                    &[fact.evidence_id],
                    GraphConfidence::Proven,
                )?;
                assert!(scripts.insert(script.proposal_id()));
            }
            assert!(!matches!(
                element.role,
                XmlElementRole::Include | XmlElementRole::Script
            ));
            let address = finished
                .assertion(&key(GraphAssertionKind::Entity, occurrence.proposal_id()))
                .ok_or("XML occurrence address missing")?;
            occurrence_nodes
                .entry(&row.occurrence_id)
                .or_default()
                .push((
                    package,
                    lookup
                        .entity(&scope, address, stop)?
                        .accepted()
                        .node()
                        .node_id()
                        .clone(),
                ));
            assert_eq!(
                load.source_path(package, local).as_deref(),
                Some(row.document.as_str())
            );
        }
        assert!(!occurrence_nodes.is_empty() && !scripts.is_empty());
        for scoped in occurrence_nodes.values() {
            assert_eq!(scoped.len(), 2);
            assert_ne!(scoped[0].0, scoped[1].0);
            assert_ne!(scoped[0].1, scoped[1].1);
        }
        assert_eq!(
            xml.batch()
                .entity_proposals()
                .iter()
                .filter(|p| p.entity_kind_id() == "xml_source_document")
                .count(),
            documents.len()
        );
        assert_eq!(
            xml.batch()
                .entity_proposals()
                .iter()
                .filter(|p| p.entity_kind_id() == "xml_source_occurrence")
                .count(),
            source.xml_containment().len()
        );
        assert_eq!(
            xml.batch()
                .entity_proposals()
                .iter()
                .filter(|p| p.entity_kind_id() == "xml_source_script_site")
                .count(),
            scripts.len()
        );
        assert!(
            !source
                .xml_facts()
                .iter()
                .any(|f| matches!(f.kind, ProjectXmlFactKind::Parent { .. }))
        );
        assert!(!xml.batch().entity_proposals().iter().any(|p| matches!(
            p.entity_kind_id(),
            "xml_source_load_site" | "xml_source_parent_reference"
        )));
    }
    let role_edges = edges
        .values()
        .filter(|proposal| {
            matches!(
                proposal.relation_kind_id(),
                "source_project_contains_package"
                    | "source_project_contains_file"
                    | "source_project_contains_raw_member"
                    | "source_package_contains_raw_member"
                    | "source_package_selects_toc"
                    | "source_toc_defines_variant"
                    | "source_toc_contains_occurrence"
                    | "source_toc_occurrence_loads"
                    | "source_toc_occurs_before"
                    | "source_toc_defines_load_policy"
                    | "xml_file_contains_document"
                    | "xml_document_contains_occurrence"
                    | "xml_lexical_contains"
                    | "xml_occurrence_source_span"
                    | "xml_occurrence_owns_script_site"
                    | "xml_occurrence_owns_load_site"
                    | "xml_include_target"
                    | "xml_external_script_target"
                    | "xml_occurrence_owns_parent_reference"
            )
        })
        .map(|proposal| proposal.proposal_id())
        .collect::<BTreeSet<_>>();
    assert_eq!(checked, role_edges);

    // Both older native owners stay usable; /2 is retained, not reconstructed.
    original.validate(stop)?;
    spans.validate(stop)?;
    let unchanged = build_platform_graph_proposal_plan_with_inventory_spans(view, stop)?;
    assert_eq!(
        unchanged.profile(),
        PLATFORM_DIRECT_GRAPH_WITH_INVENTORY_SPANS_PROFILE
    );
    assert_eq!(unchanged.registry(), spans.registry());
    assert_eq!(unchanged.foundation(), spans.foundation());
    assert_eq!(
        unchanged.scope().generation,
        *spans.foundation().generation()
    );
    assert_eq!(unchanged.inventory_span_omissions(), Some(omitted));
    Ok(())
}

#[test]
fn direct_package_stages_bind_native_predecessors_and_preserve_replay() -> ResultOf {
    let stop = AtomicBool::new(false);
    let root = FixtureRoot::new()?;
    let packages = root.packages(&stop)?;
    assert!(!root.0.exists());
    let legacy = publish(&packages, false, &stop)?;
    let publisher = publish(&packages, true, &stop)?;
    let legacy_view = legacy.open_current()?;
    let view = publisher.open_current()?;
    let legacy_archive = serde_json::to_vec(&ProjectReplay::capture(&legacy, &stop)?)?;
    let archive = serde_json::to_vec(&ProjectReplay::capture(&publisher, &stop)?)?;
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&legacy_archive)?["schema"],
        "wow-project/native-project-replay/6"
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&archive)?["schema"],
        "wow-project/native-project-replay/7"
    );
    let (registry, monolithic, coverage, provenance, limits) =
        build_source_graph_proposals(&view, &stop)?.into_parts();
    let (old_registry, old_batch, old_coverage, old_source, old_limits) =
        build_source_graph_proposals(&legacy_view, &stop)?.into_parts();
    let wrong_owner = GraphPartitionSnapshot::new(
        old_registry,
        GraphSnapshot::build(
            old_batch.universe().clone(),
            old_batch.generation().clone(),
            old_limits,
            Vec::new(),
            Vec::new(),
            old_coverage.clone(),
        )?,
        old_batch.source_context_id(),
        &stop,
    )?;
    let mut plan = build_platform_graph_proposal_plan(&view, &stop)?;
    assert_eq!(plan.registry(), &registry);
    assert_eq!(plan.limits(), limits);
    assert_eq!(
        plan.scope().source_context_id,
        provenance.context().context_id()
    );
    assert!(plan.raw_inventory_batch().is_none());
    assert_eq!(
        plan.producer_order(),
        &[
            PlatformGraphProducer::Inventory,
            PlatformGraphProducer::TocLoad,
            PlatformGraphProducer::AnalyzerStructure,
            PlatformGraphProducer::XmlStructure
        ]
    );
    let mut owner = initial(&plan, &stop)?;
    refused(plan.build_stage(PlatformGraphProducer::Inventory, &wrong_owner, &stop))?;
    refused(plan.build_stage(PlatformGraphProducer::TocLoad, &owner, &stop))?;
    let surplus = tombstone(&owner, &stop)?;
    assert!(
        surplus
            .partition("fixture:unrelated-empty")
            .ok_or("tombstone missing")?
            .report()
            .accepted_entities()
            .is_empty()
    );
    refused(plan.build_stage(PlatformGraphProducer::Inventory, &surplus, &stop))?;
    let mut prefixes = vec![owner.clone()];
    for &producer in plan.producer_order() {
        let stage = plan.build_stage(producer, &owner, &stop)?;
        assert_eq!(stage.producer(), producer);
        let version = stage.producer_version();
        let (batch, stage_coverage) = stage.into_parts();
        assert_eq!(
            plan.build_stage(producer, &owner, &stop)?.into_parts(),
            (batch.clone(), stage_coverage.clone())
        );
        if producer == PlatformGraphProducer::Inventory {
            // Building a pending stage never substitutes for native admission.
            refused(plan.build_stage(PlatformGraphProducer::TocLoad, &owner, &stop))?;
            let different_current =
                admit(&owner, batch.clone(), "2", stage_coverage.clone(), &stop)?;
            refused(plan.build_stage(PlatformGraphProducer::TocLoad, &different_current, &stop))?;
        }
        owner = admit(&owner, batch, version, stage_coverage, &stop)?;
        let partition = owner
            .partition(producer.partition_id())
            .ok_or("stage partition missing")?;
        assert!(partition.report().rejections().is_empty());
        prefixes.push(owner.clone());
    }
    assert_eq!(owner.partitions().len(), 4);
    let overlay = admit(
        &owner,
        monolithic.clone(),
        env!("CARGO_PKG_VERSION"),
        coverage.clone(),
        &stop,
    )?;
    assert!(overlay.partition(SOURCE_GRAPH_PARTITION).is_some());
    refused(replan(&view, &prefixes, &stop)?.finish(&overlay, &stop))?;
    let surplus = tombstone(&owner, &stop)?;
    refused(replan(&view, &prefixes, &stop)?.finish(&surplus, &stop))?;
    let finished = plan.finish(&owner, &stop)?;
    assert_eq!(finished.project().snapshot_id(), view.snapshot_id());
    assert!(std::ptr::eq(finished.graph(), &owner));
    assert_eq!(finished.source(), &provenance);
    assert_eq!(finished.evidence_catalog().context(), provenance.context());
    for (id, handle) in provenance.source_handles() {
        assert_eq!(finished.evidence_catalog().source_handle(id), Some(handle));
        assert_eq!(
            view.source_handle(
                handle.path().as_str(),
                handle.span(),
                handle.entity_key().cloned()
            )?,
            *handle
        );
    }
    for (id, evidence) in provenance.evidence() {
        assert_eq!(finished.evidence_catalog().evidence(id), Some(evidence));
    }
    let lookup = owner.producer_lookup(&stop)?;
    let mut external_relations = BTreeSet::new();
    let mut external_derivations = 0;
    let mut local_inheritance = 0;
    for partition in owner.partitions() {
        for accepted in partition.report().accepted_entities() {
            let assertion = key(GraphAssertionKind::Entity, accepted.proposal_id());
            let reference = finished
                .assertion(&assertion)
                .ok_or("entity address missing")?;
            let resolved = lookup.entity(finished.scope(), reference, &stop)?;
            assert_eq!(resolved.reference(), *reference);
            assert_eq!(resolved.partition(), partition);
            assert_eq!(resolved.accepted(), accepted);
            assert_eq!(
                resolved.proposal(),
                monolithic
                    .entity_proposal(accepted.proposal_id())
                    .ok_or("original entity missing")?
            );
        }
        for accepted in partition.report().accepted_relations() {
            let assertion = key(GraphAssertionKind::Relation, accepted.proposal_id());
            let reference = finished
                .assertion(&assertion)
                .ok_or("relation address missing")?;
            let resolved = lookup.relation(finished.scope(), reference, &stop)?;
            assert_eq!(resolved.reference(), *reference);
            assert_eq!(resolved.partition(), partition);
            assert_eq!(resolved.accepted(), accepted);
            let original = monolithic
                .relation_proposal(accepted.proposal_id())
                .ok_or("original relation missing")?;
            assert_eq!(resolved.proposal().confidence(), original.confidence());
            assert_eq!(
                resolved.proposal().source_handle_ids(),
                original.source_handle_ids()
            );
            assert_eq!(resolved.proposal().evidence_ids(), original.evidence_ids());
            for (actual, original) in resolved
                .proposal()
                .endpoints()
                .into_iter()
                .zip(original.endpoints())
            {
                let GraphProposalEndpoint::Proposed(id) = original else {
                    return Err("original endpoint must be Proposed".into());
                };
                let endpoint = finished
                    .assertion(&key(GraphAssertionKind::Entity, id))
                    .ok_or("endpoint address missing")?;
                let entity = lookup.entity(finished.scope(), endpoint, &stop)?;
                match actual {
                    GraphProposalEndpoint::Existing(node) => {
                        assert_eq!(node, entity.accepted().node().node_id());
                        assert!(lookup.input_view().node(node).is_some());
                        assert_ne!(entity.partition().partition_id(), partition.partition_id());
                        external_relations.insert(partition.partition_id());
                    }
                    GraphProposalEndpoint::Proposed(local) => {
                        assert_eq!(local, id);
                        assert_eq!(entity.partition().partition_id(), partition.partition_id());
                    }
                }
            }
            if resolved.proposal().relation_kind_id() == "source_xml_inherits" {
                assert!(
                    resolved
                        .proposal()
                        .endpoints()
                        .into_iter()
                        .all(|endpoint| matches!(endpoint, GraphProposalEndpoint::Proposed(_)))
                );
                local_inheritance += 1;
            }
        }
        let records = partition
            .batch()
            .assertion_records()
            .ok_or("native derivations missing")?;
        assert_eq!(&records.scope, finished.scope());
        for derivation in &records.derivations {
            let original = monolithic
                .assertion_records()
                .ok_or("original derivations missing")?
                .derivations
                .iter()
                .find(|record| record.output == derivation.output)
                .ok_or("original derivation missing")?;
            assert_eq!(derivation.rule_id, original.rule_id);
            assert_eq!(derivation.rule_version, 1);
            assert!(derivation.rebuttals.is_empty() && derivation.missing.is_empty());
            assert!(finished.assertion(&derivation.output).is_some());
            for input in &derivation.inputs {
                if let GraphAssertionRef::Producer { assertion, .. } = input {
                    assert_eq!(finished.assertion(assertion), Some(input));
                    let resolved = lookup.entity(finished.scope(), input, &stop)?;
                    assert_ne!(
                        resolved.partition().partition_id(),
                        partition.partition_id()
                    );
                    external_derivations += 1;
                }
            }
        }
    }
    for producer in [
        PlatformGraphProducer::TocLoad,
        PlatformGraphProducer::AnalyzerStructure,
        PlatformGraphProducer::XmlStructure,
    ] {
        assert!(external_relations.contains(producer.partition_id()));
    }
    assert!(external_derivations > 0 && local_inheritance >= 2);
    assert!(provenance.package_loads().iter().any(|load| matches!(
        load.outcome,
        ProjectGraphPackageLoadOutcome::Projected { .. }
    )));
    assert!(
        provenance
            .xml_inheritance()
            .iter()
            .any(|row| row.outcome == ProjectGraphXmlReferenceOutcome::Unresolved)
    );
    let report = view
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("compiled function report missing")?;
    assert!(!provenance.functions().is_empty());
    for function in provenance.functions() {
        let native = report
            .functions()
            .iter()
            .find(|f| f.fact_id() == function.function_id)
            .ok_or("native callable missing")?;
        assert_eq!(native.path(), function.path);
        assert_eq!(native.span(), function.span);
        let handle = provenance
            .source_handles()
            .get(&function.source_handle_id)
            .ok_or("callable support missing")?;
        assert_eq!(handle.span(), native.span());
        let file = provenance
            .files()
            .iter()
            .find(|file| file.path == function.path)
            .ok_or("callable file missing")?;
        assert_eq!(handle.content_digest(), &file.content_digest);
        let records = owner
            .partition(PlatformGraphProducer::AnalyzerStructure.partition_id())
            .ok_or("analyzer stage missing")?
            .batch()
            .assertion_records()
            .ok_or("analyzer records missing")?;
        let derivation = records
            .derivations
            .iter()
            .find(|d| d.output == key(GraphAssertionKind::Entity, &function.proposal_id))
            .ok_or("callable derivation missing")?;
        let input = finished
            .assertion(&key(GraphAssertionKind::Entity, &file.proposal_id))
            .ok_or("file address missing")?;
        assert!(derivation.inputs.contains(input));
    }
    let declaration = packages
        .load_plan()
        .package_plan("Alpha")
        .ok_or("Alpha plan missing")?
        .xml_references()
        .declarations()
        .values()
        .find(|row| row.name.as_deref() == Some("Receiver"))
        .ok_or("Receiver missing")?;
    let duplicates = provenance
        .xml_declarations()
        .iter()
        .filter(|row| row.occurrence_id == declaration.occurrence_id)
        .collect::<Vec<_>>();
    assert_eq!(duplicates.len(), 2);
    assert_ne!(duplicates[0].path, duplicates[1].path);
    assert_ne!(
        finished.assertion(&key(GraphAssertionKind::Entity, &duplicates[0].proposal_id)),
        finished.assertion(&key(GraphAssertionKind::Entity, &duplicates[1].proposal_id))
    );

    // Native replay retains the selected recipe; direct staging has no project effects.
    assert_eq!(
        serde_json::to_vec(&ProjectReplay::capture(&publisher, &stop)?)?,
        archive
    );
    assert_eq!(
        serde_json::to_vec(&ProjectReplay::capture(&legacy, &stop)?)?,
        legacy_archive
    );
    let restored = ProjectReplay::from_json(&archive, &stop)?.hydrate(&stop)?;
    assert_eq!(restored.snapshot_id(), view.snapshot_id());
    assert_eq!(restored.analyzer_snapshot_id(), view.analyzer_snapshot_id());
    let (_, restored_batch, restored_coverage, restored_source, _) =
        build_source_graph_proposals(&restored, &stop)?.into_parts();
    assert_eq!(restored_batch, monolithic);
    assert_eq!(restored_coverage, coverage);
    assert_eq!(restored_source, provenance);
    let mut replay_plan = build_platform_graph_proposal_plan(&restored, &stop)?;
    let mut replay_owner = initial(&replay_plan, &stop)?;
    for &producer in replay_plan.producer_order() {
        let stage = replay_plan.build_stage(producer, &replay_owner, &stop)?;
        let version = stage.producer_version();
        let (batch, coverage) = stage.into_parts();
        replay_owner = admit(&replay_owner, batch, version, coverage, &stop)?;
    }
    assert_eq!(replay_owner, owner);
    let replay_finished = replay_plan.finish(&replay_owner, &stop)?;
    assert_eq!(replay_finished.source(), finished.source());
    let legacy_restored = ProjectReplay::from_json(&legacy_archive, &stop)?.hydrate(&stop)?;
    assert_eq!(legacy_restored.snapshot_id(), legacy_view.snapshot_id());
    let (_, unchanged, unchanged_coverage, unchanged_source, _) =
        build_source_graph_proposals(&legacy_restored, &stop)?.into_parts();
    assert_eq!(unchanged, old_batch);
    assert_eq!(unchanged_coverage, old_coverage);
    assert_eq!(unchanged_source, old_source);

    // The same admitted bytes also feed the selected raw prelude and four stages.
    let raw_publisher = publish_with_graph_profile(
        &packages,
        Some(PlatformGraphProfile::PackageProjectionWithRawInventoryV1),
        &stop,
    )?;
    let raw_view = raw_publisher.open_current()?;
    let raw_archive = serde_json::to_vec(&ProjectReplay::capture(&raw_publisher, &stop)?)?;
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&raw_archive)?["schema"],
        "wow-project/native-project-replay/8"
    );
    let raw_proposals = build_source_graph_proposals(&raw_view, &stop)?;
    let raw_batch = raw_proposals
        .inventory_batch()
        .ok_or("original raw batch missing")?
        .clone();
    let (_, raw_monolithic, raw_coverage, raw_source, _) = raw_proposals.into_parts();
    let mut raw_plan = build_platform_graph_proposal_plan(&raw_view, &stop)?;
    assert_eq!(raw_plan.raw_inventory_batch(), Some(&raw_batch));
    let mut raw_owner = initial(&raw_plan, &stop)?;
    assert_eq!(raw_owner.partitions().len(), 1);
    let raw_partition_id = wow_project::graph::PLATFORM_RAW_INVENTORY_PARTITION;
    let prelude = raw_owner
        .partition(raw_partition_id)
        .ok_or("native raw prelude missing")?;
    assert_eq!(prelude.batch(), &raw_batch);
    assert_eq!(
        prelude.producer_version(),
        raw_plan.raw_inventory_producer_version()
    );
    assert!(prelude.coverage().is_empty() && prelude.report().rejections().is_empty());
    assert_eq!(prelude.report().accepted_entities().len(), 6);
    for &producer in raw_plan.producer_order() {
        let stage = raw_plan.build_stage(producer, &raw_owner, &stop)?;
        let version = stage.producer_version();
        let (batch, coverage) = stage.into_parts();
        raw_owner = admit(&raw_owner, batch, version, coverage, &stop)?;
    }
    assert_eq!(raw_owner.partitions().len(), 5);
    let raw_finished = raw_plan.finish(&raw_owner, &stop)?;
    assert_eq!(raw_finished.source(), &raw_source);
    let manifest = raw_finished
        .source()
        .raw_inventory()
        .ok_or("raw manifest missing")?;
    assert_eq!(manifest.members().len(), 6);
    let raw_lookup = raw_owner.producer_lookup(&stop)?;
    for member in manifest.members() {
        let reference = raw_finished
            .assertion(&key(GraphAssertionKind::Entity, &member.proposal_id))
            .ok_or("raw member address missing")?;
        let resolved = raw_lookup.entity(raw_finished.scope(), reference, &stop)?;
        assert_eq!(resolved.reference(), *reference);
        assert_eq!(resolved.partition().partition_id(), raw_partition_id);
        assert_eq!(resolved.partition().batch(), &raw_batch);
        assert_eq!(
            resolved.proposal(),
            raw_batch
                .entity_proposal(&member.proposal_id)
                .ok_or("raw proposal missing")?
        );
        assert_eq!(
            raw_finished
                .evidence_catalog()
                .source_handle(&member.source_handle.handle_id()),
            Some(&member.source_handle)
        );
        assert_eq!(
            raw_finished
                .evidence_catalog()
                .evidence(&member.evidence.evidence_id()),
            Some(&member.evidence)
        );
    }
    let spans_owner = inventory_spans(&raw_view, &raw_owner, &raw_source, &stop)?;
    structural_roles(&raw_view, &raw_owner, &spans_owner, &raw_source, &stop)?;
    assert!(!root.0.exists());
    let unchanged_raw = build_source_graph_proposals(&raw_view, &stop)?;
    assert_eq!(unchanged_raw.inventory_batch(), Some(&raw_batch));
    let (_, unchanged_raw_batch, unchanged_raw_coverage, unchanged_raw_source, _) =
        unchanged_raw.into_parts();
    assert_eq!(unchanged_raw_batch, raw_monolithic);
    assert_eq!(unchanged_raw_coverage, raw_coverage);
    assert_eq!(unchanged_raw_source, raw_source);
    assert_eq!(
        serde_json::to_vec(&ProjectReplay::capture(&raw_publisher, &stop)?)?,
        raw_archive
    );
    Ok(())
}
