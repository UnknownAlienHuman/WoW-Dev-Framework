//! One real service lifecycle over the direct layout and an exact frozen catalog.
use super::{
    LiveProjectPublishRequest, LiveProjectStore, PlatformFixtureSource, PlatformGraphProfile,
    PlatformStoreSelection, ProjectStore, ReadSelector, ServiceErrorCode, TestResult, catalog_for,
    platform_bundle_with_profile_source, publication, publish_input_in_namespace, root,
};
use std::{collections::BTreeSet, sync::atomic::AtomicBool};
use wow_graph::{GraphEdge, GraphPartitionSnapshot, GraphRelationKind};
use wow_project::{ProjectInputBundle, graph::PlatformGraphProducer};
use wow_recognizers::{
    source_bridge::W2_PARTITION,
    source_calls::SOURCE_CALL_PARTITION,
    source_construction::SOURCE_CONSTRUCTION_PARTITION,
    source_mixins::{SOURCE_MIXIN_ASSIGNMENT_PARTITION, SOURCE_MIXIN_PARTITION},
    source_scripts::{SOURCE_SCRIPT_PARTITION, W5_HOOK_PARTITION},
    source_signals::{W1_SIGNAL_PARTITION, W3_SIGNAL_PARTITION, W4_PARTITION},
    source_state::{SOURCE_STATE_LIBRARY_PARTITION, SOURCE_STATE_PARTITION},
    source_state_core::SourceStateCoreFamily,
    source_toc::SourceTocFamily,
    source_xml::SourceXmlFamily,
};

fn input(bundle: &ProjectInputBundle) -> TestResult<crate::LocalProjectInput> {
    let packages = bundle
        .configuration()
        .platform_packages()
        .ok_or("retained platform packages missing")?;
    let reference = wow_reference::ReferenceView::new(
        bundle.configuration().reference_generation().to_string(),
        Vec::new(),
        Vec::new(),
    )?;
    Ok(crate::LocalProjectInput::new_with_package_plans(
        bundle.clone(),
        reference,
        packages.load_plan().clone(),
        packages.main_plan().clone(),
    )?)
}

fn expected_partitions() -> BTreeSet<&'static str> {
    let mut partitions = BTreeSet::from([
        wow_project::graph::PLATFORM_RAW_INVENTORY_PARTITION,
        SOURCE_CALL_PARTITION,
        SOURCE_CONSTRUCTION_PARTITION,
        SOURCE_MIXIN_PARTITION,
        SOURCE_MIXIN_ASSIGNMENT_PARTITION,
        SOURCE_SCRIPT_PARTITION,
        SOURCE_STATE_PARTITION,
        W1_SIGNAL_PARTITION,
        W2_PARTITION,
        W3_SIGNAL_PARTITION,
        W4_PARTITION,
        W5_HOOK_PARTITION,
        SOURCE_STATE_LIBRARY_PARTITION,
    ]);
    partitions.extend(
        [
            PlatformGraphProducer::Inventory,
            PlatformGraphProducer::TocLoad,
            PlatformGraphProducer::AnalyzerStructure,
            PlatformGraphProducer::XmlStructure,
        ]
        .map(PlatformGraphProducer::partition_id),
    );
    partitions.extend(SourceTocFamily::ALL.map(SourceTocFamily::partition_id));
    partitions.insert(SourceTocFamily::SavedVariableRoot.partition_id());
    partitions.extend(SourceXmlFamily::ALL.map(SourceXmlFamily::partition_id));
    partitions.extend(SourceStateCoreFamily::ALL.map(SourceStateCoreFamily::partition_id));
    partitions
}

fn final_membership(graph: &GraphPartitionSnapshot, stop: &AtomicBool) -> TestResult {
    graph.validate(stop)?;
    let input = graph.input_view(stop)?;
    let mut nodes = std::collections::BTreeMap::new();
    for original in input.nodes() {
        let mut matches = graph.snapshot().nodes().iter().filter(|node| {
            node.kind() == original.kind() && node.owner_key() == original.owner_key()
        });
        let final_node = matches
            .next()
            .ok_or("input node missing from final graph")?;
        assert!(matches.next().is_none());
        assert!(
            original
                .evidence_ids()
                .iter()
                .all(|id| final_node.evidence_ids().contains(id))
        );
        assert!(
            nodes
                .insert(original.node_id().clone(), final_node.node_id().clone())
                .is_none()
        );
    }
    assert_eq!(
        nodes.values().collect::<BTreeSet<_>>(),
        graph
            .snapshot()
            .nodes()
            .iter()
            .map(|node| node.node_id())
            .collect::<BTreeSet<_>>()
    );
    let mut edges = BTreeSet::new();
    for partition in graph.partitions() {
        assert!(partition.report().rejections().is_empty());
        assert_eq!(
            partition.report().accepted_entities().len(),
            partition.batch().entity_proposals().len()
        );
        assert!(partition.coverage().iter().all(|c| !c.negative_authority()));
        for accepted in partition.report().accepted_entities() {
            assert!(nodes.contains_key(accepted.node().node_id()));
        }
        for accepted in partition.report().accepted_relations() {
            let proposal = partition
                .batch()
                .relation_proposal(accepted.proposal_id())
                .ok_or("accepted relation proposal missing")?;
            assert!(!proposal.source_handle_ids().is_empty());
            assert!(!proposal.evidence_ids().is_empty());
            let original = accepted.edge();
            let rebound = GraphEdge::new(
                nodes
                    .get(original.from())
                    .ok_or("final source missing")?
                    .clone(),
                nodes
                    .get(original.to())
                    .ok_or("final target missing")?
                    .clone(),
                original.relation(),
                original.confidence(),
                original.evidence_ids().to_vec(),
                graph.snapshot().limits(),
            )?;
            let final_edge = graph
                .snapshot()
                .edge(rebound.edge_id())
                .ok_or("accepted relation missing from final graph")?;
            assert_eq!(final_edge.from(), rebound.from());
            assert_eq!(final_edge.to(), rebound.to());
            assert_eq!(final_edge.relation(), rebound.relation());
            assert!(
                original
                    .evidence_ids()
                    .iter()
                    .all(|id| final_edge.evidence_ids().contains(id))
            );
            edges.insert(final_edge.edge_id().clone());
        }
    }
    assert_eq!(
        edges,
        graph
            .snapshot()
            .edges()
            .iter()
            .map(|edge| edge.edge_id().clone())
            .collect()
    );
    Ok(())
}

fn signal_membership(record: &serde_json::Value, graph: &GraphPartitionSnapshot) -> TestResult {
    let signal_nodes = record["signal_nodes"]
        .as_array()
        .ok_or("signal nodes missing")?;
    assert!(!signal_nodes.is_empty());
    for node in signal_nodes {
        let id = node["node_id"].as_str().ok_or("signal node ID missing")?;
        assert!(
            graph
                .snapshot()
                .nodes()
                .iter()
                .any(|node| node.node_id().as_str() == id)
        );
    }
    let signal_edges = record["signal_edges"]
        .as_array()
        .ok_or("signal edges missing")?;
    assert!(!signal_edges.is_empty());
    for edge in signal_edges {
        let id = edge["edge_id"].as_str().ok_or("signal edge ID missing")?;
        let native = graph
            .snapshot()
            .edges()
            .iter()
            .find(|edge| edge.edge_id().as_str() == id)
            .ok_or("signal edge not in final graph")?;
        assert_eq!(
            native.from().as_str(),
            edge["function_node_id"]
                .as_str()
                .ok_or("signal function ID missing")?
        );
        assert_eq!(
            native.to().as_str(),
            edge["target_node_id"]
                .as_str()
                .ok_or("signal target ID missing")?
        );
        assert_eq!(serde_json::to_value(native.relation())?, edge["relation"]);
        assert_eq!(
            serde_json::to_value(native.confidence())?,
            edge["confidence"]
        );
        for endpoint in [native.from(), native.to()] {
            assert!(graph.snapshot().node(endpoint).is_some());
        }
    }
    Ok(())
}

#[test]
fn direct_platform_service_reopens_v9_and_frozen_v8_refuses() -> TestResult {
    let path = root("direct-platform-service-v9")?;
    {
        let stop = AtomicBool::new(false);
        let toc = format!(
            "## SavedVariables: StateCoreAccountDB\n## SavedVariablesPerCharacter: StateCoreCharacterDB\n{}",
            include_str!("../../../../../wow-project/tests/data/xml-facts/Fixture.toc")
        );
        let lua = format!(
            "{}\n{}\nfunction LibStub(name) return {{}} end\nfunction Require() return LibStub(\"FixtureLibrary-1\") end\n",
            include_str!("../../../../../wow-project/tests/data/xml-facts/defs.lua"),
            include_str!("../../../../tests/data/state-core-pipeline/state.lua")
        );
        let fixture = PlatformFixtureSource {
            toc: toc.as_bytes(),
            lua: lua.as_bytes(),
        };
        let source_path = path.join("input");
        let bundle = platform_bundle_with_profile_source(
            &source_path,
            1,
            true,
            Some(PlatformGraphProfile::DirectPlatformProducersWithRawInventoryV1),
            Some(&fixture),
            &stop,
        )?;
        assert!(!source_path.exists());
        let packages = bundle
            .configuration()
            .platform_packages()
            .ok_or("platform source missing")?;
        let source = packages.source();
        assert_eq!(source.source_bytes("UI/opaque.bin")?, &[0xff, 0xfe, 0, 1]);
        for member in ["UI/unloaded.lua", "UI/unloaded.xml"] {
            assert!(!source.source_bytes(member)?.is_empty());
        }
        assert!(source.receipt().inventory().entries.iter().any(|entry| {
            entry.path == "UI/omitted.txt"
                && matches!(
                    &entry.disposition,
                    wow_project::platform_source::PlatformEntryDisposition::Excluded { .. }
                )
        }));
        let selection = PlatformStoreSelection::new(
            bundle.configuration().project_id().clone(),
            source.profile().profile_id().clone(),
        )?;
        let request = crate::graph::GraphBuildRequest::new(
            bundle.configuration().project_id().as_str().into(),
            "current".into(),
        )?
        .with_platform_graph_profile(
            PlatformGraphProfile::DirectPlatformProducersWithRawInventoryV1,
        );
        let request_record = serde_json::to_value(&request)?;
        assert_eq!(
            request_record["schema"],
            "wow-service/graph-build-request/12"
        );
        assert_eq!(
            request_record["projection"],
            "wow-project/source-load-proposals/22"
        );
        let (direct_publication, _) =
            request.live_publication_in_namespace(input(&bundle)?, selection.namespace(), &stop)?;
        let artifact = crate::graph::execute_graph_build(input(&bundle)?, &request, &stop)?;
        // JSON is inspected as output data; native acquisition below admits the owners.
        let artifact_record: serde_json::Value =
            serde_json::from_slice(&artifact.canonical_bytes()?)?;
        assert_eq!(
            artifact_record["schema"],
            "wow-service/graph-build-result/19"
        );
        assert!(artifact_record.get("failure").is_none());
        assert!(artifact_record.get("state_recognition").is_none());
        assert!(artifact_record.get("script_recognition").is_none());
        assert_eq!(
            artifact_record["state_assertion_recognition"]["profile"],
            "wow-recognizers/source-saved-variable-assertions/1"
        );
        assert_eq!(
            artifact_record["script_assertion_recognition"]["profile"],
            "wow-recognizers/source-xml-script-assertions/1"
        );
        let artifact_graph: serde_json::Value = serde_json::from_slice(
            &artifact
                .snapshot_bytes()?
                .ok_or("service native graph missing")?,
        )?;
        let store_path = path.join("selected");
        let publish_request =
            LiveProjectPublishRequest::new("fixture:direct-service-v9", "absent", true, true)?;
        let published = publish_input_in_namespace(
            input(&bundle)?,
            &request,
            &store_path,
            &selection,
            &publish_request,
            &stop,
        )?;
        assert_eq!(published.exit_code(), 2);
        let published_record: serde_json::Value =
            serde_json::from_slice(&published.canonical_bytes()?)?;
        assert_eq!(published_record["status"], "activated");
        let store = LiveProjectStore::open_in_namespace(&store_path, &selection)?;
        let current = store.current()?.ok_or("direct Current missing")?;
        let epoch = store.store.epoch().clone();
        let read = store.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
        assert_eq!(read.namespace(), Some(selection.namespace()));
        assert_eq!(read.current_at_acquisition(), Some(&current));
        assert_eq!(read.project().configuration(), bundle.configuration());
        assert_eq!(
            wow_project::graph::source_graph_profile(read.project().configuration()),
            "wow-project/source-load-proposals/22"
        );
        assert_eq!(serde_json::to_value(read.graph())?, artifact_graph);
        assert_eq!(
            read.graph()
                .partitions()
                .iter()
                .map(|p| p.partition_id())
                .collect::<BTreeSet<_>>(),
            expected_partitions()
        );
        assert!(
            read.graph()
                .partition(wow_project::graph::SOURCE_GRAPH_PARTITION)
                .is_none()
        );
        final_membership(read.graph(), &stop)?;
        signal_membership(&artifact_record, read.graph())?;
        let library = read
            .graph()
            .partition(SOURCE_STATE_LIBRARY_PARTITION)
            .ok_or("library partition missing")?;
        assert_eq!(library.report().accepted_entities().len(), 1);
        assert_eq!(library.report().accepted_relations().len(), 1);
        let library_entity = &library.report().accepted_entities()[0];
        assert_eq!(
            library
                .batch()
                .entity_proposal(library_entity.proposal_id())
                .ok_or("accepted library entity proposal missing")?
                .entity_kind_id(),
            "library"
        );
        let library_relation = &library.report().accepted_relations()[0];
        assert_eq!(
            library
                .batch()
                .relation_proposal(library_relation.proposal_id())
                .ok_or("accepted library relation proposal missing")?
                .relation_kind_id(),
            "lua_requires_library"
        );
        assert_eq!(
            library_relation.edge().to(),
            library_entity.node().node_id()
        );
        for (partition_id, kind) in [
            (SOURCE_SCRIPT_PARTITION, GraphRelationKind::SetsScript),
            (
                SourceStateCoreFamily::Read.partition_id(),
                GraphRelationKind::ReadsState,
            ),
            (
                SourceStateCoreFamily::Write.partition_id(),
                GraphRelationKind::WritesState,
            ),
        ] {
            let partition = read
                .graph()
                .partition(partition_id)
                .ok_or("required output missing")?;
            assert!(!partition.report().accepted_relations().is_empty());
            assert!(
                partition
                    .report()
                    .accepted_relations()
                    .iter()
                    .all(|accepted| accepted.edge().relation() == kind)
            );
        }
        assert!(
            !read
                .graph()
                .partition(SOURCE_STATE_PARTITION)
                .ok_or("state access missing")?
                .report()
                .accepted_relations()
                .is_empty()
        );
        assert!(
            !read
                .graph()
                .partition(SourceTocFamily::SavedVariableRoot.partition_id())
                .ok_or("state roots missing")?
                .report()
                .accepted_entities()
                .is_empty()
        );
        assert!(read.read.manifest().members.iter().any(|member| {
            member.key == "live.project.replay" && member.schema == "wow-project.live-replay.v9"
        }));
        let replay: serde_json::Value = read
            .read
            .record("live.project.replay", &stop)?
            .ok_or("v9 replay missing")?
            .decode()?;
        assert_eq!(replay["schema"], "wow-project/native-project-replay/9");
        let project_id = read.project().snapshot_id().to_owned();
        let analyzer_id = read.project().analyzer_snapshot_id().to_owned();
        let graph = read.graph().clone();
        let members = read.read.manifest().members.clone();
        let set_id = read.publication_set_id().to_owned();
        drop(read);
        drop(store);

        let store = LiveProjectStore::open_in_namespace(&store_path, &selection)?;
        assert_eq!(store.current()?, Some(current.clone()));
        assert_eq!(store.store.epoch(), &epoch);
        let read = store.read_in_namespace(
            &selection,
            &ReadSelector::Publication(current.record_id),
            &stop,
        )?;
        assert_eq!(read.project().snapshot_id(), project_id);
        assert_eq!(read.project().analyzer_snapshot_id(), analyzer_id);
        assert_eq!(read.graph(), &graph);
        assert_eq!(read.read.manifest().members, members);
        assert_eq!(read.publication_set_id(), set_id);
        let reopened = read
            .project()
            .configuration()
            .platform_packages()
            .ok_or("reopened packages missing")?;
        assert_eq!(reopened.source().receipt(), source.receipt());
        assert_eq!(reopened.binding(), packages.binding());
        assert_eq!(reopened.load_plan(), packages.load_plan());
        assert_eq!(reopened.main_plan(), packages.main_plan());
        for member in [
            "UI/Fixture.toc",
            "UI/defs.lua",
            "UI/frames.xml",
            "UI/opaque.bin",
            "UI/unloaded.lua",
            "UI/unloaded.xml",
        ] {
            assert_eq!(
                reopened.source().source_bytes(member)?,
                source.source_bytes(member)?
            );
        }
        assert!(!source_path.exists());
        drop(read);
        drop(store);

        let legacy_source_path = path.join("legacy-input");
        let legacy_bundle = platform_bundle_with_profile_source(
            &legacy_source_path,
            1,
            true,
            Some(PlatformGraphProfile::PackageProjectionWithRawInventoryV1),
            Some(&fixture),
            &stop,
        )?;
        assert!(!legacy_source_path.exists());
        let legacy_request = crate::graph::GraphBuildRequest::new(
            legacy_bundle.configuration().project_id().as_str().into(),
            "current".into(),
        )?
        .with_platform_graph_profile(PlatformGraphProfile::PackageProjectionWithRawInventoryV1);
        let (baseline, _) = legacy_request.live_publication_in_namespace(
            input(&legacy_bundle)?,
            selection.namespace(),
            &stop,
        )?;
        let legacy_path = path.join("frozen-v8");
        let mut legacy = LiveProjectStore {
            store: ProjectStore::create_with_namespace(
                &legacy_path,
                selection.namespace(),
                catalog_for(publication::STORAGE_SCHEMAS_V8)?,
            )?,
        };
        legacy.publish_bundle(baseline, "fixture:direct-v8-baseline", None, &stop)?;
        let old_current = legacy.current()?.ok_or("V8 Current missing")?;
        let old_epoch = legacy.store.epoch().clone();
        let old_read = legacy.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
        assert!(
            old_read
                .read
                .manifest()
                .members
                .iter()
                .any(|member| member.key == "live.project.replay"
                    && member.schema == "wow-project.live-replay.v8")
        );
        assert!(
            old_read
                .graph()
                .partition(wow_project::graph::SOURCE_GRAPH_PARTITION)
                .is_some()
        );
        let old_project = old_read.project().snapshot_id().to_owned();
        let old_graph = old_read.graph().clone();
        let old_members = old_read.read.manifest().members.clone();
        let old_set = old_read.publication_set_id().to_owned();
        drop(old_read);
        assert_eq!(
            legacy
                .publish_bundle(
                    direct_publication,
                    "fixture:direct-v9-refused",
                    Some(old_current.record_id.clone()),
                    &stop
                )
                .err()
                .ok_or("V8 admitted v9 replay")?
                .code(),
            ServiceErrorCode::IdentityMismatch
        );
        assert_eq!(legacy.current()?, Some(old_current.clone()));
        assert_eq!(legacy.store.epoch(), &old_epoch);
        assert!(legacy.reconcile("fixture:direct-v9-refused")?.is_none());
        drop(legacy);
        let legacy = LiveProjectStore::open_in_namespace(&legacy_path, &selection)?;
        assert_eq!(legacy.current()?, Some(old_current.clone()));
        assert_eq!(legacy.store.epoch(), &old_epoch);
        let read = legacy.read_in_namespace(
            &selection,
            &ReadSelector::Publication(old_current.record_id),
            &stop,
        )?;
        assert_eq!(read.project().snapshot_id(), old_project);
        assert_eq!(read.graph(), &old_graph);
        assert_eq!(read.read.manifest().members, old_members);
        assert_eq!(read.publication_set_id(), old_set);
    }
    std::fs::remove_dir_all(&path)?;
    Ok(())
}
