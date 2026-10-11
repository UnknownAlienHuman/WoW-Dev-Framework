//! One native finite model: local OnLoad is allowed only under exact Scripts.
mod named_xml_neighborhood;

use super::*;
use crate::graph::{
    FunctionDirectNeighborhoodRecipe, FunctionFingerprintSelection, GRAPH_RESULT_MAX_BYTES,
    GraphReadStatus, execute_graph_build,
};
use wow_emmy::function_calls::SourceFunctionKind;
use wow_project::graph::fingerprints::{
    FunctionDirectNeighborhoodSelection, FunctionNeighborDirection, FunctionNeighborKind,
    FunctionNeighborhoodComponent,
};

fn enumeration_xsd() -> String {
    CLASSIFICATION_XSD
        .replace(r#"targetNamespace="http://www.blizzard.com/wow/ui/">"#,
            r#"targetNamespace="http://www.blizzard.com/wow/ui/" elementFormDefault="qualified">"#)
        .replace(r#"<xs:complexType name="FrameRefType"/>"#,
            r#"<xs:complexType name="FrameRefType"><xs:attribute name="name" use="required"/></xs:complexType>"#)
        .replace(r#"<xs:extension base="u:FrameRefType"/>"#,
            r#"<xs:extension base="u:FrameRefType"><xs:sequence><xs:element ref="u:Scripts"/></xs:sequence></xs:extension>"#)
        .replace(r#"<xs:element name="Ui"/><xs:element name="Scripts"/><xs:element name="OnLoad"/>"#,
            r#"<xs:element name="Ui"><xs:complexType><xs:sequence>
                <xs:element ref="u:Frame"/><xs:element ref="u:Include" minOccurs="0"/>
            </xs:sequence></xs:complexType></xs:element>
            <xs:element name="Scripts"><xs:complexType><xs:sequence>
                <xs:element name="OnLoad" minOccurs="0"><xs:complexType>
                    <xs:attribute name="function" use="required"/>
                </xs:complexType></xs:element>
            </xs:sequence></xs:complexType></xs:element>
            <xs:element name="Include"><xs:complexType><xs:attribute name="file" use="required"/></xs:complexType></xs:element>
            <xs:element name="OnLoad"/>"#)
}

#[test]
fn finite_native_member_census_changes_xml_degree_and_preserves_legacy_partial() -> TestResult {
    const LUA: &str = "function Handle() end\n";
    const TOC: &str = "## Interface: 120100\n## LoadOnDemand: 0\nmain.lua\nframes.xml\n";
    let stop = AtomicBool::new(false);
    let xsd = enumeration_xsd();
    let mut comparable = Vec::new();
    for (xml, degree) in [
        (
            r#"<Ui xmlns="http://www.blizzard.com/wow/ui/"><Frame name="NativeFrame"><Scripts><OnLoad function="Handle"/></Scripts></Frame></Ui>"#,
            Some(1_u64),
        ),
        (
            r#"<Ui xmlns="http://www.blizzard.com/wow/ui/"><Frame name="NativeFrame"><Scripts/></Frame></Ui>"#,
            Some(0_u64),
        ),
        (
            r#"<Ui xmlns="http://www.blizzard.com/wow/ui/"><Frame name="NativeFrame"><Scripts><OnNotReviewed function="Handle"/></Scripts></Frame></Ui>"#,
            None,
        ),
        (
            r#"<Ui xmlns="http://www.blizzard.com/wow/ui/"><Frame name="NativeFrame"><Scripts><OnLoad function="ChooseHandler()"/></Scripts></Frame></Ui>"#,
            None,
        ),
        (
            r#"<Ui xmlns="http://www.blizzard.com/wow/ui/"><Frame name="NativeFrame"><Scripts><OnLoad function="Handle"/></Scripts></Frame><Include file="missing.xml"/></Ui>"#,
            None,
        ),
    ] {
        let (root, _config) = classification_fixture_members(&xsd, xml, Some(LUA), Some(TOC))?;
        let path = root.0.join("project.json");
        let (project, request, ids, source_bytes) =
            super::super::function_fingerprints::declaration_query(&path, &stop)?;
        let packages = project
            .configuration()
            .platform_packages()
            .ok_or("held packages")?;
        assert!(
            !packages.load_plan().coverage().complete(),
            "legacy XML semantics remain Partial"
        );
        assert!(packages.load_plan().issues().iter().any(|issue| issue.kind
            == wow_project::load::ProjectPackageLoadIssueKind::PackageFileClosurePartial));
        let report = project
            .snapshot()
            .analyzer_binding()
            .function_call_report()
            .ok_or("held report")?;
        let function = report
            .functions()
            .iter()
            .find(|fact| fact.kind() == SourceFunctionKind::Closure)
            .ok_or("physical Handle closure")?;
        assert!(report.calls().is_empty());
        let selection = FunctionFingerprintSelection::new(
            ids.clone(),
            ids.len(),
            source_bytes,
            GRAPH_RESULT_MAX_BYTES,
        )?
        .with_direct_neighborhood(
            FunctionDirectNeighborhoodRecipe::ComposedCallsAndXmlHandlersV1,
        )?;
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
        let record = result
            .function_fingerprints()
            .ok_or("manifest")?
            .records()
            .iter()
            .find(|record| record.function_id() == function.fact_id())
            .ok_or("Handle record")?;
        match (degree, record.direct_neighborhood()) {
            (Some(expected), FunctionDirectNeighborhoodSelection::Selected(value)) => {
                let FunctionNeighborhoodComponent::Comparable { digest, groups } = value.as_ref()
                else {
                    return Err("finite named native census did not reach Closed".into());
                };
                let xml = groups
                    .iter()
                    .find(|group| {
                        group.neighbor_kind() == FunctionNeighborKind::XmlSourceDeclaration
                            && group.direction() == FunctionNeighborDirection::Incoming
                    })
                    .ok_or("XML degree")?;
                assert_eq!(xml.multiplicity(), expected);
                assert!(
                    groups
                        .iter()
                        .filter(|group| group.neighbor_kind()
                            == FunctionNeighborKind::LuaSourceFunction)
                        .all(|group| group.multiplicity() == 0)
                );
                comparable.push(*digest);
            }
            (None, FunctionDirectNeighborhoodSelection::Selected(value)) => assert!(matches!(
                value.as_ref(),
                FunctionNeighborhoodComponent::Unavailable { .. }
            )),
            _ => return Err("selected XML census outcome mismatch".into()),
        }
        // The unselected component never invokes the selected witness/census.
        let ordinary = request.with_function_fingerprints(FunctionFingerprintSelection::new(
            ids.clone(),
            ids.len(),
            source_bytes,
            GRAPH_RESULT_MAX_BYTES,
        )?)?;
        let plain = execute_graph_build(
            LocalProjectInput::from_config_path(&path, &stop)?,
            &ordinary,
            &stop,
        )?;
        assert_eq!(plain.snapshot_bytes()?, result.snapshot_bytes()?);
        assert!(
            plain
                .function_fingerprints()
                .ok_or("plain manifest")?
                .records()
                .iter()
                .all(|record| matches!(
                    record.direct_neighborhood(),
                    FunctionDirectNeighborhoodSelection::NotSelected
                ))
        );
    }
    assert_eq!(comparable.len(), 2);
    assert_ne!(
        comparable[0], comparable[1],
        "adding exact named XML handler changes degree"
    );
    Ok(())
}

#[test]
fn referenced_model_checks_its_owning_schema_defaults() -> TestResult {
    use wow_project::graph::fingerprints::FunctionNeighborhoodUnavailable;
    use wow_project::load::schema::{
        XmlSchemaComponentKind as Kind, XmlSchemaComponentState, XmlSchemaReferenceKind as RefKind,
        XmlSchemaReferenceState,
    };

    const LUA: &str = "function Handle() end\n";
    const TOC: &str = "## Interface: 120100\n## LoadOnDemand: 0\nmain.lua\nframes.xml\n";
    const XML: &str = r#"<Ui xmlns="http://www.blizzard.com/wow/ui/"><Frame name="NativeFrame"><Scripts><OnLoad function="Handle"/></Scripts></Frame></Ui>"#;
    const INLINE_UI: &str = r#"<xs:element name="Ui"><xs:complexType><xs:sequence>
                <xs:element ref="u:Frame"/><xs:element ref="u:Include" minOccurs="0"/>
            </xs:sequence></xs:complexType></xs:element>"#;
    const MODEL: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
        xmlns:u="http://www.blizzard.com/wow/ui/" targetNamespace="http://www.blizzard.com/wow/ui/">
      <xs:complexType name="ExternalUiBase"/>
      <xs:complexType name="ExternalUiType"><xs:complexContent>
        <xs:extension base="u:ExternalUiBase"><xs:sequence><xs:element ref="u:Frame"/></xs:sequence></xs:extension>
      </xs:complexContent></xs:complexType>
    </xs:schema>"#;

    let stop = AtomicBool::new(false);
    let xsd = enumeration_xsd();
    assert!(xsd.contains(INLINE_UI));
    let xsd = xsd.replace(
        INLINE_UI,
        r#"<xs:element name="Ui" type="u:ExternalUiType"/>"#,
    );
    let controlled = MODEL.replace(
        r#"targetNamespace="http://www.blizzard.com/wow/ui/">"#,
        r#"targetNamespace="http://www.blizzard.com/wow/ui/" finalDefault="extension">"#,
    );
    assert_ne!(MODEL, controlled);
    for (model, has_control) in [(MODEL, false), (controlled.as_str(), true)] {
        // Both physical schema files are sealed before source/catalog/profile admission.
        let (root, _) = classification_fixture_schema_members(
            &[
                ("UI/classification.xsd", xsd.as_str()),
                ("UI/model.xsd", model),
            ],
            XML,
            Some(LUA),
            Some(TOC),
        )?;
        let path = root.0.join("project.json");
        let (project, request, ids, source_bytes) =
            super::super::function_fingerprints::declaration_query(&path, &stop)?;
        let catalog = project
            .configuration()
            .xml_classification_profile()
            .ok_or("reviewed native classifier")?
            .catalog();
        assert_eq!(catalog.documents().len(), 2);
        assert!(catalog.issues().is_empty(), "{:?}", catalog.issues());
        let ui = fixture_schema_global(catalog, Kind::GlobalElement, "Ui")?;
        let model_type = fixture_schema_global(catalog, Kind::NamedComplexType, "ExternalUiType")?;
        let base = fixture_schema_global(catalog, Kind::NamedComplexType, "ExternalUiBase")?;
        let content = fixture_schema_child(catalog, model_type, Kind::ComplexContent)?;
        let extension = fixture_schema_child(catalog, content, Kind::Extension)?;
        for (source, kind, target) in [
            (ui, RefKind::Type, model_type),
            (extension, RefKind::Base, base),
        ] {
            let references = catalog
                .references()
                .iter()
                .filter(|reference| reference.source() == source && reference.kind() == kind)
                .collect::<Vec<_>>();
            let [reference] = references.as_slice() else {
                return Err("native cross-document model link is not unique".into());
            };
            assert_eq!(reference.state(), XmlSchemaReferenceState::Unique);
            assert_eq!(reference.candidates(), &[target]);
        }
        let ui_component = catalog
            .components()
            .iter()
            .find(|c| c.id() == ui)
            .ok_or("Ui")?;
        let type_component = catalog
            .components()
            .iter()
            .find(|c| c.id() == model_type)
            .ok_or("referenced model")?;
        assert_eq!(ui_component.document(), "UI/classification.xsd");
        assert_eq!(type_component.document(), "UI/model.xsd");
        let owning_schema = catalog
            .components()
            .iter()
            .find(|c| c.kind() == Kind::Schema && c.document() == "UI/model.xsd")
            .ok_or("model owning schema")?;
        assert_eq!(owning_schema.state(), XmlSchemaComponentState::Observed);
        assert_eq!(
            owning_schema.attribute("finalDefault").is_some(),
            has_control
        );

        let report = project
            .snapshot()
            .analyzer_binding()
            .function_call_report()
            .ok_or("held native report")?;
        let function = report
            .functions()
            .iter()
            .find(|fact| fact.kind() == SourceFunctionKind::Closure)
            .ok_or("physical Handle closure")?;
        assert!(report.calls().is_empty());
        let selection = FunctionFingerprintSelection::new(
            ids.clone(),
            ids.len(),
            source_bytes,
            GRAPH_RESULT_MAX_BYTES,
        )?
        .with_direct_neighborhood(
            FunctionDirectNeighborhoodRecipe::ComposedCallsAndXmlHandlersV1,
        )?;
        let result = execute_graph_build(
            LocalProjectInput::from_config_path(&path, &stop)?,
            &request.with_function_fingerprints(selection)?,
            &stop,
        )?;
        assert_eq!(
            result.status(),
            GraphReadStatus::Partial,
            "{:?}",
            result.failure_message()
        );
        let record = result
            .function_fingerprints()
            .ok_or("selected native manifest")?
            .records()
            .iter()
            .find(|record| record.function_id() == function.fact_id())
            .ok_or("Handle record")?;
        let FunctionDirectNeighborhoodSelection::Selected(value) = record.direct_neighborhood()
        else {
            return Err("native XML census was not selected".into());
        };
        match (has_control, value.as_ref()) {
            (false, FunctionNeighborhoodComponent::Comparable { groups, .. }) => {
                let xml = groups
                    .iter()
                    .find(|group| {
                        group.neighbor_kind() == FunctionNeighborKind::XmlSourceDeclaration
                            && group.direction() == FunctionNeighborDirection::Incoming
                    })
                    .ok_or("Closed XML census incoming group")?;
                assert_eq!(xml.multiplicity(), 1);
            }
            (
                true,
                FunctionNeighborhoodComponent::Unavailable {
                    reason: FunctionNeighborhoodUnavailable::IncompleteXmlScope,
                },
            ) => {}
            _ => return Err("referenced owning-schema control outcome mismatch".into()),
        }
    }
    Ok(())
}
