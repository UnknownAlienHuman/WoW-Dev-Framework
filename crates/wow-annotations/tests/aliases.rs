//! Explicit alias resources remain outside Blizzard truth. These tests exercise
//! real Emmy parsing and the native source-to-render path, not a JSON substitute.
#![allow(clippy::unwrap_used)]
use std::sync::atomic::AtomicBool;
use wow_annotations::native::{project, project_with_alias_catalog};
use wow_reference::native::{
    DocumentationDocument, NativeErrorCode, ingest_document, source_digest,
};
use wow_reference::native_aliases::{AliasDocument, ingest_aliases};

const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DONOR: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
fn doc(text: &str) -> DocumentationDocument {
    ingest_document(
        REV,
        "API.lua",
        text,
        &source_digest(text.as_bytes()),
        &AtomicBool::new(false),
    )
    .unwrap()
}
fn aliases(text: &str) -> AliasDocument {
    ingest_aliases(
        DONOR,
        "Types.lua",
        text,
        &source_digest(text.as_bytes()),
        &AtomicBool::new(false),
    )
    .unwrap()
}
fn docs() -> Vec<DocumentationDocument> {
    vec![doc(
        r#"APIDocumentation:AddDocumentationTable({Name="ObjectAPI",Type="ScriptObject",Functions={{Name="Set",Type="Function",Arguments={{Name="value",Type="ExternalId",Nilable=false}}}}})"#,
    )]
}
fn projected<'a>(
    docs: &'a [DocumentationDocument],
    aliases: &'a AliasDocument,
) -> wow_annotations::native::NativeLibrary<'a> {
    project_with_alias_catalog(
        docs,
        "Mainline",
        None,
        Some(aliases),
        &AtomicBool::new(false),
    )
    .unwrap()
}

#[test]
fn native_alias_resource_reaches_real_library_without_replacing_blizzard_facts() {
    let docs = docs();
    let catalog = aliases("---@meta _\n---@alias ExternalId number|string\n");
    let before = serde_json::to_value(&docs).unwrap();
    let result = projected(&docs, &catalog);
    assert_eq!(result.schema, "wow-native-annotation-library/5");
    assert_eq!(result.projection, "projected_with_sidecars");
    assert!(!result.negative_authority);
    let report = result.aliases.as_ref().unwrap();
    assert_eq!(report.authority, "external_annotation_overlay");
    assert_eq!(report.source.revision(), DONOR);
    assert_eq!(result.revision, REV);
    assert_eq!(report.outcomes[0].status, "emitted");
    assert!(result.files[0].text.contains("---@param value ExternalId"));
    assert!(
        result
            .files
            .last()
            .unwrap()
            .text
            .contains("---@alias ExternalId number|string\n")
    );
    assert_eq!(before, serde_json::to_value(&docs).unwrap());
}
#[test]
fn aliases_are_opt_in_and_unconfigured_serialization_remains_v3() {
    let docs = docs();
    let plain = project(&docs, "Mainline", &AtomicBool::new(false)).unwrap();
    let via_none =
        project_with_alias_catalog(&docs, "Mainline", None, None, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        serde_json::to_vec(&plain).unwrap(),
        serde_json::to_vec(&via_none).unwrap()
    );
    assert!(
        serde_json::to_value(&plain)
            .unwrap()
            .get("aliases")
            .is_none()
    );
}
#[test]
fn original_resource_description_and_exact_bom_spans_survive() {
    let input = "\u{feff}---@meta _\r\n---@alias ExternalId number original description\r\n";
    let catalog = aliases(input);
    assert_eq!(catalog.text(), input);
    let span = catalog.aliases()[0].span;
    assert!(
        input
            .get(span.start..span.end)
            .unwrap()
            .contains("ExternalId number")
    );
    let docs = docs();
    let result = projected(&docs, &catalog);
    let file = result.files.last().unwrap();
    assert!(file.text.contains("Copyright (c) 2020 Ketho"));
    assert!(!file.text.contains("original description"));
    let map = &file.mappings[0];
    assert_eq!(map.source.scope, Some("annotation_alias_catalog"));
    assert_eq!(map.source.sha256, catalog.sha256());
    assert_eq!(map.source.span, span);
    assert_eq!(
        &file.text[map.generated.start..map.generated.end],
        "---@alias ExternalId number"
    );
}
#[test]
fn every_emitted_alias_parses_as_an_annotation_and_not_a_runtime_statement() {
    use emmylua_parser::{LuaAstNode, LuaDocTag, LuaLanguageLevel, LuaParser, ParserConfig};
    let docs = docs();
    let catalog = aliases("---@alias ExternalId number|string\n---@alias OtherId ExternalId\n");
    let result = projected(&docs, &catalog);
    let file = result.files.last().unwrap();
    let parsed = LuaParser::parse(
        &file.text,
        ParserConfig::with_level(LuaLanguageLevel::Lua51),
    );
    assert!(parsed.get_errors().is_empty(), "{:?}", parsed.get_errors());
    assert_eq!(
        parsed
            .get_chunk_node()
            .get_block()
            .unwrap()
            .get_stats()
            .count(),
        0
    );
    assert_eq!(
        parsed
            .get_chunk_node()
            .syntax()
            .descendants()
            .filter_map(LuaDocTag::cast)
            .filter(|tag| matches!(tag, LuaDocTag::Alias(_)))
            .count(),
        2
    );
}
#[test]
fn dependency_order_is_not_source_order_and_shared_graph_does_not_expand() {
    let docs = docs();
    let catalog = aliases(
        "---@alias Leaf Shared|ObjectAPI\n---@alias Shared ExternalId\n---@alias ExternalId string\n",
    );
    let result = projected(&docs, &catalog);
    assert!(result.issues.is_empty());
    assert_eq!(result.aliases.as_ref().unwrap().outcomes.len(), 3);
    let text = &result.files.last().unwrap().text;
    assert!(text.contains("---@alias Leaf Shared|ObjectAPI"));
    assert!(text.find("---@alias ExternalId").unwrap() < text.find("---@alias Leaf").unwrap());
    let reordered = aliases(
        "---@alias ExternalId string\n---@alias Shared ExternalId\n---@alias Leaf Shared|ObjectAPI\n",
    );
    assert_eq!(
        result.files.last().unwrap().sha256,
        projected(&docs, &reordered).files.last().unwrap().sha256
    );
}
#[test]
fn cycles_missing_targets_and_duplicate_aliases_never_widen_to_any() {
    let docs = docs();
    for text in [
        "---@alias ExternalId ExternalId\n",
        "---@alias ExternalId Other\n---@alias Other ExternalId\n",
        "---@alias ExternalId Missing\n---@alias Other ExternalId\n",
        "---@alias ExternalId string\n---@alias ExternalId string\n---@alias Other ExternalId\n",
    ] {
        let catalog = aliases(text);
        let result = projected(&docs, &catalog);
        assert_eq!(result.projection, "partial");
        assert_eq!(result.files.len(), 1);
        assert!(
            result
                .aliases
                .as_ref()
                .unwrap()
                .outcomes
                .iter()
                .all(|o| o.status != "emitted")
        );
        assert!(!result.files[0].text.contains(" any"));
        assert!(result.files[0].text.contains("---@param value ExternalId"));
    }
}
#[test]
fn source_declarations_and_reserved_names_cannot_be_overwritten_by_aliases() {
    let docs = vec![doc(
        r#"APIDocumentation:AddDocumentationTable({Name="ObjectAPI",Type="ScriptObject",Tables={{Name="Struct",Type="Structure",Fields={}}},Events={{Name="Ready",Type="Event",LiteralName="READY"}}})"#,
    )];
    let catalog = aliases(
        "---@alias ObjectAPI number\n---@alias Struct string\n---@alias FrameEvent string\n---@alias Enum string\n---@alias Constants string\n---@alias Good ObjectAPI|Struct\n",
    );
    let result = projected(&docs, &catalog);
    let report = result.aliases.as_ref().unwrap();
    assert_eq!(
        report
            .outcomes
            .iter()
            .filter(|o| o.status == "source_name_conflict")
            .count(),
        5
    );
    assert_eq!(report.outcomes.last().unwrap().status, "emitted");
    assert!(
        result
            .files
            .last()
            .unwrap()
            .text
            .contains("---@alias Good ObjectAPI|Struct")
    );
}
#[test]
fn aliases_cannot_use_failed_source_declarations_as_resolved_dependencies() {
    let docs = vec![doc(
        r#"APIDocumentation:AddDocumentationTable({Name="ObjectAPI",Type="ScriptObject",Tables={{Name="BadType",Type="Future"}}})"#,
    )];
    let catalog = aliases("---@alias ExternalId BadType\n---@alias BadType string\n");
    let result = projected(&docs, &catalog);
    let report = result.aliases.unwrap();
    assert_eq!(report.outcomes[0].status, "unresolved_alias_target");
    assert_eq!(report.outcomes[1].status, "source_name_conflict");
}
#[test]
fn aliases_cannot_treat_namespace_values_as_types() {
    let docs = vec![doc(
        r#"APIDocumentation:AddDocumentationTable({Name="System",Type="System",Namespace="C_Example",Functions={{Name="Do",Type="Function"}}})"#,
    )];
    let catalog = aliases("---@alias C_Example table\n---@alias Other C_Example\n");
    let result = projected(&docs, &catalog);
    assert!(
        result
            .aliases
            .unwrap()
            .outcomes
            .iter()
            .all(|o| o.status != "emitted")
    );
}
#[test]
fn independent_valid_alias_survives_unsupported_types_and_primitive_alias_names() {
    let docs = docs();
    let catalog = aliases(
        "---@alias ExternalId string\n---@alias Other fun():string\n---@alias Array number[]\n---@alias any string\n",
    );
    let result = projected(&docs, &catalog);
    assert_eq!(result.projection, "partial");
    assert_eq!(
        result
            .aliases
            .unwrap()
            .outcomes
            .iter()
            .filter(|o| o.status == "emitted")
            .count(),
        1
    );
    assert!(
        result
            .files
            .last()
            .unwrap()
            .text
            .contains("---@alias ExternalId string")
    );
}
#[test]
fn explicit_any_is_preserved_only_when_present_in_the_resource() {
    let docs = docs();
    let catalog = aliases("---@alias ExternalId any\n");
    let result = projected(&docs, &catalog);
    assert!(
        result
            .files
            .last()
            .unwrap()
            .text
            .contains("---@alias ExternalId any")
    );
    assert_eq!(
        result.aliases.unwrap().source.aliases()[0]
            .terms
            .as_ref()
            .unwrap(),
        &["any"]
    );
}
#[test]
fn executable_input_and_directive_side_effects_are_rejected_before_output() {
    for text in [
        "---@alias ExternalId number\nrequire('payload')",
        "---@alias ExternalId number\nlocal data = {}",
        "---@alias ExternalId number\n---@diagnostic disable",
        "---@alias ExternalId number\n---@namespace Rogue",
        "---@alias ExternalId number\n---@class Rogue",
        "---@meta module\n---@alias ExternalId number",
        "--[[---@alias ExternalId number]]",
        "---@alias ExternalId\n---| 'one'\n",
    ] {
        assert!(
            ingest_aliases(
                DONOR,
                "Types.lua",
                text,
                &source_digest(text.as_bytes()),
                &AtomicBool::new(false)
            )
            .is_err(),
            "{text}"
        );
    }
}
#[test]
fn exact_identity_digest_limits_and_cancellation_are_checked() {
    let text = "---@alias ExternalId number\n";
    let hash = source_digest(text.as_bytes());
    assert_eq!(
        ingest_aliases("main", "Types.lua", text, &hash, &AtomicBool::new(false))
            .unwrap_err()
            .code,
        NativeErrorCode::InvalidIdentity
    );
    assert!(ingest_aliases(DONOR, "../Types.lua", text, &hash, &AtomicBool::new(false)).is_err());
    assert_eq!(
        ingest_aliases(
            DONOR,
            "Types.lua",
            text,
            "sha256:bad",
            &AtomicBool::new(false)
        )
        .unwrap_err()
        .code,
        NativeErrorCode::DigestMismatch
    );
    assert_eq!(
        ingest_aliases(DONOR, "Types.lua", text, &hash, &AtomicBool::new(true))
            .unwrap_err()
            .code,
        NativeErrorCode::Cancelled
    );
    let long = format!("---@alias Long {}number{}", "(".repeat(80), ")".repeat(80));
    assert_eq!(
        ingest_aliases(
            DONOR,
            "Types.lua",
            &long,
            &source_digest(long.as_bytes()),
            &AtomicBool::new(false)
        )
        .unwrap_err()
        .code,
        NativeErrorCode::Limit
    );
}

#[test]
fn malformed_source_cannot_be_repaired_by_an_external_catalog() {
    let docs = vec![doc(
        r#"APIDocumentation:AddDocumentationTable({Name="Broken",Namespace="C_Broken",Tables={{Name="MalformedType",Fields={}}}})"#,
    )];
    let catalog = aliases(
        "---@alias Broken table\n---@alias C_Broken table\n---@alias MalformedType table\n---@alias Other MalformedType\n",
    );
    let result = projected(&docs, &catalog);
    assert!(
        result
            .aliases
            .unwrap()
            .outcomes
            .iter()
            .all(|o| o.status != "emitted")
    );
    assert!(result.files.is_empty());
}
#[test]
fn enum_targets_use_only_actual_rendered_qualified_types() {
    let docs = vec![doc(
        r#"APIDocumentation:AddDocumentationTable({Tables={{Name="Choice",Type="Enumeration",Fields={{Name="One",Type="Choice",EnumValue=1}}}}})"#,
    )];
    let catalog =
        aliases("---@alias Good Enum.Choice\n---@alias Bad Choice\n---@alias Choice number\n");
    let result = projected(&docs, &catalog);
    let outcomes = &result.aliases.as_ref().unwrap().outcomes;
    assert_eq!(outcomes[0].status, "emitted");
    assert_eq!(outcomes[1].status, "unresolved_alias_target");
    assert_eq!(outcomes[2].status, "source_name_conflict");
}
#[test]
fn selected_environment_does_not_borrow_a_foreign_type() {
    let docs = vec![doc(
        r#"APIDocumentation:AddDocumentationTable({Name="OtherAPI",Type="ScriptObject",Environment="Other"}) APIDocumentation:AddDocumentationTable({Name="SelectedAPI",Type="ScriptObject",Environment="Mainline"})"#,
    )];
    let catalog = aliases("---@alias Available SelectedAPI\n---@alias Missing OtherAPI\n");
    let result = projected(&docs, &catalog);
    assert_eq!(
        result.aliases.as_ref().unwrap().outcomes[0].status,
        "emitted"
    );
    assert_eq!(
        result.aliases.as_ref().unwrap().outcomes[1].status,
        "unresolved_alias_target"
    );
}
#[test]
fn cancellation_of_optional_alias_path_returns_no_artifact() {
    let docs = docs();
    let catalog = aliases("---@alias ExternalId string\n");
    assert_eq!(
        project_with_alias_catalog(
            &docs,
            "Mainline",
            None,
            Some(&catalog),
            &AtomicBool::new(true)
        )
        .unwrap_err(),
        wow_annotations::ketho::RenderError::Cancelled
    );
}
#[test]
fn many_dependencies_are_iterative_and_outputs_do_not_expand() {
    let docs = docs();
    let mut input = String::from("---@alias A0 string\n");
    for i in 1..2000 {
        input.push_str(&format!("---@alias A{i} A{}\n", i - 1));
    }
    let catalog = aliases(&input);
    let result = projected(&docs, &catalog);
    assert!(result.issues.is_empty());
    assert_eq!(result.aliases.unwrap().outcomes.len(), 2000);
    assert!(
        result
            .files
            .last()
            .unwrap()
            .text
            .contains("---@alias A1999 A1998")
    );
}

#[test]
fn external_qualified_alias_cannot_extend_a_source_namespace_or_primitive() {
    let docs = vec![doc(
        r#"APIDocumentation:AddDocumentationTable({Name="Example",Type="System",Namespace="C_Example",Functions={{Name="Read"}}})"#,
    )];
    let catalog = aliases(
        "---@alias C_Example.Read number\n---@alias Enum.Unknown number\n---@alias string.Fake table\n---@alias ExternalId number\n",
    );
    let result = projected(&docs, &catalog);
    let outcomes = &result.aliases.as_ref().unwrap().outcomes;
    assert_eq!(outcomes.len(), 4);
    assert!(
        outcomes[..3]
            .iter()
            .all(|v| v.status == "source_name_conflict")
    );
    assert_eq!(outcomes[3].status, "emitted");
    assert!(
        !result
            .files
            .last()
            .unwrap()
            .text
            .contains("---@alias C_Example")
    );
}

#[test]
fn ambiguous_doc_suffix_is_local_syntax_failure_not_type_repair() {
    let raw =
        "---@alias Before number\n---@alias Broken number [0.0 - 1.0]\n---@alias After string\n";
    let catalog = aliases(raw);
    assert_eq!(catalog.aliases().len(), 3);
    assert!(catalog.aliases()[1].syntax_error);
    assert!(catalog.aliases()[1].terms.is_none());
    let docs = docs();
    let result = projected(&docs, &catalog);
    let report = result.aliases.as_ref().unwrap();
    assert_eq!(report.outcomes[0].status, "emitted");
    assert_eq!(report.outcomes[1].status, "alias_syntax_error");
    assert_eq!(report.outcomes[2].status, "emitted");
    assert_eq!(result.projection, "partial");
    assert!(
        !result
            .files
            .last()
            .unwrap()
            .text
            .contains("---@alias Broken")
    );
    assert!(
        result
            .files
            .last()
            .unwrap()
            .text
            .contains("---@alias Before number")
    );
    let span = catalog.aliases()[1].span;
    assert_eq!(
        &raw[span.start..span.end],
        "---@alias Broken number [0.0 - 1.0]"
    );
}
