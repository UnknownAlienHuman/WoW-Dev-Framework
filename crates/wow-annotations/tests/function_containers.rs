//! Ketho FunctionContainer resources project as static analysis declarations.
use std::sync::atomic::AtomicBool;
use wow_annotations::native::project_with_alias_catalog;
use wow_annotations::navigation::{GeneratedLookup, NavigationIndex, SourceFile, SourceLookup};
use wow_reference::native::{ingest_document, source_digest};
use wow_reference::native_aliases::ingest_alias_catalog;

const SOURCE_REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DONOR_REVISION: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn project(
    raw: &str,
) -> Result<wow_annotations::native::NativeLibrary<'static>, Box<dyn std::error::Error>> {
    let source = r#"APIDocumentation:AddDocumentationTable({Name="Probe",Type="System",Namespace="C_Probe",Functions={{Name="Read"}}})"#;
    let documents = Box::leak(Box::new([ingest_document(
        SOURCE_REVISION,
        "API.lua",
        source,
        &source_digest(source.as_bytes()),
        &AtomicBool::new(false),
    )?]));
    let catalog = Box::leak(Box::new(ingest_alias_catalog(
        DONOR_REVISION,
        "FunctionContainer.lua",
        raw,
        &source_digest(raw.as_bytes()),
        &AtomicBool::new(false),
    )?));
    Ok(project_with_alias_catalog(
        documents,
        "Mainline",
        None,
        Some(catalog),
        &AtomicBool::new(false),
    )?)
}

#[test]
fn current_static_container_shape_emits_exact_methods_and_bidirectional_maps()
-> Result<(), Box<dyn std::error::Error>> {
    let raw = "---@meta _\n\n---@class FunctionContainer\nlocal FunctionContainer = {}\n\nfunction FunctionContainer:Cancel() end\n\n---@return boolean\nfunction FunctionContainer:IsCancelled() end\n\nfunction FunctionContainer:Invoke() end\n";
    let library = project(raw)?;
    assert_eq!(library.projection, "projected_with_sidecars");
    let report = library.aliases.as_ref().ok_or("alias report")?;
    assert_eq!(report.schema, "wow-native-alias-projection/8");
    assert_eq!(report.function_container_outcomes.len(), 1);
    assert_eq!(report.function_container_outcomes[0].status, "emitted");
    assert!(report.unresolved_function_container_returns.is_empty());
    let file = library.files.last().ok_or("external output")?;
    let expected = "---@class FunctionContainer\nlocal FunctionContainer = {}\n\nfunction FunctionContainer:Cancel() end\n\n---@return boolean\nfunction FunctionContainer:IsCancelled() end\n\nfunction FunctionContainer:Invoke() end";
    assert!(file.text.contains(expected));
    assert_eq!(file.mappings.len(), 4);
    let index = NavigationIndex::new(&library, &AtomicBool::new(false))?;
    for mapping in &file.mappings {
        assert_eq!(mapping.source.scope, Some("annotation_alias_catalog"));
        assert_eq!(mapping.source.path, "FunctionContainer.lua");
        let SourceLookup::Mapped { candidates, .. } = index.source_at(
            &file.path,
            &file.sha256,
            mapping.generated.start,
            &AtomicBool::new(false),
        )? else {
            return Err("generated method did not navigate".into());
        };
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].revision, DONOR_REVISION);
    }
    let source = index.bind_source(
        SourceFile {
            scope: Some("annotation_alias_catalog"),
            revision: DONOR_REVISION,
            path: report.source.path(),
            sha256: report.source.sha256(),
        },
        raw,
        &AtomicBool::new(false),
    )?;
    let method = &report.source.function_containers()[0].methods[1];
    assert!(matches!(
        source.generated_at(method.span.start, &AtomicBool::new(false))?,
        GeneratedLookup::Mapped { .. }
    ));
    Ok(())
}

#[test]
fn unknown_return_type_is_retained_but_keeps_the_overlay_partial()
-> Result<(), Box<dyn std::error::Error>> {
    let raw = "---@class FunctionContainer\nlocal FunctionContainer = {}\n\n---@return MissingType\nfunction FunctionContainer:GetMissing() end\n";
    let library = project(raw)?;
    assert_eq!(library.projection, "partial");
    let report = library.aliases.as_ref().ok_or("alias report")?;
    assert_eq!(report.schema, "wow-native-alias-projection/8");
    assert_eq!(report.function_container_outcomes[0].status, "emitted");
    assert_eq!(report.unresolved_function_container_returns.len(), 1);
    assert!(
        library
            .files
            .last()
            .ok_or("external output")?
            .text
            .contains("---@return MissingType\nfunction FunctionContainer:GetMissing() end")
    );
    assert!(library.issues.iter().any(|issue| {
        issue.code == "unresolved_function_container_return_type"
            && issue.source.scope == Some("annotation_alias_catalog")
    }));
    Ok(())
}
