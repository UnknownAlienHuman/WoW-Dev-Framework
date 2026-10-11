//! Shared native fixture lifecycle: selected publication/update/removal, exact
//! source-absent reopen and candidate-required Skeleton. No decoded authority.
use super::*;
use crate::{
    ServiceErrorCode,
    graph::{
        FunctionDirectNeighborhoodRecipe, FunctionFingerprintSelection, GRAPH_RESULT_MAX_BYTES,
        GraphBuildRequest, GraphReadStatus, XmlFingerprintSelection, XmlFingerprintTarget,
        XmlSchemaTypeShapeRecipe, execute_graph_build,
    },
    live_project::{
        LIVE_SKELETON_REQUEST_SCHEMA, LIVE_SKELETON_RESULT_SCHEMA, LiveProjectLibraryMode,
        LiveProjectPublishRequest, LiveProjectStore, LiveProjectUpdateRequest,
        PLATFORM_PROJECT_SELECTION_SCHEMA, PlatformProjectSelection, PlatformStoreSelection,
        execute_live_skeleton_read, publish_input_in_namespace, publish_selected_local_project,
        read_selected_live_project, update_selected_local_project,
    },
};
use wow_emmy::function_calls::PhysicalFunctionDeclarationRecipe;
use wow_graph::{
    GraphDirection, GraphEvidenceResolveLimits, GraphExplainLimits, GraphNeighborQuery,
    GraphNeighborReadLimits, GraphNeighborReadQuery, GraphPathConfidence, GraphQueryState,
};
use wow_project::{
    ProjectView,
    load::schema::classification::RegionTypeWitness,
    skeleton_input::{OpenSkeletonInputRequest, SkeletonInputLimits},
};
use wow_store::project::{ReadSelector, StoreGenerationId};

#[test]
fn selected_candidate_source_absent_reopen_reaches_skeleton_and_absence_refuses() -> TestResult {
    let stop = AtomicBool::new(false);
    // Reuse the existing reviewed classification fixture and its positive closed
    // TextureType shape; no second fixture manifest/schema authority is invented.
    let xsd = skeleton_xsd();
    let (root, mut config) = classification_fixture(&xsd)?;
    config["physical_function_declarations"] =
        serde_json::to_value(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1)?;
    write_json(&root, "project.json", &config)?;
    let path = root.0.join("project.json");
    let (project, request, ids, _, intent) = selected_skeleton_query(&path, &config, &stop)?;

    let ordinary_selection = PlatformStoreSelection::new(
        project.configuration().project_id().clone(),
        project
            .configuration()
            .platform_packages()
            .ok_or("admitted packages")?
            .source()
            .profile()
            .profile_id()
            .clone(),
    )?
    .with_xml_classification_catalog()
    .with_physical_function_declarations_catalog();
    let selected_selection = ordinary_selection.clone().with_composed_candidate_catalog();
    let mut selection_wire = json!({
        "schema": PLATFORM_PROJECT_SELECTION_SCHEMA,
        "owner_project_id": project.configuration().project_id(),
        "source_profile_id": project.configuration().platform_packages().ok_or("source profile")?
            .source().profile().profile_id(),
        "catalogs": {
            "xml_schema_classification": true,
            "physical_function_declarations": "declared_syntax_v1",
            "physical_function_signatures": "absent",
            "composed_candidate": true
        }
    });
    let selected_transport =
        PlatformProjectSelection::from_json_slice(&serde_json::to_vec(&selection_wire)?, &stop)?;
    let selected_root = root.0.join("selected-candidate-store");
    let publish =
        LiveProjectPublishRequest::new("fixture:skeleton-selected", "absent", true, true)?;
    let published = publish_selected_local_project(
        &path,
        &request,
        &selected_root,
        &selected_transport,
        &publish,
        &stop,
    )
    .map_err(|error| format!("selected candidate publication: {error:?}"))?;
    let published = serde_json::to_value(published)?;
    assert_eq!(published["status"], "activated");
    let exact = published["operation"]["generation_id"]
        .as_str()
        .ok_or("selected store generation")?
        .to_owned();

    // Genuine ordinary epoch supplies the absence premise, not a manufactured
    // candidate receipt or a raw ProjectView/graph constructor.
    let ordinary_root = root.0.join("ordinary-component-store");
    let ordinary_input = LocalProjectInput::from_config_path(&path, &stop)?;
    let ordinary_request =
        GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
            .with_input_profile(&ordinary_input);
    let ordinary_publish =
        LiveProjectPublishRequest::new("fixture:skeleton-ordinary", "absent", true, true)?;
    publish_input_in_namespace(
        ordinary_input,
        &ordinary_request,
        &ordinary_root,
        &ordinary_selection,
        &ordinary_publish,
        &stop,
    )
    .map_err(|error| format!("ordinary comparison publication: {error:?}"))?;
    drop(project);
    // These are only the existing temporary fixture files when ROOT runs this
    // test. No production source operation is performed by this private patch.
    std::fs::remove_dir_all(root.0.join("UI"))?;
    std::fs::remove_file(root.0.join("version.txt"))?;
    assert!(!root.0.join("UI/main.lua").exists());
    assert!(!root.0.join("UI/classification.xsd").exists());

    let (request, _) = source_absent_skeleton(
        &selected_root,
        &selected_transport,
        &selected_selection,
        &exact,
        &ids,
        &intent,
        &stop,
    )?;

    let ordinary = LiveProjectStore::open_in_namespace(&ordinary_root, &ordinary_selection)?;
    let absent = ordinary.read_in_namespace(&ordinary_selection, &ReadSelector::Current, &stop)?;
    assert!(absent.candidate().is_none());
    drop(absent);
    drop(ordinary);
    selection_wire["catalogs"]["composed_candidate"] = json!(false);
    let absent_transport =
        PlatformProjectSelection::from_json_slice(&serde_json::to_vec(&selection_wire)?, &stop)?;
    // Intent without candidate selection is rejected before any Skeleton read.
    assert_eq!(
        execute_live_skeleton_read(
            &ordinary_root,
            &absent_transport,
            "current",
            &request,
            &stop,
        )
        .err()
        .ok_or("absent-candidate request succeeded")?
        .code(),
        ServiceErrorCode::InvalidRequest
    );
    // Selecting candidate intent cannot reinterpret an ordinary epoch either.
    assert_eq!(
        execute_live_skeleton_read(
            &ordinary_root,
            &selected_transport,
            "current",
            &request,
            &stop,
        )
        .err()
        .ok_or("ordinary epoch bypassed candidate prerequisite")?
        .code(),
        ServiceErrorCode::IdentityMismatch
    );
    Ok(())
}

fn skeleton_xsd() -> String {
    CLASSIFICATION_XSD.replace(
        "<xs:complexType name=\"TextureType\"/>",
        r#"<xs:complexType name="TextureType"><xs:sequence>
          <xs:element ref="u:ShapeLeaf"/>
        </xs:sequence></xs:complexType>
        <xs:complexType name="ShapeLeafType"><xs:sequence>
          <xs:element name="leaf" minOccurs="0" maxOccurs="2"><xs:complexType/></xs:element>
        </xs:sequence></xs:complexType>
        <xs:element name="ShapeLeaf" type="u:ShapeLeafType"/>"#,
    )
}

type SelectedSkeletonQuery = (
    ProjectView,
    GraphBuildRequest,
    Vec<Box<str>>,
    usize,
    RegionTypeWitness,
);
fn selected_skeleton_query(
    path: &std::path::Path,
    config: &Value,
    stop: &AtomicBool,
) -> TestResult<SelectedSkeletonQuery> {
    let (project, request, ids, source_bytes) =
        super::super::function_fingerprints::declaration_query(path, stop)?;
    let functions = FunctionFingerprintSelection::new(
        ids.clone(),
        ids.len(),
        source_bytes,
        GRAPH_RESULT_MAX_BYTES,
    )?
    .with_direct_neighborhood(FunctionDirectNeighborhoodRecipe::ComposedCallsAndXmlHandlersV1)?;
    let request = request.with_function_fingerprints(functions)?;
    // This ordinary operation discovers exact native XML proposal IDs only.
    // It is never used as candidate authority or an independently chosen final owner.
    let discovery = execute_graph_build(
        LocalProjectInput::from_config_path(path, stop)?,
        &request,
        stop,
    )?;
    assert_eq!(
        discovery.status(),
        GraphReadStatus::Partial,
        "{:?}",
        discovery.failure_message()
    );
    let wire = serde_json::to_value(&discovery)?;
    let documents = wire["provenance"]["xml_classification"]["documents"]
        .as_array()
        .ok_or("native classification documents")?;
    let [document] = documents.as_slice() else {
        return Err("expected one fixture classification document".into());
    };
    let observations = document["report"]["occurrences"]
        .as_array()
        .ok_or("native XML observations")?;
    let projections = document["occurrences"]
        .as_array()
        .ok_or("native XML projections")?;
    assert_eq!(observations.len(), projections.len());
    let texture = observations
        .iter()
        .zip(projections)
        .filter(|(observation, _)| observation["qname"]["name"]["local_name"] == "Texture")
        .collect::<Vec<_>>();
    let [(observation, projection)] = texture.as_slice() else {
        return Err("expected one exact fixture Texture occurrence".into());
    };
    assert_eq!(observation["occurrence"], projection["occurrence"]);
    let intent: RegionTypeWitness = serde_json::from_value(
        config["xml_schema_classification"]["profile_selection"]["region_like"]["members"][0]
            .clone(),
    )?;
    let target = XmlFingerprintTarget::new(
        projection["occurrence_proposal_id"]
            .as_str()
            .ok_or("native Texture proposal")?
            .into(),
        Some(
            projection["class_proposal_id"]
                .as_str()
                .ok_or("native Texture class")?
                .into(),
        ),
    )
    .with_type_shape(intent.element, intent.type_component)?;
    let xml = XmlFingerprintSelection::new_with_type_shapes(
        vec![target],
        1,
        GRAPH_RESULT_MAX_BYTES,
        XmlSchemaTypeShapeRecipe::ClosedComplexTreeV1,
    )?;
    assert_eq!(
        serde_json::to_value(&xml)?["schema"],
        "wow-service/xml-fingerprint-selection/selected-components/1"
    );
    let request = request.with_xml_fingerprints(xml)?;
    Ok((project, request, ids, source_bytes, intent))
}

// Acquires the actual sealed candidate again, then uses the registered service
// operation after releasing the first store/read/lookup. JSON is inert assertions.
fn source_absent_skeleton(
    selected_root: &std::path::Path,
    selected_transport: &PlatformProjectSelection,
    selected_selection: &PlatformStoreSelection,
    exact: &str,
    ids: &[Box<str>],
    intent: &RegionTypeWitness,
    stop: &AtomicBool,
) -> TestResult<(Vec<u8>, Value)> {
    let reopened = read_selected_live_project(selected_root, selected_transport, exact, stop)
        .map_err(|error| format!("source-absent selected reopen: {error:?}"))?;
    assert_eq!(reopened.exit_code(), 2);
    let reopened: Value = serde_json::from_slice(reopened.canonical_bytes())?;
    let store = LiveProjectStore::open_in_namespace(selected_root, selected_selection)
        .map_err(|error| format!("typed selected store open: {error:?}"))?;
    let selector = ReadSelector::Exact(StoreGenerationId::parse(exact)?);
    let read = store
        .read_in_namespace(selected_selection, &selector, stop)
        .map_err(|error| format!("typed source-absent read: {error:?}"))?;
    let candidate = read
        .candidate()
        .ok_or("selected rebuild omitted genuine candidate")?;
    assert_eq!(
        reopened["candidate"]["candidate_id"],
        candidate.manifest().candidate_id()
    );
    assert_eq!(
        reopened["candidate"]["candidate_digest"],
        candidate.manifest().candidate_digest()
    );
    let functions = candidate
        .function_fingerprints()
        .ok_or("candidate function manifest")?;
    assert_eq!(functions.records().len(), ids.len());
    assert_eq!(
        functions
            .records()
            .iter()
            .map(|record| record.function_id())
            .collect::<Vec<_>>(),
        ids.iter().map(AsRef::as_ref).collect::<Vec<&str>>()
    );
    assert!(
        functions
            .records()
            .iter()
            .all(|record| record.declared_header().is_some())
    );
    assert_eq!(
        functions
            .selected_components()
            .ok_or("function component receipt")?
            .direct_neighborhood(),
        Some(FunctionDirectNeighborhoodRecipe::ComposedCallsAndXmlHandlersV1)
    );
    assert!(functions.neighborhood_binding().is_some());
    let xml = candidate
        .xml_fingerprints()
        .ok_or("candidate selected XML manifest")?;
    assert_eq!(
        xml.schema(),
        "wow-service/xml-fingerprint-manifest/selected-components/1"
    );
    assert_eq!(
        xml.type_shape_recipe(),
        Some(XmlSchemaTypeShapeRecipe::ClosedComplexTreeV1)
    );
    assert_eq!(
        xml.type_shape_for(0)
            .ok_or("candidate type-shape mapping")?
            .type_component(),
        intent.type_component
    );
    let raw = xml.type_shape_support(0).ok_or("candidate XML support")?;
    assert!(!raw.is_empty());
    assert!(
        raw.iter()
            .all(|support| support.source().path().as_str() == "UI/classification.xsd")
    );

    // Select the exact fixture function already named in the query. Resolve its
    // genuine native support in the candidate's one final immutable owner.
    let function = functions
        .records()
        .iter()
        .find(|record| record.function_id() == ids[0].as_ref())
        .ok_or("selected native function record")?;
    let lookup = candidate.graph().producer_lookup(stop)?;
    let accepted_node = lookup
        .entity(
            candidate.native().scope(),
            function.support().entity_assertion(),
            stop,
        )?
        .accepted()
        .node();
    // Accepted producer IDs belong to the input graph. The consumer selects the
    // same exact entity rebound by the graph owner into the final generation.
    let rebound = wow_graph::GraphNode::new(
        candidate.graph().snapshot().universe().clone(),
        candidate.graph().snapshot().generation().clone(),
        accepted_node.kind(),
        accepted_node.owner_key(),
        accepted_node.evidence_ids().to_vec(),
        candidate.graph().snapshot().limits(),
    )?;
    let root_node = candidate
        .graph()
        .snapshot()
        .node(rebound.node_id())
        .ok_or("exact selected entity is absent from final materialization")?
        .node_id()
        .clone();
    let relation = lookup
        .relation(
            candidate.native().scope(),
            function.support().incoming_ownership_assertion(),
            stop,
        )?
        .accepted()
        .edge()
        .relation();
    let neighborhood = GraphNeighborReadQuery::new(
        candidate.graph().snapshot().snapshot_id().clone(),
        GraphNeighborQuery::new(root_node.clone(), GraphDirection::Both, vec![relation], 8)?,
        GraphNeighborReadLimits::default(),
    )?
    .with_confidence(GraphPathConfidence::IncludeCandidate);
    let limits = SkeletonInputLimits {
        max_entities: 64,
        max_support_records: 4096,
        max_output_bytes: 1_048_576,
        root_explanation: GraphExplainLimits::default(),
        root_evidence: GraphEvidenceResolveLimits::default(),
    };
    let input =
        OpenSkeletonInputRequest::new(candidate.native(), candidate.graph(), neighborhood, limits)?;
    assert_eq!(
        serde_json::to_value(input.binding())?,
        reopened["candidate"]["skeleton_binding"]
    );
    let request = serde_json::to_vec(&json!({
        "schema": LIVE_SKELETON_REQUEST_SCHEMA,
        "candidate_id": candidate.manifest().candidate_id(),
        "candidate_digest": candidate.manifest().candidate_digest(),
        "input": input,
    }))?;
    let native = wow_project::skeleton_input::open_skeleton_input_view(
        candidate.native(),
        candidate.graph(),
        input.clone(),
        stop,
    )
    .map_err(|error| format!("held native Skeleton open: {error:?}"))?;
    let native_page = native
        .read(stop)
        .map_err(|error| format!("held native Skeleton page: {error:?}"))?;
    assert_eq!(native_page.state(), GraphQueryState::Partial);
    drop(native_page);
    drop(native);
    drop(lookup);
    drop(read);
    drop(store);
    let observed =
        execute_live_skeleton_read(selected_root, selected_transport, exact, &request, stop)
            .map_err(|error| format!("selected Skeleton service read: {error:?}"))?;
    assert_eq!(observed.state(), GraphQueryState::Partial);
    assert!(observed.canonical_bytes().len() <= limits.max_output_bytes as usize);
    let observed: Value = serde_json::from_slice(observed.canonical_bytes())?;
    assert_eq!(observed["schema"], LIVE_SKELETON_RESULT_SCHEMA);
    assert_eq!(
        observed["candidate_id"],
        reopened["candidate"]["candidate_id"]
    );
    assert_eq!(
        observed["candidate_digest"],
        reopened["candidate"]["candidate_digest"]
    );
    assert_eq!(observed["store_generation_id"], exact);
    assert_eq!(observed["page"]["body"]["state"], "partial");
    assert_eq!(observed["page"]["body"]["no_new_evidence"], false);
    assert_eq!(
        observed["page"]["body"]["neighborhood"]["root"]["node_id"],
        root_node.as_str()
    );
    assert!(
        observed["page"]["body"]["counts"]["returned_root_support_records"]
            .as_u64()
            .ok_or("support count")?
            > 0
    );
    assert!(
        observed["page"]["body"]["counts"]["root_source_handles"]
            .as_u64()
            .ok_or("handle count")?
            > 0
    );
    assert!(
        !observed["page"]["body"]["not_evaluated"]
            .as_array()
            .ok_or("explicit unavailable facets")?
            .is_empty()
    );
    Ok((request, observed))
}

#[test]
fn selected_candidate_update_removal_and_query_fences_reopen_to_skeleton() -> TestResult {
    let stop = AtomicBool::new(false);
    let xsd = skeleton_xsd();
    let extra = [(
        "UI/removable.lua",
        "function RemovedFunction() return false end\n",
    )];
    let both_toc = "main.lua\nremovable.lua\nframes.xml\n";
    let make =
        |lua: &str, toc: &str, extra_main: &[(&str, &str)]| -> TestResult<(FixtureRoot, Value)> {
            let (root, mut config) = classification_fixture_schema_and_main_members(
                &[("UI/classification.xsd", &xsd)],
                CLASSIFICATION_XML,
                Some(lua),
                Some(toc),
                extra_main,
            )?;
            config["physical_function_declarations"] =
                serde_json::to_value(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1)?;
            write_json(&root, "project.json", &config)?;
            Ok((root, config))
        };
    let (root, config) = make(
        "function KeptFunction() return true end\n",
        both_toc,
        &extra,
    )?;
    let path = root.0.join("project.json");
    let (project, graph, ids, _, intent) = selected_skeleton_query(&path, &config, &stop)?;
    let ordinary = PlatformStoreSelection::new(
        project.configuration().project_id().clone(),
        project
            .configuration()
            .platform_packages()
            .ok_or("source profile")?
            .source()
            .profile()
            .profile_id()
            .clone(),
    )?
    .with_xml_classification_catalog()
    .with_physical_function_declarations_catalog();
    let selected = ordinary.with_composed_candidate_catalog();
    let selection_wire = json!({
        "schema": PLATFORM_PROJECT_SELECTION_SCHEMA,
        "owner_project_id": project.configuration().project_id(),
        "source_profile_id": project.configuration().platform_packages().ok_or("source profile")?
            .source().profile().profile_id(),
        "catalogs": { "xml_schema_classification": true,
            "physical_function_declarations": "declared_syntax_v1",
            "physical_function_signatures": "absent", "composed_candidate": true }
    });
    let transport =
        PlatformProjectSelection::from_json_slice(&serde_json::to_vec(&selection_wire)?, &stop)?;
    let store_root = root.0.join("update-candidate-store");
    let initial = serde_json::to_value(publish_selected_local_project(
        &path,
        &graph,
        &store_root,
        &transport,
        &LiveProjectPublishRequest::new("fixture:candidate-update-base", "absent", true, true)?,
        &stop,
    )?)?;
    assert_eq!(initial["status"], "activated");
    let initial_exact = initial["operation"]["generation_id"]
        .as_str()
        .ok_or("base generation")?
        .to_owned();
    let base = LiveProjectStore::open_in_namespace(&store_root, &selected)?
        .current()?
        .ok_or("base Current")?;

    let (updated_root, updated_config) = make(
        "function KeptFunction() return false end\n",
        both_toc,
        &extra,
    )?;
    let updated_path = updated_root.0.join("project.json");
    let (updated_project, updated_graph, updated_ids, updated_bytes, updated_intent) =
        selected_skeleton_query(&updated_path, &updated_config, &stop)?;
    // Exact loaded Main declarations, not fixture annotations. A changed Lua
    // digest MUST reach activated, never be accepted as a declaration refusal.
    let before = LocalProjectInput::from_config_path(&path, &stop)?;
    let after = LocalProjectInput::from_config_path(&updated_path, &stop)?;
    assert_eq!(
        before.project_bundle().configuration().project_id(),
        after.project_bundle().configuration().project_id()
    );
    let before_main = before
        .project_bundle()
        .configuration()
        .platform_packages()
        .ok_or("old packages")?
        .main_plan()
        .resolve_source("Fixture", "main.lua")
        .ok_or("old Main path")?;
    let after_main = after
        .project_bundle()
        .configuration()
        .platform_packages()
        .ok_or("new packages")?
        .main_plan()
        .resolve_source("Fixture", "main.lua")
        .ok_or("new Main path")?;
    assert_eq!(before_main, after_main);
    let previous = before
        .project_bundle()
        .inventory()
        .files()
        .iter()
        .find(|file| file.relative_path().as_str() == before_main)
        .ok_or("old Main file")?;
    let next = after
        .project_bundle()
        .inventory()
        .files()
        .iter()
        .find(|file| file.relative_path().as_str() == after_main)
        .ok_or("new Main file")?;
    assert_eq!(previous.role(), next.role());
    assert_eq!(previous.language_kind(), next.language_kind());
    assert_eq!(previous.source_fixture_ref(), None);
    assert_eq!(next.source_fixture_ref(), previous.source_fixture_ref());
    assert_ne!(previous.content_digest(), next.content_digest());
    let removable_path = before
        .project_bundle()
        .configuration()
        .platform_packages()
        .ok_or("old packages")?
        .main_plan()
        .resolve_source("Fixture", "removable.lua")
        .ok_or("removable Main path")?
        .to_owned();
    let removable_id = project
        .file_by_path(&removable_path)?
        .ok_or("removable native file")?
        .file_id()
        .clone();
    let updated = serde_json::to_value(update_selected_local_project(
        &updated_path,
        &updated_graph,
        &store_root,
        &transport,
        &LiveProjectUpdateRequest::new(
            "fixture:candidate-content-update",
            base.record_id.as_str(),
            LiveProjectLibraryMode::Keep,
            true,
        )?,
        &stop,
    )?)?;
    assert_eq!(
        updated["status"], "activated",
        "positive text update was refused"
    );
    let updated_exact = updated["operation"]["generation_id"]
        .as_str()
        .ok_or("updated generation")?
        .to_owned();
    assert_ne!(updated_exact, initial_exact);
    let content_current = LiveProjectStore::open_in_namespace(&store_root, &selected)?
        .current()?
        .ok_or("updated Current")?;
    assert_ne!(content_current.record_id, base.record_id);
    let store = LiveProjectStore::open_in_namespace(&store_root, &selected)?;
    let active = store.read_in_namespace(
        &selected,
        &ReadSelector::Exact(StoreGenerationId::parse(&updated_exact)?),
        &stop,
    )?;
    assert_eq!(
        active.project().snapshot_id(),
        updated_project.snapshot_id()
    );
    assert_eq!(
        active
            .project()
            .file_by_path(after_main)?
            .ok_or("updated Main file")?
            .content_digest(),
        next.content_digest()
    );
    drop(active);
    drop(store);
    drop(before);
    drop(after);

    // The project/config/source is IDENTICAL: only the final query selection is
    // narrowed to one native fact. A changed candidate must still publish.
    assert!(updated_ids.len() > 1);
    let query_ids = vec![updated_ids[0].clone()];
    let query_graph = updated_graph.clone().with_function_fingerprints(
        FunctionFingerprintSelection::new(
            query_ids.clone(),
            1,
            updated_bytes,
            GRAPH_RESULT_MAX_BYTES,
        )?
        .with_direct_neighborhood(
            FunctionDirectNeighborhoodRecipe::ComposedCallsAndXmlHandlersV1,
        )?,
    )?;
    let queried = serde_json::to_value(update_selected_local_project(
        &updated_path,
        &query_graph,
        &store_root,
        &transport,
        &LiveProjectUpdateRequest::new(
            "fixture:candidate-query-update",
            content_current.record_id.as_str(),
            LiveProjectLibraryMode::Keep,
            true,
        )?,
        &stop,
    )?)?;
    assert_eq!(
        queried["status"], "activated",
        "query-only candidate change was discarded"
    );
    let query_exact = queried["operation"]["generation_id"]
        .as_str()
        .ok_or("query generation")?
        .to_owned();
    assert_ne!(query_exact, updated_exact);
    let store = LiveProjectStore::open_in_namespace(&store_root, &selected)?;
    let query_current = store.current()?.ok_or("query Current")?;
    assert_ne!(query_current.record_id, content_current.record_id);
    let old = store.read_in_namespace(
        &selected,
        &ReadSelector::Exact(StoreGenerationId::parse(&updated_exact)?),
        &stop,
    )?;
    let new = store.read_in_namespace(
        &selected,
        &ReadSelector::Exact(StoreGenerationId::parse(&query_exact)?),
        &stop,
    )?;
    assert_eq!(old.project().snapshot_id(), updated_project.snapshot_id());
    assert_eq!(new.project().snapshot_id(), old.project().snapshot_id());
    assert_eq!(
        new.project().project_generation(),
        old.project().project_generation()
    );
    assert_eq!(
        new.project().configuration().configuration_digest(),
        old.project().configuration().configuration_digest()
    );
    assert_eq!(
        new.project().analyzer_snapshot_id(),
        old.project().analyzer_snapshot_id()
    );
    assert_eq!(new.graph(), old.graph());
    assert_ne!(
        new.candidate()
            .ok_or("query candidate")?
            .manifest()
            .candidate_id(),
        old.candidate()
            .ok_or("content candidate")?
            .manifest()
            .candidate_id()
    );
    assert_eq!(
        new.candidate()
            .ok_or("query candidate")?
            .function_fingerprints()
            .ok_or("query manifest")?
            .records()
            .len(),
        1
    );
    drop(new);
    drop(old);
    drop(store);
    let unchanged = serde_json::to_value(update_selected_local_project(
        &updated_path,
        &query_graph,
        &store_root,
        &transport,
        &LiveProjectUpdateRequest::new(
            "fixture:candidate-query-no-change",
            query_current.record_id.as_str(),
            LiveProjectLibraryMode::Keep,
            true,
        )?,
        &stop,
    )?)?;
    assert_eq!(unchanged["status"], "no_change");
    assert!(unchanged.get("operation").is_none());
    let store = LiveProjectStore::open_in_namespace(&store_root, &selected)?;
    assert_eq!(store.current()?, Some(query_current.clone()));
    assert!(
        store
            .reconcile("fixture:candidate-query-no-change")?
            .is_none()
    );
    drop(store);

    let stale = LiveProjectUpdateRequest::new(
        "fixture:candidate-update-stale",
        base.record_id.as_str(),
        LiveProjectLibraryMode::Keep,
        true,
    )?;
    assert_eq!(
        update_selected_local_project(
            &updated_path,
            &query_graph,
            &store_root,
            &transport,
            &stale,
            &stop
        )
        .err()
        .ok_or("stale Current accepted")?
        .code(),
        ServiceErrorCode::StoreCurrentConflict
    );
    let cancelled = LiveProjectUpdateRequest::new(
        "fixture:candidate-update-cancelled",
        query_current.record_id.as_str(),
        LiveProjectLibraryMode::Keep,
        true,
    )?;
    // Pre-admission cancellation only; no timing claim about a late CAS probe.
    assert_eq!(
        update_selected_local_project(
            &updated_path,
            &query_graph,
            &store_root,
            &transport,
            &cancelled,
            &AtomicBool::new(true)
        )
        .err()
        .ok_or("cancelled update accepted")?
        .code(),
        ServiceErrorCode::Cancelled
    );
    let store = LiveProjectStore::open_in_namespace(&store_root, &selected)?;
    assert_eq!(store.current()?, Some(query_current.clone()));
    assert!(store.reconcile("fixture:candidate-update-stale")?.is_none());
    assert!(
        store
            .reconcile("fixture:candidate-update-cancelled")?
            .is_none()
    );
    drop(store);

    // Remove the physical member from BOTH exact source manifest and native TOC,
    // while keeping main.lua so the existing nonempty Main contract is satisfied.
    let (removed_root, removed_config) = make(
        "function KeptFunction() return false end\n",
        "main.lua\nframes.xml\n",
        &[],
    )?;
    let removed_path = removed_root.0.join("project.json");
    let (removed_project, removed_graph, removed_ids, _, removed_intent) =
        selected_skeleton_query(&removed_path, &removed_config, &stop)?;
    let packages = removed_project
        .configuration()
        .platform_packages()
        .ok_or("removed packages")?;
    assert!(
        packages
            .source()
            .receipt()
            .inventory()
            .entries
            .iter()
            .all(|entry| entry.path != "UI/removable.lua")
    );
    assert!(
        packages
            .main_plan()
            .resolve_source("Fixture", "removable.lua")
            .is_none()
    );
    assert!(
        packages
            .main_plan()
            .resolve_source("Fixture", "main.lua")
            .is_some()
    );
    assert!(!removed_root.0.join("UI/removable.lua").exists());
    assert!(removed_project.file_by_id(&removable_id).is_none());
    let removed_report = removed_project
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("removed report")?;
    assert!(
        removed_report
            .functions()
            .iter()
            .all(|fact| fact.path() != removable_path)
    );
    assert!(removed_ids.len() < updated_ids.len());
    let removed = serde_json::to_value(update_selected_local_project(
        &removed_path,
        &removed_graph,
        &store_root,
        &transport,
        &LiveProjectUpdateRequest::new(
            "fixture:candidate-member-remove",
            query_current.record_id.as_str(),
            LiveProjectLibraryMode::Keep,
            true,
        )?,
        &stop,
    )?)?;
    assert_eq!(
        removed["status"], "activated",
        "positive physical removal was refused"
    );
    let removed_exact = removed["operation"]["generation_id"]
        .as_str()
        .ok_or("removal generation")?
        .to_owned();
    assert_ne!(removed_exact, query_exact);
    let store = LiveProjectStore::open_in_namespace(&store_root, &selected)?;
    let removal_current = store.current()?.ok_or("removal Current")?;
    assert_ne!(removal_current.record_id, query_current.record_id);
    let read = store.read_in_namespace(
        &selected,
        &ReadSelector::Exact(StoreGenerationId::parse(&removed_exact)?),
        &stop,
    )?;
    assert_eq!(read.project().snapshot_id(), removed_project.snapshot_id());
    assert!(read.project().file_by_id(&removable_id).is_none());
    assert!(read.project().file_by_path(&removable_path)?.is_none());
    assert!(
        read.project()
            .snapshot()
            .analyzer_binding()
            .function_call_report()
            .ok_or("reopened removed report")?
            .functions()
            .iter()
            .all(|fact| fact.path() != removable_path)
    );
    assert_ne!(
        read.project().project_generation(),
        updated_project.project_generation()
    );
    drop(read);
    drop(store);
    let updated_snapshot = updated_project.snapshot_id().to_owned();
    let removed_snapshot = removed_project.snapshot_id().to_owned();
    drop(project);
    drop(updated_project);
    drop(removed_project);
    // Optional root-owned CLI export: preserve final bytes before fixture deletion.
    let cli_export = export_cli_fixture_if_requested(
        &removed_root,
        &[&root, &updated_root, &removed_root],
        &removed_graph,
        &selection_wire,
    )?;
    // Only temporary fixture files, only when root executes the test. Every
    // reopened generation below must rebuild from the same retained owner route.
    for fixture in [&root, &updated_root, &removed_root] {
        std::fs::remove_dir_all(fixture.0.join("UI"))?;
        std::fs::remove_file(fixture.0.join("version.txt"))?;
        assert!(!fixture.0.join("UI/main.lua").exists());
        assert!(!fixture.0.join("UI/classification.xsd").exists());
    }
    let (_, initial_skeleton) = source_absent_skeleton(
        &store_root,
        &transport,
        &selected,
        &initial_exact,
        &ids,
        &intent,
        &stop,
    )?;
    let (content_request, content_skeleton) = source_absent_skeleton(
        &store_root,
        &transport,
        &selected,
        &updated_exact,
        &updated_ids,
        &updated_intent,
        &stop,
    )?;
    let (_, query_skeleton) = source_absent_skeleton(
        &store_root,
        &transport,
        &selected,
        &query_exact,
        &query_ids,
        &updated_intent,
        &stop,
    )?;
    let (removed_request, removed_skeleton) = source_absent_skeleton(
        &store_root,
        &transport,
        &selected,
        &removed_exact,
        &removed_ids,
        &removed_intent,
        &stop,
    )?;
    assert_ne!(
        initial_skeleton["candidate_id"],
        content_skeleton["candidate_id"]
    );
    assert_ne!(
        content_skeleton["candidate_id"],
        query_skeleton["candidate_id"]
    );
    assert_ne!(
        query_skeleton["candidate_id"],
        removed_skeleton["candidate_id"]
    );
    // Exact source-absent snapshots, removal invalidation and final Current all
    // survive actual selected reopen, not only the prior in-memory observations.
    let store = LiveProjectStore::open_in_namespace(&store_root, &selected)?;
    let content = store.read_in_namespace(
        &selected,
        &ReadSelector::Exact(StoreGenerationId::parse(&updated_exact)?),
        &stop,
    )?;
    assert_eq!(content.project().snapshot_id(), updated_snapshot);
    assert!(content.project().file_by_id(&removable_id).is_some());
    let removal = store.read_in_namespace(
        &selected,
        &ReadSelector::Exact(StoreGenerationId::parse(&removed_exact)?),
        &stop,
    )?;
    assert_eq!(removal.project().snapshot_id(), removed_snapshot);
    assert!(removal.project().file_by_id(&removable_id).is_none());
    assert!(
        removal
            .project()
            .snapshot()
            .analyzer_binding()
            .function_call_report()
            .ok_or("source-absent removed report")?
            .functions()
            .iter()
            .all(|fact| fact.path() != removable_path)
    );
    assert_eq!(store.current()?, Some(removal_current));
    if let Some(destination) = &cli_export {
        // Existing genuine content candidate is stale against the final removal.
        std::fs::write(destination.join("skeleton-stale.json"), &content_request)?;
        std::fs::write(destination.join("skeleton.json"), &removed_request)?;
    }
    Ok(())
}

/// Test-only explicit export. No file/serialization work when the root flag is absent.
fn export_cli_fixture_if_requested(
    source: &FixtureRoot,
    fixtures: &[&FixtureRoot],
    graph: &GraphBuildRequest,
    selection: &Value,
) -> TestResult<Option<std::path::PathBuf>> {
    let Some(destination) = std::env::var_os("WOW_W17_116_CLI_EXPORT_ROOT") else {
        return Ok(None);
    };
    let destination = std::path::PathBuf::from(destination);
    if !destination.is_absolute() {
        return Err("CLI fixture export requires an explicit absolute directory".into());
    }
    // Resolve the existing parent before creating the fresh output; it must
    // survive every FixtureRoot destructor and the existing source removals.
    let parent = destination
        .parent()
        .ok_or("CLI export parent")?
        .canonicalize()?;
    let destination = parent.join(destination.file_name().ok_or("CLI export directory name")?);
    for fixture in fixtures {
        if destination.starts_with(fixture.0.canonicalize()?) {
            return Err("CLI export must be outside the temporary fixture roots".into());
        }
    }
    std::fs::create_dir(&destination)?;
    std::fs::create_dir(destination.join("UI"))?;
    // Exact final removal fixture; no directory scan, store copy or second parser.
    for path in [
        "project.json",
        "manifest.json",
        "reference.json",
        "compatibility.json",
        "version.txt",
        "UI/Fixture.toc",
        "UI/main.lua",
        "UI/frames.xml",
        "UI/classification.xsd",
    ] {
        std::fs::copy(source.0.join(path), destination.join(path))?;
    }
    std::fs::write(
        destination.join("selection.json"),
        serde_json::to_vec(selection)?,
    )?;
    let request = serde_json::to_value(graph)?;
    for (file, field) in [
        ("function-query.json", "function_fingerprints"),
        ("xml-query.json", "xml_fingerprints"),
    ] {
        let query = request
            .get(field)
            .filter(|value| value.is_object())
            .ok_or("CLI export requires both actual selected query recipes")?;
        std::fs::write(destination.join(file), serde_json::to_vec(query)?)?;
    }
    Ok(Some(destination))
}
