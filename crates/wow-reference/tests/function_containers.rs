//! Static FunctionContainer resources are syntax data, never executed Lua.
use std::sync::atomic::AtomicBool;
use wow_reference::native::source_digest;
use wow_reference::native_aliases::ingest_alias_catalog;

const REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn ingest(text: &str) -> wow_reference::native_aliases::AliasDocument {
    ingest_alias_catalog(
        REVISION,
        "FunctionContainer.lua",
        text,
        &source_digest(text.as_bytes()),
        &AtomicBool::new(false),
    )
    .expect("valid function container")
}

#[test]
fn class_empty_binding_methods_and_returns_keep_exact_source_spans() {
    let raw = "---@meta _\n\n---@class FunctionContainer\nlocal FunctionContainer = {}\n\nfunction FunctionContainer:Cancel() end\n\n---@return boolean\nfunction FunctionContainer:IsCancelled() end\n\nfunction FunctionContainer:Invoke() end\n";
    let document = ingest(raw);
    assert!(document.aliases().is_empty());
    assert!(document.structures().is_empty());
    assert!(document.namespaces().is_empty());
    let containers = document.function_containers();
    assert_eq!(containers.len(), 1);
    let container = &containers[0];
    assert_eq!(container.name, "FunctionContainer");
    assert_eq!(container.methods.len(), 3);
    assert_eq!(container.methods[0].name, "Cancel");
    assert!(container.methods[0].returns.is_empty());
    assert_eq!(container.methods[1].name, "IsCancelled");
    assert_eq!(container.methods[1].returns[0].terms, ["boolean"]);
    assert_eq!(container.methods[2].name, "Invoke");
    assert_eq!(
        raw.get(container.header_span.start..container.header_span.end),
        Some("---@class FunctionContainer\nlocal FunctionContainer = {}")
    );
    assert_eq!(
        raw.get(container.methods[1].span.start..container.methods[1].span.end),
        Some("---@return boolean\nfunction FunctionContainer:IsCancelled() end")
    );
    let value = serde_json::to_value(document).expect("serialize resource");
    assert_eq!(value["schema"], "wow-native-alias-resource/6");
}

#[test]
fn executable_or_ambiguous_static_class_shapes_are_rejected() {
    for raw in [
        "---@class FunctionContainer\nlocal FunctionContainer = { value = true }\nfunction FunctionContainer:Cancel() end\n",
        "---@class FunctionContainer\nlocal Other = {}\nfunction FunctionContainer:Cancel() end\n",
        "---@class FunctionContainer\nlocal FunctionContainer = {}\nfunction FunctionContainer.Cancel() end\n",
        "---@class FunctionContainer\nlocal FunctionContainer = {}\nfunction FunctionContainer:Cancel(value) end\n",
        "---@class FunctionContainer\nlocal FunctionContainer = {}\nfunction FunctionContainer:Cancel() return true end\n",
        "---@class FunctionContainer\nlocal FunctionContainer = {}\nfunction Other:Cancel() end\n",
        "---@class FunctionContainer\nlocal FunctionContainer = {}\nos.execute('never')\n",
        "---@class FunctionContainer\nlocal FunctionContainer = {}\n---@param value number\nfunction FunctionContainer:Cancel() end\n",
        "---@class FunctionContainer\nlocal FunctionContainer = {}\n---@return boolean value\nfunction FunctionContainer:IsCancelled() end\n",
    ] {
        assert!(
            ingest_alias_catalog(
                REVISION,
                "FunctionContainer.lua",
                raw,
                &source_digest(raw.as_bytes()),
                &AtomicBool::new(false),
            )
            .is_err(),
            "{raw}"
        );
    }
}
