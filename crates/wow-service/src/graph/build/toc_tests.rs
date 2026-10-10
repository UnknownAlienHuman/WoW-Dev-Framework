use super::*;
use std::error::Error;
use wow_core::{
    CanonicalResult, ContentDigest, ProfileIdentityBuilder, ProfileKind, ReferenceGenerationId,
    SchemaVersionEntry, SourceKind, SourceLogicalSnapshot, ToolVersion,
};
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
};
use wow_project::disk::{ProjectDiskFile, ProjectInputDirectory};
use wow_project::load::{ProjectPackageInput, ProjectPackageVariantInput, TocLoadContext};
use wow_project::{
    AnalyzerBindingDeclaration, ProjectBudgetPolicy, ProjectCapabilityPolicy,
    ProjectConfigurationBuilder, ProjectId, ProjectInputBundle, ProjectKind, ProjectPublisher,
    ProjectSourceOriginId, ProjectWorkspaceId,
};

#[test]
fn named_toc_families_publish_through_matcher_with_exact_graph_readback()
-> Result<(), Box<dyn Error>> {
    let stop = AtomicBool::new(false);
    let profile = ProfileIdentityBuilder::new(
        "profile:fixture:toc-pipeline-v1".parse()?,
        ProfileKind::Fixture,
        "retail",
        120_100,
        SourceKind::SyntheticFixture,
        "027d26c3406d3de2cbd2b1f67d468fe033a1bcd4",
        ContentDigest::<SourceLogicalSnapshot>::from_bytes([7; 32]),
    )
    .schema_versions(vec![SchemaVersionEntry::new(
        "schema:wow:fixture-e0".parse()?,
        ToolVersion::parse("1.0.0")?,
    )])
    .fixture_scope("wow-toc-pipeline-fixture")
    .build()?;
    let directory = ProjectInputDirectory::open(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/toc-pipeline"),
    )?;
    let packages = ["Main", "Required", "Optional"].map(|name| {
        ProjectPackageInput::new(
            name,
            name,
            name == "Main",
            vec![ProjectPackageVariantInput::new(
                ProjectDiskFile::new(format!("{name}.toc")),
                true,
            )],
        )
    });
    let context = TocLoadContext {
        game_types: std::collections::BTreeMap::from([("retail".into(), true)]),
        family: None,
        game: None,
        text_locale: None,
        location: None,
        environment: None,
    };
    let (files, load, main) = directory
        .read_package_project_with_context(&packages, &profile, Some(&context), &stop)?
        .into_namespaced_main()?
        .into_parts();
    let backend = EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(EMMYLUA_CODE_ANALYSIS_VERSION),
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        format!("sha256:{}", "1".repeat(64)),
        format!("sha256:{}", "2".repeat(64)),
    )?;
    let library = LuaWorkspaceSnapshot::build(
        backend.clone(),
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "library/fixture.lua",
            "---@meta _\n",
        )],
        LuaWorkspaceLimits::new(8, 16_384, 256 * 1024, 512 * 1024)?,
    )?;
    let binding = AnalyzerBindingDeclaration::new(
        "wow-emmy/e0-c/1",
        format!("emmy-pin:{EMMYLUA_REVISION}"),
        backend.compatibility_report_sha256().to_owned(),
        ContentDigest::<CanonicalResult>::from_bytes([3; 32]),
        "wow-emmy/e0-c/1",
        "wow-emmy-e0-c-library-v1",
        backend,
    )?;
    let config = ProjectConfigurationBuilder::new(
        ProjectId::new("fixture-toc-pipeline")?,
        ProjectKind::Fixture,
        profile,
        ReferenceGenerationId::derive(&"fixture-toc-reference")?,
        binding,
    )
    .workspace_id(ProjectWorkspaceId::new("workspace:toc-pipeline")?)
    .source_origin_id(ProjectSourceOriginId::new("project-origin:toc-pipeline")?)
    .logical_root("fixtures/toc-pipeline")
    .capability_policy(ProjectCapabilityPolicy::strict_e0()?)
    .budget_policy(ProjectBudgetPolicy::fixture_e0()?)
    .package_load_plan(&load, &main)?
    .build()?;
    let mut publisher = ProjectPublisher::new();
    publisher.publish_initial(ProjectInputBundle::closed(config, files, vec![library])?)?;
    let project = publisher.open_current()?;
    let (registry, batch, coverage, provenance, limits) =
        build_source_graph_proposals(&project, &stop)?.into_parts();
    let foundation = GraphSnapshot::build(
        batch.universe().clone(),
        batch.generation().clone(),
        limits,
        Vec::new(),
        Vec::new(),
        coverage.clone(),
    )?;
    let owner =
        GraphPartitionSnapshot::new(registry, foundation, batch.source_context_id(), &stop)?;
    let source = owner.prepare_replacement(
        GraphPartitionReplacement {
            expected_snapshot_id: owner.snapshot().snapshot_id().clone(),
            expected_partition_digest: None,
            producer_version: env!("CARGO_PKG_VERSION").into(),
            batch,
            coverage,
        },
        &stop,
    )?;
    let (snapshot, recognition) = toc::publish(source.candidate(), &provenance, &stop)
        .map_err(|error| std::io::Error::other(format!("TOC publication: {error}")))?;
    snapshot.validate(&stop)?;
    let addresses = SourceGraphAddressCrosswalk::legacy(&provenance, source.candidate(), &stop)?;
    let topology = toc::maps(&snapshot, &addresses, &recognition, &stop)
        .map_err(|error| std::io::Error::other(format!("TOC crosswalk: {error}")))?;
    assert_eq!(recognition.len(), 5);
    for family in wow_recognizers::source_toc::SourceTocFamily::ALL {
        let partition = snapshot
            .partition(family.partition_id())
            .ok_or("missing TOC family partition")?;
        assert!(!partition.report().accepted_relations().is_empty());
        assert!(
            partition
                .coverage()
                .iter()
                .all(|coverage| !coverage.negative_authority())
        );
        assert!(
            recognition
                .iter()
                .any(|r| r.family == family && !r.receipts.is_empty())
        );
    }
    for relation in [
        wow_graph::GraphRelationKind::Contains,
        wow_graph::GraphRelationKind::Defines,
        wow_graph::GraphRelationKind::Loads,
        wow_graph::GraphRelationKind::LoadsBefore,
        wow_graph::GraphRelationKind::DependsOn,
        wow_graph::GraphRelationKind::OptionalDependsOn,
        wow_graph::GraphRelationKind::Owns,
    ] {
        assert!(
            snapshot
                .snapshot()
                .edges()
                .iter()
                .any(|edge| edge.relation() == relation)
        );
    }
    assert!(!topology.nodes.is_empty());
    assert!(!topology.edges.is_empty());
    let missing = provenance
        .toc_facts()
        .iter()
        .find(|fact| {
            matches!(&fact.kind,
        wow_project::graph::ProjectTocFactKind::Dependency { name, .. } if name == "MissingAddon")
        })
        .ok_or("missing unresolved dependency fact")?;
    assert!(
        recognition
            .iter()
            .flat_map(|item| &item.omissions)
            .any(|omission| omission.blocker == "toc.dependency_unresolved"
                && omission.fact_ids.contains(&missing.fact_id))
    );
    for node in &topology.nodes {
        assert!(snapshot.snapshot().node(&node.node_id).is_some());
    }
    for item in &topology.edges {
        let edge = snapshot
            .snapshot()
            .edge(&item.edge_id)
            .ok_or("unmaterialized TOC edge")?;
        assert_eq!(edge.from(), &item.from_node_id);
        assert_eq!(edge.to(), &item.to_node_id);
        assert_eq!(edge.relation(), item.relation);
        assert_eq!(edge.confidence(), item.confidence);
        assert!(!edge.evidence_ids().is_empty());
    }
    let saved = snapshot
        .partition(wow_recognizers::source_toc::SourceTocFamily::SavedVariables.partition_id())
        .ok_or("missing SavedVariables partition")?;
    assert_eq!(saved.batch().entity_proposals().len(), 1);
    let root = &saved.batch().entity_proposals()[0];
    for fact in provenance.toc_facts().iter().filter(|fact| {
        matches!(&fact.kind,
        wow_project::graph::ProjectTocFactKind::SavedVariable { name, .. } if name == "PipelineDB")
    }) {
        assert!(root.evidence_ids().contains(&fact.evidence_id));
        assert!(root.source_handle_ids().contains(&fact.source_handle_id));
    }
    for item in &recognition {
        for evaluation in &item.evaluations {
            evaluation.fact_bundle.validate(
                provenance.context(),
                wow_recognizers::RecognizerFactLimits::default(),
            )?;
            evaluation.output.validate()?;
            for proposal in evaluation
                .output
                .outcomes()
                .iter()
                .flat_map(|outcome| outcome.proposals())
            {
                let coverage_ids = match proposal {
                    wow_recognizers::RecognizerProposedAssertion::Entity {
                        coverage_ids, ..
                    }
                    | wow_recognizers::RecognizerProposedAssertion::Relation {
                        coverage_ids, ..
                    } => coverage_ids,
                };
                for id in coverage_ids {
                    assert!(
                        evaluation
                            .fact_bundle
                            .coverage()
                            .iter()
                            .any(|record| record.coverage_id() == *id)
                    );
                }
            }
        }
    }
    wow_core::canonical_json_bytes(&recognition)?;
    wow_core::canonical_json_bytes(&topology)?;
    Ok(())
}
