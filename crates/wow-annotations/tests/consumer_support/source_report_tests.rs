use super::*;
use crate::consumer_support::{Workspace, fixture::SOURCE_FILES, source};

fn build() -> Result<(Workspace, Value, String)> {
    let workspace = Workspace::new()?;
    let input = workspace.path.join("input");
    source::prepare(&input, SOURCE_FILES)?;
    let report: Value = serde_json::from_slice(&fs::read(input.join("source-report.json"))?)?;
    let revision = report["revision"]
        .as_str()
        .ok_or("missing fixture revision")?
        .to_owned();
    Ok((workspace, report, revision))
}

#[test]
fn git_toc_receipt_and_emitted_bytes_survive_dirty_source_files() -> Result<()> {
    let (workspace, report, revision) = build()?;
    let input = workspace.path.join("input");
    assert_eq!(
        fs::read_to_string(input.with_extension("source").join("API.toc"))?,
        "uncommitted.lua\n"
    );
    validate(&report, &input, &revision, SOURCE_FILES)?;
    assert!(array(&report["library"]["files"])?.iter().any(|file| {
        file["text"]
            .as_str()
            .is_some_and(|text| text.contains("function C_Probe.Read(id) end"))
    }));
    Ok(())
}

#[test]
fn partial_stale_missing_and_duplicate_receipts_cannot_pass() -> Result<()> {
    let (workspace, report, revision) = build()?;
    let input = workspace.path.join("input");
    for (pointer, value) in [
        ("/schema", json!("unknown")),
        ("/revision", json!("0".repeat(40))),
        ("/environment", json!("other")),
        ("/status", json!("partial")),
        ("/negative_authority", json!(true)),
        ("/candidate_files", json!(1)),
        ("/admitted_files", json!(1)),
        ("/source_order", json!([])),
        ("/input_failures", json!([{}])),
        ("/toc/sha256", json!("sha256:wrong")),
        ("/library/revision", json!("0".repeat(40))),
        ("/library/projection", json!("partial")),
        ("/library/issues", json!([{}])),
        ("/library/sources", json!([])),
        ("/library/sources/0/sha256", json!("sha256:wrong")),
        ("/library/files", json!([])),
        ("/library/files/0/mappings", json!([])),
        ("/library/files/0/sha256", json!("sha256:wrong")),
        ("/library/files/0/path", json!("../foreign.lua")),
        ("/library/files/0/mappings/0/generated/end", json!(u64::MAX)),
        (
            "/library/files/0/mappings/0/source/sha256",
            json!("sha256:wrong"),
        ),
        (
            "/library/files/0/mappings/0/source/span/end",
            json!(u64::MAX),
        ),
    ] {
        let mut changed = report.clone();
        *changed
            .pointer_mut(pointer)
            .ok_or("missing mutation target")? = value;
        assert!(
            validate(&changed, &input, &revision, SOURCE_FILES).is_err(),
            "{pointer}"
        );
    }
    for field in ["sources", "files"] {
        let mut changed = report.clone();
        let items = changed["library"][field]
            .as_array_mut()
            .ok_or("missing array")?;
        items.push(items[0].clone());
        assert!(validate(&changed, &input, &revision, SOURCE_FILES).is_err());
    }
    Ok(())
}

#[test]
fn changed_missing_or_unreported_artifacts_reject() -> Result<()> {
    let (workspace, report, revision) = build()?;
    let input = workspace.path.join("input");
    let name = report["library"]["files"][0]["path"]
        .as_str()
        .ok_or("missing file")?;
    let path = input.join(name);
    let original = fs::read(&path)?;
    fs::write(&path, "-- tampered\n")?;
    assert!(validate(&report, &input, &revision, SOURCE_FILES).is_err());
    fs::remove_file(&path)?;
    assert!(validate(&report, &input, &revision, SOURCE_FILES).is_err());
    fs::write(&path, original)?;
    fs::write(input.join("unexpected.lua"), "---@meta\n")?;
    assert!(validate(&report, &input, &revision, SOURCE_FILES).is_err());
    fs::remove_file(input.join("unexpected.lua"))?;
    validate(&report, &input, &revision, SOURCE_FILES)?;
    Ok(())
}

#[test]
fn map_ranges_must_be_nonempty_utf8_boundaries() -> Result<()> {
    span(&json!({"start":0,"end":2}), "é")?;
    for range in [
        json!({"start":0,"end":1}),
        json!({"start":1,"end":2}),
        json!({"start":0,"end":0}),
        json!({"start":2,"end":0}),
        json!({"start":0,"end":3}),
        json!({"start":-1,"end":2}),
    ] {
        assert!(span(&range, "é").is_err());
    }
    Ok(())
}
