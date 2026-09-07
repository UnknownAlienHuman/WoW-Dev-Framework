//! Native callbacks retain ordered signatures instead of being omitted.
use emmylua_parser::{LuaAstNode, LuaDocFuncType, LuaLanguageLevel, LuaParser, ParserConfig};
use std::{collections::BTreeSet, sync::atomic::AtomicBool};
use wow_annotations::{
    ketho::{Field, Owner, RenderError, Renderer, System, Table},
    native::project,
};
use wow_reference::native::{DocumentationDocument, ingest_document, source_digest};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn document(text: &str) -> Result<DocumentationDocument> {
    Ok(ingest_document(
        REV,
        "Callbacks.lua",
        text,
        &source_digest(text.as_bytes()),
        &AtomicBool::new(false),
    )?)
}

#[test]
fn native_arrays_returns_and_argument_packs_preserve_order_and_source_maps() -> Result<()> {
    let raw = r#"APIDocumentation:AddDocumentationTable({Tables={
{Name="Transform",Type="CallbackType",Arguments={{Name="values",Type="table",InnerType="cstring"},{Name="enabled",Type="bool",Default=false}},Returns={{Name="ok",Type="bool"},{Name="values",Type="table",InnerType="number",Nilable=true}}},
{Name="Visit",Type="CallbackType",Arguments={{Name="first",Type="number"},{Name="rest",Type="cstring",StrideIndex=1}}},
{Name="Original",Type="CallbackType",Arguments={{Name="value",Type="bool"}}}
}})"#;
    let docs = [document(raw)?];
    let library = project(&docs, "Mainline", &AtomicBool::new(false))?;
    assert_eq!(library.projection, "projected_with_sidecars");
    assert!(library.issues.is_empty());
    let file = library.files.first().ok_or("callback file")?;
    for expected in [
        "---@alias Transform FunctionContainer|fun(values: string[], enabled?: boolean): (boolean, number[]?)",
        "---@alias Visit FunctionContainer|fun(first: number, ...: string)",
        "---@alias Original FunctionContainer|fun(value: boolean)",
    ] {
        assert!(file.text.contains(expected), "{expected}");
        let map = file
            .mappings
            .iter()
            .find(|m| file.text.get(m.generated.start..m.generated.end) == Some(expected))
            .ok_or("callback mapping")?;
        assert_eq!(map.source.path, "Callbacks.lua");
        assert_eq!(map.source.sha256, source_digest(raw.as_bytes()));
        assert!(raw.get(map.source.span.start..map.source.span.end).is_some());
    }
    let tree = LuaParser::parse(&file.text, ParserConfig::with_level(LuaLanguageLevel::Lua51));
    assert!(tree.get_errors().is_empty(), "{:?}", tree.get_errors());
    let functions = tree
        .get_chunk_node()
        .syntax()
        .descendants()
        .filter_map(LuaDocFuncType::cast)
        .collect::<Vec<_>>();
    assert_eq!(functions.len(), 3);
    assert_eq!(functions[0].get_params().count(), 2);
    assert_eq!(
        functions[0]
            .get_return_type_list()
            .ok_or("returns")?
            .get_return_type_list()
            .count(),
        2
    );
    assert_eq!(functions[1].get_params().count(), 2);
    assert_eq!(file.sha256, source_digest(file.text.as_bytes()));
    assert!(!library.negative_authority);
    Ok(())
}

#[test]
fn unrepresentable_callback_stays_partial_without_erasing_valid_neighbor() -> Result<()> {
    for fields in [
        "Arguments={{Name='rest',Type='string',StrideIndex=1},{Name='last',Type='number'}}",
        "Returns={{Name='rest',Type='string',StrideIndex=1}}",
        "Arguments={{Name='rest',Type='string',StrideIndex=1,Default='invalid'}}",
        "Returns={{Name='same',Type='number'},{Name='same',Type='bool'}}",
        "Returns={{Name='value',Type='fun()'}}",
    ] {
        let raw = [
            "APIDocumentation:AddDocumentationTable({Tables={{Name='Bad',Type='CallbackType',",
            fields,
            "},{Name='Good',Type='CallbackType',Returns={{Name='ok',Type='bool'}}}}})",
        ]
        .concat();
        let docs = [document(&raw)?];
        let result = project(&docs, "Mainline", &AtomicBool::new(false))?;
        assert_eq!(result.projection, "partial", "{fields}");
        assert_eq!(result.issues.len(), 1, "{fields}");
        let file = result.files.first().ok_or("valid neighbor")?;
        assert!(file.text.contains("---@alias Good FunctionContainer|fun(): (boolean)"));
        assert!(!file.text.contains("---@alias Bad"));
    }
    Ok(())
}

fn field(name: &str, ty: &str) -> Field {
    Field {
        name: name.into(),
        type_name: ty.into(),
        inner_type: None,
        nilable: false,
        default_text: None,
        variadic: false,
    }
}

#[test]
fn function_type_return_tuple_keeps_union_array_and_optional_precedence() -> Result<()> {
    let mut value = field("result", "number|string");
    value.nilable = true;
    let mut array = field("items", "table");
    array.inner_type = Some("number|string".into());
    let system = System {
        owner: Owner::Global,
        functions: vec![],
        tables: vec![Table::CallbackSignature {
            name: "Choice".into(),
            arguments: vec![],
            returns: vec![value, array],
        }],
    };
    let renderer = Renderer::new(BTreeSet::new(), 4096)?;
    let text = renderer.render(&system)?;
    assert!(text.contains("fun(): ((number|string)?, (number|string)[])"));
    let tree = LuaParser::parse(&text, ParserConfig::with_level(LuaLanguageLevel::Lua51));
    assert!(tree.get_errors().is_empty(), "{:?}", tree.get_errors());
    let fun = tree
        .get_chunk_node()
        .syntax()
        .descendants()
        .find_map(LuaDocFuncType::cast)
        .ok_or("function type")?;
    assert_eq!(
        fun.get_return_type_list()
            .ok_or("returns")?
            .get_return_type_list()
            .count(),
        2
    );
    assert_eq!(
        Renderer::new(BTreeSet::new(), text.len() - 1)?.render(&system),
        Err(RenderError::OutputLimit)
    );
    assert_eq!(Renderer::new(BTreeSet::new(), text.len())?.render(&system)?, text);
    Ok(())
}

#[test]
fn legacy_callback_profile_is_not_silently_reinterpreted() -> Result<()> {
    let mut array = field("values", "table");
    array.inner_type = Some("number".into());
    let mut system = System {
        owner: Owner::Global,
        functions: vec![],
        tables: vec![Table::Callback {
            name: "Legacy".into(),
            arguments: vec![array.clone()],
        }],
    };
    let renderer = Renderer::new(BTreeSet::new(), 4096)?;
    assert_eq!(renderer.render(&system), Err(RenderError::UnsupportedType));
    system.tables = vec![Table::CallbackSignature {
        name: "Native".into(),
        arguments: vec![array],
        returns: vec![],
    }];
    assert!(renderer.render(&system)?.contains("fun(values: number[])"));
    Ok(())
}
