//! Ketho-style resource syntax; synthetic values are not a client inventory.
use std::sync::atomic::AtomicBool;
use wow_annotations::native::project_with_alias_catalog;
use wow_reference::native::{NativeError, NativeErrorCode, ingest_document, source_digest};
use wow_reference::native_aliases::{AliasDocument, ingest_alias_catalog, ingest_aliases};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DONOR: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn catalog(text: &str) -> std::result::Result<AliasDocument, NativeError> {
    ingest_alias_catalog(
        DONOR,
        "Types.lua",
        text,
        &source_digest(text.as_bytes()),
        &AtomicBool::new(false),
    )
}

fn project(text: &str) -> Result<serde_json::Value> {
    let source = r#"APIDocumentation:AddDocumentationTable({Name="Probe",Type="System",Namespace="C_Probe",Functions={{Name="Choose",Arguments={{Name="choice",Type="Choice"}}}}})"#;
    let docs = [ingest_document(
        REV,
        "API.lua",
        source,
        &source_digest(source.as_bytes()),
        &AtomicBool::new(false),
    )?];
    let aliases = catalog(text)?;
    let library = project_with_alias_catalog(
        &docs,
        "Mainline",
        None,
        Some(&aliases),
        &AtomicBool::new(false),
    )?;
    Ok(serde_json::to_value(library)?)
}

#[test]
fn multiline_ketho_literals_reach_the_library_with_exact_scoped_maps() -> Result<()> {
    let raw = "\u{feff}---@meta _\r\n---@alias Choice #41\r\n---|\"FIRST\"\r\n---|'SECOND'\r\n";
    let library = project(raw)?;
    assert_eq!(library["schema"], "wow-native-annotation-library/5");
    assert_eq!(library["projection"], "projected_with_sidecars");
    assert_eq!(library["negative_authority"], false);
    assert_eq!(
        library["aliases"]["schema"],
        "wow-native-alias-projection/2"
    );
    assert_eq!(
        library["aliases"]["source"]["schema"],
        "wow-native-alias-resource/2"
    );
    assert_eq!(library["aliases"]["source"]["revision"], DONOR);
    assert_eq!(library["aliases"]["source"]["text"], raw);
    let file = library["files"]
        .as_array()
        .ok_or("files")?
        .last()
        .ok_or("file")?;
    let text = file["text"].as_str().ok_or("text")?;
    assert!(text.contains("---@alias Choice \"FIRST\"|\"SECOND\"\n"));
    let map = &file["mappings"][0];
    assert_eq!(map["source"]["scope"], "annotation_alias_catalog");
    let start = map["source"]["span"]["start"].as_u64().ok_or("start")? as usize;
    let end = map["source"]["span"]["end"].as_u64().ok_or("end")? as usize;
    assert!(raw.get(start..end).ok_or("span")?.contains("---|'SECOND'"));
    Ok(())
}

#[test]
fn inline_and_multiline_literals_have_identical_generated_bytes() -> Result<()> {
    let inline = project("---@alias Choice \"FIRST\"|'SECOND'\n")?;
    let multiline = project("---@alias Choice\n---|\"FIRST\"\n---|'SECOND'\n")?;
    assert_eq!(inline["files"][1]["text"], multiline["files"][1]["text"]);
    assert_eq!(
        inline["files"][1]["sha256"],
        multiline["files"][1]["sha256"]
    );
    Ok(())
}

#[test]
fn named_dependencies_can_resolve_to_a_closed_string_enum_without_expansion() -> Result<()> {
    let library = project(
        "---@alias Choice LiteralChoice|nil\n---@alias LiteralChoice\n---|\"A|B\"\n---|\"number\"\n",
    )?;
    assert_eq!(library["projection"], "projected_with_sidecars");
    let text = library["files"][1]["text"].as_str().ok_or("text")?;
    assert!(text.contains("---@alias Choice LiteralChoice|nil"));
    assert!(text.contains("---@alias LiteralChoice \"A|B\"|\"number\""));
    Ok(())
}

#[test]
fn rejected_literal_forms_preserve_independent_aliases_and_remain_partial() -> Result<()> {
    for rejected in [
        "---@alias Broken \"A\"|string\n",
        "---@alias Broken \"A\"|42\n",
        "---@alias Broken \"A\"|\"A\"\n",
        "---@alias Broken \"a\\nb\"\n",
        "---@alias Broken \"é\"\n",
        "---@alias Broken fun():string\n",
        "---@alias Broken\n---|number\n",
    ] {
        let library = project(&format!("{rejected}---@alias Choice \"GOOD\"\n"))?;
        assert_eq!(library["projection"], "partial", "{rejected}");
        assert_ne!(library["aliases"]["outcomes"][0]["status"], "emitted");
        assert_eq!(library["aliases"]["outcomes"][1]["status"], "emitted");
    }
    Ok(())
}

#[test]
fn malformed_multiline_alias_does_not_consume_or_repair_its_neighbor() -> Result<()> {
    let library =
        project("---@alias Broken\n---|\"unterminated\n---@alias Choice\n---|\"GOOD\"\n")?;
    assert_eq!(library["projection"], "partial");
    // Emmy reports no parse error for this incomplete doc token. Preserve that
    // observation, while the closed-literal adapter must still reject its type.
    assert_eq!(
        library["aliases"]["outcomes"][0]["status"],
        "unsupported_alias_type"
    );
    assert_eq!(
        library["aliases"]["source"]["aliases"][0]["syntax_error"],
        false
    );
    assert_eq!(
        library["aliases"]["source"]["aliases"][0]["terms"],
        serde_json::Value::Null
    );
    assert!(
        library["aliases"]["source"]["aliases"][0]
            .get("string_values")
            .is_none()
    );
    assert_eq!(library["aliases"]["outcomes"][1]["status"], "emitted");
    Ok(())
}

#[test]
fn alias_name_conflicts_do_not_become_merged_string_enums() -> Result<()> {
    for text in [
        "---@alias Choice \"A\"\n---@alias Choice \"B\"\n",
        "---@alias C_Probe \"A\"\n",
    ] {
        let library = project(text)?;
        assert_eq!(library["projection"], "partial");
        assert_eq!(library["files"].as_array().ok_or("files")?.len(), 1);
    }
    Ok(())
}

#[test]
fn orphan_continuations_and_non_alias_side_effects_reject() {
    for text in [
        "---|\"ORPHAN\"\n---@alias Choice number\n",
        "---@alias Choice\n\n---|\"ORPHAN\"\n",
        "---@meta _\n---|\"ORPHAN\"\n---@alias Choice number\n",
        "---@alias Choice\n---|\"OK\"\n---@diagnostic disable\n",
        "---@alias Choice\n---|\"OK\"\nreturn {}\n",
    ] {
        assert!(catalog(text).is_err(), "{text}");
    }
}

#[test]
fn continuation_budget_accepts_512_values_and_rejects_513_before_parsing() -> Result<()> {
    let mut raw = String::from("---@alias Choice\n");
    for i in 0..512 {
        raw.push_str(&format!("---|\"SYNTHETIC_{i}\"\n"));
    }
    let accepted = catalog(&raw)?;
    assert_eq!(
        accepted.aliases()[0]
            .string_values
            .as_ref()
            .ok_or("values")?
            .len(),
        512
    );
    raw.push_str("---|\"OVER_BUDGET\"\n");
    let Err(error) = catalog(&raw) else {
        return Err("over-budget catalog was accepted".into());
    };
    assert_eq!(error.code, NativeErrorCode::Limit);
    Ok(())
}

#[test]
fn old_named_profile_and_uncataloged_bytes_are_unchanged() -> Result<()> {
    let raw = "---@meta _\n---@alias Choice bool|cstring|luaIndex\n";
    let old = ingest_aliases(
        DONOR,
        "Types.lua",
        raw,
        &source_digest(raw.as_bytes()),
        &AtomicBool::new(false),
    )?;
    let new = catalog(raw)?;
    assert_eq!(serde_json::to_value(old)?, serde_json::to_value(new)?);
    assert_eq!(
        project(raw)?["aliases"]["schema"],
        "wow-native-alias-projection/1"
    );
    Ok(())
}

#[test]
fn explicit_inline_open_base_matches_multiline_hints() -> Result<()> {
    let inline = project("---@alias Choice string|\"FIRST\"|'SECOND'\n")?;
    let multiline = project("---@alias Choice string #hints\n---|\"FIRST\"\n---|'SECOND'\n")?;
    for library in [&inline, &multiline] {
        assert_eq!(library["projection"], "projected_with_sidecars");
        assert_eq!(
            library["aliases"]["source"]["aliases"][0]["string_base"],
            "string"
        );
        let text = library["files"][1]["text"].as_str().ok_or("output")?;
        assert!(text.contains("---@alias Choice string|\"FIRST\"|\"SECOND\""));
    }
    assert_eq!(inline["files"][1]["text"], multiline["files"][1]["text"]);
    Ok(())
}

#[test]
fn large_open_hint_catalog_remains_open_and_retains_every_value() -> Result<()> {
    let mut raw = String::from("---@alias EmoteToken string\n");
    for i in 0..263 {
        raw.push_str(&format!("---|\"EMOTE_{i}\"\n"));
    }
    let library = project(&raw)?;
    assert_eq!(library["projection"], "projected_with_sidecars");
    let fact = &library["aliases"]["source"]["aliases"][0];
    assert_eq!(fact["string_base"], "string");
    assert_eq!(fact["string_values"].as_array().ok_or("values")?.len(), 263);
    let text = library["files"][1]["text"].as_str().ok_or("output")?;
    assert!(text.contains("---@alias EmoteToken string|\"EMOTE_0\""));
    assert!(text.contains("|\"EMOTE_262\""));
    Ok(())
}
