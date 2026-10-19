from pathlib import Path

path = Path("apps/wow-reference-builder/src/lib.rs")
text = path.read_text()
old = '''    let partial = !failures.is_empty() || library.projection == "partial";
    let report = serde_json::json!({
        "schema": "wow-native-source-build/1",
        "revision": revision,
        "selector": selector,
        "freshness": "not_network_verified",
        "environment": environment,
        "toc": {"path": toc_path, "sha256": source_digest(&toc_bytes)},
        "source_order": &paths,
        "candidate_files": paths.len(),
        "admitted_files": documents.len(),
        "input_failures": &failures,
        "status": if partial {"partial"} else {"projected_with_sidecars"},
        "negative_authority": false,
        "library": &library,
    });
    let report_bytes = serde_json::to_vec_pretty(&report)?;
    if report_bytes.len() > 512 * LIMIT {
        return Err("report exceeds byte limit".into());
    }
'''
new = '''    let partial = !failures.is_empty() || library.projection == "partial";
    let source_inputs = documents
        .iter()
        .map(|document| {
            serde_json::json!({
                "path": document.path(),
                "sha256": document.sha256(),
                "bytes": document.source_bytes(),
            })
        })
        .collect::<Vec<_>>();
    let alias_inputs = alias_catalogs
        .iter()
        .map(|catalog| {
            serde_json::json!({
                "revision": catalog.revision(),
                "path": catalog.path(),
                "sha256": catalog.sha256(),
            })
        })
        .collect::<Vec<_>>();
    let annotation_files = library
        .files
        .iter()
        .map(|file| {
            serde_json::json!({
                "path": &file.path,
                "sha256": &file.sha256,
                "bytes": file.text.len(),
                "mappings": &file.mappings,
            })
        })
        .collect::<Vec<_>>();
    let report = serde_json::json!({
        "schema": "wow-native-source-build/2",
        "revision": revision,
        "selector": selector,
        "freshness": "not_network_verified",
        "environment": environment,
        "toc": {"path": toc_path, "sha256": source_digest(&toc_bytes)},
        "source_order": &paths,
        "source_inputs": source_inputs,
        "alias_inputs": alias_inputs,
        "candidate_files": paths.len(),
        "admitted_files": documents.len(),
        "input_failures": &failures,
        "status": if partial {"partial"} else {"projected_with_sidecars"},
        "negative_authority": false,
        "annotation_files": annotation_files,
        "projection_issues": &library.issues,
        "metadata_sidecars": &library.metadata_sidecars,
        "scalar_resolutions": &library.scalar_resolutions,
        "name_projections": &library.name_projections,
        "corrections": &library.corrections,
        "limitations": &library.limitations,
    });
    let report_bytes = serde_json::to_vec_pretty(&report)?;
    if report_bytes.len() > TOTAL_LIMIT {
        return Err("report exceeds byte limit".into());
    }
'''
if text.count(old) != 1:
    raise SystemExit("unexpected source-report block")
path.write_text(text.replace(old, new))

readme = Path("apps/wow-reference-builder/README.md")
text = readme.read_text()
old = '''The destination must not exist. Output is generated `.lua` files plus
`source-report.json`; exit `3` means useful partial output, `0` means complete
projection, `2` is usage, `5` is generation failure, and `6` is cancellation.
'''
new = '''The destination must not exist. Output is generated `.lua` files plus a bounded
`source-report.json` with selected input identities, omissions and source maps;
it does not duplicate source or generated Lua text. Exit `3` means useful partial
output, `0` means complete projection, `2` is usage, `5` is generation failure,
and `6` is cancellation.
'''
if text.count(old) != 1:
    raise SystemExit("unexpected builder README block")
readme.write_text(text.replace(old, new))
