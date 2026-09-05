#![allow(clippy::unwrap_used)]
use std::sync::atomic::AtomicBool;
use wow_annotations::{ketho::RenderError, native::project};
use wow_reference::native::{DocumentationDocument, ingest_document, source_digest};
const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
fn doc(path: &str, text: &str) -> DocumentationDocument {
    ingest_document(
        REV,
        path,
        text,
        &source_digest(text.as_bytes()),
        &AtomicBool::new(false),
    )
    .unwrap()
}
const SOURCE: &str = r#"local a={Name="Example",Type="System",Namespace="C_Example",Environment="All",
Functions={{Name="Read",Type="Function",SecretReturns=true,Documentation={"Read."},Arguments={{Name="enabled",Type="bool",Nilable=false,Default=false}},Returns={{Name="mode",Type="Mode"},{Name="count",Type="number"}}}},
Events={{Name="Changed",LiteralName="EXAMPLE_CHANGED",Payload={{Name="value",Type="number"}}}},
Tables={{Name="Mode",Type="Enumeration",Fields={{Name="Off",EnumValue=0},{Name="On",EnumValue=0x10}}},{Name="Record",Type="Structure",Fields={{Name="flag",Type="bool",Nilable=true}}}},
FutureField={X=nil,Predicate=Enum.Secrets.Test}}
APIDocumentation:AddDocumentationTable(a)"#;
#[test]
fn lua_to_library_is_entirely_native_and_retains_raw_metadata() {
    let docs = [doc("API.lua", SOURCE)];
    let library = project(&docs, "Mainline", &AtomicBool::new(false)).unwrap();
    assert!(library.issues.is_empty(), "{:?}", library.issues);
    assert!(!library.negative_authority);
    let text = library
        .files
        .iter()
        .map(|f| f.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("---@param enabled? boolean Default = false"));
    assert!(text.contains("---@return Enum.Mode mode"));
    assert!(text.contains("function C_Example.Read(enabled) end"));
    assert!(text.contains("---@enum Enum.Mode"));
    assert!(text.contains("On = 16"));
    assert!(text.contains("EXAMPLE_CHANGED\" # `value`"));
    assert!(text.contains("---@class Record"));
    let report = serde_json::to_string(&library).unwrap();
    assert!(report.contains("SecretReturns"));
    assert!(report.contains("FutureField"));
    for file in &library.files {
        assert_eq!(file.sha256, source_digest(file.text.as_bytes()));
        for map in &file.mappings {
            assert!(
                file.text
                    .get(map.generated.start..map.generated.end)
                    .is_some()
            );
            assert!(
                SOURCE
                    .get(map.source.span.start..map.source.span.end)
                    .is_some()
            );
        }
    }
}
#[test]
fn methods_with_same_name_on_distinct_objects_stay_distinct() {
    let docs = [
        doc(
            "A.lua",
            r#"local a={Name="ObjectA",Type="ScriptObject",Functions={{Name="Show"}}} APIDocumentation:AddDocumentationTable(a)"#,
        ),
        doc(
            "B.lua",
            r#"local a={Name="ObjectB",Type="ScriptObject",Functions={{Name="Show"}}} APIDocumentation:AddDocumentationTable(a)"#,
        ),
    ];
    let library = project(&docs, "Mainline", &AtomicBool::new(false)).unwrap();
    assert!(library.issues.is_empty());
    assert!(library.files[0].text.contains("ObjectA:Show()"));
    assert!(library.files[1].text.contains("ObjectB:Show()"));
}
#[test]
fn duplicates_never_become_first_or_last_wins() {
    let docs = [doc("A.lua", SOURCE), doc("B.lua", SOURCE)];
    let library = project(&docs, "Mainline", &AtomicBool::new(false)).unwrap();
    assert!(library.files.is_empty());
    assert!(!library.issues.is_empty());
    assert_eq!(library.projection, "partial");
}
#[test]
fn unsupported_metadata_is_not_lost_with_the_raw_source() {
    let docs = [doc(
        "A.lua",
        r#"APIDocumentation:AddDocumentationTable({Tables={{Name="CB",Type="CallbackType",Returns={{Name="ok",Type="bool"}}},{Name="Wide",Type="Enumeration",Fields={{Name="TooWide",EnumValue=9223372036854775807}}},{Name="New",Type="FutureType"}}})"#,
    )];
    let library = project(&docs, "Mainline", &AtomicBool::new(false)).unwrap();
    assert!(library.files.is_empty());
    assert!(library.issues.len() >= 3);
    assert_eq!(library.sources.len(), 1);
}
#[test]
fn cross_generation_duplicate_paths_and_cancel_reject_globally() {
    let a = doc("A.lua", SOURCE);
    let b = ingest_document(
        &"b".repeat(40),
        "B.lua",
        SOURCE,
        &source_digest(SOURCE.as_bytes()),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(
        project(&[a.clone(), b], "Mainline", &AtomicBool::new(false)).unwrap_err(),
        RenderError::InvalidSource
    );
    assert_eq!(
        project(&[a.clone(), a.clone()], "Mainline", &AtomicBool::new(false)).unwrap_err(),
        RenderError::InvalidSource
    );
    assert_eq!(
        project(&[a], "Mainline", &AtomicBool::new(true)).unwrap_err(),
        RenderError::Cancelled
    );
}
#[test]
fn corpus_order_does_not_change_artifacts() {
    let a = doc("A.lua", SOURCE);
    let b = doc(
        "B.lua",
        r#"local a={Name="Other",Type="System",Functions={{Name="GlobalRead"}}} APIDocumentation:AddDocumentationTable(a)"#,
    );
    let left = [a.clone(), b.clone()];
    let right = [b, a];
    assert_eq!(
        serde_json::to_string(&project(&left, "Mainline", &AtomicBool::new(false)).unwrap())
            .unwrap(),
        serde_json::to_string(&project(&right, "Mainline", &AtomicBool::new(false)).unwrap())
            .unwrap()
    );
}
#[test]
fn environment_selection_never_invents_availability() {
    let docs = [doc(
        "A.lua",
        &SOURCE.replace("Environment=\"All\"", "Environment=\"Glue\""),
    )];
    let library = project(&docs, "Mainline", &AtomicBool::new(false)).unwrap();
    assert!(library.files.is_empty());
    assert_eq!(library.issues[0].code, "environment_not_selected");
}
#[test]
fn generated_library_has_one_literal_root_and_parses_in_emmy_frontend() {
    let docs = [
        doc("A.lua", SOURCE),
        doc(
            "B.lua",
            r#"local a={Name="Other",Type="System",Events={{Name="Other",LiteralName="OTHER"}},Tables={{Name="Second",Type="Enumeration",Fields={{Name="One",EnumValue=1}}}}} APIDocumentation:AddDocumentationTable(a)"#,
        ),
    ];
    let library = project(&docs, "Mainline", &AtomicBool::new(false)).unwrap();
    assert!(library.issues.is_empty());
    assert_eq!(
        library
            .files
            .iter()
            .filter(|f| f.text.contains("---@alias FrameEvent"))
            .count(),
        1
    );
    assert_eq!(
        library
            .files
            .iter()
            .filter(|f| f.text.contains("Enum = {}"))
            .count(),
        1
    );
    for file in &library.files {
        let tree = emmylua_parser::LuaParser::parse(
            &file.text,
            emmylua_parser::ParserConfig::with_level(emmylua_parser::LuaLanguageLevel::Lua51),
        );
        assert!(
            tree.get_errors().is_empty(),
            "{} {:?}",
            file.path,
            tree.get_errors()
        );
    }
}
#[test]
fn exact_nonconflicted_enum_defaults_are_resolved_without_execution() {
    let source = SOURCE.replace("Default=false", "Default=Enum.Mode.On");
    let docs = [doc("A.lua", &source)];
    let library = project(&docs, "Mainline", &AtomicBool::new(false)).unwrap();
    assert!(library.issues.is_empty());
    assert!(library.files[0].text.contains("Default = 0x10"));
    let missing = source.replace("Enum.Mode.On", "Enum.Mode.Unknown");
    let docs = [doc("A.lua", &missing)];
    let library = project(&docs, "Mainline", &AtomicBool::new(false)).unwrap();
    assert!(
        library
            .issues
            .iter()
            .any(|i| i.code.starts_with("callable_"))
    );
}
