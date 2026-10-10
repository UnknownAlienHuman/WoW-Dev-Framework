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
use wow_project::graph::build_source_graph_proposals;
use wow_project::load::TocLoadContext;
use wow_project::{
    AnalyzerBindingDeclaration, ProjectBudgetPolicy, ProjectCapabilityPolicy,
    ProjectConfigurationBuilder, ProjectId, ProjectInputBundle, ProjectKind, ProjectPublisher,
    ProjectSourceOriginId, ProjectWorkspaceId,
};

fn xml_fixture_profile() -> Result<wow_core::ProfileIdentity, Box<dyn Error>> {
    Ok(ProfileIdentityBuilder::new(
        "profile:fixture:xml-pipeline-v1".parse()?,
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
    .fixture_scope("wow-xml-pipeline-fixture")
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

/// One standalone TOC/XML pipeline: owner projection, source publication, TOC publication,
/// the four XML rule families and their final read-back. Source-declared structure only;
/// no runtime frame, receiver, dispatch or client authority is claimed.
#[test]
fn xml_pipeline_publishes_object_parentage_and_template_references() -> Result<(), Box<dyn Error>> {
    let stop = std::sync::atomic::AtomicBool::new(false);
    let directory = ProjectInputDirectory::open(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/xml-pipeline"),
    )?;
    let (files, plan) = directory
        .read_toc_project_with_context(
            ".",
            &ProjectDiskFile::new("XmlPipeline.toc"),
            &xml_fixture_profile()?,
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
            "library/xml_pipeline.lua",
            "---@meta _\n",
        )],
        LuaWorkspaceLimits::new(8, 16_384, 256 * 1024, 512 * 1024)?,
    )?;
    let config = ProjectConfigurationBuilder::new(
        ProjectId::new("fixture-xml-pipeline")?,
        ProjectKind::Fixture,
        xml_fixture_profile()?,
        ReferenceGenerationId::derive(&"fixture-xml-reference")?,
        binding,
    )
    .workspace_id(ProjectWorkspaceId::new("workspace:xml-pipeline")?)
    .source_origin_id(ProjectSourceOriginId::new("project-origin:xml-pipeline")?)
    .logical_root("fixtures/xml-pipeline")
    .capability_policy(ProjectCapabilityPolicy::strict_e0()?)
    .budget_policy(ProjectBudgetPolicy::fixture_e0()?)
    .load_plan(&plan)?
    .build()?;
    // The standard graph-build route enables the bounded function/call fact
    // sidecar, and the scripts stage reads that report. Without it the function
    // call report is None and script publication is skipped.
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
    // Template and Object need the materialized toc_variant owner, so TOC
    // publication must precede XML publication.
    let (after_toc, _toc_recognition) = toc::publish(source.candidate(), &provenance, &stop)
        .map_err(|error| std::io::Error::other(format!("TOC publication: {error}")))?;
    let (snapshot, recognition) = xml::publish(&after_toc, &provenance, &stop)
        .map_err(|error| std::io::Error::other(format!("XML publication: {error}")))?;
    snapshot.validate(&stop)?;
    let addresses = SourceGraphAddressCrosswalk::legacy(&provenance, source.candidate(), &stop)?;
    let topology = xml::maps(&snapshot, &addresses, &recognition, &stop)
        .map_err(|error| std::io::Error::other(format!("XML crosswalk: {error}")))?;
    // Five publication phases share four unique frozen rule IDs: Object and
    // ObjectParentage both publish core.xml.object@1 from distinct partitions.
    assert_eq!(recognition.len(), 5);
    let mut rule_ids: Vec<&str> = recognition.iter().map(|r| r.family.rule_id()).collect();
    rule_ids.sort();
    rule_ids.dedup();
    assert_eq!(
        rule_ids,
        vec![
            "core.xml.inherits",
            "core.xml.object",
            "core.xml.script",
            "core.xml.template"
        ]
    );
    let partition_ids: Vec<&str> = recognition
        .iter()
        .map(|r| r.family.partition_id())
        .collect();
    assert_eq!(partition_ids.len(), 5);
    for recognition_entry in &recognition {
        let partition = snapshot
            .partition(recognition_entry.family.partition_id())
            .ok_or("missing XML family partition")?;
        assert!(!partition.report().accepted_relations().is_empty());
        assert!(!recognition_entry.receipts.is_empty());
    }
    // ObjectParentage is the sole ParentOf producer: exactly one explicit
    // parent edge, and the lexical-only nested child never becomes ParentOf.
    let parentage = recognition
        .iter()
        .find(|r| {
            r.family.rule_id() == "core.xml.object"
                && r.family.partition_id() == "wow-recognizers.xml-object-parentage"
        })
        .ok_or("missing object parentage phase")?;
    assert_eq!(parentage.family.rule_id(), "core.xml.object");
    assert!(
        recognition
            .iter()
            .any(|r| r.family.partition_id() == "wow-recognizers.xml-object"
                && r.family != parentage.family)
    );
    // The lexical-only nested child is retained as a declaration fact with
    // captured containment but no explicit parent name reference, so it must
    // never appear in a ParentOf edge.
    let lexical_declaration = provenance
        .xml_facts()
        .iter()
        .find(|fact| {
            matches!(
                &fact.kind,
                wow_project::graph::ProjectXmlFactKind::Declaration { declaration, .. }
                    if declaration.parent_key.as_deref() == Some("LexicalChild")
            )
        })
        .ok_or("lexical-only nested child is missing from the retained facts")?;
    let lexical_fact_id = &lexical_declaration.fact_id;
    let parent_of_edges: Vec<&xml::XmlEdge> = topology
        .edges
        .iter()
        .filter(|edge| edge.relation == wow_graph::GraphRelationKind::ParentOf)
        .collect();
    assert_eq!(
        parent_of_edges.len(),
        1,
        "exactly one explicit ParentOf edge"
    );
    assert!(
        parent_of_edges
            .iter()
            .all(|edge| !edge.fact_ids.contains(lexical_fact_id)),
        "the lexical-only nested child must not be a ParentOf endpoint"
    );
    // The explicit parent edge reaches materialized nodes, and the resolved
    // inheritance plus its template reference are both published.
    for edge in &parent_of_edges {
        assert!(snapshot.snapshot().node(&edge.from_node_id).is_some());
        assert!(snapshot.snapshot().node(&edge.to_node_id).is_some());
    }
    let lexical_node = topology
        .nodes
        .iter()
        .find(|node| node.rule_id == "core.xml.object" && node.fact_ids.contains(lexical_fact_id))
        .ok_or("missing lexical object node")?;
    let object_profile =
        wow_graph::GraphAxisProfile::bind(snapshot.registry(), wow_graph::GraphAxis::Object)?;
    assert_eq!(
        object_profile.shape(),
        wow_graph::GraphAxisShape::MultiParent
    );
    assert_eq!(object_profile.relations().len(), 1);
    assert_eq!(
        object_profile.relations()[0].relation(),
        wow_graph::GraphRelationKind::ParentOf
    );
    let axis = wow_graph::GraphAxisQuery::new(
        snapshot.snapshot().snapshot_id().clone(),
        &object_profile,
        vec![parent_of_edges[0].from_node_id.clone()],
        wow_graph::GraphAxisTraversal::Children,
        wow_graph::GraphSubgraphLimits {
            max_depth: 1,
            ..wow_graph::GraphSubgraphLimits::default()
        },
    )?;
    assert_eq!(
        parent_of_edges[0].confidence,
        wow_graph::GraphConfidence::Possible,
        "partial captured-structure coverage cannot become Derived"
    );
    let default_axis = axis.execute(&snapshot, &object_profile, &stop)?;
    assert!(
        default_axis
            .projection()
            .nodes()
            .iter()
            .all(|node| node.node().node_id() != &parent_of_edges[0].to_node_id)
    );
    let axis = axis
        .with_confidence(wow_graph::GraphPathConfidence::IncludePossible)
        .execute(&snapshot, &object_profile, &stop)?;
    assert!(
        axis.projection()
            .nodes()
            .iter()
            .any(|node| node.node().node_id() == &parent_of_edges[0].to_node_id)
    );
    assert!(
        axis.projection()
            .nodes()
            .iter()
            .all(|node| node.node().node_id() != &lexical_node.node_id)
    );
    assert!(!axis.absence_authoritative());
    let inheritance_edges: Vec<&xml::XmlEdge> = topology
        .edges
        .iter()
        .filter(|edge| {
            matches!(
                edge.relation,
                wow_graph::GraphRelationKind::Inherits
                    | wow_graph::GraphRelationKind::ReferencesTemplate
            )
        })
        .collect();
    assert!(
        inheritance_edges
            .iter()
            .any(|edge| edge.relation == wow_graph::GraphRelationKind::Inherits),
        "the resolved inherits name must publish an Inherits relation"
    );
    assert!(
        inheritance_edges
            .iter()
            .any(|edge| edge.relation == wow_graph::GraphRelationKind::ReferencesTemplate),
        "the same resolved name must publish ReferencesTemplate"
    );
    // Both the inline OnClick body and the named OnLoad handler are admitted
    // script bindings. The unresolved named handler is retained as an explicit
    // omission rather than dropped or treated as a clean negative.
    let script_bindings = provenance.script_bindings();
    assert!(
        script_bindings
            .iter()
            .any(|binding| binding.handler_kind == "lua_source_function"
                && binding.semantic_context.is_none()
                && binding.confidence == wow_graph::GraphConfidence::Derived)
    );
    assert!(
        script_bindings
            .iter()
            .any(|binding| binding.handler_kind == "xml_source_handler"
                && binding.semantic_context.is_some()
                && binding.confidence == wow_graph::GraphConfidence::Possible),
        "the inline OnClick body must retain an admitted binding"
    );
    for binding in script_bindings {
        assert!(
            topology.edges.iter().any(|edge| edge.relation
                == wow_graph::GraphRelationKind::SetsScript
                && edge.confidence == wow_graph::GraphConfidence::Possible
                && edge.fact_ids.contains(&binding.binding_id)),
            "each admitted binding must retain its witness and partial confidence"
        );
    }
    let unresolved_script = provenance
        .xml_facts()
        .iter()
        .find(|fact| {
            matches!(&fact.kind,
        wow_project::graph::ProjectXmlFactKind::Script { function_reference: Some(name), .. }
        if name == "XmlMissingHandler.OnEnter")
        })
        .ok_or("missing unresolved named script fact")?;
    assert!(
        topology.omissions.iter().any(|omission| omission.blocker
            == "xml.script_handler_not_admitted"
            && omission.fact_ids.contains(&unresolved_script.fact_id)),
        "the unresolved named handler must be an explicit omission"
    );
    assert!(
        topology
            .omissions
            .iter()
            .any(|omission| omission.blocker == "xml.inheritance_unresolved"),
        "the unresolved inherits name must be an explicit omission"
    );
    // Final crosswalk IDs are materialized, not proposal placeholders.
    assert!(
        provenance
            .xml_facts()
            .iter()
            .any(|fact| matches!(&fact.kind,
        wow_project::graph::ProjectXmlFactKind::InheritanceUnresolved { name, resolution, .. }
        if name == "XmlAmbiguousTemplate" && matches!(resolution,
            wow_project::load::xml_references::XmlReferenceResolution::AmbiguousName { .. })))
    );
    let chunk = provenance
        .script_sources()
        .iter()
        .find(|source| source.script_name == "Script")
        .ok_or("missing source-only Script chunk")?;
    assert!(
        provenance
            .script_sites()
            .iter()
            .any(|site| site.script_id == chunk.script_id
                && site.consumer_id.is_none()
                && site.binding_ids.is_empty())
    );
    assert!(
        provenance
            .inline_handlers()
            .iter()
            .all(|handler| handler.script_id != chunk.script_id)
    );
    assert!(!topology.nodes.is_empty());
    assert!(!topology.edges.is_empty());
    for node in &topology.nodes {
        assert!(snapshot.snapshot().node(&node.node_id).is_some());
    }
    for edge in &topology.edges {
        assert!(snapshot.snapshot().edge(&edge.edge_id).is_some());
    }
    Ok(())
}
