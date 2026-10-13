//! Exact original XML declaration input; retained fixture, no source generation.
use super::*;

const WORK: &str = ".codex/work/platform-xml-declaration-native-20261012";
const MAX_DECLARATIONS: usize = 4096;

mod common_path;
mod consumer;

#[test]
#[ignore = "one bounded original native declaration preparation, not product acceptance"]
fn prepare_platform_xml_declaration_root() -> TestResult {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let work = repo.join(WORK);
    std::fs::create_dir_all(&work)?;
    let original = repo.join(BASELINE_RELATIVE);
    let host_bytes = bounded_bytes(&original.join("host.json"), MAX_HOST_BYTES)?;
    let request_bytes = bounded_bytes(&original.join("inspect-request.json"), MAX_REQUEST_BYTES)?;
    let host: Value = serde_json::from_slice(&host_bytes)?;
    let mut request: Value = serde_json::from_slice(&request_bytes)?;
    let stop = AtomicBool::new(false);
    let selection = PlatformProjectSelection::from_json_slice(
        &wow_core::canonical_json_bytes(&host["platform"]["selection"])?,
        &stop,
    )?;
    let store_path = Path::new(
        host["platform"]["project_store_root"]
            .as_str()
            .ok_or("retained store")?,
    );
    let generation = wow_store::project::StoreGenerationId::parse(
        request["platform"]["store_generation_id"]
            .as_str()
            .ok_or("exact generation")?,
    )?;
    let timer = Instant::now();
    let store = LiveProjectStore::open_in_namespace(store_path, selection.owner())?;
    let read =
        store.read_in_namespace(selection.owner(), &ReadSelector::Exact(generation), &stop)?;
    let observed = (|| -> TestResult<Value> {
        assert_eq!(
            read.publication_set_id(),
            request["platform"]["publication_set_id"]
                .as_str()
                .ok_or("publication")?
        );
        let candidate = read.candidate().ok_or("original native candidate")?;
        assert_eq!(
            candidate.manifest().candidate_id(),
            request["platform"]["candidate_id"]
                .as_str()
                .ok_or("candidate ID")?
        );
        assert_eq!(
            candidate.manifest().candidate_digest(),
            request["platform"]["candidate_digest"]
                .as_str()
                .ok_or("candidate digest")?
        );
        let native = candidate.native();
        let source = native.source();
        let graph = candidate.graph();
        let declarations = source.xml_declarations();
        let counts = json!({"xml_declarations":declarations.len(),"graph_nodes":graph.snapshot().nodes().len(),
            "inline_handlers":source.inline_handlers().len(),"script_sources":source.script_sources().len()});
        if declarations.len() > MAX_DECLARATIONS || graph.snapshot().nodes().len() > MAX_GRAPH_NODES
        {
            return Err(format!(
                "original native declaration input exceeds bound; counts={counts}"
            )
            .into());
        }
        // Choose one explicitly recorded fixture identity, then qualify its
        // original receipt. This preparation does not implement Service root
        // selection, nor claim full declaration coverage.
        let declaration = declarations
            .iter()
            .min_by_key(|declaration| &declaration.occurrence_id)
            .ok_or_else(|| format!("no retained declaration; counts={counts}"))?;
        let assertion = native
            .assertion(&GraphLocalAssertion {
                kind: GraphAssertionKind::Entity,
                proposal_id: declaration.proposal_id.clone().into(),
            })
            .ok_or("original declaration assertion")?;
        let lookup = graph.producer_lookup(&stop)?;
        let accepted = lookup.entity(native.scope(), assertion, &stop)?;
        assert_eq!(
            accepted.proposal().entity_kind_id(),
            "xml_source_declaration"
        );
        assert_eq!(
            accepted.proposal().source_handle_ids(),
            std::slice::from_ref(&declaration.source_handle_id)
        );
        assert_eq!(
            accepted.proposal().evidence_ids(),
            std::slice::from_ref(&declaration.evidence_id)
        );
        let input_node = accepted.accepted().node();
        let rebound = GraphNode::new(
            graph.snapshot().universe().clone(),
            graph.snapshot().generation().clone(),
            input_node.kind(),
            input_node.owner_key(),
            input_node.evidence_ids().to_vec(),
            graph.snapshot().limits(),
        )?;
        let root = graph
            .snapshot()
            .node(rebound.node_id())
            .ok_or("final accepted declaration node")?;
        let mut platform_input = request["platform"]["input"].clone();
        platform_input["neighborhood"]["query"]["node_id"] = json!(root.node_id());
        let input = OpenSkeletonInputRequest::from_json_slice(
            &wow_core::canonical_json_bytes(&platform_input)?,
            &stop,
        )?;
        let view = open_skeleton_input_view(native, graph, input, &stop)?;
        let page = view.read(&stop)?;
        assert!(std::ptr::eq(root, page.neighborhood().root()));
        let support = page.root_support();
        assert!(support.explanation().support_complete());
        assert!(support.explanation().derivation_complete());
        assert!(support.explanation().conflicts().is_empty());
        assert!(support.evidence_resolution().closure_complete());
        let handle = source
            .source_handles()
            .get(&declaration.source_handle_id)
            .ok_or("original handle")?;
        let evidence = source
            .evidence()
            .get(&declaration.evidence_id)
            .ok_or("original evidence")?;
        assert_eq!(handle.path().as_str(), declaration.path);
        assert!(
            support
                .evidence_resolution()
                .source_handles()
                .contains(&handle)
        );
        assert!(
            support
                .evidence_resolution()
                .evidence_records()
                .contains(&evidence)
        );
        let metadata = json!({"selection":"xml_declaration_metadata",
            "interpretation":"original_accepted_root_xml_declaration_identity_span_no_classification_or_runtime_resolution",
            "source_context":source.context(),"assertion":assertion,"root":root,"declaration":declaration,
            "source_handle":handle,"evidence":evidence,"root_support":support});
        request["platform"]["input"] = platform_input;
        request["inspection"]["platform_root_facet"] = json!("xml_declaration_metadata");
        Ok(
            json!({"counts":counts,"original":metadata,"selected_inspect_request":request,
            "root":root.node_id(),"original_page":page,"wrong_kind_request":serde_json::from_slice::<Value>(&request_bytes)?,
            "host_sha256":digest_hex(&host_bytes),"original_request_sha256":digest_hex(&request_bytes)}),
        )
    })();
    let closed_read = read.close();
    let closed_store = store.close();
    closed_read?;
    closed_store?;
    let mut observed = observed?;
    observed["preparation_ms"] = json!(timer.elapsed().as_millis());
    observed["resource_closure"] = json!("closed");
    write_new_capped(
        &work.join("prepared.json"),
        &wow_core::canonical_json_bytes(&observed)?,
        MAX_SELECTED_EXPECTED_BYTES,
    )?;
    write_new_capped(
        &work.join("inspect-request.json"),
        &wow_core::canonical_json_bytes(&observed["selected_inspect_request"])?,
        MAX_REQUEST_BYTES,
    )?;
    write_new_capped(&work.join("host.json"), &host_bytes, MAX_HOST_BYTES)?;
    println!(
        "original declaration preparation {}ms, counts={}",
        observed["preparation_ms"], observed["counts"]
    );
    Ok(())
}
