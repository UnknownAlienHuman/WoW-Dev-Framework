//! Exact v1 wire format produced by `cargo xtask manifest`.
//! Typed decoding rejects unknown and duplicate fields before digest verification.
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceManifest {
    pub schema_version: u64,
    pub source: Source,
    pub selection: Selection,
    pub coverage: Coverage,
    pub files: Vec<Member>,
    pub manifest_sha256: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Source {
    pub source_id: String,
    pub selector: String,
    pub revision: String,
    pub git_object_format: String,
    pub version: String,
    pub acquisition: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Selection {
    pub extensions: Vec<String>,
    pub version_path: String,
    pub non_regular_entries: String,
    pub working_tree: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Coverage {
    pub tracked_files: u64,
    pub included_files: u64,
    pub excluded_files: u64,
    pub included_bytes: u64,
    #[serde(default)]
    pub kind_version: Option<u64>,
    #[serde(default)]
    pub kind_generated_api: Option<u64>,
    #[serde(default)]
    pub kind_lua: Option<u64>,
    #[serde(default)]
    pub kind_toc: Option<u64>,
    #[serde(default)]
    pub kind_xml: Option<u64>,
    #[serde(default)]
    pub kind_schema: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Member {
    pub path: String,
    pub kind: String,
    pub bytes: u64,
    pub git_blob_algorithm: String,
    pub git_blob_id: String,
    pub content_sha256: String,
}
