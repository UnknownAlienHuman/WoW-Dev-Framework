//! Selected named XML mutation uses fresh native member/catalog admission.
use super::*;

#[test]
fn selected_named_xml_neighborhood_tracks_distinct_bindings_with_unchanged_handler() -> TestResult {
    use crate::graph::{
        FunctionDirectNeighborhoodRecipe, FunctionFingerprintSelection, GraphReadStatus,
        execute_graph_build,
    };
    use wow_emmy::function_calls::SourceFunctionKind;
    use wow_graph::{
        GraphAssertionKind, GraphAssertionRef, GraphConfidence, GraphEdge, GraphLocalAssertion,
        GraphNode, GraphPartitionSnapshot, GraphRelationKind,
    };
    use wow_project::graph::fingerprints::{
        FunctionDirectNeighborhoodSelection, FunctionNeighborDirection, FunctionNeighborKind,
        FunctionNeighborhoodComponent, FunctionSourceSpanComponent,
    };
    use wow_project::load::ProjectPackageLoadIssueKind;

    const HANDLE: &str = "function Handle()\n    return true\nend\n";
    const FIRST: &str =
        r#"<Frame name="First"><Scripts><OnLoad function="Handle"/></Scripts></Frame>"#;
    const SECOND: &str =
        r#"<Frame name="Second"><Scripts><OnLoad function="Handle"/></Scripts></Frame>"#;
    const MISSING: &str =
        r#"<Frame name="Second"><Scripts><OnLoad function="MissingHandle"/></Scripts></Frame>"#;
    const TOC: &str = "## Interface: 120100\n## LoadOnDemand: 0\nmain.lua\nframes.xml\n";
    let stop = AtomicBool::new(false);
    // The existing genuine schema fixture supports finite 0/1 particles.
    // Two distinct sequential Frame refs admit one or two receivers; no repetition.
    let xsd = super::enumeration_xsd().replacen(
        r#"<xs:element ref="u:Frame"/><xs:element ref="u:Include" minOccurs="0"/>"#,
        r#"<xs:element ref="u:Frame"/><xs:element ref="u:Frame" minOccurs="0"/><xs:element ref="u:Include" minOccurs="0"/>"#,
        1,
    );
    let recipe = FunctionDirectNeighborhoodRecipe::ComposedCallsAndXmlHandlersV1;
    let mut span_observation = None;
    let mut comparable_digests = Vec::new();

    // Two distinct XML receivers avoid duplicate-edge inflation. The control
    // retains the valid first binding but cannot prove a complete neighborhood.
    for (extra, expected_xml) in [("", Some(1_u64)), (SECOND, Some(2)), (MISSING, None)] {
        let xml = format!(r#"<Ui xmlns="http://www.blizzard.com/wow/ui/">{FIRST}{extra}</Ui>"#);
        // Build every changed member before native source/catalog/profile admission.
        // No later manifest mutation or relabelling of sealed catalog IDs.
        let (root, _config) =
            super::super::classification_fixture_members(&xsd, &xml, Some(HANDLE), Some(TOC))?;
        let path = root.0.join("project.json");
        let (project, request, _, _) =
            super::super::super::function_fingerprints::declaration_query(&path, &stop)?;
        assert!(
            project
                .configuration()
                .xml_classification_profile()
                .is_some()
        );
        assert_eq!(
            project.configuration().physical_function_declarations(),
            None
        );
        assert_eq!(project.configuration().physical_function_signatures(), None);
        let packages = project
            .configuration()
            .platform_packages()
            .ok_or("native packages")?;
        assert!(!packages.load_plan().coverage().complete());
        assert!(
            packages.load_plan().issues().iter().any(|issue| {
                issue.kind == ProjectPackageLoadIssueKind::PackageFileClosurePartial
                    && issue.blocks_complete
            }),
            "legacy XML semantic completeness stays Partial"
        );
        assert_eq!(packages.main_plan().files().len(), 1);
        let main_file = packages
            .main_plan()
            .files()
            .first()
            .ok_or("physical Main file")?;
        let report = project
            .snapshot()
            .analyzer_binding()
            .function_call_report()
            .ok_or("genuine native function report")?;
        report.validate()?;
        assert!(report.source_health_complete());
        assert!(report.calls().is_empty());
        let mut closures = report.functions().iter().filter(|fact| {
            fact.kind() == SourceFunctionKind::Closure && fact.path() == main_file.project_path
        });
        let fact = closures.next().ok_or("genuine global Handle closure")?;
        assert!(closures.next().is_none(), "one physical handler closure");
        let file = project
            .snapshot()
            .analyzer_binding()
            .main_workspace()
            .file(fact.path())
            .ok_or("held physical Main source")?;
        let start = usize::try_from(fact.span().byte_start().ok_or("exact Handle span start")?)?;
        let end = usize::try_from(fact.span().byte_end().ok_or("exact Handle span end")?)?;
        let handler_bytes = file
            .text()
            .as_bytes()
            .get(start..end)
            .ok_or("native Handle byte span")?
            .to_vec();
        // Reuse the adjacent XML query's finite service output allowance.
        // Source-graph configuration and product defaults remain unchanged.
        let metadata_bytes = crate::graph::GRAPH_RESULT_MAX_BYTES;
        let selection = FunctionFingerprintSelection::new(
            vec![fact.fact_id().into()],
            1,
            handler_bytes.len(),
            metadata_bytes,
        )?
        .with_direct_neighborhood(recipe)?;
        let request = request.with_function_fingerprints(selection)?;
        let result = execute_graph_build(
            LocalProjectInput::from_config_path(&path, &stop)?,
            &request,
            &stop,
        )?;
        assert_eq!(
            result.status(),
            GraphReadStatus::Partial,
            "{:?}",
            result.failure_message()
        );
        let manifest = result
            .function_fingerprints()
            .ok_or("selected named XML manifest")?;
        assert!(manifest.is_partial());
        assert!(manifest.scope_conflicts().is_empty());
        assert_eq!(
            manifest.binding().project_snapshot_id(),
            project.snapshot_id()
        );
        assert_eq!(
            manifest.binding().function_analysis_id(),
            report.analysis_id()
        );
        let selected = manifest
            .selected_components()
            .ok_or("selected component receipt")?;
        assert_eq!(selected.direct_neighborhood(), Some(recipe));
        assert_eq!(selected.physical_declarations(), None);
        assert_eq!(selected.physical_signatures(), None);
        assert_eq!(selected.package_load_role(), None);
        assert_eq!(manifest.records().len(), 1);
        let record = manifest.records().first().ok_or("Handle record")?;
        assert_eq!(record.function_id(), fact.fact_id());
        assert!(record.declared_header().is_none());
        assert_eq!(
            serde_json::to_value(record.signature())?,
            "missing_signature"
        );
        assert_eq!(
            serde_json::to_value(record.declaration_shape())?,
            "missing_shape"
        );
        assert_eq!(
            serde_json::to_value(record.package_load_role())?,
            "not_selected"
        );
        let FunctionSourceSpanComponent::Available {
            digest,
            content_sha256,
            byte_length,
        } = record.source_span()
        else {
            return Err("Handle's native source span was not available".into());
        };
        assert_eq!(usize::try_from(*byte_length)?, handler_bytes.len());
        let span = (*digest, *content_sha256, *byte_length);
        if let Some((original_bytes, original_span)) = &span_observation {
            assert_eq!(&handler_bytes, original_bytes);
            assert_eq!(
                &span, original_span,
                "Handle source-span bytes/digests stay fixed"
            );
        } else {
            span_observation = Some((handler_bytes, span));
        }

        let graph: GraphPartitionSnapshot =
            serde_json::from_slice(&result.snapshot_bytes()?.ok_or("final native graph")?)?;
        let lookup = graph.producer_lookup(&stop)?;
        assert_eq!(lookup.scope(), manifest.binding().graph_scope());
        let scripts = graph
            .partition("wow-recognizers.xml-script-bindings")
            .ok_or("accepted named script partition")?;
        let calls = graph
            .partition("wow-recognizers.lua-direct-calls")
            .ok_or("accepted call partition")?;
        assert!(scripts.report().ready_for_publication());
        assert!(calls.report().accepted_relations().is_empty());
        let binding = manifest.neighborhood_binding().ok_or("shared binding")?;
        assert_eq!(binding.recipe(), recipe);
        assert_eq!(
            binding.prefix_snapshot_id(),
            manifest.binding().published_graph_snapshot_id()
        );
        assert_eq!(binding.final_snapshot_id(), graph.snapshot().snapshot_id());
        assert_eq!(binding.final_generation(), graph.snapshot().generation());
        assert_eq!(
            binding.registry_digest(),
            graph.registry().registry_digest()
        );
        assert_eq!(binding.call_partition_digest(), calls.partition_digest());
        assert_eq!(
            binding.call_validation_report_id(),
            calls.report().report_id()
        );
        assert_eq!(
            binding.script_partition_digest(),
            scripts.partition_digest()
        );
        assert_eq!(
            binding.script_validation_report_id(),
            scripts.report().report_id()
        );
        let handler = lookup.entity(lookup.scope(), record.support().entity_assertion(), &stop)?;
        assert_eq!(
            handler.accepted().node().node_id(),
            record.exact_owner().node_id()
        );
        let ownership = lookup.relation(
            lookup.scope(),
            record.support().incoming_ownership_assertion(),
            &stop,
        )?;
        assert_eq!(
            ownership.proposal().relation_kind_id(),
            "source_declaration_owns"
        );
        assert_eq!(
            ownership.accepted().edge().to(),
            record.exact_owner().node_id()
        );

        // Read original service receipts/facts only; do not deserialize a native
        // report or fabricate an admission. Exact proposals are resolved below.
        let wire = serde_json::to_value(&result)?;
        let recognition = &wire["script_assertion_recognition"];
        assert_eq!(recognition["scope"], serde_json::to_value(lookup.scope())?);
        let receipts = recognition["receipts"]
            .as_array()
            .ok_or("native script receipts")?;
        let facts = wire["provenance"]["script_bindings"]
            .as_array()
            .ok_or("native script binding facts")?;
        let sites = wire["provenance"]["script_sites"]
            .as_array()
            .ok_or("native named script sites")?;
        let admitted = expected_xml.unwrap_or(1);
        assert_eq!(u64::try_from(receipts.len())?, admitted);
        assert_eq!(u64::try_from(facts.len())?, admitted);
        assert_eq!(
            u64::try_from(scripts.report().accepted_relations().len())?,
            admitted
        );
        assert_eq!(sites.len(), if extra.is_empty() { 1 } else { 2 });
        let mut distinct_receivers = std::collections::BTreeSet::new();
        for accepted in scripts.report().accepted_relations() {
            let receipt = receipts
                .iter()
                .find(|r| r["proposal_id"] == accepted.proposal_id())
                .ok_or("accepted relation's real receipt")?;
            let fact = facts
                .iter()
                .find(|f| f["binding_id"] == receipt["binding_id"])
                .ok_or("receipt's real native binding")?;
            assert_eq!(fact["handler_kind"], "lua_source_function");
            let address = GraphAssertionRef::Producer {
                partition_id: scripts.partition_id().into(),
                batch_id: scripts.batch().batch_id().into(),
                assertion: GraphLocalAssertion {
                    kind: GraphAssertionKind::Relation,
                    proposal_id: accepted.proposal_id().into(),
                },
            };
            let resolved = lookup.relation(lookup.scope(), &address, &stop)?;
            assert_eq!(
                resolved.proposal().relation_kind_id(),
                "source_xml_sets_script"
            );
            assert_eq!(resolved.accepted().edge(), accepted.edge());
            assert_eq!(
                serde_json::to_value(resolved.proposal().source_handle_ids())?,
                fact["source_handle_ids"]
            );
            assert_eq!(
                serde_json::to_value(resolved.proposal().evidence_ids())?,
                fact["evidence_ids"]
            );
            let edge = accepted.edge();
            assert_eq!(edge.relation(), GraphRelationKind::SetsScript);
            assert_eq!(edge.confidence(), GraphConfidence::Derived);
            assert_eq!(edge.to(), record.exact_owner().node_id());
            let receiver = lookup
                .input_view()
                .node(edge.from())
                .ok_or("native XML receiver")?;
            assert_eq!(receiver.kind(), "xml_source_declaration");
            assert!(distinct_receivers.insert(edge.from().clone()));
            let mut final_nodes = Vec::new();
            for node in [receiver, handler.accepted().node()] {
                let published = GraphNode::new(
                    graph.snapshot().universe().clone(),
                    graph.snapshot().generation().clone(),
                    node.kind(),
                    node.owner_key(),
                    node.evidence_ids().to_vec(),
                    graph.snapshot().limits(),
                )?;
                assert!(graph.snapshot().node(published.node_id()).is_some());
                final_nodes.push(published.node_id().clone());
            }
            let final_edge = GraphEdge::new(
                final_nodes[0].clone(),
                final_nodes[1].clone(),
                edge.relation(),
                edge.confidence(),
                edge.evidence_ids().to_vec(),
                graph.snapshot().limits(),
            )?;
            assert_eq!(
                graph.snapshot().edge(final_edge.edge_id()),
                Some(&final_edge)
            );
        }

        let FunctionDirectNeighborhoodSelection::Selected(component) = record.direct_neighborhood()
        else {
            return Err("named XML neighborhood was not selected".into());
        };
        if let Some(expected_xml) = expected_xml {
            let FunctionNeighborhoodComponent::Comparable { digest, groups } = component.as_ref()
            else {
                return Err("closed native named-site census was not comparable".into());
            };
            assert_eq!(groups[0].relation_definition_id(), "lua_direct_calls");
            assert_eq!(groups[0].direction(), FunctionNeighborDirection::Incoming);
            assert_eq!(
                groups[0].neighbor_kind(),
                FunctionNeighborKind::LuaSourceFunction
            );
            assert_eq!(groups[0].multiplicity(), 0);
            assert_eq!(groups[1].relation_definition_id(), "lua_direct_calls");
            assert_eq!(groups[1].direction(), FunctionNeighborDirection::Outgoing);
            assert_eq!(
                groups[1].neighbor_kind(),
                FunctionNeighborKind::LuaSourceFunction
            );
            assert_eq!(groups[1].multiplicity(), 0);
            assert_eq!(groups[2].relation_definition_id(), "source_xml_sets_script");
            assert_eq!(groups[2].direction(), FunctionNeighborDirection::Incoming);
            assert_eq!(
                groups[2].neighbor_kind(),
                FunctionNeighborKind::XmlSourceDeclaration
            );
            assert_eq!(groups[2].multiplicity(), expected_xml);
            assert!(
                sites
                    .iter()
                    .all(|site| site["blockers"].as_array().is_some_and(Vec::is_empty))
            );
            comparable_digests.push(*digest);
        } else {
            let missing = sites
                .iter()
                .find(|site| {
                    site["queries"].as_array().is_some_and(|queries| {
                        queries
                            .iter()
                            .any(|query| query["query"] == "MissingHandle")
                    })
                })
                .ok_or("actual unresolved named site")?;
            let query = missing["queries"]
                .as_array()
                .ok_or("native site queries")?
                .iter()
                .find(|query| query["query"] == "MissingHandle")
                .ok_or("actual missing-name query")?;
            assert_eq!(query["outcome"]["state"], "not_unique");
            assert!(
                missing["binding_ids"]
                    .as_array()
                    .ok_or("missing site bindings")?
                    .is_empty()
            );
            assert!(matches!(
                component.as_ref(),
                FunctionNeighborhoodComponent::Unavailable { .. }
            ));
            let component_wire = serde_json::to_value(component)?;
            assert!(component_wire.get("digest").is_none());
            assert!(
                component_wire.get("groups").is_none(),
                "one valid edge cannot prove closure"
            );
        }
    }
    assert_eq!(comparable_digests.len(), 2);
    assert_ne!(comparable_digests[0], comparable_digests[1]);
    Ok(())
}
