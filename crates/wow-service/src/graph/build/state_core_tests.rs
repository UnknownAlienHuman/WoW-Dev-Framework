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
use wow_graph::{GraphPartitionReplacement, GraphPartitionSnapshot, GraphSnapshot};
use wow_project::disk::{ProjectDiskFile, ProjectInputDirectory};
use wow_project::graph::build_source_graph_proposals;
use wow_project::load::TocLoadContext;
use wow_project::{
    AnalyzerBindingDeclaration, ProjectBudgetPolicy, ProjectCapabilityPolicy,
    ProjectConfigurationBuilder, ProjectId, ProjectInputBundle, ProjectKind, ProjectPublisher,
    ProjectSourceOriginId, ProjectWorkspaceId,
};

fn state_core_profile() -> Result<wow_core::ProfileIdentity, Box<dyn Error>> {
    Ok(ProfileIdentityBuilder::new(
        "profile:fixture:state-core-pipeline-v1".parse()?,
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
    .fixture_scope("wow-state-core-pipeline-fixture")
    .build()?)
}

fn backend() -> Result<EmmyBackendIdentity, Box<dyn Error>> {
    Ok(EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(EMMYLUA_CODE_ANALYSIS_VERSION),
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        format!("sha256:{}", "1".repeat(64)),
        format!("sha256:{}", "2".repeat(64)),
    )?)
}

/// One state-only pipeline: the source owner projects both SavedVariables roots,
/// the legacy state recognizer joins analyzer global-slot accesses, TOC publication
/// supplies the variant owner, and the three core state rules then publish with the
/// final crosswalk. Static structure only; no runtime value or client authority.
#[test]
fn state_core_pipeline_publishes_roots_and_literal_paths() -> Result<(), Box<dyn Error>> {
    let stop = std::sync::atomic::AtomicBool::new(false);
    let directory = ProjectInputDirectory::open(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/state-core-pipeline"),
    )?;
    let (files, plan) = directory
        .read_toc_project_with_context(
            ".",
            &ProjectDiskFile::new("StateCore.toc"),
            &state_core_profile()?,
            Some(&TocLoadContext {
                game_types: std::collections::BTreeMap::from([("retail".into(), true)]),
                family: None,
                game: None,
                text_locale: None,
                location: None,
                environment: None,
            }),
            &stop,
        )?
        .into_parts();
    let identity = backend()?;
    let binding = AnalyzerBindingDeclaration::new(
        "wow-emmy/e0-c/1",
        format!("emmy-pin:{EMMYLUA_REVISION}"),
        identity.compatibility_report_sha256().to_owned(),
        ContentDigest::<CanonicalResult>::from_bytes([3; 32]),
        "wow-emmy/e0-c/1",
        "wow-emmy-e0-c-library-v1",
        identity.clone(),
    )?;
    let library = LuaWorkspaceSnapshot::build(
        identity,
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "library/state_core.lua",
            "---@meta _\n",
        )],
        LuaWorkspaceLimits::new(8, 16_384, 256 * 1024, 512 * 1024)?,
    )?;
    let config = ProjectConfigurationBuilder::new(
        ProjectId::new("fixture-state-core-pipeline")?,
        ProjectKind::Fixture,
        state_core_profile()?,
        ReferenceGenerationId::derive(&"fixture-state-core-reference")?,
        binding,
    )
    .workspace_id(ProjectWorkspaceId::new("workspace:state-core-pipeline")?)
    .source_origin_id(ProjectSourceOriginId::new(
        "project-origin:state-core-pipeline",
    )?)
    .logical_root("fixtures/state-core-pipeline")
    .capability_policy(ProjectCapabilityPolicy::strict_e0()?)
    .budget_policy(ProjectBudgetPolicy::fixture_e0()?)
    .load_plan(&plan)?
    .build()?;
    // The function/call fact sidecar must be enabled: legacy state publication
    // reads that report and would otherwise be skipped entirely.
    let mut publisher = ProjectPublisher::with_function_call_facts();
    publisher.publish_initial(ProjectInputBundle::closed(config, files, vec![library])?)?;
    let view = publisher.open_current()?;
    let (registry, batch, coverage, provenance, limits) =
        build_source_graph_proposals(&view, &stop)?.into_parts();
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
    // The owner projects both exact roots before any recognizer runs. The account
    // and character scopes are distinct declarations, so neither root is ambiguous.
    assert!(
        provenance
            .state_roots()
            .iter()
            .any(|root| root.name == "StateCoreAccountDB"
                && !root.ambiguous
                && root.scope == wow_project::load::TocSavedVariableScope::Account)
    );
    assert!(
        provenance
            .state_roots()
            .iter()
            .any(|root| root.name == "StateCoreCharacterDB"
                && !root.ambiguous
                && root.scope == wow_project::load::TocSavedVariableScope::Character)
    );
    // Legacy state recognition joins the analyzer global-slot accesses.
    let (after_state, legacy) = state::publish(source.candidate(), &provenance, &stop)
        .map_err(|error| std::io::Error::other(format!("state publication: {error}")))?;
    rejects_changed_access_handles(&after_state, &provenance, &legacy, &stop)?;
    // TOC publication supplies the materialized variant owner that Template-style
    // roots reference, then the TOC state-root phase runs.
    let (after_toc, _toc_recognition) = toc::publish(&after_state, &provenance, &stop)
        .map_err(|error| std::io::Error::other(format!("TOC publication: {error}")))?;
    let (after_root, root_recognition) = toc::publish_state_root(&after_toc, &provenance, &stop)
        .map_err(|error| std::io::Error::other(format!("TOC state root: {error}")))?;
    // Core state publication consumes the materialized roots and the legacy
    // recognition, then the final crosswalk runs across the accepted partitions.
    let (final_snapshot, core_recognition) =
        state_core::publish(&after_root, &provenance, &legacy, &stop)
            .map_err(|error| std::io::Error::other(format!("core state: {error}")))?;
    final_snapshot.validate(&stop)?;
    let addresses = SourceGraphAddressCrosswalk::legacy(&provenance, source.candidate(), &stop)?;
    let root_topology = toc::maps(
        &final_snapshot,
        &addresses,
        std::slice::from_ref(&root_recognition),
        &stop,
    )
    .map_err(|error| std::io::Error::other(format!("TOC crosswalk: {error}")))?;
    let topology = state_core::maps(&final_snapshot, &addresses, &core_recognition, &stop)
        .map_err(|error| std::io::Error::other(format!("core crosswalk: {error}")))?;
    // Three core rule phases: saved_variable_root plus literal read and write.
    assert_eq!(core_recognition.len(), 2);
    let mut rule_ids: Vec<&str> = core_recognition
        .iter()
        .map(|recognition| recognition.family.rule_id())
        .collect();
    rule_ids.push(root_recognition.family.rule_id());
    rule_ids.sort();
    rule_ids.dedup();
    assert_eq!(
        rule_ids,
        vec![
            "core.state.literal_path_read",
            "core.state.literal_path_write",
            "core.state.saved_variable_root",
        ]
    );
    for recognition in &core_recognition {
        let partition = final_snapshot
            .partition(recognition.family.partition_id())
            .ok_or("missing core state partition")?;
        assert!(
            partition
                .coverage()
                .iter()
                .all(|record| !record.negative_authority())
        );
        assert!(!recognition.evaluations.is_empty());
        for proposal in partition.batch().entity_proposals() {
            assert_eq!(proposal.entity_kind_id(), "state_path");
            assert!(matches!(
                proposal.semantic_key().get("root"),
                Some(wow_graph::GraphProposalValue::Identifier(_))
            ));
            assert!(matches!(
                proposal.semantic_key().get("path"),
                Some(wow_graph::GraphProposalValue::String(_))
            ));
            let source_partition = final_snapshot
                .partition(wow_project::graph::SOURCE_GRAPH_PARTITION)
                .ok_or("missing source partition")?;
            assert!(
                provenance.state_paths().iter().any(|path| source_partition
                    .batch()
                    .entity_proposal(&path.proposal_id)
                    .is_some_and(|source| source.semantic_key() == proposal.semantic_key())),
                "core path identity must equal an admitted owner path"
            );
        }
        for evaluation in &recognition.evaluations {
            if evaluation.fact_bundle.facts().is_empty() {
                assert!(
                    evaluation
                        .fact_bundle
                        .coverage()
                        .iter()
                        .all(|coverage| coverage.state()
                            == wow_recognizers::RecognizerFactCoverageState::NotEvaluated)
                );
            }
        }
    }
    // The saved-variable-root phase emits both distinct scopes. Account and
    // character are separate declarations, so neither root is ambiguous.
    assert_eq!(
        root_recognition.family.rule_id(),
        "core.state.saved_variable_root"
    );
    assert!(!root_recognition.receipts.is_empty());
    // Every emitted root retains exact support and a materialized node.
    for node in &root_topology.nodes {
        assert!(!node.fact_ids.is_empty());
        assert!(final_snapshot.snapshot().node(&node.node_id).is_some());
    }
    assert!(!root_topology.nodes.is_empty());
    // Both literal path phases publish relations. Every relation carries Possible
    // confidence because the matcher forces it under Partial coverage.
    for rule_id in [
        "core.state.literal_path_read",
        "core.state.literal_path_write",
    ] {
        let phase = core_recognition
            .iter()
            .find(|recognition| recognition.family.rule_id() == rule_id)
            .ok_or("missing core state path phase")?;
        assert!(!phase.receipts.is_empty());
        let edges: Vec<&state_core::StateCoreEdge> = topology
            .edges
            .iter()
            .filter(|edge| edge.rule_id == rule_id)
            .collect();
        assert!(!edges.is_empty());
        for edge in &edges {
            assert_eq!(edge.confidence, wow_graph::GraphConfidence::Possible);
            assert!(final_snapshot.snapshot().edge(&edge.edge_id).is_some());
            assert!(final_snapshot.snapshot().node(&edge.from_node_id).is_some());
            assert!(final_snapshot.snapshot().node(&edge.to_node_id).is_some());
        }
    }
    // The alias read reaches the root through a lexical binding, so at least one
    // read edge exists beyond the direct literal access.
    let read_edges: Vec<&state_core::StateCoreEdge> = topology
        .edges
        .iter()
        .filter(|edge| edge.rule_id == "core.state.literal_path_read")
        .collect();
    assert!(read_edges.len() >= 2);
    let report = provenance
        .function_call_report()
        .ok_or("missing access report")?;
    assert!(
        provenance.state_bindings().iter().any(|binding| report
            .global_accesses()
            .iter()
            .any(|access| access.fact_id() == binding.access_id && access.is_alias())
            && read_edges
                .iter()
                .any(|edge| edge.fact_ids.contains(&binding.access_id))),
        "an admitted alias read must retain its exact access witness"
    );
    let dynamic = provenance
        .state_sites()
        .iter()
        .find(|site| {
            site.outcome == wow_project::graph::ProjectGraphStateOutcome::DynamicOrUnsupportedKey
        })
        .ok_or("missing explicit dynamic-key omission")?;
    assert!(
        topology
            .edges
            .iter()
            .all(|edge| !edge.fact_ids.contains(&dynamic.access_id)),
        "dynamic keys cannot manufacture an exact path relation"
    );
    let text = include_str!("../../../tests/data/state-core-pipeline/state.lua");
    let shadow_start = text
        .find("local function shadowed_local()")
        .ok_or("missing shadow fixture")?;
    let shadow_end = text
        .find("-- Dynamic key:")
        .ok_or("missing shadow boundary")?;
    let shadow = report.functions().iter().find(|function| matches!(
        (function.span().byte_start(), function.span().byte_end()),
        (Some(start), Some(end)) if shadow_start <= start as usize && end as usize <= shadow_end
    )).ok_or("missing shadowed function fact")?;
    assert!(
        provenance.state_bindings().iter().all(|binding| report
            .global_accesses()
            .iter()
            .find(|access| access.fact_id() == binding.access_id)
            .is_none_or(|access| access.function_id() != shadow.fact_id())),
        "a local shadow cannot bind to the SavedVariables global"
    );
    // Every surviving edge keeps exact support.
    assert!(
        topology.edges.iter().all(|edge| !edge.fact_ids.is_empty()),
        "every core state edge keeps exact support"
    );
    let first = topology.edges.first().ok_or("missing core relation")?;
    let explanation = wow_graph::GraphExplainQuery::new(
        final_snapshot.snapshot().snapshot_id().clone(),
        wow_graph::GraphExplainSubject::Relation(first.edge_id.clone()),
        wow_graph::GraphExplainLimits::default(),
    )?
    .execute(&final_snapshot, &stop)?;
    assert!(
        !explanation.derivations().is_empty(),
        "real core producer retains its exact graph prerequisites"
    );
    assert!(
        !explanation.assertion_supports().is_empty(),
        "explain follows the admitted predecessor assertion"
    );
    assert!(
        explanation.derivation_complete(),
        "the state chain closes through source assertions and exact captured files"
    );
    assert!(explanation.derivations().iter().any(|observation| {
        observation
            .record
            .rule_id
            .starts_with("wow-project.source-graph.entity.")
    }));
    let shallow = wow_graph::GraphExplainLimits {
        max_derivation_depth: 0,
        ..wow_graph::GraphExplainLimits::default()
    };
    let truncated = wow_graph::GraphExplainQuery::new(
        final_snapshot.snapshot().snapshot_id().clone(),
        wow_graph::GraphExplainSubject::Relation(first.edge_id.clone()),
        shallow,
    )?
    .execute(&final_snapshot, &stop)?;
    assert!(!truncated.derivation_complete());
    assert!(
        truncated
            .truncations()
            .contains(&wow_graph::GraphExplanationTruncation::DerivationDepth)
    );
    assert!(!explanation.absence_authoritative());
    Ok(())
}

fn rejects_changed_access_handles(
    owner: &GraphPartitionSnapshot,
    provenance: &ProjectGraphProvenance,
    recognition: &SourceStateRecognition,
    stop: &AtomicBool,
) -> Result<(), Box<dyn Error>> {
    use wow_recognizers::source_state_core::{
        SourceStateCoreFamily, SourceStateCoreInput, recognize_source_state_core,
    };
    let partition = owner
        .partition(wow_recognizers::source_state::SOURCE_STATE_PARTITION)
        .ok_or("missing access partition")?;
    let batch = partition.batch();
    let mut relations = partition
        .report()
        .accepted_relations()
        .iter()
        .map(|accepted| {
            batch
                .relation_proposal(accepted.proposal_id())
                .cloned()
                .ok_or("missing admitted proposal")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let index = relations
        .iter()
        .position(|relation| relation.relation_kind_id() == "source_reads_state")
        .ok_or("missing read proposal")?;
    let original = &relations[index];
    let other_handle = owner
        .partition(wow_project::graph::SOURCE_GRAPH_PARTITION)
        .ok_or("missing source partition")?
        .batch()
        .entity_proposals()
        .iter()
        .flat_map(|entity| entity.source_handle_ids())
        .find(|id| !original.source_handle_ids().contains(id))
        .ok_or("missing different admitted handle")?;
    let mut changed = serde_json::to_value(original)?;
    changed["source_handle_ids"] = serde_json::to_value(vec![*other_handle])?;
    relations[index] = serde_json::from_value(changed)?;
    let changed_batch = wow_graph::GraphProposalBatch::build(
        batch.registry_bundle_id(),
        batch.registry_digest(),
        batch.universe().clone(),
        batch.generation().clone(),
        batch.source_context_id(),
        batch.producer_partition_id(),
        batch.entity_proposals().to_vec(),
        relations,
    )?;
    let prepared = owner.prepare_replacement(
        GraphPartitionReplacement {
            expected_snapshot_id: owner.snapshot().snapshot_id().clone(),
            expected_partition_digest: Some(partition.partition_digest().into()),
            producer_version: env!("CARGO_PKG_VERSION").into(),
            batch: changed_batch,
            coverage: partition.coverage().to_vec(),
        },
        stop,
    )?;
    let outcome = recognize_source_state_core(
        SourceStateCoreInput {
            owner: prepared.candidate(),
            context: provenance.context(),
            recognition,
        },
        SourceStateCoreFamily::Read,
        stop,
    );
    assert!(
        matches!(outcome, Err(error) if error.code() == wow_recognizers::RecognizerErrorCode::AdapterIdentityMismatch),
        "unchanged endpoints/evidence cannot authorize a substituted source handle"
    );
    Ok(())
}
