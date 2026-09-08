//! Explicit source-guarded widget inheritance through the production projection.
use std::sync::atomic::AtomicBool;
use wow_annotations::native::{NativeLibrary, project, project_with_corrections};
use wow_reference::native::{DocumentationDocument, ingest_document, source_digest};
use wow_reference::native_corrections::{
    Correction, CorrectionSet, Evidence, INHERITANCE_SCHEMA, NORMALIZER, Projection, SCHEMA,
    Status, Target, ValidatedCorrections, Value, raw_digest,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn widget(name: &str) -> Result<DocumentationDocument> {
    let raw = format!(
        "APIDocumentation:AddDocumentationTable({{Name=\"{name}\",Type=\"ScriptObject\",Functions={{{{Name=\"OwnMethod\"}}}}}})"
    );
    Ok(ingest_document(
        REV,
        &format!("{name}.lua"),
        &raw,
        &source_digest(raw.as_bytes()),
        &AtomicBool::new(false),
    )?)
}

fn record(
    document: &DocumentationDocument,
    projection: Projection,
    after: &str,
) -> Result<Correction> {
    Ok(Correction {
        id: format!("{}:{after}", document.path()),
        target: Target {
            path: document.path().into(),
            registration: 0,
            projection,
        },
        expected_source_sha256: document.sha256().into(),
        expected_raw_sha256: raw_digest(&document.registrations()[0].value)?,
        before: Value::Absent,
        after: Value::Text(after.into()),
        reviewer: "synthetic fixture".into(),
        rationale: "Explicit synthetic parent; not a client widget inventory".into(),
        evidence: vec![Evidence {
            revision: REV.into(),
            path: "reviewed/synthetic.lua".into(),
            sha256: source_digest(b"synthetic review"),
        }],
    })
}

fn base(child: &DocumentationDocument, parent: &DocumentationDocument) -> Result<Correction> {
    record(
        child,
        Projection::WidgetBase {
            parent_path: parent.path().into(),
            parent_registration: 0,
            expected_parent_source_sha256: parent.sha256().into(),
            expected_parent_raw_sha256: raw_digest(&parent.registrations()[0].value)?,
        },
        parent.path().strip_suffix(".lua").ok_or("parent name")?,
    )
}

fn pack(records: Vec<Correction>) -> Result<ValidatedCorrections> {
    Ok(ValidatedCorrections::new(CorrectionSet {
        schema: INHERITANCE_SCHEMA.into(),
        version: 1,
        revision: REV.into(),
        environment: "Mainline".into(),
        normalizer: NORMALIZER.into(),
        records,
    })?)
}

fn output(library: &NativeLibrary<'_>) -> String {
    library
        .files
        .iter()
        .map(|f| f.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn guarded_parent_chain_reaches_renderer_and_retains_raw_and_final_maps() -> Result<()> {
    let docs = [widget("Child")?, widget("Parent")?, widget("Root")?];
    let original = docs.clone();
    let corrections = pack(vec![base(&docs[0], &docs[1])?, base(&docs[1], &docs[2])?])?;
    let stopped = AtomicBool::new(false);
    let library = project_with_corrections(&docs, "Mainline", Some(&corrections), &stopped)?;
    assert_eq!(library.projection, "projected_with_sidecars");
    assert_eq!(docs, original);
    let text = output(&library);
    assert!(text.contains("---@class Child : Parent\nlocal Child = {}"));
    assert!(text.contains("---@class Parent : Root\nlocal Parent = {}"));
    assert!(text.contains("function Child:OwnMethod() end"));
    for file in &library.files {
        for map in &file.mappings {
            assert!(
                file.text
                    .get(map.generated.start..map.generated.end)
                    .is_some()
            );
        }
        assert_eq!(file.sha256, source_digest(file.text.as_bytes()));
    }
    assert!(!output(&project(&docs, "Mainline", &stopped)?).contains(" : "));
    let mut legacy = corrections.set().clone();
    legacy.schema = SCHEMA.into();
    assert!(ValidatedCorrections::new(legacy).is_err());
    let reversed = [docs[2].clone(), docs[0].clone(), docs[1].clone()];
    let repeated = project_with_corrections(&reversed, "Mainline", Some(&corrections), &stopped)?;
    assert_eq!(
        serde_json::to_value(library)?,
        serde_json::to_value(repeated)?
    );
    Ok(())
}

#[test]
fn parent_rename_keeps_original_alias_and_independent_child_base() -> Result<()> {
    let docs = [widget("Child")?, widget("Parent")?];
    let mut rename = record(&docs[1], Projection::WidgetOwner, "RenamedParent")?;
    rename.before = Value::Text("Parent".into());
    let corrections = pack(vec![base(&docs[0], &docs[1])?, rename])?;
    let library = project_with_corrections(
        &docs,
        "Mainline",
        Some(&corrections),
        &AtomicBool::new(false),
    )?;
    assert_eq!(library.projection, "projected_with_sidecars");
    let text = output(&library);
    assert!(text.contains("---@class Child : Parent"));
    assert!(text.contains("---@class RenamedParent\nlocal RenamedParent = {}"));
    assert!(text.contains("---@alias Parent RenamedParent"));
    Ok(())
}

#[test]
fn stale_parent_guards_do_not_erase_the_child_or_claim_an_applied_base() -> Result<()> {
    let docs = [widget("Child")?, widget("Parent")?];
    for mutation in 0..4 {
        let mut correction = base(&docs[0], &docs[1])?;
        let Projection::WidgetBase {
            parent_path,
            parent_registration,
            expected_parent_source_sha256,
            expected_parent_raw_sha256,
        } = &mut correction.target.projection
        else {
            return Err("base projection".into());
        };
        match mutation {
            0 => *parent_path = "Missing.lua".into(),
            1 => *parent_registration = 1,
            2 => *expected_parent_source_sha256 = source_digest(b"stale"),
            _ => *expected_parent_raw_sha256 = source_digest(b"stale"),
        }
        let corrections = pack(vec![correction])?;
        let library = project_with_corrections(
            &docs,
            "Mainline",
            Some(&corrections),
            &AtomicBool::new(false),
        )?;
        assert_eq!(library.projection, "partial");
        let application = &library
            .corrections
            .as_ref()
            .ok_or("corrections")?
            .applications[0];
        assert_eq!(application.status, Status::Expired);
        assert!(application.after.is_none());
        let text = output(&library);
        assert!(text.contains("---@class Child\nlocal Child = {}"));
        assert!(text.contains("function Child:OwnMethod() end"));
        assert!(!text.contains(" : "));
    }
    Ok(())
}

#[test]
fn competing_parents_self_edges_and_cycles_cannot_be_applied() -> Result<()> {
    let docs = [widget("Child")?, widget("Parent")?, widget("Root")?];
    for records in [
        vec![base(&docs[0], &docs[0])?],
        vec![base(&docs[0], &docs[1])?, base(&docs[0], &docs[2])?],
        vec![
            base(&docs[0], &docs[1])?,
            base(&docs[1], &docs[0])?,
            base(&docs[2], &docs[0])?,
        ],
    ] {
        let corrections = pack(records)?;
        let library = project_with_corrections(
            &docs,
            "Mainline",
            Some(&corrections),
            &AtomicBool::new(false),
        )?;
        assert_eq!(library.projection, "partial");
        assert!(!output(&library).contains(" : "));
        assert!(
            library
                .corrections
                .as_ref()
                .ok_or("corrections")?
                .applications
                .iter()
                .all(|a| a.status == Status::Conflict && a.after.is_none())
        );
    }
    Ok(())
}

#[test]
fn parent_receiver_conflict_blocks_only_the_inheritance_projection() -> Result<()> {
    let mut docs = vec![widget("Child")?, widget("Parent")?];
    let raw = "APIDocumentation:AddDocumentationTable({Name=\"Other\",Type=\"System\",Namespace=\"Parent\"})";
    docs.push(ingest_document(
        REV,
        "Other.lua",
        raw,
        &source_digest(raw.as_bytes()),
        &AtomicBool::new(false),
    )?);
    let corrections = pack(vec![base(&docs[0], &docs[1])?])?;
    let library = project_with_corrections(
        &docs,
        "Mainline",
        Some(&corrections),
        &AtomicBool::new(false),
    )?;
    assert_eq!(library.projection, "partial");
    assert!(
        library
            .issues
            .iter()
            .any(|i| i.code == "widget_base_not_projected")
    );
    assert!(output(&library).contains("---@class Child\nlocal Child = {}"));
    assert!(!output(&library).contains("---@class Child : Parent"));
    assert!(
        project_with_corrections(
            &docs,
            "Mainline",
            Some(&corrections),
            &AtomicBool::new(true)
        )
        .is_err()
    );
    Ok(())
}
