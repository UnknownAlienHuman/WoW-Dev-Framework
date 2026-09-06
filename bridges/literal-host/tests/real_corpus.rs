//! Explicit online-workflow input, but this test itself only reads local Git.
#[path = "support/compare.rs"]
mod compare;
#[path = "../../../crates/wow-annotations/examples/support/mod.rs"]
mod driver;
use serde_json::{Value, json};
use std::{error::Error, ffi::OsString, fs, path::PathBuf};
use wow_literal_host::{Limits, ModuleHandle, ModuleSlot, module_digest};
#[test]
#[ignore = "requires a selected local Gethe/Ketho corpus and two real guests; source CI only"]
fn selected_current_corpus_matches_native_with_two_guests() -> Result<(), Box<dyn Error>> {
    let get = |key| std::env::var(key);
    let source = get("WDF_CORPUS_CHECKOUT")?;
    let revision = get("WDF_CORPUS_REVISION")?;
    let toc = get("WDF_CORPUS_TOC")?;
    let environment = get("WDF_CORPUS_ENVIRONMENT")?;
    let donor = get("WDF_ALIAS_CHECKOUT")?;
    let donor_revision = get("WDF_ALIAS_REVISION")?;
    let donor_path = get("WDF_ALIAS_PATH")?;
    let root = PathBuf::from(get("WDF_OUTPUT_ROOT")?);
    let baseline: Value =
        serde_json::from_slice(&fs::read(root.join("native-aliased/source-report.json"))?)?;
    assert_eq!(baseline["revision"], revision);
    let a = fs::read(get("WDF_WASM_A")?)?;
    let b = fs::read(get("WDF_WASM_B")?)?;
    assert_ne!(module_digest(&a), module_digest(&b));
    let first = ModuleHandle::load(&a, &module_digest(&a), Limits::default())?;
    let second = ModuleHandle::load(&b, &module_digest(&b), Limits::default())?;
    let slot = ModuleSlot::new(first);
    let mut summaries = Vec::new();
    for (index, name) in ["wasm-a", "wasm-b"].into_iter().enumerate() {
        if index == 1 {
            slot.replace(slot.snapshot()?.selection(), second.clone())?;
        }
        let snapshot = slot.snapshot()?;
        let mut args: Vec<OsString> = [&source, &revision, &toc, &environment]
            .into_iter()
            .map(Into::into)
            .collect();
        args.push(root.join(name).into_os_string());
        args.extend(
            ["--alias-catalog", &donor, &donor_revision, &donor_path]
                .into_iter()
                .map(Into::into),
        );
        let partial = driver::run(args, Some(&snapshot))?;
        let report: Value =
            serde_json::from_slice(&fs::read(root.join(name).join("source-report.json"))?)?;
        assert_eq!(partial, baseline["status"] == "partial");
        compare::compare(&baseline, &report)?;
        assert_eq!(
            report["library"]["literal_execution"]["module"]["sha256"],
            snapshot.selection().module_sha256()
        );
        let files = report["library"]["files"]
            .as_array()
            .ok_or("missing files")?;
        for file in files {
            let path = file["path"].as_str().ok_or("missing file path")?;
            assert_eq!(
                fs::read(root.join(name).join(path))?,
                fs::read(root.join("native-aliased").join(path))?
            );
        }
        summaries.push(json!({"output":name,"module":report["library"]["literal_execution"]["module"],"candidate_files":report["candidate_files"],"admitted_files":report["admitted_files"],"input_failures":report["input_failures"].as_array().map(Vec::len),"annotation_files":files.len(),"calls":report["library"]["literal_execution"]["calls"].as_array().map(Vec::len),"status":report["status"],"native_files_and_metadata_identical":true}));
    }
    fs::write(
        root.join("source-wasm-comparison.json"),
        serde_json::to_vec_pretty(
            &json!({"source_revision":revision,"alias_revision":donor_revision,"runs":summaries,"semantic_consumer_compatibility":"not_evaluated"}),
        )?,
    )?;
    println!(
        "Both real guests matched the same native source generation without rebuilding the host"
    );
    Ok(())
}
