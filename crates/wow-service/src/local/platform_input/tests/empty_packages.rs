use super::*;

const INLINE_XML: &str = "<Ui xmlns=\"http://www.blizzard.com/wow/ui/\"><Frame name=\"InlineFrame\"><Scripts><OnLoad>local inlineLoaded = true</OnLoad></Scripts></Frame></Ui>";

fn mixed_fixture() -> TestResult<(FixtureRoot, Value)> {
    let (root, mut config) = fixture()?;
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(root.0.join("manifest.json"))?)?;
    for package in ["Inline", "Metadata"] {
        std::fs::create_dir(root.0.join(package))?;
        config["source_profile"]["roots"]
            .as_array_mut()
            .ok_or("source roots")?
            .push(json!({"root":package,"selected_tocs":[format!("{package}/{package}.toc")]}));
        config["packages"]
            .as_array_mut()
            .ok_or("packages")?
            .push(serde_json::to_value(ProjectPackageInput::new(
                package,
                package,
                true,
                vec![ProjectPackageVariantInput::new(
                    ProjectDiskFile::new(format!("{package}.toc")),
                    true,
                )],
            ))?);
    }
    let files = manifest["files"].as_array_mut().ok_or("manifest files")?;
    for (path, kind, text) in [
        ("Inline/Inline.toc", "toc", "frames.xml\n"),
        ("Inline/frames.xml", "xml", INLINE_XML),
        (
            "Metadata/Metadata.toc",
            "toc",
            "## SavedVariables: MetadataState\n",
        ),
    ] {
        std::fs::write(root.0.join(path), text)?;
        files.push(json!({"path":path,"kind":kind,"bytes":text.len(),
            "git_blob_algorithm":"sha1","git_blob_id":"2".repeat(40),
            "content_sha256":format!("{:x}",Sha256::digest(text.as_bytes()))}));
    }
    files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    let bytes: u64 = files
        .iter()
        .map(|file| file["bytes"].as_u64().unwrap_or(0))
        .sum();
    manifest["coverage"] = json!({"tracked_files":7,"included_files":7,
        "excluded_files":0,"included_bytes":bytes,"kind_version":1,
        "kind_lua":1,"kind_toc":3,"kind_xml":2});
    manifest
        .as_object_mut()
        .ok_or("manifest object")?
        .remove("manifest_sha256");
    manifest["manifest_sha256"] = json!(format!(
        "{:x}",
        Sha256::digest(wow_core::canonical_json_bytes(&manifest)?)
    ));
    let file = write_json(&root, "manifest.json", &manifest)?;
    config["source"]["manifest"] = serde_json::to_value(file)?;
    config["source"]["origin"]["revision"]["digest"] =
        serde_json::to_value(digest(&serde_json::to_vec(&manifest)?))?;
    write_json(&root, "project.json", &config)?;
    Ok((root, config))
}

#[test]
fn empty_package_members_preserve_native_graph_and_replay_with_physical_main() -> TestResult {
    let (root, config) = mixed_fixture()?;
    let stop = AtomicBool::new(false);
    let path = root.0.join("project.json");
    let input = LocalProjectInput::from_config_path(&path, &stop)?;
    let bundle = input.project_bundle();
    assert_eq!(bundle.inventory().files().len(), 1);
    let profile = bundle.configuration().selected_profile();
    let plan = bundle
        .configuration()
        .package_load_plan()
        .ok_or("load plan")?;
    assert_eq!(plan.packages().len(), 3);
    for package in ["Inline", "Metadata"] {
        plan.package_plan(package)
            .ok_or("member plan")?
            .validate_main_files(&[])?;
    }
    let packages: Vec<ProjectPackageInput> = serde_json::from_value(config["packages"].clone())?;
    let directory = ProjectInputDirectory::open(&root.0)?;
    let disk = directory
        .read_package_project_with_context(&packages, profile, None, &stop)?
        .into_namespaced_main()?;
    assert_eq!(disk.load_plan(), plan);
    assert_eq!(disk.files(), bundle.inventory().files());
    for (package, toc) in [("Inline", "Inline.toc"), ("Metadata", "Metadata.toc")] {
        assert_eq!(
            directory
                .read_toc_project_with_context(
                    package,
                    &ProjectDiskFile::new(toc),
                    profile,
                    None,
                    &stop
                )
                .err()
                .ok_or("standalone empty Main admitted")?
                .message(),
            "selected TOC closure contains no analyzable Lua files"
        );
    }
    let request = crate::graph::GraphBuildRequest::new(
        "fixture-platform-transport".into(),
        "current".into(),
    )?
    .with_input_profile(&input);
    // Native publication and replay must use the admitted bytes, including XML
    // whose package has no physical Main file.
    std::fs::remove_file(root.0.join("Inline/frames.xml"))?;
    let (publication, _) = crate::graph::live_publication(input, &request, &stop)?;
    let (records, bindings) = publication.into_parts();
    let graph: wow_graph::GraphSnapshot = records
        .iter()
        .find(|record| record.key() == "graph.materialized")
        .ok_or("graph record")?
        .decode()?;
    assert_eq!(
        Some(graph.snapshot_id().as_str()),
        bindings.get("graph_snapshot_id").map(String::as_str)
    );
    assert_eq!(
        graph
            .nodes()
            .iter()
            .filter(|node| node.kind() == "xml_source_virtual_lua_unit")
            .count(),
        2
    );
    assert!(
        graph
            .nodes()
            .iter()
            .any(|node| node.kind() == "xml_source_virtual_lua_map_piece")
    );
    let replay: wow_project::replay::ProjectReplay = records
        .iter()
        .find(|record| record.key() == "live.project.replay")
        .ok_or("replay record")?
        .decode()?;
    let reopened = replay.hydrate(&stop)?;
    assert_eq!(
        Some(reopened.snapshot_id()),
        bindings.get("project_snapshot_id").map(String::as_str)
    );
    assert_eq!(
        Some(reopened.analyzer_snapshot_id()),
        bindings.get("analyzer_snapshot_id").map(String::as_str)
    );
    assert_eq!(reopened.file_manifest().len(), 1);
    assert_eq!(
        reopened
            .configuration()
            .package_load_plan()
            .ok_or("replayed plan")?
            .packages()
            .len(),
        3
    );
    let xml = reopened
        .snapshot()
        .analyzer_binding()
        .package_xml_bindings()
        .ok_or("replayed XML bindings")?;
    let inline = xml
        .groups()
        .iter()
        .find(|group| group.scope().package() == "Inline")
        .ok_or("inline group")?;
    assert_eq!(inline.documents().len(), 1);
    // Inline bodies are native virtual units, without a named callable lookup.
    assert!(inline.bindings().is_empty());
    assert_eq!(
        reopened
            .snapshot()
            .analyzer_binding()
            .xml_lua_analysis()
            .ok_or("inline analysis")?
            .units()
            .len(),
        2
    );
    assert!(replay.hydrate(&AtomicBool::new(true)).is_err());

    // The aggregate owner still rejects an entirely physical-Lua-free Main.
    // Restore the exact admitted XML, rather than changing its manifest identity.
    std::fs::write(root.0.join("Inline/frames.xml"), INLINE_XML)?;
    let mut empty = config;
    empty["packages"]
        .as_array_mut()
        .ok_or("packages")?
        .retain(|package| package["name"] != "Fixture");
    empty["source_profile"]["roots"][0]["selected_tocs"] = json!([]);
    write_json(&root, "project.json", &empty)?;
    let error = LocalProjectInput::from_config_path(&path, &stop)
        .err()
        .ok_or("empty aggregate admitted")?;
    assert_eq!(error.code(), crate::ServiceErrorCode::InvalidConfiguration);
    assert_eq!(
        error.message(),
        "package Main namespace contains no Lua files"
    );
    Ok(())
}
