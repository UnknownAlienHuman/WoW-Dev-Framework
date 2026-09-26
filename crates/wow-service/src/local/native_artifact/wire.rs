//! Closed native cache envelope. This is retained producer data, not an analyzer
//! session, a full Reference Pack or proof that source/consumer checks ran.
use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use wow_core::{CanonicalResult, ContentDigest, ProfileIdentity, SourceContent};
use wow_reference::{CoverageStatus, ReferenceRecordKind, ReferenceView};

use super::super::input::invalid;
use super::super::native_resources::present;
use crate::ServiceResult;

pub const ARTIFACT_SCHEMA: &str = "wow-service/native-input-artifact/1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Artifact {
    pub schema: String,
    pub profile: ProfileIdentity,
    pub reference_view: ReferenceView,
    #[serde(deserialize_with = "files")]
    pub library_files: Vec<LibraryFile>,
    pub source_report: SourceReport,
    pub negative_authority: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceReport {
    pub json: String,
    pub sha256: ContentDigest<SourceContent>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LibraryFile {
    pub path: String,
    pub text: String,
    pub sha256: ContentDigest<SourceContent>,
    pub byte_length: u64,
}

#[derive(Serialize)]
pub(super) struct ArtifactOutput<'a> {
    pub schema: &'static str,
    pub profile: &'a ProfileIdentity,
    pub reference_view: &'a ReferenceView,
    pub library_files: Vec<LibraryFileOutput<'a>>,
    pub source_report: SourceReportOutput<'a>,
    pub negative_authority: bool,
}

#[derive(Serialize)]
pub(super) struct LibraryFileOutput<'a> {
    pub path: &'a str,
    pub text: &'a str,
    pub sha256: ContentDigest<SourceContent>,
    pub byte_length: u64,
}

#[derive(Serialize)]
pub(super) struct SourceReportOutput<'a> {
    pub json: &'a str,
    pub sha256: String,
}

/// Check the native view's *allowed capability ceiling*, not source truth.
/// Do not let an artifact turn a Partial API slice into Complete or Secret data.
pub(super) fn reference(view: &ReferenceView) -> ServiceResult<()> {
    view.validate()
        .map_err(|_| invalid("native artifact ReferenceView was rejected"))?;
    if view.partitions().len() != 1 {
        return Err(invalid("native artifact requires one callable partition"));
    }
    for partition in view.partitions() {
        if partition.id() != wow_reference::native_view::NATIVE_API_PARTITION
            || partition.coverage() != CoverageStatus::Partial
            || partition.records().iter().any(|record| {
                record.kind() != ReferenceRecordKind::Api
                    || !record.key().starts_with("function:")
                    || !record.restrictions().is_empty()
            })
        {
            return Err(invalid(
                "native artifact exceeds the supported reference capability",
            ));
        }
    }
    Ok(())
}

/// Cross-check the envelope against the retained producer report. Detailed raw
/// metadata, correction decisions and maps remain inert claims in the exact
/// report; importing them does not rerun or certify their producers.
pub(super) fn report(
    report_json: &str,
    selected_profile: &ProfileIdentity,
    reference_view: &ReferenceView,
    library_files: &[LibraryFileOutput<'_>],
) -> ServiceResult<()> {
    let report: ReportBinding = serde_json::from_str(report_json)
        .map_err(|_| invalid("native artifact producer report was rejected"))?;
    let mode = match report.schema.as_str() {
        "wow-service/native-input-report/1" => 1,
        "wow-service/native-input-report/2" => 2,
        "wow-service/native-input-report/3" => 3,
        "wow-service/native-input-report/4" => {
            return Err(invalid(
                "authority-bearing native reports require source reacquisition and cannot be cached",
            ));
        }
        _ => return Err(invalid("unsupported retained native report schema")),
    };
    let profile = serde_json::to_value(selected_profile)
        .map_err(|_| invalid("native artifact profile cannot be encoded"))?;
    let expected_binding = if report.source_manifest.is_some() {
        "source_manifest_selected_toc"
    } else {
        "explicit_digest_pinned_manifest"
    };
    let library_version_matches = match report.library.schema.as_str() {
        "wow-native-annotation-library/3" => {
            mode != 3 && report.library.corrections.is_none() && report.library.aliases.is_none()
        }
        "wow-native-annotation-library/4" => {
            mode == 3 && report.library.corrections.is_some() && report.library.aliases.is_none()
        }
        "wow-native-annotation-library/5" => mode == 3 && report.library.aliases.is_some(),
        _ => false,
    };
    if &report.profile != selected_profile
        || &report.reference.view != reference_view
        || report.negative_authority
        || report.negative_authority_scope.is_some()
        || report.reference.negative_authority
        || report.library.negative_authority
        || report.reference.schema != wow_reference::native_view::NATIVE_VIEW_PROFILE
        || report.freshness != "unverified-current"
        || report.environment.is_empty()
        || report.environment.len() > 128
        || !report
            .environment
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        || !matches!(report.library.revision.len(), 40 | 64)
        || !report
            .library
            .revision
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || profile["source_kind"].as_str() != Some("blizzard_snapshot")
        || profile["source_revision"].as_str() != Some(report.library.revision.as_str())
        || report.library.source_map_profile != "wow-native-field-maps/1"
        || !library_version_matches
        || !matches!(
            report.library.projection.as_str(),
            "partial" | "projected_with_sidecars"
        )
        || (mode == 1 && (report.source_manifest.is_some() || report.annotation_inputs.is_some()))
        || (mode == 2 && (report.source_manifest.is_none() || report.annotation_inputs.is_some()))
        || (mode == 3 && report.annotation_inputs.is_none())
        || report.source_binding != expected_binding
    {
        return Err(invalid(
            "native artifact and producer report identities differ",
        ));
    }
    let expected: BTreeMap<_, _> = library_files.iter().map(|file| (file.path, file)).collect();
    if expected.len() != library_files.len() || expected.len() != report.library.files.len() {
        return Err(invalid(
            "native artifact Library inventory differs from its report",
        ));
    }
    let mut remaining = expected;
    for file in report.library.files {
        let Some(expected) = remaining.remove(file.path.as_str()) else {
            return Err(invalid("unlisted or repeated native report Library file"));
        };
        if file.sha256 != expected.sha256 || file.text != expected.text {
            return Err(invalid(
                "native artifact Library bytes differ from its report",
            ));
        }
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReportBinding {
    schema: String,
    profile: ProfileIdentity,
    #[serde(rename = "analyzer_input_configuration")]
    _analyzer_input_configuration: ContentDigest<CanonicalResult>,
    #[serde(rename = "analyzer_bound_configuration")]
    _analyzer_bound_configuration: ContentDigest<CanonicalResult>,
    environment: String,
    source_binding: String,
    #[serde(default, deserialize_with = "present")]
    source_manifest: Option<OpaqueObject>,
    freshness: String,
    #[serde(rename = "source_files", deserialize_with = "files")]
    _source_files: Vec<IgnoredAny>,
    #[serde(rename = "input_failures", deserialize_with = "files")]
    _input_failures: Vec<IgnoredAny>,
    reference: ReferenceBinding,
    library: LibraryBinding,
    #[serde(default, deserialize_with = "present")]
    annotation_inputs: Option<OpaqueObject>,
    negative_authority: bool,
    #[serde(default, deserialize_with = "present")]
    negative_authority_scope: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReferenceBinding {
    schema: String,
    view: ReferenceView,
    #[serde(rename = "candidates")]
    _candidates: IgnoredAny,
    #[serde(rename = "sources")]
    _sources: IgnoredAny,
    #[serde(rename = "issues")]
    _issues: IgnoredAny,
    negative_authority: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LibraryBinding {
    schema: String,
    source_map_profile: String,
    revision: String,
    projection: String,
    negative_authority: bool,
    #[serde(rename = "sources", deserialize_with = "files")]
    _sources: Vec<IgnoredAny>,
    #[serde(deserialize_with = "files")]
    files: Vec<ReportFile>,
    #[serde(rename = "issues")]
    _issues: IgnoredAny,
    #[serde(rename = "metadata_sidecars")]
    _metadata_sidecars: IgnoredAny,
    #[serde(rename = "scalar_resolutions")]
    _scalar_resolutions: IgnoredAny,
    #[serde(rename = "name_projections")]
    _name_projections: IgnoredAny,
    #[serde(default, deserialize_with = "present")]
    corrections: Option<OpaqueObject>,
    #[serde(default, deserialize_with = "present")]
    aliases: Option<OpaqueObject>,
    #[serde(rename = "limitations")]
    _limitations: IgnoredAny,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReportFile {
    path: String,
    text: String,
    sha256: ContentDigest<SourceContent>,
    #[serde(rename = "mappings")]
    _mappings: IgnoredAny,
}

/// Admit an object without allocating a second copy of its opaque metadata.
/// Its contents remain only in the pinned original report, not current facts.
struct OpaqueObject;

impl<'de> Deserialize<'de> for OpaqueObject {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ObjectVisitor;
        impl<'de> Visitor<'de> for ObjectVisitor {
            type Value = OpaqueObject;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a retained producer metadata object")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
                Ok(OpaqueObject)
            }
        }
        deserializer.deserialize_map(ObjectVisitor)
    }
}

/// Stop collection growth during decoding, not after an enormous Vec allocation.
fn files<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Files<T>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>> Visitor<'de> for Files<T> {
        type Value = Vec<T>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("at most 1024 native artifact files")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
            let mut values = Vec::new();
            while let Some(value) = sequence.next_element::<T>()? {
                if values.len() == 1024 {
                    return Err(serde::de::Error::custom("native artifact file-count limit"));
                }
                values.push(value);
            }
            Ok(values)
        }
    }
    deserializer.deserialize_seq(Files(PhantomData))
}
