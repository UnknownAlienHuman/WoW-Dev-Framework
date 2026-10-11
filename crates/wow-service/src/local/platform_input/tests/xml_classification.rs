use super::*;

const CLASSIFICATION_XSD: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
    xmlns:u="http://www.blizzard.com/wow/ui/" targetNamespace="http://www.blizzard.com/wow/ui/">
  <xs:complexType name="FrameRefType"/>
  <xs:complexType name="FrameType"><xs:complexContent><xs:extension base="u:FrameRefType"/></xs:complexContent></xs:complexType>
  <xs:element name="FrameRef" abstract="true" type="u:FrameRefType"/>
  <xs:element name="Frame" type="u:FrameType" substitutionGroup="u:FrameRef"/>
  <xs:complexType name="TextureType"/><xs:element name="Texture" type="u:TextureType"/>
  <xs:complexType name="FontStringType"/><xs:element name="FontString" type="u:FontStringType"/>
  <xs:complexType name="LineType"/><xs:element name="Line" type="u:LineType"/>
  <xs:complexType name="ActorType"/><xs:element name="Actor" type="u:ActorType"/>
  <xs:complexType name="MaskTextureType"/><xs:element name="MaskTexture" type="u:MaskTextureType"/>
  <xs:element name="Layers"><xs:complexType><xs:sequence>
    <xs:element name="Layer"><xs:complexType><xs:sequence><xs:choice>
      <xs:element ref="u:Texture"/><xs:element ref="u:FontString"/><xs:element ref="u:Line"/>
      <xs:element ref="u:Actor"/><xs:element ref="u:MaskTexture"/>
    </xs:choice></xs:sequence></xs:complexType></xs:element>
  </xs:sequence></xs:complexType></xs:element>
  <xs:element name="Ui"/><xs:element name="Scripts"/><xs:element name="OnLoad"/>
</xs:schema>"#;
const CLASSIFICATION_XML: &str = r#"<Ui xmlns="http://www.blizzard.com/wow/ui/"><Frame name="NativeFrame"><Scripts><OnLoad>local loaded = true</OnLoad></Scripts></Frame><Texture name="NativeRegion"/><Custom/><FrameRef/></Ui>"#;

#[test]
fn selected_declared_headers_compose_with_xml_classification_and_exact_catalog() -> TestResult {
    use crate::graph::{GraphReadStatus, execute_graph_build};
    use crate::live_project::{
        LiveProjectPublishRequest, LiveProjectStore, PlatformStoreSelection,
        publish_input_in_namespace,
    };
    use wow_emmy::function_calls::PhysicalFunctionDeclarationRecipe;
    use wow_project::replay::ProjectReplay;
    use wow_store::project::ReadSelector;

    let stop = AtomicBool::new(false);
    let (root, mut config) = classification_fixture(CLASSIFICATION_XSD)?;
    let recipe = PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1;
    config["physical_function_declarations"] = serde_json::to_value(recipe)?;
    write_json(&root, "project.json", &config)?;
    let path = root.0.join("project.json");
    let (project, request, _, _) = super::function_fingerprints::declaration_query(&path, &stop)?;
    let classifier_digest = project
        .configuration()
        .xml_classification_profile()
        .ok_or("combined classifier")?
        .digest();
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
    assert!(
        result
            .function_fingerprints()
            .ok_or("combined fingerprints")?
            .records()
            .iter()
            .all(|record| record.declared_header().is_some())
    );
    let wire = serde_json::to_value(&result)?;
    assert!(!wire["provenance"]["xml_classification"].is_null());
    assert!(!wire["xml_classification_membership"].is_null());
    let ordinary = PlatformStoreSelection::new(
        project.configuration().project_id().clone(),
        project
            .configuration()
            .platform_packages()
            .ok_or("combined packages")?
            .source()
            .profile()
            .profile_id()
            .clone(),
    )?;
    let xml_only = ordinary.clone().with_xml_classification_catalog();
    let declaration_only = ordinary
        .clone()
        .with_physical_function_declarations_catalog();
    let selection = xml_only
        .clone()
        .with_physical_function_declarations_catalog();
    let captured_input = LocalProjectInput::from_config_path(&path, &stop)?;
    let durable_input = LocalProjectInput::from_config_path(&path, &stop)?;
    std::fs::remove_dir_all(root.0.join("UI"))?;
    std::fs::remove_file(root.0.join("version.txt"))?;
    let (bundle, _) =
        request.live_publication_in_namespace(captured_input, selection.namespace(), &stop)?;
    let (records, _) = bundle.into_parts();
    let record = records
        .iter()
        .find(|record| record.key() == "live.project.replay")
        .ok_or("combined replay record")?;
    let replay: ProjectReplay = record.decode()?;
    let expected_record = wow_store::project::PartitionRecord::new(
        record.key(),
        "wow-project.live-replay.xml-schema-classification-and-physical-function-declarations.v1",
        &replay,
    )?;
    assert_eq!(record.version(), expected_record.version());
    assert_eq!(
        serde_json::to_value(&replay)?["schema"],
        "wow-project/native-project-replay/xml-schema-classification-and-physical-function-declarations/1"
    );
    let decoded = ProjectReplay::from_json(&serde_json::to_vec(&replay)?, &stop)?;
    let hydrated = decoded.hydrate(&stop)?;
    assert_eq!(hydrated.snapshot_id(), project.snapshot_id());
    assert_eq!(
        hydrated.configuration().physical_function_declarations(),
        Some(recipe)
    );
    assert_eq!(
        hydrated
            .configuration()
            .xml_classification_profile()
            .ok_or("hydrated classifier")?
            .digest(),
        classifier_digest
    );

    let store_root = root.0.join("combined-store");
    let publish =
        LiveProjectPublishRequest::new("fixture:combined-declarations", "absent", true, true)?;
    let published = publish_input_in_namespace(
        durable_input,
        &request,
        &store_root,
        &selection,
        &publish,
        &stop,
    )?;
    assert_eq!(serde_json::to_value(published)?["status"], "activated");
    for wrong in [&ordinary, &xml_only, &declaration_only] {
        assert_eq!(
            LiveProjectStore::open_in_namespace(&store_root, wrong)
                .err()
                .ok_or("different component catalog reopened combined epoch")?
                .code(),
            crate::ServiceErrorCode::IdentityMismatch
        );
    }
    let store = LiveProjectStore::open_in_namespace(&store_root, &selection)?;
    let read = store.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
    assert_eq!(read.project().snapshot_id(), project.snapshot_id());
    assert_eq!(
        read.project()
            .configuration()
            .physical_function_declarations(),
        Some(recipe)
    );
    assert_eq!(
        read.project()
            .configuration()
            .xml_classification_profile()
            .ok_or("read classifier")?
            .digest(),
        classifier_digest
    );
    Ok(())
}

fn classification_fixture(xsd: &str) -> TestResult<(FixtureRoot, Value)> {
    use wow_project::load::schema::XmlSchemaComponentKind as Kind;
    use wow_project::load::schema::{
        XmlSchemaMemberSelection, XmlSchemaSelection, admit_xml_schema, classification::*,
    };

    let (root, mut config) = fixture()?;
    std::fs::write(root.0.join("UI/frames.xml"), CLASSIFICATION_XML)?;
    std::fs::write(root.0.join("UI/classification.xsd"), xsd)?;
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(root.0.join("manifest.json"))?)?;
    let files = manifest["files"].as_array_mut().ok_or("manifest files")?;
    let xml = files
        .iter_mut()
        .find(|file| file["path"] == "UI/frames.xml")
        .ok_or("fixture XML member")?;
    xml["bytes"] = json!(CLASSIFICATION_XML.len());
    xml["content_sha256"] = json!(format!(
        "{:x}",
        Sha256::digest(CLASSIFICATION_XML.as_bytes())
    ));
    files.push(
        json!({"path":"UI/classification.xsd","kind":"schema","bytes":xsd.len(),
        "git_blob_algorithm":"sha1","git_blob_id":"2".repeat(40),
        "content_sha256":format!("{:x}",Sha256::digest(xsd.as_bytes()))}),
    );
    files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    let total = files
        .iter()
        .try_fold(0_u64, |sum, file| -> TestResult<u64> {
            Ok(sum
                .checked_add(file["bytes"].as_u64().ok_or("member bytes")?)
                .ok_or("fixture byte sum")?)
        })?;
    assert_eq!(files.len(), 5);
    manifest["coverage"] = json!({"tracked_files":5,"included_files":5,"excluded_files":0,
        "included_bytes":total,"kind_version":1,"kind_lua":1,"kind_toc":1,"kind_xml":1,"kind_schema":1});
    manifest
        .as_object_mut()
        .ok_or("manifest object")?
        .remove("manifest_sha256");
    manifest["manifest_sha256"] = json!(format!(
        "{:x}",
        Sha256::digest(wow_core::canonical_json_bytes(&manifest)?)
    ));
    config["source"]["manifest"] =
        serde_json::to_value(write_json(&root, "manifest.json", &manifest)?)?;
    // Required fixture wire OIDs are assertions; this synthetic source never attests Git membership.
    config["source"]["origin"]["revision"] =
        serde_json::to_value(PlatformSourceRevision::Fixture {
            digest: digest(&serde_json::to_vec(&manifest)?),
        })?;
    write_json(&root, "project.json", &config)?;

    let stop = AtomicBool::new(false);
    let directory = ProjectInputDirectory::open(&root.0)?;
    let transport: ManifestedPlatformInput = serde_json::from_value(config["source"].clone())?;
    let profile =
        BlizzardUiSourceProfile::new(serde_json::from_value(config["source_profile"].clone())?)?;
    let source = Arc::new(
        directory
            .admit_platform_source_manifest(&profile, &transport, &stop)?
            .into_parts()
            .0,
    );
    let receipt = source.receipt();
    let schema_selection = XmlSchemaSelection {
        source_snapshot_id: receipt.source_snapshot_id().to_owned(),
        profile_digest: receipt.profile_digest(),
        content_manifest_digest: receipt.content_manifest_digest(),
        admission_digest: receipt.admission_digest(),
        members: vec![XmlSchemaMemberSelection {
            path: "UI/classification.xsd".into(),
            content_digest: digest(xsd.as_bytes()),
            byte_length: u64::try_from(xsd.len())?,
        }],
    };
    let catalog = Arc::new(admit_xml_schema(&source, &schema_selection, &stop)?);
    assert!(catalog.issues().is_empty(), "{:?}", catalog.issues());
    let layers_element = fixture_schema_global(&catalog, Kind::GlobalElement, "Layers")?;
    let layers_type = fixture_schema_child(&catalog, layers_element, Kind::AnonymousComplexType)?;
    let layers_sequence = fixture_schema_child(&catalog, layers_type, Kind::Sequence)?;
    let layer_element = fixture_schema_child(&catalog, layers_sequence, Kind::LocalElement)?;
    let layer_type = fixture_schema_child(&catalog, layer_element, Kind::AnonymousComplexType)?;
    let layer_sequence = fixture_schema_child(&catalog, layer_type, Kind::Sequence)?;
    let choice = fixture_schema_child(&catalog, layer_sequence, Kind::Choice)?;
    let selected = XmlClassificationProfileSelection {
        binding: XmlClassificationBinding {
            source_snapshot_id: catalog.receipt().source_snapshot_id().to_owned(),
            source_profile_digest: catalog.receipt().source_profile_digest(),
            content_manifest_digest: catalog.receipt().content_manifest_digest(),
            admission_digest: catalog.receipt().admission_digest(),
            catalog_selection_digest: catalog.receipt().selection_digest(),
            catalog_digest: catalog.receipt().digest(),
        },
        frame_like: FrameLikeAnchors {
            element: fixture_schema_global(&catalog, Kind::GlobalElement, "FrameRef")?,
            type_component: fixture_schema_global(
                &catalog,
                Kind::NamedComplexType,
                "FrameRefType",
            )?,
        },
        region_like: RegionLikeAnchors {
            layers_element,
            layers_type,
            layers_sequence,
            layer_element,
            layer_type,
            layer_sequence,
            choice,
            members: [
                fixture_region_witness(&catalog, choice, "Texture", "TextureType")?,
                fixture_region_witness(&catalog, choice, "FontString", "FontStringType")?,
                fixture_region_witness(&catalog, choice, "Line", "LineType")?,
                fixture_region_witness(&catalog, choice, "Actor", "ActorType")?,
                fixture_region_witness(&catalog, choice, "MaskTexture", "MaskTextureType")?,
            ],
        },
        explicit: vec![
            ExplicitClassAnchor {
                element: fixture_schema_global(&catalog, Kind::GlobalElement, "Ui")?,
                class: XmlExplicitSchemaClass::Structural,
            },
            ExplicitClassAnchor {
                element: fixture_schema_global(&catalog, Kind::GlobalElement, "Scripts")?,
                class: XmlExplicitSchemaClass::Structural,
            },
            ExplicitClassAnchor {
                element: fixture_schema_global(&catalog, Kind::GlobalElement, "OnLoad")?,
                class: XmlExplicitSchemaClass::Script,
            },
        ],
    };
    // Native catalog IDs form intent; LocalProjectInput admits the reviewed profile
    // against its own held source before deriving configuration and generation.
    config["xml_schema_classification"] =
        json!({"schema_selection":schema_selection,"profile_selection":selected});
    config["graph_profile"] = serde_json::to_value(PlatformGraphProfile::DirectCapabilitiesV1(
        PlatformGraphCapabilities::new(&[
            PlatformGraphCapability::InventorySpansV1,
            PlatformGraphCapability::StructuralRolesV1,
            PlatformGraphCapability::XmlSourceMapsV1,
            PlatformGraphCapability::XmlSchemaClassificationV1,
        ])?,
    ))?;
    write_json(&root, "project.json", &config)?;
    Ok((root, config))
}

fn fixture_schema_global(
    catalog: &wow_project::load::schema::AdmittedXmlSchema,
    kind: wow_project::load::schema::XmlSchemaComponentKind,
    name: &str,
) -> TestResult<ContentDigest<CanonicalResult>> {
    let matches: Vec<_> = catalog
        .components()
        .iter()
        .filter(|component| {
            component.kind() == kind
                && component.name().is_some_and(|expanded| {
                    expanded.namespace() == Some("http://www.blizzard.com/wow/ui/")
                        && expanded.local_name() == name
                })
        })
        .collect();
    let [component] = matches.as_slice() else {
        return Err("fixture global is not unique".into());
    };
    assert_eq!(
        component.state(),
        wow_project::load::schema::XmlSchemaComponentState::Observed
    );
    Ok(component.id())
}

fn fixture_schema_child(
    catalog: &wow_project::load::schema::AdmittedXmlSchema,
    parent: ContentDigest<CanonicalResult>,
    kind: wow_project::load::schema::XmlSchemaComponentKind,
) -> TestResult<ContentDigest<CanonicalResult>> {
    let matches: Vec<_> = catalog
        .components()
        .iter()
        .filter(|component| component.parent() == Some(parent) && component.kind() == kind)
        .collect();
    let [component] = matches.as_slice() else {
        return Err("fixture child is not unique".into());
    };
    assert_eq!(
        component.state(),
        wow_project::load::schema::XmlSchemaComponentState::Observed
    );
    Ok(component.id())
}

fn fixture_region_witness(
    catalog: &wow_project::load::schema::AdmittedXmlSchema,
    choice: ContentDigest<CanonicalResult>,
    element_name: &str,
    type_name: &str,
) -> TestResult<wow_project::load::schema::classification::RegionTypeWitness> {
    use wow_project::load::schema::{
        XmlSchemaComponentKind as Kind, XmlSchemaReferenceKind as RefKind, XmlSchemaReferenceState,
    };
    let element = fixture_schema_global(catalog, Kind::GlobalElement, element_name)?;
    let type_component = fixture_schema_global(catalog, Kind::NamedComplexType, type_name)?;
    let references: Vec<_> = catalog
        .references()
        .iter()
        .filter(|reference| {
            reference.kind() == RefKind::Element
                && reference.candidates() == [element]
                && catalog.components().iter().any(|component| {
                    component.id() == reference.source()
                        && component.kind() == Kind::ElementReference
                        && component.parent() == Some(choice)
                })
        })
        .collect();
    let [reference] = references.as_slice() else {
        return Err("fixture region reference is not unique".into());
    };
    assert_eq!(reference.state(), XmlSchemaReferenceState::Unique);
    Ok(
        wow_project::load::schema::classification::RegionTypeWitness {
            element_reference: reference.source(),
            element,
            type_component,
        },
    )
}

fn verify_classification_graph(
    report: &Value,
    graph: &wow_graph::GraphPartitionSnapshot,
    profile_digest: ContentDigest<CanonicalResult>,
    catalog_digest: ContentDigest<CanonicalResult>,
    selected_toc: &str,
    stop: &AtomicBool,
) -> TestResult {
    use wow_graph::{
        GraphAssertionKind, GraphAssertionRef, GraphConfidence, GraphProposalEndpoint,
        GraphProposalValue,
    };
    use wow_project::load::schema::classification::{
        XmlClassOutcome, XmlClassificationUnknown, XmlSchemaClass,
    };

    assert_eq!(
        report["profile_digest"],
        serde_json::to_value(profile_digest)?
    );
    assert_eq!(
        report["binding"]["catalog_digest"],
        serde_json::to_value(catalog_digest)?
    );
    assert_eq!(
        report["counts_by_class"],
        json!({"frame_like":1,"region_like":1,"structural":2,"script":1})
    );
    assert_eq!(
        report["unmatched_names"],
        json!([{
            "name":{"namespace":"http://www.blizzard.com/wow/ui/","local_name":"Custom"},"occurrences":1
        }])
    );
    let documents = report["documents"]
        .as_array()
        .ok_or("classification documents")?;
    let [document] = documents.as_slice() else {
        return Err("expected one native XML scope".into());
    };
    assert_eq!(document["scope"]["package"], "Fixture");
    assert_eq!(document["scope"]["selected_toc"], selected_toc);
    assert_eq!(
        document["report"]["coverage"],
        json!({
            "occurrences":7,"classified":5,"unknown":2,"conflicted":0,"catalog_issues":0
        })
    );
    let classified = document["report"]["occurrences"]
        .as_array()
        .ok_or("leaf occurrences")?;
    let scoped = document["occurrences"]
        .as_array()
        .ok_or("project occurrences")?;
    assert_eq!(scoped.len(), 7);
    let definition = graph
        .registry()
        .relation_kind("xml_occurrence_schema_class")
        .ok_or("exact classification definition missing")?;
    assert_eq!(
        definition.relation(),
        wow_graph::GraphRelationKind::HasSchemaClass
    );
    assert!(definition.allows_source_kind("xml_source_occurrence"));
    assert!(definition.allows_target_kind("xml_schema_class"));
    let coverage = graph
        .snapshot()
        .coverage()
        .iter()
        .find(|coverage| coverage.relation() == wow_graph::GraphRelationKind::HasSchemaClass)
        .ok_or("classification coverage")?;
    // The aggregate requires coverage from every partition. Independent
    // recognizers do not evaluate schema classification; keep that limitation.
    assert_eq!(
        coverage.state(),
        wow_graph::GraphCoverageState::NotEvaluated
    );
    assert!(
        coverage
            .blocker_ids()
            .iter()
            .any(|id| id.as_ref() == "graph.partition.coverage_missing")
    );
    assert!(!coverage.negative_authority());
    let admitted = graph
        .partitions()
        .iter()
        .map(|partition| {
            partition
                .report()
                .accepted_relations()
                .iter()
                .filter(|accepted| {
                    partition
                        .batch()
                        .relation_proposal(accepted.proposal_id())
                        .is_some_and(|proposal| {
                            proposal.relation_kind_id() == "xml_occurrence_schema_class"
                        })
                })
                .count()
        })
        .sum::<usize>();
    assert_eq!(admitted, 5);
    let lookup = graph.producer_lookup(stop)?;
    // Fixed semantic expectations for ONE source document, not generated golden outcomes.
    for (name, outcome) in [
        (
            "Ui",
            XmlClassOutcome::Classified(XmlSchemaClass::Structural),
        ),
        (
            "Frame",
            XmlClassOutcome::Classified(XmlSchemaClass::FrameLike),
        ),
        (
            "Scripts",
            XmlClassOutcome::Classified(XmlSchemaClass::Structural),
        ),
        (
            "OnLoad",
            XmlClassOutcome::Classified(XmlSchemaClass::Script),
        ),
        (
            "Texture",
            XmlClassOutcome::Classified(XmlSchemaClass::RegionLike),
        ),
        (
            "Custom",
            XmlClassOutcome::Unknown(XmlClassificationUnknown::MissingGlobalDeclaration),
        ),
        (
            "FrameRef",
            XmlClassOutcome::Unknown(XmlClassificationUnknown::AbstractOccurrence),
        ),
    ] {
        let matches: Vec<_> = classified
            .iter()
            .filter(|row| row["qname"]["name"]["local_name"] == name)
            .collect();
        let [leaf] = matches.as_slice() else {
            return Err("expected QName occurrence is not unique".into());
        };
        assert_eq!(
            leaf["qname"]["name"]["namespace"],
            "http://www.blizzard.com/wow/ui/"
        );
        assert_eq!(leaf["outcome"], serde_json::to_value(outcome)?, "{name}");
        let matches: Vec<_> = scoped
            .iter()
            .filter(|row| row["occurrence"] == leaf["occurrence"])
            .collect();
        let [row] = matches.as_slice() else {
            return Err("scope/leaf occurrence join is not unique".into());
        };
        let XmlClassOutcome::Classified(class) = outcome else {
            assert!(row.get("class_proposal_id").is_none(), "{name}");
            assert!(row.get("relation_proposal_id").is_none(), "{name}");
            continue;
        };
        let relation_id = row["relation_proposal_id"]
            .as_str()
            .ok_or("classified relation missing")?;
        let class_id = row["class_proposal_id"]
            .as_str()
            .ok_or("classified entity missing")?;
        let occurrence_id = row["occurrence_proposal_id"]
            .as_str()
            .ok_or("native occurrence proposal")?;
        let partitions: Vec<_> = graph
            .partitions()
            .iter()
            .filter(|partition| partition.batch().relation_proposal(relation_id).is_some())
            .collect();
        let [partition] = partitions.as_slice() else {
            return Err("classification partition is not unique".into());
        };
        let owner_coverage = partition
            .coverage()
            .iter()
            .find(|row| row.relation() == wow_graph::GraphRelationKind::HasSchemaClass)
            .ok_or("classification owner coverage")?;
        assert_eq!(
            owner_coverage.state(),
            wow_graph::GraphCoverageState::Partial
        );
        assert!(!owner_coverage.negative_authority());
        let proposal = partition
            .batch()
            .relation_proposal(relation_id)
            .ok_or("relation")?;
        assert_eq!(proposal.relation_kind_id(), "xml_occurrence_schema_class");
        assert_eq!(proposal.confidence(), GraphConfidence::Derived);
        assert_eq!(
            proposal.endpoints(),
            [
                &GraphProposalEndpoint::Proposed(occurrence_id.into()),
                &GraphProposalEndpoint::Proposed(class_id.into()),
            ]
        );
        assert!(
            partition
                .report()
                .accepted_relations()
                .iter()
                .any(|accepted| accepted.proposal_id() == relation_id)
        );
        assert!(
            partition
                .report()
                .accepted_entities()
                .iter()
                .any(|accepted| {
                    accepted.proposal_id() == class_id
                        && accepted.node().kind() == "xml_schema_class"
                })
        );
        let class_proposal = partition
            .batch()
            .entity_proposal(class_id)
            .ok_or("class proposal")?;
        assert_eq!(
            class_proposal.semantic_key().get("profile_digest"),
            Some(&GraphProposalValue::String(
                profile_digest.canonical().into()
            ))
        );
        assert_eq!(
            class_proposal.semantic_key().get("catalog_digest"),
            Some(&GraphProposalValue::String(
                catalog_digest.canonical().into()
            ))
        );
        assert_eq!(
            class_proposal.semantic_key().get("class"),
            Some(&GraphProposalValue::String(
                String::from_utf8(wow_core::canonical_json_bytes(&class)?)?.into()
            ))
        );
        let declaration = leaf["candidates"].as_array().ok_or("class candidates")?;
        assert_eq!(declaration.len(), 1);
        let declaration_id: ContentDigest<CanonicalResult> =
            serde_json::from_value(declaration[0].clone())?;
        assert_eq!(
            class_proposal.semantic_key().get("declaration"),
            Some(&GraphProposalValue::String(
                declaration_id.canonical().into()
            ))
        );
        let witnesses = report["catalog_support"]["witnesses"]
            .as_object()
            .ok_or("catalog support")?;
        let native = witnesses
            .values()
            .find(|witness| witness["component_id"] == declaration[0])
            .ok_or("native declaration support")?;
        assert_eq!(native["document"], "UI/classification.xsd");
        assert_eq!(
            native["content_digest"],
            serde_json::to_value(digest(CLASSIFICATION_XSD.as_bytes()))?
        );
        let handles = serde_json::to_value(proposal.source_handle_ids())?;
        assert!(
            handles
                .as_array()
                .ok_or("relation handles")?
                .contains(&native["source_handle_id"])
        );
        assert!(
            handles
                .as_array()
                .ok_or("relation handles")?
                .contains(&row["source_handle_id"])
        );
        let evidence = serde_json::to_value(proposal.evidence_ids())?;
        assert!(
            evidence
                .as_array()
                .ok_or("relation evidence")?
                .contains(&native["evidence_id"])
        );
        assert!(
            evidence
                .as_array()
                .ok_or("relation evidence")?
                .contains(&row["evidence_id"])
        );
        let records = partition
            .batch()
            .assertion_records()
            .ok_or("native derivations")?;
        let derivation = records
            .derivations
            .iter()
            .find(|record| {
                record.output.kind == GraphAssertionKind::Relation
                    && record.output.proposal_id.as_ref() == relation_id
            })
            .ok_or("classification relation derivation")?;
        assert_eq!(
            derivation.rule_id.as_ref(),
            "wow-project.source-graph.xml-schema-classification"
        );
        let mut input_ids = std::collections::BTreeSet::new();
        for input in &derivation.inputs {
            let proposal = match input {
                GraphAssertionRef::Local { assertion } => {
                    assert_eq!(assertion.kind, GraphAssertionKind::Entity);
                    let proposal = partition
                        .batch()
                        .entity_proposal(&assertion.proposal_id)
                        .ok_or("local derivation entity")?;
                    let accepted = partition
                        .report()
                        .accepted_entities()
                        .iter()
                        .find(|row| row.proposal_id() == proposal.proposal_id())
                        .ok_or("local derivation acceptance")?;
                    assert_eq!(
                        lookup.input_view().node(accepted.node().node_id()),
                        Some(accepted.node())
                    );
                    proposal
                }
                GraphAssertionRef::Producer { .. } => {
                    lookup.entity(&records.scope, input, stop)?.proposal()
                }
            };
            input_ids.insert(proposal.proposal_id().to_owned());
        }
        assert!(
            input_ids.contains(
                native["raw_member_proposal_id"]
                    .as_str()
                    .ok_or("XSD raw proposal")?
            )
        );
        assert!(input_ids.contains(occurrence_id));
        assert!(input_ids.contains(class_id));
    }
    Ok(())
}

fn verify_classification_membership(
    report: &Value,
    membership: &Value,
    graph: &wow_graph::GraphPartitionSnapshot,
    stop: &AtomicBool,
) -> TestResult {
    use std::collections::BTreeSet;
    use wow_graph::{
        GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRef, GraphLocalAssertion,
    };

    let lookup = graph.producer_lookup(stop)?;
    let scope: GraphAssertionRecordScope = serde_json::from_value(membership["scope"].clone())?;
    assert_eq!(&scope, lookup.scope());
    assert_eq!(
        membership["registry_digest"],
        graph.registry().registry_digest()
    );
    // This is the native source-prefix snapshot; later recognizer partitions
    // produce the final service graph. Do not equate their snapshot IDs.
    let mut expected: BTreeSet<GraphLocalAssertion> =
        serde_json::from_value(report["proposals"].clone())?;
    for document in report["documents"]
        .as_array()
        .ok_or("classification documents")?
    {
        for row in document["occurrences"]
            .as_array()
            .ok_or("native occurrences")?
        {
            expected.insert(GraphLocalAssertion {
                kind: GraphAssertionKind::Entity,
                proposal_id: row["occurrence_proposal_id"]
                    .as_str()
                    .ok_or("occurrence proposal")?
                    .into(),
            });
        }
    }
    for witness in report["catalog_support"]["witnesses"]
        .as_object()
        .ok_or("native catalog witnesses")?
        .values()
    {
        expected.insert(GraphLocalAssertion {
            kind: GraphAssertionKind::Entity,
            proposal_id: witness["raw_member_proposal_id"]
                .as_str()
                .ok_or("raw XSD member")?
                .into(),
        });
    }
    let mut actual = BTreeSet::new();
    let mut partition_ids = BTreeSet::new();
    for row in membership["dispositions"]
        .as_array()
        .ok_or("native dispositions")?
    {
        assert_eq!(row["disposition"], "accepted");
        let proposal: GraphLocalAssertion = serde_json::from_value(row["proposal"].clone())?;
        let reference: GraphAssertionRef = serde_json::from_value(row["reference"].clone())?;
        let (partition, resolved_proposal, resolved_reference) = match proposal.kind {
            GraphAssertionKind::Entity => {
                let resolved = lookup.entity(&scope, &reference, stop)?;
                (
                    resolved.partition(),
                    resolved.proposal().proposal_id(),
                    resolved.reference(),
                )
            }
            GraphAssertionKind::Relation => {
                let resolved = lookup.relation(&scope, &reference, stop)?;
                (
                    resolved.partition(),
                    resolved.proposal().proposal_id(),
                    resolved.reference(),
                )
            }
        };
        assert_eq!(resolved_proposal, proposal.proposal_id.as_ref());
        assert_eq!(resolved_reference, reference);
        assert!(actual.insert(proposal), "duplicate membership disposition");
        partition_ids.insert(partition.partition_id().to_owned());
    }
    assert_eq!(actual, expected);
    let mut retained_ids = BTreeSet::new();
    for row in membership["partitions"]
        .as_array()
        .ok_or("native partition identities")?
    {
        let id = row["partition_id"].as_str().ok_or("partition ID")?;
        let partition = graph
            .partitions()
            .iter()
            .find(|partition| partition.partition_id() == id)
            .ok_or("membership partition absent from actual graph")?;
        assert_eq!(row["partition_digest"], partition.partition_digest());
        assert_eq!(row["batch_id"], partition.batch().batch_id());
        assert_eq!(row["report_id"], partition.report().report_id());
        assert!(partition.report().ready_for_publication());
        assert!(
            retained_ids.insert(id.to_owned()),
            "duplicate partition identity"
        );
    }
    assert_eq!(retained_ids, partition_ids);
    Ok(())
}

#[test]
fn selected_xml_classification_reaches_service_replay_and_publication() -> TestResult {
    use crate::graph::{GraphBuildRequest, GraphReadStatus, execute_graph_build};
    use crate::live_project::{
        LiveProjectPublishRequest, LiveProjectStore, PlatformStoreSelection,
        publish_input_in_namespace,
    };
    use wow_project::load::schema::classification::{
        XmlClassificationProfileSelection, XmlExplicitSchemaClass,
    };
    use wow_project::replay::{ProjectReplay, publication};
    use wow_store::project::ReadSelector;

    let (root, config) = classification_fixture(CLASSIFICATION_XSD)?;
    let path = root.0.join("project.json");
    let stop = AtomicBool::new(false);
    let input = LocalProjectInput::from_config_path(&path, &stop)?;
    let held = input
        .project_bundle()
        .configuration()
        .xml_classification_profile()
        .ok_or("selected classifier absent")?;
    let profile_digest = held.digest();
    let catalog_digest = held.catalog().receipt().digest();
    let ui = fixture_schema_global(
        held.catalog(),
        wow_project::load::schema::XmlSchemaComponentKind::GlobalElement,
        "Ui",
    )?;
    let packages = input
        .project_bundle()
        .configuration()
        .package_load_plan()
        .ok_or("package plans")?;
    let plan = packages
        .package_plan("Fixture")
        .ok_or("Fixture selected plan")?;
    let selected_toc = packages
        .source_path("Fixture", plan.selected_toc())
        .ok_or("qualified selected TOC")?;
    let default_selection = PlatformStoreSelection::new(
        input.project_bundle().configuration().project_id().clone(),
        input
            .project_bundle()
            .configuration()
            .platform_packages()
            .ok_or("packages")?
            .source()
            .profile()
            .profile_id()
            .clone(),
    )?;
    let selection = default_selection.clone().with_xml_classification_catalog();
    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    assert_eq!(
        serde_json::to_value(&request)?["xml_classification_profile_digest"],
        serde_json::to_value(profile_digest)?
    );
    let result = execute_graph_build(input, &request, &stop)?;
    assert_eq!(
        result.status(),
        GraphReadStatus::Partial,
        "{}",
        serde_json::to_string(&result)?
    );
    let snapshot: wow_graph::GraphPartitionSnapshot =
        serde_json::from_slice(&result.snapshot_bytes()?.ok_or("selected graph absent")?)?;

    // Inspect serialized observations only; genuine graph admission is checked below.
    let result_value: Value = serde_json::from_slice(&result.canonical_bytes()?)?;
    assert_eq!(
        result_value["schema"],
        "wow-service/graph-build-result/xml-schema-classification/1"
    );
    let report = &result_value["provenance"]["xml_classification"];
    let membership = &result_value["xml_classification_membership"];
    verify_classification_graph(
        report,
        &snapshot,
        profile_digest,
        catalog_digest,
        &selected_toc,
        &stop,
    )?;
    verify_classification_membership(report, membership, &snapshot, &stop)?;

    let mut alternate = config.clone();
    let mut alternate_profile: XmlClassificationProfileSelection = serde_json::from_value(
        alternate["xml_schema_classification"]["profile_selection"].clone(),
    )?;
    alternate_profile
        .explicit
        .iter_mut()
        .find(|anchor| anchor.element == ui)
        .ok_or("Ui policy anchor")?
        .class = XmlExplicitSchemaClass::Load;
    alternate["xml_schema_classification"]["profile_selection"] =
        serde_json::to_value(alternate_profile)?;
    write_json(&root, "project.json", &alternate)?;
    let alternate_input = LocalProjectInput::from_config_path(&path, &stop)?;
    assert_ne!(
        alternate_input
            .project_bundle()
            .configuration()
            .xml_classification_profile()
            .ok_or("alternate reviewed owner")?
            .digest(),
        profile_digest
    );
    let wrong_request =
        GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
            .with_input_profile(&alternate_input);
    drop(alternate_input);
    write_json(&root, "project.json", &config)?;
    let refused = execute_graph_build(
        LocalProjectInput::from_config_path(&path, &stop)?,
        &wrong_request,
        &stop,
    )?;
    assert_eq!(refused.status(), GraphReadStatus::Failed);
    assert_eq!(
        serde_json::to_value(&refused)?["failure"],
        serde_json::to_value(crate::ServiceErrorCode::IdentityMismatch)?
    );
    assert!(refused.snapshot_bytes()?.is_none());

    // A UNIQUE retained Type/Base self-cycle must refuse profile admission,
    // rather than become a selected RegionLike observation.
    assert_eq!(
        CLASSIFICATION_XSD
            .matches("<xs:complexType name=\"TextureType\"/>")
            .count(),
        1
    );
    let cyclic_xsd = CLASSIFICATION_XSD.replace(
        "<xs:complexType name=\"TextureType\"/>",
        "<xs:complexType name=\"TextureType\"><xs:complexContent><xs:extension base=\"u:TextureType\"/></xs:complexContent></xs:complexType>",
    );
    let (cyclic_root, _) = classification_fixture(&cyclic_xsd)?;
    let cycle = LocalProjectInput::from_config_path(&cyclic_root.0.join("project.json"), &stop)
        .err()
        .ok_or("selected Region Type/Base cycle admitted")?;
    assert_eq!(cycle.code(), crate::ServiceErrorCode::InvalidConfiguration);
    assert_eq!(
        cycle.message(),
        "reviewed region member has an unsupported or cyclic typed path"
    );

    let publication_input = LocalProjectInput::from_config_path(&path, &stop)?;
    let store_input = LocalProjectInput::from_config_path(&path, &stop)?;
    let default_store_input = LocalProjectInput::from_config_path(&path, &stop)?;
    // Capture and replay must retain XSD and XML authority, rather than reopen them.
    std::fs::remove_file(root.0.join("UI/classification.xsd"))?;
    std::fs::remove_file(root.0.join("UI/frames.xml"))?;
    let (bundle, _) =
        request.live_publication_in_namespace(publication_input, selection.namespace(), &stop)?;
    let (records, bindings) = bundle.into_parts();
    let replay: ProjectReplay = records
        .iter()
        .find(|record| record.key() == "live.project.replay")
        .ok_or("native replay record")?
        .decode()?;
    assert_eq!(
        serde_json::to_value(&replay)?["schema"],
        "wow-project/native-project-replay/xml-schema-classification/1"
    );
    let decoded = ProjectReplay::from_json(&serde_json::to_vec(&replay)?, &stop)?;
    let reopened = decoded.hydrate(&stop)?;
    assert_eq!(
        Some(reopened.snapshot_id()),
        bindings.get("project_snapshot_id").map(String::as_str)
    );
    let reviewed = reopened
        .configuration()
        .xml_classification_profile()
        .ok_or("reopened classifier")?;
    assert_eq!(reviewed.digest(), profile_digest);
    assert_eq!(reviewed.catalog().receipt().digest(), catalog_digest);
    let rebuilt_bundle = publication::ProjectPublicationBundle::from_replay_in_namespace(
        &decoded,
        &snapshot,
        selection.namespace(),
        &stop,
    )?;
    let (rebuilt_records, rebuilt_bindings) = rebuilt_bundle.into_parts();
    assert_eq!(rebuilt_bindings, bindings);
    assert_eq!(rebuilt_records.len(), records.len());
    for (rebuilt, original) in rebuilt_records.iter().zip(&records) {
        assert_eq!(rebuilt.key(), original.key());
        assert_eq!(rebuilt.version(), original.version());
        assert_eq!(rebuilt.decode::<Value>()?, original.decode::<Value>()?);
    }

    let store_root = root.0.join("classification-store");
    let publish =
        LiveProjectPublishRequest::new("fixture:xml-classification", "absent", true, true)?;
    let default_refusal = publish_input_in_namespace(
        default_store_input,
        &request,
        &store_root,
        &default_selection,
        &publish,
        &stop,
    )
    .err()
    .ok_or("default storage admitted classified configuration")?;
    assert_eq!(
        default_refusal.code(),
        crate::ServiceErrorCode::IdentityMismatch
    );
    assert!(
        !store_root.exists(),
        "configuration refusal must precede store creation"
    );
    let published = publish_input_in_namespace(
        store_input,
        &request,
        &store_root,
        &selection,
        &publish,
        &stop,
    )?;
    assert_eq!(serde_json::to_value(&published)?["status"], "activated");
    let wrong_catalog = LiveProjectStore::open_in_namespace(&store_root, &default_selection)
        .err()
        .ok_or("ordinary selection reopened classification epoch")?;
    assert_eq!(
        wrong_catalog.code(),
        crate::ServiceErrorCode::IdentityMismatch
    );
    let store = LiveProjectStore::open_in_namespace(&store_root, &selection)?;
    let current = store.current()?.ok_or("publication Current absent")?;
    let read = store.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
    assert_eq!(read.project().snapshot_id(), reopened.snapshot_id());
    assert_eq!(
        read.graph().snapshot().snapshot_id(),
        snapshot.snapshot().snapshot_id()
    );
    assert_eq!(
        read.project()
            .configuration()
            .xml_classification_profile()
            .ok_or("durable classifier")?
            .digest(),
        profile_digest
    );
    verify_classification_graph(
        report,
        read.graph(),
        profile_digest,
        catalog_digest,
        &selected_toc,
        &stop,
    )?;
    verify_classification_membership(report, membership, read.graph(), &stop)?;

    drop(read);
    drop(store);
    let store = LiveProjectStore::open_in_namespace(&store_root, &selection)?;
    assert_eq!(store.current()?, Some(current));
    let read = store.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
    verify_classification_graph(
        report,
        read.graph(),
        profile_digest,
        catalog_digest,
        &selected_toc,
        &stop,
    )?;
    verify_classification_membership(report, membership, read.graph(), &stop)?;
    assert_eq!(
        read.project()
            .configuration()
            .xml_classification_profile()
            .ok_or("reopened durable classifier")?
            .catalog()
            .receipt()
            .digest(),
        catalog_digest
    );
    let cancelled = result.into_cancelled()?;
    assert_eq!(cancelled.status(), GraphReadStatus::Cancelled);
    assert!(cancelled.snapshot_bytes()?.is_none());
    let cancelled_wire = serde_json::to_value(cancelled)?;
    assert!(cancelled_wire.get("provenance").is_none());
    assert!(
        cancelled_wire
            .get("xml_classification_membership")
            .is_none()
    );
    Ok(())
}

fn native_xml_fingerprint_targets(
    report: &Value,
) -> TestResult<Vec<crate::graph::XmlFingerprintTarget>> {
    let mut ids = Vec::new();
    for document in report["documents"]
        .as_array()
        .ok_or("native XML documents")?
    {
        for row in document["occurrences"]
            .as_array()
            .ok_or("native XML occurrences")?
        {
            let occurrence: Box<str> = row["occurrence_proposal_id"]
                .as_str()
                .ok_or("native occurrence proposal ID")?
                .into();
            let class = row
                .get("class_proposal_id")
                .map(|id| {
                    id.as_str()
                        .map(Box::<str>::from)
                        .ok_or("native class proposal ID")
                })
                .transpose()?;
            ids.push((occurrence, class));
        }
    }
    ids.sort_by(|left, right| left.0.cmp(&right.0));
    assert!(!ids.is_empty());
    assert!(ids.windows(2).all(|pair| pair[0].0 < pair[1].0));
    Ok(ids
        .into_iter()
        .map(|(occurrence, class)| crate::graph::XmlFingerprintTarget::new(occurrence, class))
        .collect())
}

type XmlQueryComparables = std::collections::BTreeMap<
    String,
    (
        wow_project::graph::xml_fingerprints::XmlSchemaClassComparable,
        ContentDigest<CanonicalResult>,
    ),
>;

fn fixture_xml_query_components(
    manifest: &crate::graph::XmlFingerprintManifest,
) -> TestResult<XmlQueryComparables> {
    use wow_graph::GraphCoverageState;
    use wow_project::graph::xml_fingerprints::XmlSchemaClassUnavailable;
    use wow_project::load::schema::classification::{
        XmlClassOutcome, XmlClassificationUnknown, XmlSchemaClass,
    };

    assert_eq!(manifest.coverage(), GraphCoverageState::Partial);
    assert_eq!(manifest.records().len(), 7);
    assert_eq!(manifest.records().len(), manifest.components().len());
    let mut comparables = XmlQueryComparables::new();
    let mut missing_named_type = 0;
    let mut unknown = 0;
    for (record, (id, component)) in manifest.records().iter().zip(manifest.components()) {
        assert_eq!(id, &record.projection().occurrence_proposal_id);
        let observation = record.observation();
        let name = observation.qname().name().ok_or("expanded fixture QName")?;
        assert_eq!(name.namespace(), Some("http://www.blizzard.com/wow/ui/"));
        match name.local_name() {
            "Frame" | "Texture" => {
                let comparable = component
                    .comparable()
                    .ok_or("named-type component unavailable")?;
                let expected = if name.local_name() == "Frame" {
                    XmlSchemaClass::FrameLike
                } else {
                    XmlSchemaClass::RegionLike
                };
                assert_eq!(observation.outcome(), XmlClassOutcome::Classified(expected));
                assert_eq!(comparable.class(), expected);
                assert_eq!(comparable.declaration(), name);
                assert!(!comparable.qualified_proof_targets().is_empty());
                assert_eq!(component.coverage(), GraphCoverageState::Complete);
                assert!(component.unavailable_reason().is_none());
                assert!(
                    comparables
                        .insert(
                            name.local_name().into(),
                            (
                                comparable.clone(),
                                component.component_digest().ok_or("available digest")?,
                            )
                        )
                        .is_none()
                );
            }
            "Ui" | "Scripts" | "OnLoad" => {
                let expected = if name.local_name() == "OnLoad" {
                    XmlSchemaClass::Script
                } else {
                    XmlSchemaClass::Structural
                };
                assert_eq!(observation.outcome(), XmlClassOutcome::Classified(expected));
                assert!(matches!(
                    component.unavailable_reason(),
                    Some(XmlSchemaClassUnavailable::MissingNamedTypeWitness)
                ));
                assert!(component.comparable().is_none());
                assert!(component.component_digest().is_none());
                assert_eq!(component.coverage(), GraphCoverageState::Partial);
                missing_named_type += 1;
            }
            "Custom" | "FrameRef" => {
                let expected = if name.local_name() == "Custom" {
                    XmlClassificationUnknown::MissingGlobalDeclaration
                } else {
                    XmlClassificationUnknown::AbstractOccurrence
                };
                assert_eq!(observation.outcome(), XmlClassOutcome::Unknown(expected));
                assert!(matches!(component.unavailable_reason(),
                    Some(XmlSchemaClassUnavailable::ClassificationUnknown(reason)) if reason == expected));
                assert!(component.comparable().is_none());
                assert!(component.component_digest().is_none());
                assert_eq!(component.coverage(), GraphCoverageState::Partial);
                assert!(record.projection().class_proposal_id.is_none());
                assert!(record.projection().relation_proposal_id.is_none());
                unknown += 1;
            }
            _ => return Err("unexpected native fixture occurrence".into()),
        }
    }
    assert_eq!(comparables.len(), 2);
    assert_eq!(missing_named_type, 3);
    assert_eq!(unknown, 2);
    Ok(comparables)
}

#[test]
fn selected_xml_schema_components_preserve_unknowns_and_prefix_alias_comparison() -> TestResult {
    use crate::{
        ServiceErrorCode,
        graph::{
            GRAPH_RESULT_MAX_BYTES, GraphBuildRequest, GraphReadStatus, XmlFingerprintSelection,
            execute_graph_build,
        },
    };

    let stop = AtomicBool::new(false);
    let (root, _) = classification_fixture(CLASSIFICATION_XSD)?;
    let path = root.0.join("project.json");
    let input = LocalProjectInput::from_config_path(&path, &stop)?;
    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    let ordinary = execute_graph_build(input, &request, &stop)?;
    assert_eq!(
        ordinary.status(),
        GraphReadStatus::Partial,
        "{:?}",
        ordinary.failure_message()
    );
    assert!(ordinary.xml_fingerprints().is_none());
    let ordinary_wire = serde_json::to_value(&ordinary)?;
    assert!(ordinary_wire.get("xml_fingerprints").is_none());
    assert!(ordinary_wire["request"].get("xml_fingerprints").is_none());
    let targets =
        native_xml_fingerprint_targets(&ordinary_wire["provenance"]["xml_classification"])?;
    // Reuse one existing finite service ceiling for the cumulative binding/index/output ledger.
    let limit = GRAPH_RESULT_MAX_BYTES;
    let selection = XmlFingerprintSelection::new(targets.clone(), targets.len(), limit)?;
    let selection_wire = serde_json::to_value(&selection)?;
    let selection_bytes = serde_json::to_vec(&selection)?;
    let selected_request = request
        .clone()
        .with_xml_fingerprints(XmlFingerprintSelection::from_json_slice(&selection_bytes)?)?;
    let result = execute_graph_build(
        LocalProjectInput::from_config_path(&path, &stop)?,
        &selected_request,
        &stop,
    )?;
    assert_eq!(
        result.status(),
        GraphReadStatus::Partial,
        "{:?}",
        result.failure_message()
    );
    let manifest = result
        .xml_fingerprints()
        .ok_or("selected native XML manifest")?;
    let normalized = fixture_xml_query_components(manifest)?;
    let result_wire: Value = serde_json::from_slice(&result.canonical_bytes()?)?;
    assert_eq!(
        result_wire["schema"],
        "wow-service/graph-build-result/xml-fingerprints/1"
    );
    assert_eq!(
        result_wire["request"]["schema"],
        "wow-service/graph-build-request/xml-fingerprints/1"
    );
    assert_eq!(result_wire["request"]["xml_fingerprints"], selection_wire);
    assert_eq!(
        result_wire["xml_fingerprints"]["component_recipe"],
        "wow-project/xml-schema-class-component/named-proof-witnesses/1"
    );
    // Existing verifier and native service joins own the support closure.
    let graph: wow_graph::GraphPartitionSnapshot =
        serde_json::from_slice(&result.snapshot_bytes()?.ok_or("selected graph")?)?;
    verify_classification_membership(
        &result_wire["provenance"]["xml_classification"],
        &result_wire["xml_classification_membership"],
        &graph,
        &stop,
    )?;
    // Query/nonquery comparison on this build; no predecessor byte-freeze claim.
    assert_eq!(result.snapshot_bytes()?, ordinary.snapshot_bytes()?);
    for row in result_wire["xml_fingerprints"]["records"]
        .as_array()
        .ok_or("canonical records")?
    {
        if row["projection"].get("class_proposal_id").is_none() {
            assert!(row["projection"].get("relation_proposal_id").is_none());
            assert!(row["support"].get("class_assertion").is_none());
            assert!(row["support"].get("class_node_id").is_none());
            assert!(row["support"].get("classification_assertion").is_none());
        }
    }
    let mut null = selection_wire.clone();
    let unknown = null["targets"]
        .as_array_mut()
        .ok_or("targets")?
        .iter_mut()
        .find(|target| target.get("class_id").is_none())
        .ok_or("unclassified target")?;
    unknown["class_id"] = Value::Null;
    assert_eq!(
        XmlFingerprintSelection::from_json_slice(&serde_json::to_vec(&null)?)
            .err()
            .ok_or("strict scanner admitted null class ID")?
            .code(),
        ServiceErrorCode::InvalidRequest
    );
    assert_eq!(
        XmlFingerprintSelection::new(vec![targets[0].clone(), targets[0].clone()], 2, limit)
            .err()
            .ok_or("duplicate IDs admitted")?
            .code(),
        ServiceErrorCode::InvalidRequest
    );
    assert_eq!(
        XmlFingerprintSelection::new(targets.clone(), targets.len(), 0)
            .err()
            .ok_or("zero metadata limit admitted")?
            .code(),
        ServiceErrorCode::InvalidRequest
    );
    assert_eq!(
        XmlFingerprintSelection::from_json_slice_cancellable(
            &selection_bytes,
            &AtomicBool::new(true),
        )
        .err()
        .ok_or("cancelled decoder continued")?
        .code(),
        ServiceErrorCode::Cancelled
    );

    // One meaningful mutation: only QName prefix aliases change in the synthetic XSD.
    assert!(CLASSIFICATION_XSD.contains("xmlns:u="));
    assert!(CLASSIFICATION_XSD.contains("\"u:"));
    let renamed_xsd = CLASSIFICATION_XSD
        .replace("xmlns:u=", "xmlns:v=")
        .replace("\"u:", "\"v:");
    let (renamed_root, _) = classification_fixture(&renamed_xsd)?;
    let renamed_path = renamed_root.0.join("project.json");
    let input = LocalProjectInput::from_config_path(&renamed_path, &stop)?;
    let renamed_request =
        GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
            .with_input_profile(&input);
    let renamed_ordinary = execute_graph_build(input, &renamed_request, &stop)?;
    assert_eq!(
        renamed_ordinary.status(),
        GraphReadStatus::Partial,
        "{:?}",
        renamed_ordinary.failure_message()
    );
    let renamed_wire = serde_json::to_value(&renamed_ordinary)?;
    let renamed_targets =
        native_xml_fingerprint_targets(&renamed_wire["provenance"]["xml_classification"])?;
    let count = renamed_targets.len();
    let renamed_request = renamed_request.with_xml_fingerprints(XmlFingerprintSelection::new(
        renamed_targets,
        count,
        limit,
    )?)?;
    let renamed = execute_graph_build(
        LocalProjectInput::from_config_path(&renamed_path, &stop)?,
        &renamed_request,
        &stop,
    )?;
    assert_eq!(
        renamed.status(),
        GraphReadStatus::Partial,
        "{:?}",
        renamed.failure_message()
    );
    let renamed_normalized = fixture_xml_query_components(
        renamed
            .xml_fingerprints()
            .ok_or("renamed native XML manifest")?,
    )?;
    let renamed_wire: Value = serde_json::from_slice(&renamed.canonical_bytes()?)?;
    assert_ne!(
        result_wire["xml_fingerprints"]["binding"]["source"]["source_snapshot_id"],
        renamed_wire["xml_fingerprints"]["binding"]["source"]["source_snapshot_id"]
    );
    assert_ne!(
        result_wire["xml_fingerprints"]["binding"]["source"]["catalog_digest"],
        renamed_wire["xml_fingerprints"]["binding"]["source"]["catalog_digest"]
    );
    // Fresh native IDs/support are custody; comparable fields/digests keep the same recipe.
    assert_eq!(normalized, renamed_normalized);
    Ok(())
}

fn selected_type_shape_fixture(xsd: &str) -> TestResult<crate::graph::GraphBuildResult> {
    use crate::graph::{
        GRAPH_RESULT_MAX_BYTES, GraphBuildRequest, GraphReadStatus, XmlFingerprintSelection,
        XmlFingerprintTarget, XmlSchemaTypeShapeRecipe, execute_graph_build,
    };
    use wow_project::load::schema::classification::RegionTypeWitness;

    let stop = AtomicBool::new(false);
    let (root, config) = classification_fixture(xsd)?;
    // This is the explicit Texture witness already admitted by the fixture's
    // reviewed region policy, not a production by-name/first-target selection.
    let intent: RegionTypeWitness = serde_json::from_value(
        config["xml_schema_classification"]["profile_selection"]["region_like"]["members"][0]
            .clone(),
    )?;
    let path = root.0.join("project.json");
    let input = LocalProjectInput::from_config_path(&path, &stop)?;
    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    let ordinary = execute_graph_build(input, &request, &stop)?;
    assert_eq!(ordinary.status(), GraphReadStatus::Partial);
    let ordinary_wire = serde_json::to_value(&ordinary)?;
    let documents = ordinary_wire["provenance"]["xml_classification"]["documents"]
        .as_array()
        .ok_or("native classification documents")?;
    let [document] = documents.as_slice() else {
        return Err("expected one classification document".into());
    };
    let observations = document["report"]["occurrences"]
        .as_array()
        .ok_or("native classification observations")?;
    let projections = document["occurrences"]
        .as_array()
        .ok_or("native classification projections")?;
    assert_eq!(observations.len(), projections.len());
    let matches: Vec<_> = observations
        .iter()
        .zip(projections)
        .filter(|(observation, _)| observation["qname"]["name"]["local_name"] == "Texture")
        .collect();
    let [(observation, projection)] = matches.as_slice() else {
        return Err("expected one native Texture occurrence".into());
    };
    assert_eq!(observation["occurrence"], projection["occurrence"]);
    let occurrence = projection["occurrence_proposal_id"]
        .as_str()
        .ok_or("native Texture proposal")?;
    let class = projection["class_proposal_id"]
        .as_str()
        .ok_or("native Texture class")?;
    let target = XmlFingerprintTarget::new(occurrence.into(), Some(class.into()))
        .with_type_shape(intent.element, intent.type_component)?;
    let selection = XmlFingerprintSelection::new_with_type_shapes(
        vec![target],
        1,
        GRAPH_RESULT_MAX_BYTES,
        XmlSchemaTypeShapeRecipe::ClosedComplexTreeV1,
    )?;
    let selection_wire = serde_json::to_value(&selection)?;
    assert_eq!(
        selection_wire["schema"],
        "wow-service/xml-fingerprint-selection/selected-components/1"
    );
    let selection = XmlFingerprintSelection::from_json_slice(&serde_json::to_vec(&selection)?)?;
    let selected = execute_graph_build(
        LocalProjectInput::from_config_path(&path, &stop)?,
        &request.with_xml_fingerprints(selection)?,
        &stop,
    )?;
    assert_eq!(
        selected.status(),
        GraphReadStatus::Partial,
        "{:?}",
        selected.failure_message()
    );
    assert_eq!(selected.snapshot_bytes()?, ordinary.snapshot_bytes()?);
    let manifest = selected
        .xml_fingerprints()
        .ok_or("selected type-shape manifest")?;
    assert_eq!(manifest.coverage(), wow_graph::GraphCoverageState::Partial);
    assert_eq!(manifest.records().len(), 1);
    assert_eq!(manifest.type_shapes().ok_or("type batch")?.len(), 1);
    let row = manifest
        .type_shape_for(0)
        .ok_or("exact occurrence/type mapping")?;
    assert_eq!(row.type_component(), intent.type_component);
    assert_eq!(manifest.records()[0].type_shape_index(), Some(0));
    assert!(manifest.records()[0].type_handoff().is_some());
    assert!(!row.witnesses().is_empty());
    assert!(!row.references().is_empty());
    let raw = manifest
        .type_shape_support(0)
        .ok_or("complete closure raw support")?;
    assert_eq!(raw.len(), 1);
    assert_eq!(raw[0].source().path().as_str(), "UI/classification.xsd");
    assert_eq!(*raw[0].source().content_digest(), digest(xsd.as_bytes()));
    assert!(
        raw[0]
            .evidence()
            .source_handle_ids()
            .contains(&raw[0].source().handle_id())
    );
    assert_eq!(
        manifest.schema(),
        "wow-service/xml-fingerprint-manifest/selected-components/1"
    );
    Ok(selected)
}

#[test]
fn selected_xml_type_shape_tracks_nested_occurs_and_refuses_incoming_substitution() -> TestResult {
    use crate::graph::{XmlSchemaTypeShapeEvent, XmlSchemaTypeShapeUnavailable};
    use wow_project::load::schema::XmlSchemaReferenceKind;

    let xsd = CLASSIFICATION_XSD.replace(
        "<xs:complexType name=\"TextureType\"/>",
        r#"<xs:complexType name="TextureType"><xs:sequence>
          <xs:element ref="u:ShapeLeaf"/>
        </xs:sequence></xs:complexType>
        <xs:complexType name="ShapeLeafType"><xs:sequence>
          <xs:element name="leaf" minOccurs="0" maxOccurs="2"><xs:complexType/></xs:element>
        </xs:sequence></xs:complexType>
        <xs:element name="ShapeLeaf" type="u:ShapeLeafType"/>"#,
    );
    let base = selected_type_shape_fixture(&xsd)?;
    let base_row = base
        .xml_fingerprints()
        .ok_or("base manifest")?
        .type_shape_for(0)
        .ok_or("base type row")?;
    let base_component = base_row.component();
    let base_shape = base_component
        .comparable()
        .ok_or("base complete complex closure")?;
    let base_digest = base_component.digest().ok_or("base digest")?;

    // Only one supported nested particle bound changes; the QName/ref topology
    // is unchanged. The finite whole-tree component must observe the mutation.
    assert_eq!(xsd.matches("name=\"leaf\" minOccurs=\"0\"").count(), 1);
    let mutated_xsd = xsd.replace(
        "name=\"leaf\" minOccurs=\"0\"",
        "name=\"leaf\" minOccurs=\"1\"",
    );
    let mutated = selected_type_shape_fixture(&mutated_xsd)?;
    let mutated_component = mutated
        .xml_fingerprints()
        .ok_or("mutated manifest")?
        .type_shape_for(0)
        .ok_or("mutated type row")?
        .component();
    let mutated_shape = mutated_component
        .comparable()
        .ok_or("mutated complete complex closure")?;
    assert_ne!(
        base_digest,
        mutated_component.digest().ok_or("mutated digest")?
    );
    assert_eq!(base_shape.events().len(), mutated_shape.events().len());
    let changes: Vec<_> = base_shape
        .events()
        .iter()
        .zip(mutated_shape.events())
        .filter(|(before, after)| before != after)
        .collect();
    let [
        (
            XmlSchemaTypeShapeEvent::Element {
                occurs: Some(before),
                ..
            },
            XmlSchemaTypeShapeEvent::Element {
                occurs: Some(after),
                ..
            },
        ),
    ] = changes.as_slice()
    else {
        return Err("expected exactly one nested element cardinality mutation".into());
    };
    assert_eq!(before.min_occurs(), 0);
    assert_eq!(after.min_occurs(), 1);
    assert_eq!(before.max_occurs(), after.max_occurs());

    // A new incoming substitution member is outside the direct type tree, but
    // changes the reached global head's schema context. Never hash its old prefix.
    let substituted_xsd = xsd.replace("</xs:schema>",
        r#"<xs:element name="ShapeLeafMember" type="u:ShapeLeafType" substitutionGroup="u:ShapeLeaf"/></xs:schema>"#);
    let substituted = selected_type_shape_fixture(&substituted_xsd)?;
    let substituted_row = substituted
        .xml_fingerprints()
        .ok_or("substitution manifest")?
        .type_shape_for(0)
        .ok_or("substitution type row")?;
    assert_eq!(
        substituted_row.component().unavailable_reason(),
        Some(&XmlSchemaTypeShapeUnavailable::UnsupportedReferenceKind(
            XmlSchemaReferenceKind::SubstitutionGroup
        ))
    );
    assert!(substituted_row.component().digest().is_none());
    assert!(substituted_row.component().comparable().is_none());
    assert!(
        substituted_row
            .references()
            .iter()
            .any(|reference| reference.kind() == XmlSchemaReferenceKind::SubstitutionGroup)
    );
    Ok(())
}
