//! Minimum native source-reference input through the existing fixture owners.
use super::*;
use wow_project::load::xml_references::{XmlReferenceKind, XmlReferenceResolution};

mod consumer;

const WORK: &str = ".codex/work/platform-xml-reference-native-20261012";
const XML: &str = r#"<Ui xmlns="http://www.blizzard.com/wow/ui/">
<Frame name="ReferenceTemplate" virtual="true"/>
<Frame name="ReferenceChild" inherits="ReferenceTemplate,MissingTemplate" parent="ReferenceTemplate"/>
<Frame name="NativeFrame"><Scripts><OnLoad>local loaded = true</OnLoad></Scripts></Frame>
<Texture name="NativeRegion"/><Custom/><FrameRef/></Ui>"#;

#[test]
#[ignore = "qualify the retained owned preparation stores once; never rebuild the fixture"]
fn qualify_retained_platform_xml_reference_root() -> TestResult {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let work = repo.join(WORK);
    let stop = AtomicBool::new(false);
    let baseline: Value = serde_json::from_slice(&bounded_bytes(
        &repo.join(BASELINE_RELATIVE).join("host.json"),
        MAX_HOST_BYTES,
    )?)?;
    let transport = PlatformProjectSelection::from_json_slice(
        &wow_core::canonical_json_bytes(&baseline["platform"]["selection"])?,
        &stop,
    )?;
    let primary_store = work.join("registered-primary-store");
    let platform_store = work.join("registered-platform-store");
    // The preparation owned and activated these fresh stores. Resolve Current
    // once per owner solely to reconcile the interrupted recording, then bind
    // exact generations; no Service request uses Current or a replacement root.
    let primary = LiveProjectStore::open(&primary_store)?;
    let primary_read = primary.read(&ReadSelector::Current, &stop)?;
    let primary_exact = primary_read.store_generation_id().clone();
    let primary_profile =
        serde_json::to_value(primary_read.project().configuration().selected_profile())?;
    let primary_reference = primary_read
        .project()
        .configuration()
        .reference_generation()
        .to_string();
    primary_read.close()?;
    primary.close()?;
    let platform = LiveProjectStore::open_in_namespace(&platform_store, transport.owner())?;
    let platform_read =
        platform.read_in_namespace(transport.owner(), &ReadSelector::Current, &stop)?;
    let platform_exact = platform_read.store_generation_id().clone();
    let platform_profile =
        serde_json::to_value(platform_read.project().configuration().selected_profile())?;
    let platform_reference = platform_read
        .project()
        .configuration()
        .reference_generation()
        .to_string();
    platform_read.close()?;
    platform.close()?;
    assert_eq!(primary_profile, baseline["primary"]["reference_profile"]);
    assert_eq!(platform_profile, primary_profile);
    assert_eq!(
        primary_reference,
        baseline["primary"]["reference"]["generation_id"]
    );
    assert_eq!(platform_reference, primary_reference);
    let (primary_root, primary_publication, _, _, _) =
        combined_primary_observation(&primary_store, &primary_exact, &stop)?;
    let timer = Instant::now();
    let original =
        qualify_reference_original(&platform_store, transport.owner(), &platform_exact, &stop)?;
    let qualification_ms = timer.elapsed().as_millis();
    let mut host = baseline;
    host["primary"]["project_store_id"] = json!("project-store:xml-reference-primary");
    host["primary"]["project_store_root"] = json!(primary_store);
    host["platform"]["project_store_id"] = json!("project-store:xml-reference-platform");
    host["platform"]["project_store_root"] = json!(platform_store);
    let request = json!({"schema":CONTEXT_COMBINED_REQUEST_SCHEMA,"operation":"inspect","profile_id":COMBINED_NATIVE_MAP_PROFILE_ID,
        "primary":{"project_store_id":"project-store:xml-reference-primary","store_generation_id":primary_exact,
            "publication_set_id":primary_publication,"map_profile_id":PRIMARY_NAVIGATION_MAP_PROFILE_ID,"root":ContextMapRoot::GraphEntity(primary_root)},
        "platform":original["platform_request"],"inspection":{"level":"l0","platform_root_facet":"xml_reference_metadata"}});
    let proof = json!({"original":original,"resource_closure":"closed","original_root_qualification_ms":qualification_ms,
        "retained_fixture_reused_without_publication":true,"selector_resolution":"owned_fixture_current_once_then_exact",
        "host_sha256":digest_hex(&wow_core::canonical_json_bytes(&host)?),
        "request_sha256":digest_hex(&wow_core::canonical_json_bytes(&request)?),
        "previous_fixture_publication_phase_times":"NotEvaluated: interrupted recording preserved",
        "relation_acceptance_current_source_runtime":"NotEvaluated"});
    write_new_capped(
        &work.join("host.json"),
        &wow_core::canonical_json_bytes(&host)?,
        MAX_HOST_BYTES,
    )?;
    write_new_capped(
        &work.join("inspect-request.json"),
        &wow_core::canonical_json_bytes(&request)?,
        MAX_REQUEST_BYTES,
    )?;
    write_new_capped(
        &work.join("prepared.json"),
        &wow_core::canonical_json_bytes(&proof)?,
        MAX_SELECTED_EXPECTED_BYTES,
    )?;
    println!(
        "retained XML reference qualification {}ms; counts={}",
        qualification_ms, proof["original"]["counts"]
    );
    Ok(())
}

#[test]
#[ignore = "one minimum original XML reference preparation, not full source/corpus acceptance"]
fn prepare_platform_xml_reference_root() -> TestResult {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let work = repo.join(WORK);
    std::fs::create_dir(&work)?;
    let stop = AtomicBool::new(false);
    let timer = Instant::now();
    let setup_started = Instant::now();
    let fixture = selected_skeleton_fixture_from_xml_seed(&stop, false, XML)?;
    let input = LocalProjectInput::from_config_path(&fixture.path, &stop)?;
    let reference = input.reference_view().clone();
    let profile = input
        .project_bundle()
        .configuration()
        .selected_profile()
        .clone();
    drop(input);
    let (primary_path, primary_query) =
        combined_primary_fixture(&fixture.root, &fixture.config, &profile, &reference, &stop)?;
    let setup_ms = setup_started.elapsed().as_millis();
    let primary_store = work.join("registered-primary-store");
    let platform_store = work.join("registered-platform-store");
    let primary_started = Instant::now();
    let primary = serde_json::to_value(publish_local_project(
        &primary_path,
        &primary_query,
        &primary_store,
        &LiveProjectPublishRequest::new("fixture:xml-reference-primary", "absent", true, true)?,
        &stop,
    )?)?;
    let primary_ms = primary_started.elapsed().as_millis();
    assert_eq!(primary["status"], "activated");
    let primary_exact = StoreGenerationId::parse(
        primary["operation"]["generation_id"]
            .as_str()
            .ok_or("Primary generation")?,
    )?;
    let (primary_root, primary_publication, _, _, _) =
        combined_primary_observation(&primary_store, &primary_exact, &stop)?;
    let platform_started = Instant::now();
    let platform = serde_json::to_value(publish_selected_local_project(
        &fixture.path,
        &fixture.request,
        &platform_store,
        &fixture.selected_transport,
        &LiveProjectPublishRequest::new("fixture:xml-reference-platform", "absent", true, true)?,
        &stop,
    )?)?;
    let platform_ms = platform_started.elapsed().as_millis();
    assert_eq!(platform["status"], "activated");
    let platform_exact = StoreGenerationId::parse(
        platform["operation"]["generation_id"]
            .as_str()
            .ok_or("platform generation")?,
    )?;
    let qualification_started = Instant::now();
    let original = qualify_reference_original(
        &platform_store,
        &fixture.selected_selection,
        &platform_exact,
        &stop,
    )?;
    let qualification_ms = qualification_started.elapsed().as_millis();
    let limits = json!({"max_selected_roots":64,"max_neighbors":1,"max_input_bytes":8_000_000,
        "max_metadata_bytes":16*1024*1024,"max_context_work":16_000_000});
    let host = json!({"schema":CONTEXT_COMBINED_HOST_SCHEMA,
        "primary":{"schema":CONTEXT_MAP_HOST_SCHEMA,"project_store_id":"project-store:xml-reference-primary",
            "project_store_root":primary_store,"project_id":"fixture-context-primary-map","reference_profile":profile,"reference":reference,"limits":limits},
        "platform":{"schema":CONTEXT_PLATFORM_HOST_SCHEMA,"project_store_id":"project-store:xml-reference-platform",
            "project_store_root":platform_store,"selection":fixture.selection_wire,"reference_profile":profile,"reference":reference,"limits":limits}});
    let request = json!({"schema":CONTEXT_COMBINED_REQUEST_SCHEMA,"operation":"inspect","profile_id":COMBINED_NATIVE_MAP_PROFILE_ID,
        "primary":{"project_store_id":"project-store:xml-reference-primary","store_generation_id":primary_exact,
            "publication_set_id":primary_publication,"map_profile_id":PRIMARY_NAVIGATION_MAP_PROFILE_ID,"root":ContextMapRoot::GraphEntity(primary_root)},
        "platform":original["platform_request"],"inspection":{"level":"l0","platform_root_facet":"xml_reference_metadata"}});
    write_new_capped(
        &work.join("host.json"),
        &wow_core::canonical_json_bytes(&host)?,
        MAX_HOST_BYTES,
    )?;
    write_new_capped(
        &work.join("inspect-request.json"),
        &wow_core::canonical_json_bytes(&request)?,
        MAX_REQUEST_BYTES,
    )?;
    let proof = json!({"original":original,"host_sha256":digest_hex(&wow_core::canonical_json_bytes(&host)?),
        "request_sha256":digest_hex(&wow_core::canonical_json_bytes(&request)?),"resource_closure":"closed",
        "phase_ms":{"fixture_and_primary_setup":setup_ms,"primary_publication":primary_ms,"platform_publication":platform_ms,
            "original_root_qualification":qualification_ms},"total_ms":timer.elapsed().as_millis(),
        "relation_acceptance_current_source_runtime":"NotEvaluated"});
    write_new_capped(
        &work.join("prepared.json"),
        &wow_core::canonical_json_bytes(&proof)?,
        MAX_SELECTED_EXPECTED_BYTES,
    )?;
    println!(
        "XML reference preparation phases={}, counts={}",
        proof["phase_ms"], proof["original"]["counts"]
    );
    Ok(())
}

fn qualify_reference_original(
    platform_store: &Path,
    selection: &PlatformStoreSelection,
    platform_exact: &StoreGenerationId,
    stop: &AtomicBool,
) -> TestResult<Value> {
    let store = LiveProjectStore::open_in_namespace(platform_store, selection)?;
    let read = store.read_in_namespace(
        selection,
        &ReadSelector::Exact(platform_exact.clone()),
        stop,
    )?;
    let original = (|| -> TestResult<Value> {
        let candidate = read.candidate().ok_or("original native candidate")?;
        let native = candidate.native();
        let source = native.source();
        let graph = candidate.graph();
        let plans = native
            .project()
            .configuration()
            .package_load_plan()
            .ok_or("original retained package plans")?;
        assert!(plans.packages().len() <= 64 && source.xml_declarations().len() <= 4096);
        let mut selected = None;
        for package in plans.packages() {
            let plan = plans
                .package_plan(&package.package)
                .ok_or("retained selected plan")?;
            assert_eq!(plan.digest(), package.selected_plan_digest);
            let report = plan.xml_references();
            assert!(report.references().len() <= 65536 && report.declarations().len() <= 65536);
            // This fixture names the input; Service receives only the frozen
            // exact occurrence/accepted node below and performs no name lookup.
            for site in report
                .declarations()
                .values()
                .filter(|site| site.name.as_deref() == Some("ReferenceChild"))
            {
                let path = plans
                    .source_path(&package.package, &site.document)
                    .ok_or("qualified source document")?;
                let mut declarations = source
                    .xml_declarations()
                    .iter()
                    .filter(|row| row.occurrence_id == site.occurrence_id && row.path == path);
                let declaration = declarations.next().ok_or("original source declaration")?;
                assert!(declarations.next().is_none());
                assert!(
                    selected
                        .replace((package, plan, report, site, declaration))
                        .is_none()
                );
            }
        }
        let (package, plan, report, site, declaration) =
            selected.ok_or("fixture declaration site")?;
        let references = report
            .references()
            .iter()
            .filter(|row| row.source_id == site.occurrence_id)
            .collect::<Vec<_>>();
        assert_eq!(references.len(), 3);
        assert!(
            references
                .iter()
                .any(|row| row.kind == XmlReferenceKind::Parent)
        );
        assert!(
            references
                .iter()
                .any(|row| row.kind == XmlReferenceKind::Inherits
                    && matches!(
                        &row.resolution,
                        XmlReferenceResolution::UniqueLocalDeclaration { .. }
                    ))
        );
        assert!(
            references
                .iter()
                .any(|row| row.kind == XmlReferenceKind::Inherits
                    && row.resolution == XmlReferenceResolution::NotInCapturedScope)
        );
        let projections = source
            .xml_inheritance()
            .iter()
            .filter(|row| {
                references
                    .iter()
                    .any(|reference| row.reference_id == reference.reference_id)
            })
            .collect::<Vec<_>>();
        assert_eq!(projections.len(), 2);
        for projection in &projections {
            assert_eq!(
                projection.package.as_deref(),
                Some(package.package.as_str())
            );
            assert_eq!(projection.load_plan_digest, Some(plan.digest()));
            assert_eq!(
                projection.document.as_deref(),
                Some(declaration.path.as_str())
            );
        }
        let assertion = native
            .assertion(&GraphLocalAssertion {
                kind: GraphAssertionKind::Entity,
                proposal_id: declaration.proposal_id.clone().into(),
            })
            .ok_or("original declaration assertion")?;
        let lookup = graph.producer_lookup(stop)?;
        let resolved = lookup.entity(native.scope(), assertion, stop)?;
        assert_eq!(
            resolved.proposal().entity_kind_id(),
            "xml_source_declaration"
        );
        assert_eq!(
            resolved.proposal().source_handle_ids(),
            std::slice::from_ref(&declaration.source_handle_id)
        );
        assert_eq!(
            resolved.proposal().evidence_ids(),
            std::slice::from_ref(&declaration.evidence_id)
        );
        let original_root = resolved.accepted().node();
        let rebound = GraphNode::new(
            graph.snapshot().universe().clone(),
            graph.snapshot().generation().clone(),
            original_root.kind(),
            original_root.owner_key(),
            original_root.evidence_ids().to_vec(),
            graph.snapshot().limits(),
        )?;
        let root = graph
            .snapshot()
            .node(rebound.node_id())
            .ok_or("exact final root")?;
        let ownership_assertion = native
            .assertion(&GraphLocalAssertion {
                kind: GraphAssertionKind::Relation,
                proposal_id: declaration.ownership_proposal_id.clone().into(),
            })
            .ok_or("original declaration ownership assertion")?;
        let ownership = lookup
            .relation(native.scope(), ownership_assertion, stop)?
            .accepted()
            .edge()
            .relation();
        let neighborhood = GraphNeighborReadQuery::new(
            graph.snapshot().snapshot_id().clone(),
            GraphNeighborQuery::new(
                root.node_id().clone(),
                GraphDirection::Both,
                vec![ownership],
                1,
            )?,
            GraphNeighborReadLimits::default(),
        )?
        .with_confidence(GraphPathConfidence::IncludeCandidate);
        let input = OpenSkeletonInputRequest::new(
            native,
            graph,
            neighborhood,
            SkeletonInputLimits {
                max_entities: 64,
                max_support_records: 4096,
                max_output_bytes: 1_048_576,
                root_explanation: GraphExplainLimits::default(),
                root_evidence: GraphEvidenceResolveLimits::default(),
            },
        )?;
        let view = open_skeleton_input_view(native, graph, input.clone(), stop)?;
        let page = view.read(stop)?;
        assert!(std::ptr::eq(root, page.neighborhood().root()));
        assert!(page.root_support().explanation().support_complete());
        assert!(page.root_support().explanation().derivation_complete());
        assert!(page.root_support().explanation().conflicts().is_empty());
        assert!(page.root_support().evidence_resolution().closure_complete());
        let handle = source
            .source_handles()
            .get(&declaration.source_handle_id)
            .ok_or("original source handle")?;
        let evidence = source
            .evidence()
            .get(&declaration.evidence_id)
            .ok_or("original evidence")?;
        assert_eq!(handle.path().as_str(), declaration.path);
        assert_eq!(handle.content_digest(), &site.content_digest);
        assert!(
            page.root_support()
                .evidence_resolution()
                .source_handles()
                .contains(&handle)
        );
        assert!(
            page.root_support()
                .evidence_resolution()
                .evidence_records()
                .contains(&evidence)
        );
        let targets = references
            .iter()
            .filter_map(|reference| match &reference.resolution {
                XmlReferenceResolution::UniqueLocalDeclaration { declaration_id }
                | XmlReferenceResolution::InvalidTarget { declaration_id } => {
                    report.declarations().get(declaration_id)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let platform_request = json!({"schema":CONTEXT_PLATFORM_REQUEST_SCHEMA,"operation":"map",
            "profile_id":PLATFORM_NATIVE_ONE_HOP_PROFILE_ID,"project_store_id":"project-store:xml-reference-platform",
            "store_generation_id":platform_exact,"publication_set_id":read.publication_set_id(),
            "candidate_id":candidate.manifest().candidate_id(),"candidate_digest":candidate.manifest().candidate_digest(),"input":input});
        Ok(json!({"root":root.node_id(),"original_root_declaration":{
            "selection":"xml_declaration_metadata",
            "interpretation":"original_accepted_root_xml_declaration_identity_span_no_classification_or_runtime_resolution",
            "source_context":source.context(),"assertion":assertion,"root":root,"declaration":declaration,
            "source_handle":handle,"evidence":evidence,"root_support":page.root_support()},
            "original_site":site,"original_references":references,"original_projection_receipts":projections,
            "original_target_sites":targets,"package":package.package,"selected_plan_digest":plan.digest(),"report_digest":report.digest(),
            "original_page":page,"platform_request":platform_request,
            "counts":{"xml_declarations":source.xml_declarations().len(),"xml_projection_receipts":source.xml_inheritance().len(),
                "root_references":references.len(),"root_inheritance_receipts":projections.len(),"graph_nodes":graph.snapshot().nodes().len()}}))
    })();
    let read_closed = read.close();
    let store_closed = store.close();
    read_closed?;
    store_closed?;
    original
}
