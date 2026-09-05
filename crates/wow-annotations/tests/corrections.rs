#![allow(clippy::unwrap_used)]
//! Ketho patch/widget semantics through the actual source/model/renderer seam.
use std::sync::atomic::AtomicBool;
use wow_annotations::{
    ketho::{Field, Function, Owner, Renderer, System},
    native::{project, project_with_corrections},
};
use wow_reference::native::{DocumentationDocument, ingest_document, source_digest};
use wow_reference::native_corrections::*;
use wow_reference::native_model::{SystemOwner, normalize_document};
const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const INPUT: &str = r#"local s={Name="Vars",Type="System",Namespace="C_Vars",Environment="All",Functions={
 {Name="Set",Arguments={{Name="unrelated",Type="bool"},{Name="value",Type="cstring",Nilable=true}},Returns={{Name="result",Type="bool"}}},
 {Name="Get"}}} APIDocumentation:AddDocumentationTable(s)"#;
fn doc(path: &str, source: &str) -> DocumentationDocument {
    ingest_document(
        REV,
        path,
        source,
        &source_digest(source.as_bytes()),
        &AtomicBool::new(false),
    )
    .unwrap()
}
fn frame(path: &str, name: &str) -> DocumentationDocument {
    doc(
        path,
        &format!(
            r#"APIDocumentation:AddDocumentationTable({{Name="{name}",Type="ScriptObject",Functions={{{{Name="Show"}}}}}})"#
        ),
    )
}
fn record(
    document: &DocumentationDocument,
    projection: Projection,
    before: Value,
    after: Value,
) -> Correction {
    let normalized = normalize_document(document);
    let system = normalized.systems[0].as_ref().unwrap();
    let raw = match &projection {
        Projection::WidgetOwner => system.raw,
        Projection::CallableField {
            function,
            lane,
            member,
            ..
        } => {
            let f = system
                .functions
                .iter()
                .find(|f| f.name == function)
                .unwrap();
            let fields = match lane {
                Lane::Arguments => &f.arguments,
                Lane::Returns => &f.returns,
            };
            fields.iter().find(|f| f.name == member).unwrap().raw
        }
    };
    Correction {
        id: format!("{}:{projection:?}", document.path()),
        target: Target {
            path: document.path().into(),
            registration: 0,
            projection,
        },
        expected_source_sha256: document.sha256().into(),
        expected_raw_sha256: raw_digest(raw).unwrap(),
        before,
        after,
        reviewer: "synthetic-review".into(),
        rationale: "Synthetic Ketho behavior regression, not platform evidence".into(),
        evidence: vec![Evidence {
            revision: REV.into(),
            path: "synthetic/donor-case.lua".into(),
            sha256: source_digest(b"synthetic reviewed evidence"),
        }],
    }
}
fn type_patch(document: &DocumentationDocument) -> Correction {
    record(
        document,
        Projection::CallableField {
            function: "Set".into(),
            lane: Lane::Arguments,
            member: "value".into(),
            property: Property::Type,
        },
        Value::Text("cstring".into()),
        Value::Text("string|number".into()),
    )
}
fn alias(document: &DocumentationDocument, before: &str, after: &str) -> Correction {
    record(
        document,
        Projection::WidgetOwner,
        Value::Text(before.into()),
        Value::Text(after.into()),
    )
}
fn pack(records: Vec<Correction>) -> CorrectionSet {
    CorrectionSet {
        schema: SCHEMA.into(),
        version: 1,
        revision: REV.into(),
        environment: "Mainline".into(),
        normalizer: NORMALIZER.into(),
        records,
    }
}
#[test]
fn ketho_name_lookup_type_and_nilability_reach_renderer_without_changing_raw() {
    let docs = [doc("Vars.lua", INPUT)];
    let original = docs.clone();
    let nilability = record(
        &docs[0],
        Projection::CallableField {
            function: "Set".into(),
            lane: Lane::Returns,
            member: "result".into(),
            property: Property::Nilable,
        },
        Value::Absent,
        Value::Boolean(true),
    );
    let pack = ValidatedCorrections::new(pack(vec![type_patch(&docs[0]), nilability])).unwrap();
    let library =
        project_with_corrections(&docs, "Mainline", Some(&pack), &AtomicBool::new(false)).unwrap();
    assert_eq!(docs, original);
    assert!(library.issues.is_empty());
    assert_eq!(library.schema, "wow-native-annotation-library/4");
    let text = &library.files[0].text;
    assert!(text.contains("---@param unrelated boolean"));
    assert!(text.contains("---@param value? (string|number)"));
    assert!(text.contains("---@return boolean? result"));
    assert!(text.contains("function C_Vars.Set(unrelated, value) end"));
    assert_eq!(library.corrections.as_ref().unwrap().applications.len(), 2);
    assert!(
        library
            .corrections
            .as_ref()
            .unwrap()
            .applications
            .iter()
            .all(|a| a.status == Status::Applied && a.after.is_some())
    );
    assert!(!library.negative_authority);
    let uncorrected = project(&docs, "Mainline", &AtomicBool::new(false)).unwrap();
    assert!(
        uncorrected.files[0]
            .text
            .contains("---@param value? string")
    );
    assert_eq!(uncorrected.schema, "wow-native-annotation-library/3");
    assert!(
        !serde_json::to_string(&uncorrected)
            .unwrap()
            .contains("\"corrections\"")
    );
}
#[test]
fn widget_receiver_uses_explicit_alias_and_retains_source_owner() {
    let docs = [frame("Frame.lua", "SimpleFrameAPI")];
    let pack =
        ValidatedCorrections::new(pack(vec![alias(&docs[0], "SimpleFrameAPI", "Frame")])).unwrap();
    let library =
        project_with_corrections(&docs, "Mainline", Some(&pack), &AtomicBool::new(false)).unwrap();
    assert!(library.files[0].text.contains("function Frame:Show() end"));
    assert!(library.files[0].text.contains("API_Frame_Show"));
    assert_eq!(
        normalize_document(&docs[0]).systems[0]
            .as_ref()
            .unwrap()
            .owner,
        SystemOwner::ScriptObject("SimpleFrameAPI")
    );
    let application = &library.corrections.as_ref().unwrap().applications[0];
    assert_eq!(
        application.before,
        Some(Value::Text("SimpleFrameAPI".into()))
    );
    assert_eq!(application.status, Status::Applied);
    assert_eq!(library.files[0].mappings[0].source.sha256, docs[0].sha256());
}
#[test]
fn revision_normalizer_source_digest_raw_digest_and_expected_value_expire() {
    let docs = [doc("Vars.lua", INPUT)];
    for mutation in 0..5 {
        let mut data = pack(vec![type_patch(&docs[0])]);
        match mutation {
            0 => data.revision = "b".repeat(40),
            1 => data.normalizer = "native-model/next".into(),
            2 => data.records[0].expected_source_sha256 = source_digest(b"changed"),
            3 => data.records[0].expected_raw_sha256 = source_digest(b"changed"),
            _ => data.records[0].before = Value::Text("number".into()),
        }
        let pack = ValidatedCorrections::new(data.clone()).unwrap();
        let library =
            project_with_corrections(&docs, "Mainline", Some(&pack), &AtomicBool::new(false))
                .unwrap();
        assert_eq!(library.projection, "partial");
        let report = library.corrections.as_ref().unwrap();
        assert_eq!(report.applications[0].status, Status::Expired);
        assert!(report.applications[0].after.is_none());
        assert_eq!(
            report.corrections.set().records[0].expected_source_sha256,
            data.records[0].expected_source_sha256
        );
        assert!(library.files[0].text.contains("---@param value? string"));
    }
}
#[test]
fn other_environment_is_not_applicable_and_does_not_change_projection() {
    let docs = [doc("Vars.lua", INPUT)];
    let pack = ValidatedCorrections::new(pack(vec![type_patch(&docs[0])])).unwrap();
    let library =
        project_with_corrections(&docs, "Glue", Some(&pack), &AtomicBool::new(false)).unwrap();
    assert_eq!(
        library.corrections.as_ref().unwrap().applications[0].status,
        Status::NotApplicable
    );
    assert_eq!(library.projection, "projected_with_sidecars");
}
#[test]
fn duplicate_corrections_conflict_even_when_replacements_agree() {
    let docs = [doc("Vars.lua", INPUT)];
    for different in [false, true] {
        let first = type_patch(&docs[0]);
        let mut second = first.clone();
        second.id = "second".into();
        if different {
            second.after = Value::Text("number".into());
        }
        let pack = ValidatedCorrections::new(pack(vec![first, second])).unwrap();
        let library =
            project_with_corrections(&docs, "Mainline", Some(&pack), &AtomicBool::new(false))
                .unwrap();
        assert!(
            library
                .corrections
                .as_ref()
                .unwrap()
                .applications
                .iter()
                .all(|a| a.status == Status::Conflict)
        );
        assert!(library.files[0].text.contains("---@param value? string"));
    }
}
#[test]
fn multiple_source_members_never_use_first_matching_name() {
    let source = INPUT.replace(
        "{Name=\"value\",Type=\"cstring\",Nilable=true}",
        "{Name=\"value\",Type=\"cstring\",Nilable=true},{Name=\"value\",Type=\"cstring\"}",
    );
    let docs = [doc("Vars.lua", &source)];
    let pack = ValidatedCorrections::new(pack(vec![type_patch(&docs[0])])).unwrap();
    let corrected = apply_to_documents(&docs, "Mainline", &pack, &AtomicBool::new(false)).unwrap();
    assert_eq!(corrected.report.applications[0].status, Status::Conflict);
    assert_eq!(
        corrected.systems()[0].1.functions[0].arguments[1].type_name,
        "cstring"
    );
}
#[test]
fn independent_order_changes_neither_set_identity_nor_artifacts() {
    let docs = [doc("Vars.lua", INPUT), frame("Frame.lua", "SimpleFrameAPI")];
    let records = vec![
        type_patch(&docs[0]),
        alias(&docs[1], "SimpleFrameAPI", "Frame"),
    ];
    let left = ValidatedCorrections::new(pack(records.clone())).unwrap();
    let right = ValidatedCorrections::new(pack(records.into_iter().rev().collect())).unwrap();
    assert_eq!(left.id(), right.id());
    let reversed = [docs[1].clone(), docs[0].clone()];
    let a =
        project_with_corrections(&docs, "Mainline", Some(&left), &AtomicBool::new(false)).unwrap();
    let b = project_with_corrections(&reversed, "Mainline", Some(&right), &AtomicBool::new(false))
        .unwrap();
    assert_eq!(
        serde_json::to_vec(&a).unwrap(),
        serde_json::to_vec(&b).unwrap()
    );
}
#[test]
fn alias_collision_reversion_rechecks_the_whole_chain() {
    let docs = [
        frame("A.lua", "A"),
        frame("B.lua", "B"),
        frame("C.lua", "C"),
        frame("D.lua", "D"),
    ];
    let pack = ValidatedCorrections::new(pack(vec![
        alias(&docs[0], "A", "B"),
        alias(&docs[1], "B", "C"),
        alias(&docs[2], "C", "D"),
    ]))
    .unwrap();
    let corrected = apply_to_documents(&docs, "Mainline", &pack, &AtomicBool::new(false)).unwrap();
    assert!(
        corrected
            .report
            .applications
            .iter()
            .all(|a| a.status == Status::Conflict && a.reason == "widget_name_collision")
    );
    for (index, name) in ["A", "B", "C", "D"].iter().enumerate() {
        assert_eq!(
            corrected.systems()[index].1.owner,
            SystemOwner::ScriptObject(name)
        );
    }
}
#[test]
fn widget_alias_cannot_collide_with_class_global_or_namespace() {
    for source in [
        r#"APIDocumentation:AddDocumentationTable({Tables={{Name="Frame",Type="Structure",Fields={}}}})"#,
        r#"APIDocumentation:AddDocumentationTable({Name="X",Type="System",Functions={{Name="Frame"}}})"#,
        r#"APIDocumentation:AddDocumentationTable({Name="X",Type="System",Namespace="Frame"})"#,
    ] {
        let docs = [frame("A.lua", "SimpleFrameAPI"), doc("B.lua", source)];
        let pack =
            ValidatedCorrections::new(pack(vec![alias(&docs[0], "SimpleFrameAPI", "Frame")]))
                .unwrap();
        let corrected =
            apply_to_documents(&docs, "Mainline", &pack, &AtomicBool::new(false)).unwrap();
        assert_eq!(corrected.report.applications[0].status, Status::Conflict);
    }
}
#[test]
fn malformed_unknown_duplicate_json_and_unreviewed_sets_reject() {
    let docs = [doc("Vars.lua", INPUT)];
    let data = pack(vec![type_patch(&docs[0])]);
    for mutation in 0..6 {
        let mut data = data.clone();
        match mutation {
            0 => data.records[0].reviewer.clear(),
            1 => data.records[0].evidence.clear(),
            2 => data.records[0].target.path = "../Vars.lua".into(),
            3 => data.records[0].after = Value::Text("string\n---@class Injected".into()),
            4 => data.records[0].after = Value::Boolean(true),
            _ => data.records[0].after = Value::Text("string||number".into()),
        }
        assert!(ValidatedCorrections::new(data).is_err());
    }
    let json = serde_json::to_string(&data).unwrap();
    let duplicate = json.replacen("{", "{\"schema\":\"ignored\",", 1);
    assert!(ValidatedCorrections::from_json(duplicate.as_bytes()).is_err());
    let unknown = json.replacen("{", "{\"execute\":\"bad\",", 1);
    assert!(ValidatedCorrections::from_json(unknown.as_bytes()).is_err());
    assert_eq!(
        ValidatedCorrections::from_json(json.as_bytes())
            .unwrap()
            .id(),
        ValidatedCorrections::new(data).unwrap().id()
    );
}
#[test]
fn malformed_document_cannot_be_repaired_by_correction_guesswork() {
    let good = doc("Vars.lua", INPUT);
    let mut patch = type_patch(&good);
    let bad = doc(
        "Vars.lua",
        r#"APIDocumentation:AddDocumentationTable({Name="X",Name="Y",Type="System"})"#,
    );
    patch.expected_source_sha256 = bad.sha256().into();
    let docs = [bad];
    let pack = ValidatedCorrections::new(pack(vec![patch])).unwrap();
    let corrected = apply_to_documents(&docs, "Mainline", &pack, &AtomicBool::new(false)).unwrap();
    assert_eq!(corrected.normalization_errors.len(), 1);
    assert!(corrected.systems().is_empty());
    assert_eq!(corrected.report.applications[0].status, Status::Expired);
}
#[test]
fn changes_to_review_or_pack_version_change_identity() {
    let docs = [doc("Vars.lua", INPUT)];
    let data = pack(vec![type_patch(&docs[0])]);
    let original = ValidatedCorrections::new(data.clone()).unwrap();
    let mut changed = data.clone();
    changed.version += 1;
    assert_ne!(
        original.id(),
        ValidatedCorrections::new(changed).unwrap().id()
    );
    let mut changed = data;
    changed.records[0].reviewer = "second-review".into();
    assert_ne!(
        original.id(),
        ValidatedCorrections::new(changed).unwrap().id()
    );
}
#[test]
fn cancellation_limits_and_mixed_revision_reject_without_changing_source() {
    let docs = [doc("Vars.lua", INPUT)];
    let original = docs.clone();
    let data = pack(vec![type_patch(&docs[0])]);
    let pack = ValidatedCorrections::new(data.clone()).unwrap();
    assert!(matches!(
        apply_to_documents(&docs, "Mainline", &pack, &AtomicBool::new(true)),
        Err(CorrectionError::Cancelled)
    ));
    assert_eq!(docs, original);
    let mut oversized = data;
    oversized.records = vec![oversized.records[0].clone(); 4097];
    assert!(matches!(
        ValidatedCorrections::new(oversized),
        Err(CorrectionError::Limit)
    ));
    assert!(matches!(
        ValidatedCorrections::from_json(&vec![b' '; 2 * 1024 * 1024 + 1]),
        Err(CorrectionError::Limit)
    ));
    let other = ingest_document(
        &"b".repeat(40),
        "Other.lua",
        INPUT,
        &source_digest(INPUT.as_bytes()),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert!(matches!(
        apply_to_documents(
            &[docs[0].clone(), other],
            "Mainline",
            &pack,
            &AtomicBool::new(false)
        ),
        Err(CorrectionError::InvalidSource)
    ));
}
#[test]
fn exact_path_and_registration_never_fall_back_to_basename() {
    let docs = [doc("Other/Vars.lua", INPUT)];
    let mut patch = type_patch(&docs[0]);
    patch.target.path = "Wanted/Vars.lua".into();
    let pack = ValidatedCorrections::new(pack(vec![patch])).unwrap();
    assert_eq!(
        apply_to_documents(&docs, "Mainline", &pack, &AtomicBool::new(false))
            .unwrap()
            .report
            .applications[0]
            .status,
        Status::Expired
    );
}
#[test]
fn alias_targeting_namespace_is_rejected_without_turning_it_into_object() {
    let docs = [doc("Vars.lua", INPUT)];
    let patch = record(
        &docs[0],
        Projection::WidgetOwner,
        Value::Text("Vars".into()),
        Value::Text("Frame".into()),
    );
    let pack = ValidatedCorrections::new(pack(vec![patch])).unwrap();
    let corrected = apply_to_documents(&docs, "Mainline", &pack, &AtomicBool::new(false)).unwrap();
    assert_eq!(corrected.report.applications[0].status, Status::Rejected);
    assert_eq!(
        corrected.systems()[0].1.owner,
        SystemOwner::Namespace("C_Vars")
    );
}
#[test]
fn supported_union_syntax_is_bounded_and_arrays_preserve_grouping() {
    let renderer = Renderer::new(Default::default(), 1024 * 1024).unwrap();
    assert_eq!(
        renderer.lower_type("string|number").unwrap(),
        "string|number"
    );
    for bad in [
        "string||number",
        "string|string",
        "bool|boolean",
        "string| number",
        "string;evil()",
        "string\n---@class Evil",
    ] {
        assert!(renderer.lower_type(bad).is_err(), "{bad}");
    }
    assert!(renderer.lower_type(&vec!["string"; 17].join("|")).is_err());
    let text = renderer
        .render(&System {
            owner: Owner::Global,
            functions: vec![Function {
                name: "Accept".into(),
                documentation: None,
                arguments: vec![Field {
                    name: "values".into(),
                    type_name: "table".into(),
                    inner_type: Some("string|number".into()),
                    nilable: false,
                    default_text: None,
                    variadic: false,
                }],
                returns: vec![],
            }],
            tables: vec![],
        })
        .unwrap();
    assert!(text.contains("---@param values (string|number)[]"));
    let parsed = emmylua_parser::LuaParser::parse(
        &text,
        emmylua_parser::ParserConfig::with_level(emmylua_parser::LuaLanguageLevel::Lua51),
    );
    assert!(parsed.get_errors().is_empty(), "{:?}", parsed.get_errors());
}

#[test]
fn alias_does_not_disguise_an_existing_source_owner_conflict() {
    let docs = [frame("A.lua", "Original"), frame("B.lua", "Original")];
    let pack = ValidatedCorrections::new(pack(vec![alias(&docs[0], "Original", "Frame")])).unwrap();
    let result =
        project_with_corrections(&docs, "Mainline", Some(&pack), &AtomicBool::new(false)).unwrap();
    assert_eq!(
        result.corrections.as_ref().unwrap().applications[0].status,
        Status::Conflict
    );
    assert!(result.files.is_empty());
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.code == "duplicate_callable")
    );
}
#[test]
fn alias_collision_is_scoped_to_selected_environment() {
    let docs = [
        frame("A.lua", "Original"),
        doc(
            "B.lua",
            r#"APIDocumentation:AddDocumentationTable({Name="Frame",Type="ScriptObject",Environment="Glue",Functions={{Name="Show"}}})"#,
        ),
    ];
    let pack = ValidatedCorrections::new(pack(vec![alias(&docs[0], "Original", "Frame")])).unwrap();
    let result =
        project_with_corrections(&docs, "Mainline", Some(&pack), &AtomicBool::new(false)).unwrap();
    assert_eq!(
        result.corrections.as_ref().unwrap().applications[0].status,
        Status::Applied
    );
    assert!(result.files[0].text.contains("function Frame:Show"));
}
#[test]
fn type_patch_cannot_silently_leave_incompatible_inner_type() {
    let source = INPUT.replace(
        "Type=\"cstring\",Nilable=true",
        "Type=\"cstring\",InnerType=\"string\",Nilable=true",
    );
    let docs = [doc("Vars.lua", &source)];
    let pack = ValidatedCorrections::new(pack(vec![type_patch(&docs[0])])).unwrap();
    let corrected = apply_to_documents(&docs, "Mainline", &pack, &AtomicBool::new(false)).unwrap();
    assert_eq!(corrected.report.applications[0].status, Status::Rejected);
}
