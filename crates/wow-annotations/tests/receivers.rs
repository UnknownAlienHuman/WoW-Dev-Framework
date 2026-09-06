//! Ketho-style local widget declarations joined to the native source lane.
//! No inheritance is inferred and no generated Lua is executed.
#![allow(clippy::unwrap_used)]
use std::collections::BTreeSet;
use std::sync::atomic::AtomicBool;
use wow_annotations::ketho::{Function, Owner, RenderError, Renderer, System, Table};
use wow_annotations::native::{project, project_with_corrections};
use wow_reference::native::{DocumentationDocument, ingest_document, source_digest};
use wow_reference::native_corrections::{
    Correction, CorrectionSet, Evidence, NORMALIZER, Projection, SCHEMA, Target,
    ValidatedCorrections, Value, raw_digest,
};

const REV: &str = "cccccccccccccccccccccccccccccccccccccccc";
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
fn object(name: &str, methods: &str) -> String {
    format!(
        r#"APIDocumentation:AddDocumentationTable({{Name="{name}",Type="ScriptObject",Functions={{{methods}}}}})"#
    )
}
fn system(name: &str, alias: Option<&str>) -> System {
    System {
        owner: Owner::ScriptObject {
            system_name: name.into(),
            annotation_name: alias.map(Into::into),
        },
        functions: vec![Function {
            name: "Show".into(),
            documentation: None,
            arguments: vec![],
            returns: vec![],
        }],
        tables: vec![],
    }
}
fn renderer() -> Renderer {
    Renderer::new(BTreeSet::new(), 1024 * 1024).unwrap()
}
fn syntax(text: &str) {
    let tree = emmylua_parser::LuaParser::parse(
        text,
        emmylua_parser::ParserConfig::with_level(emmylua_parser::LuaLanguageLevel::Lua51),
    );
    assert!(tree.get_errors().is_empty(), "{:?}", tree.get_errors());
}
fn rename(document: &DocumentationDocument, from: &str, to: &str) -> Correction {
    Correction {
        id: document.path().into(),
        target: Target {
            path: document.path().into(),
            registration: 0,
            projection: Projection::WidgetOwner,
        },
        expected_source_sha256: document.sha256().into(),
        expected_raw_sha256: raw_digest(&document.registrations()[0].value).unwrap(),
        before: Value::Text(from.into()),
        after: Value::Text(to.into()),
        reviewer: "synthetic-review".into(),
        rationale: "Synthetic receiver projection, not platform evidence".into(),
        evidence: vec![Evidence {
            revision: REV.into(),
            path: document.path().into(),
            sha256: document.sha256().into(),
        }],
    }
}
fn pack(records: Vec<Correction>) -> ValidatedCorrections {
    ValidatedCorrections::new(CorrectionSet {
        schema: SCHEMA.into(),
        version: 1,
        revision: REV.into(),
        environment: "Mainline".into(),
        normalizer: NORMALIZER.into(),
        records,
    })
    .unwrap()
}

#[test]
fn local_class_binding_precedes_methods_with_exact_mapped_ranges() {
    let rendered = renderer()
        .render_library_mapped(&system("FutureObjectAPI", None))
        .unwrap();
    assert_eq!(
        rendered.text,
        "---@meta _\n---@class FutureObjectAPI\nlocal FutureObjectAPI = {}\n\n---[Documentation](https://warcraft.wiki.gg/wiki/API_FutureObjectAPI_Show)\nfunction FutureObjectAPI:Show() end"
    );
    let receiver = rendered.receiver.unwrap();
    assert_eq!(
        &rendered.text[receiver.start..receiver.end],
        "---@class FutureObjectAPI\nlocal FutureObjectAPI = {}"
    );
    assert_eq!(rendered.declarations.len(), 1);
    assert!(
        rendered.text[rendered.declarations[0].start..rendered.declarations[0].end]
            .ends_with("function FutureObjectAPI:Show() end")
    );
    assert!(
        !rendered
            .text
            .lines()
            .any(|line| line == "FutureObjectAPI = {}")
    );
    syntax(&rendered.text);
}
#[test]
fn exact_receiver_rename_keeps_a_type_alias_but_not_a_global() {
    let rendered = renderer()
        .render_library_mapped(&system("OriginalAPI", Some("SelectedObject")))
        .unwrap();
    assert!(rendered.text.contains(
        "---@class SelectedObject\nlocal SelectedObject = {}\n---@alias OriginalAPI SelectedObject"
    ));
    assert!(rendered.text.contains("function SelectedObject:Show() end"));
    assert!(!rendered.text.contains("local OriginalAPI"));
    assert!(!rendered.text.contains("---@class OriginalAPI"));
    syntax(&rendered.text);
}
#[test]
fn same_name_projection_has_no_redundant_alias() {
    let rendered = renderer()
        .render_library_mapped(&system("Identical", Some("Identical")))
        .unwrap();
    assert!(!rendered.text.contains("---@alias"));
}
#[test]
fn empty_script_object_still_declares_its_type() {
    let mut input = system("EmptyAPI", None);
    input.functions.clear();
    let rendered = renderer().render_library_mapped(&input).unwrap();
    assert_eq!(
        rendered.text,
        "---@meta _\n---@class EmptyAPI\nlocal EmptyAPI = {}"
    );
    assert!(rendered.receiver.is_some());
    assert!(rendered.declarations.is_empty());
}
#[test]
fn non_object_and_donor_profiles_are_unchanged() {
    for owner in [Owner::Global, Owner::Namespace("C_Synthetic".into())] {
        let mut input = system("Unused", None);
        input.owner = owner;
        assert_eq!(
            renderer().render_library_mapped(&input).unwrap(),
            renderer().render_mapped(&input).unwrap()
        );
    }
    let input = system("ExactNameAPI", None);
    let legacy = renderer().render_mapped(&input).unwrap();
    assert!(legacy.receiver.is_none());
    assert!(!legacy.text.contains("---@class"));
}
#[test]
fn invalid_primitive_and_colliding_receiver_names_reject() {
    for name in [
        "end",
        "Injected\n---@class Evil",
        "Name.With.Dot",
        "string",
        "any",
        "bool",
        "cstring",
        "luaIndex",
    ] {
        assert!(
            renderer()
                .render_library_mapped(&system(name, None))
                .is_err()
        );
    }
    for name in ["Original", "Alias"] {
        let mut input = system("Original", Some("Alias"));
        input.tables.push(Table::Structure {
            name: name.into(),
            fields: vec![],
        });
        assert_eq!(
            renderer().render_library_mapped(&input),
            Err(RenderError::DuplicateName)
        );
    }
}
#[test]
fn receiver_bytes_and_count_are_inside_renderer_budgets() {
    let input = system("Bounded", None);
    let size = renderer().render_library_mapped(&input).unwrap().text.len();
    assert!(
        Renderer::new(BTreeSet::new(), size)
            .unwrap()
            .render_library_mapped(&input)
            .is_ok()
    );
    assert_eq!(
        Renderer::new(BTreeSet::new(), size - 1)
            .unwrap()
            .render_library_mapped(&input),
        Err(RenderError::OutputLimit)
    );
    let mut input = system("Many", None);
    input.functions = (0..4096)
        .map(|i| Function {
            name: format!("F{i}"),
            documentation: None,
            arguments: vec![],
            returns: vec![],
        })
        .collect();
    assert_eq!(
        renderer().render_library_mapped(&input),
        Err(RenderError::InputLimit)
    );
}
#[test]
fn native_source_links_cover_class_and_methods_independently() {
    let raw = object("ExactObject", r#"{Name="Show"}"#);
    let docs = [doc("DifferentFilename.lua", &raw)];
    let library = project(&docs, "Mainline", &AtomicBool::new(false)).unwrap();
    assert!(library.issues.is_empty());
    assert!(!library.negative_authority);
    let file = &library.files[0];
    assert_eq!(file.mappings.len(), 2);
    let class = &file.mappings[0];
    assert_eq!(class.granularity, "declaration");
    assert_eq!(class.source.sha256, docs[0].sha256());
    assert_eq!(class.source.span, docs[0].registrations()[0].value.span);
    assert_eq!(
        &file.text[class.generated.start..class.generated.end],
        "---@class ExactObject\nlocal ExactObject = {}"
    );
    let method = &file.mappings[1];
    assert_eq!(
        &raw[method.source.span.start..method.source.span.end],
        r#"{Name="Show"}"#
    );
    assert!(method.generated.start >= class.generated.end);
    syntax(&file.text);
}
#[test]
fn reviewed_rename_connects_class_alias_methods_and_preserves_raw_source() {
    let raw = object(
        "OriginalAPI",
        r#"{Name="Clone",Returns={{Name="value",Type="OriginalAPI"}}}"#,
    );
    let docs = [doc("A.lua", &raw)];
    let original = docs.clone();
    let corrections = pack(vec![rename(&docs[0], "OriginalAPI", "Object")]);
    let library = project_with_corrections(
        &docs,
        "Mainline",
        Some(&corrections),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert!(library.issues.is_empty());
    let text = &library.files[0].text;
    assert!(text.contains("---@class Object\nlocal Object = {}\n---@alias OriginalAPI Object"));
    assert!(text.contains("---@return OriginalAPI value\nfunction Object:Clone() end"));
    assert_eq!(docs, original);
    syntax(text);
}
#[test]
fn disjoint_methods_do_not_merge_duplicate_source_owners() {
    let docs = [
        doc("A.lua", &object("Same", r#"{Name="First"}"#)),
        doc("B.lua", &object("Same", r#"{Name="Second"}"#)),
    ];
    let library = project(&docs, "Mainline", &AtomicBool::new(false)).unwrap();
    assert!(library.files.is_empty());
    assert_eq!(
        library
            .issues
            .iter()
            .filter(|i| i.code == "script_object_name_conflict")
            .count(),
        2
    );
    assert_eq!(library.projection, "partial");
}
#[test]
fn owner_conflict_leaves_independent_structures_and_events() {
    let raw = r#"APIDocumentation:AddDocumentationTable({Name="Collision",Type="ScriptObject",Functions={{Name="Hidden"}},Tables={{Name="Collision",Type="Structure",Fields={{Name="ok",Type="bool"}}}},Events={{Name="Changed",LiteralName="CHANGED"}}})"#;
    let docs = [doc("A.lua", raw)];
    let library = project(&docs, "Mainline", &AtomicBool::new(false)).unwrap();
    let text = library
        .files
        .iter()
        .map(|f| f.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("---@class Collision\n---@field ok boolean"));
    assert!(text.contains("---@alias FrameEvent"));
    assert!(!text.contains("local Collision"));
    assert!(!text.contains("Hidden"));
    assert_eq!(library.issues[0].code, "script_object_name_conflict");
}
#[test]
fn receiver_conflicts_with_namespaces_globals_enum_types_and_primitive_types() {
    for other in [
        r#"{Name="Other",Type="System",Namespace="Collision",Functions={{Name="Query"}}}"#,
        r#"{Name="Other",Type="System",Functions={{Name="Collision"}}}"#,
        r#"{Tables={{Name="Collision",Type="Enumeration",Fields={{Name="One",EnumValue=1}}}}}"#,
    ] {
        let docs = [
            doc("A.lua", &object("Collision", r#"{Name="Hidden"}"#)),
            doc(
                "B.lua",
                &format!("APIDocumentation:AddDocumentationTable({other})"),
            ),
        ];
        let library = project(&docs, "Mainline", &AtomicBool::new(false)).unwrap();
        assert!(
            library
                .issues
                .iter()
                .any(|i| i.code == "script_object_name_conflict")
        );
        assert!(
            !library
                .files
                .iter()
                .any(|f| f.text.contains("Collision:Hidden"))
        );
    }
    let docs = [doc("A.lua", &object("string", r#"{Name="Hidden"}"#))];
    assert!(
        project(&docs, "Mainline", &AtomicBool::new(false))
            .unwrap()
            .files
            .is_empty()
    );
}
#[test]
fn chained_receiver_alias_collision_is_not_first_or_last_wins() {
    let docs = [
        doc("A.lua", &object("A", r#"{Name="First"}"#)),
        doc("B.lua", &object("B", r#"{Name="Second"}"#)),
    ];
    let corrections = pack(vec![rename(&docs[0], "A", "B"), rename(&docs[1], "B", "C")]);
    let library = project_with_corrections(
        &docs,
        "Mainline",
        Some(&corrections),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(library.projection, "partial");
    assert!(library.files.is_empty());
    assert_eq!(
        library
            .issues
            .iter()
            .filter(|i| i.code == "script_object_name_conflict")
            .count(),
        2
    );
}
#[test]
fn excluded_environment_does_not_create_class_conflicts() {
    let docs = [
        doc("A.lua", &object("Same", r#"{Name="First"}"#)),
        doc(
            "B.lua",
            &object("Same", r#"{Name="Second"}"#).replace(
                "Type=\"ScriptObject\"",
                "Type=\"ScriptObject\",Environment=\"Glue\"",
            ),
        ),
    ];
    let library = project(&docs, "Mainline", &AtomicBool::new(false)).unwrap();
    assert_eq!(library.files.len(), 1);
    assert!(library.files[0].text.contains("local Same = {}"));
    assert_eq!(library.issues[0].code, "environment_not_selected");
}
#[test]
fn names_do_not_infer_aliases_inheritance_or_runtime_globals() {
    let raw = object(
        "SimpleUnfamiliarAPI",
        r#"{Name="Read",Returns={{Name="value",Type="UnresolvedFuture"}}}"#,
    );
    let docs = [doc("LooksLikeFrame.lua", &raw)];
    let library = project(&docs, "Mainline", &AtomicBool::new(false)).unwrap();
    let text = &library.files[0].text;
    assert!(text.contains("---@class SimpleUnfamiliarAPI\nlocal SimpleUnfamiliarAPI = {}"));
    assert!(!text.contains("---@alias"));
    assert!(!text.contains(" : Frame"));
    assert!(text.contains("---@return UnresolvedFuture value"));
    assert!(!text.contains("---@return any"));
}
#[test]
fn object_types_survive_zero_methods_or_rejected_methods_without_fake_members() {
    for methods in [
        "",
        r#"{Name="Broken",Arguments={{Name="end",Type="number"}}}"#,
    ] {
        let docs = [doc("A.lua", &object("EmptyAPI", methods))];
        let library = project(&docs, "Mainline", &AtomicBool::new(false)).unwrap();
        assert_eq!(library.files.len(), 1);
        assert_eq!(
            library.files[0].text,
            "---@meta _\n---@class EmptyAPI\nlocal EmptyAPI = {}"
        );
        assert_eq!(library.files[0].mappings.len(), 1);
        assert_eq!(
            library.projection,
            if methods.is_empty() {
                "projected_with_sidecars"
            } else {
                "partial"
            }
        );
    }
}
#[test]
fn class_output_is_deterministic_under_corpus_order() {
    let docs = [
        doc("A.lua", &object("First", r#"{Name="Show"}"#)),
        doc("B.lua", &object("Second", r#"{Name="Show"}"#)),
    ];
    let reversed = [docs[1].clone(), docs[0].clone()];
    let render = |docs: &[DocumentationDocument]| {
        serde_json::to_vec(&project(docs, "Mainline", &AtomicBool::new(false)).unwrap()).unwrap()
    };
    assert_eq!(render(&docs), render(&reversed));
}

#[test]
fn unreviewed_inheritance_is_metadata_not_a_guessed_class_relation() {
    let raw = object("ChildAPI", r#"{Name="Show"}"#).replace(
        "Type=\"ScriptObject\"",
        "Type=\"ScriptObject\",Inherits=\"ParentAPI\"",
    );
    let docs = [doc("A.lua", &raw)];
    let library = project(&docs, "Mainline", &AtomicBool::new(false)).unwrap();
    assert!(library.files[0].text.contains("---@class ChildAPI\n"));
    assert!(!library.files[0].text.contains("---@class ChildAPI:"));
    assert!(!library.files[0].text.contains("ParentAPI"));
    assert!(
        library
            .metadata_sidecars
            .iter()
            .any(|record| record.field == "Inherits")
    );
    assert!(!library.negative_authority);
}
