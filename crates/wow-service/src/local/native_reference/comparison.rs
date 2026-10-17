//! Exact orchestration over the shared source-only native Reference acquisition.
use super::*;
use wow_reference::native_view::comparison::{
    CALLABLE_COMPARISON_PROFILE, CallableComparisonLimits, compare_reference_generations,
};

const REQUEST_SCHEMA: &str = "wow-service/source-reference-comparison-request/1";

#[cfg(test)]
mod tests;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ComparisonRequest {
    schema: String,
    comparison_profile: String,
    privacy: String,
    before: ExpectedReference,
    after: ExpectedReference,
    keys: Vec<String>,
    limits: CallableComparisonLimits,
    max_output_bytes: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedReference {
    configuration_content_digest: ContentDigest<SourceContent>,
    reference_generation: ReferenceGenerationId,
    reference_view_digest: String,
}
impl ExpectedReference {
    fn require(&self, actual: &SourceReferenceResult) -> ServiceResult<()> {
        if self.configuration_content_digest != actual.configuration_content_digest
            || self.reference_generation != actual.reference_generation
            || self.reference_view_digest != actual.reference.view.self_digest()
        {
            return Err(ServiceError::new(
                ServiceErrorCode::ExactGenerationUnavailable,
                "source Reference comparison exact binding does not match",
            ));
        }
        Ok(())
    }
}

/// Contains only approved bytes; native source-owning inputs have been released
/// before public delivery. Retrying this read recomputes from explicit inputs.
pub struct SourceReferenceComparisonResult {
    bytes: Vec<u8>,
    partial: bool,
}
impl SourceReferenceComparisonResult {
    pub const fn is_partial(&self) -> bool {
        self.partial
    }
    pub fn canonical_bytes(&self, stop: &AtomicBool) -> ServiceResult<Vec<u8>> {
        cancelled(stop)?;
        let bytes = self.bytes.clone();
        cancelled(stop)?;
        Ok(bytes)
    }
}

/// One operation, two explicit immutable native acquisitions, no effects. Config
/// guards bind the exact bytes consumed by the existing loader, not a prior stat
/// or a second unguarded config read. No symbolic generation or source selector.
pub fn compare_local_source_references(
    before_config: &Path,
    after_config: &Path,
    request_file: &Path,
    stop: &AtomicBool,
) -> ServiceResult<SourceReferenceComparisonResult> {
    cancelled(stop)?;
    let parent = request_file
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = request_file
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| name.ends_with(".json"))
        .ok_or_else(|| invalid("Reference comparison requires an explicit JSON request"))?;
    let directory =
        ProjectInputDirectory::open(parent).map_err(super::super::disk_input::acquisition_error)?;
    let bytes = directory
        .read_configuration(name, stop)
        .map_err(super::super::disk_input::acquisition_error)?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(invalid("Reference comparison request exceeds budget"));
    }
    let request: ComparisonRequest = serde_json::from_slice(&bytes)
        .map_err(|_| invalid("invalid Reference comparison request"))?;
    if request.schema != REQUEST_SCHEMA || request.comparison_profile != CALLABLE_COMPARISON_PROFILE
    {
        return Err(invalid("unsupported Reference comparison profile"));
    }
    if request.privacy != "local_metadata_only" {
        return Err(invalid("unsupported Reference comparison privacy"));
    }
    request.limits.validate().map_err(native_error)?;
    if request.keys.is_empty()
        || request.keys.len() > request.limits.max_keys
        || request.max_output_bytes == 0
        || request.max_output_bytes > 16 * 1024 * 1024
    {
        return Err(invalid("Reference comparison bounds are invalid"));
    }
    let before = read_local_source_reference(before_config, stop)?;
    request.before.require(&before)?;
    let after = read_local_source_reference(after_config, stop)?;
    request.after.require(&after)?;
    require_compatible(&before, &after)?;
    let comparison = compare_reference_generations(
        &before.reference,
        &before.revision,
        &after.reference,
        &after.revision,
        &request.keys,
        request.limits,
        stop,
    )
    .map_err(native_error)?;
    let partial = before.is_partial() || after.is_partial() || comparison.is_partial();
    #[derive(Serialize)]
    struct ResultBody<'a> {
        schema: &'static str,
        compatibility_policy: &'static str,
        privacy: &'static str,
        before: Origin<'a>,
        after: Origin<'a>,
        partial: bool,
        comparison: &'a wow_reference::native_view::comparison::NativeCallableComparison,
    }
    let result = ResultBody {
        schema: "wow-service/source-reference-comparison-result/1",
        compatibility_policy: "same_native_flavor_edition_environment_schema_explicit_build_pair/1",
        privacy: "local_metadata_only",
        before: Origin::from(&before),
        after: Origin::from(&after),
        partial,
        comparison: &comparison,
    };
    let bytes = super::super::native_input::report_bytes_with_limit(
        &result,
        request.max_output_bytes,
        stop,
    )?;
    cancelled(stop)?;
    Ok(SourceReferenceComparisonResult { bytes, partial })
}

fn require_compatible(
    before: &SourceReferenceResult,
    after: &SourceReferenceResult,
) -> ServiceResult<()> {
    if before.profile.validate().is_err()
        || after.profile.validate().is_err()
        || before.profile.profile_kind() != after.profile.profile_kind()
        || before.profile.flavor_id() != after.profile.flavor_id()
        || before.profile.edition_id() != after.profile.edition_id()
        || before.profile.schema_versions() != after.profile.schema_versions()
        || before.environment != after.environment
        || before.reference.schema != after.reference.schema
        || before.has_compatibility_aliases()
        || after.has_compatibility_aliases()
        || before.native_documents().is_some()
        || after.native_documents().is_some()
        || before.reference.type_resolution_profile().is_some()
        || after.reference.type_resolution_profile().is_some()
    {
        return Err(invalid(
            "Reference comparison profiles or environments are incompatible",
        ));
    }
    Ok(())
}

#[derive(Serialize)]
struct Origin<'a> {
    configuration_content_digest: &'a ContentDigest<SourceContent>,
    profile: &'a ProfileIdentity,
    reference_generation: ReferenceGenerationId,
    reference_view_digest: &'a str,
    revision: &'a str,
    environment: &'a str,
    source_binding: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_manifest: Option<&'a SourceManifestReceipt>,
    freshness: &'static str,
    source_files: &'a [NativeFileIdentity],
    input_failures: &'a [InputFailure],
    projection_profile: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    callable_coverage: Option<wow_reference::CoverageStatus>,
    issues: &'a [wow_reference::native_view::NativeViewIssue],
}
impl<'a> From<&'a SourceReferenceResult> for Origin<'a> {
    fn from(value: &'a SourceReferenceResult) -> Self {
        Self {
            configuration_content_digest: &value.configuration_content_digest,
            profile: &value.profile,
            reference_generation: value.reference_generation,
            reference_view_digest: value.reference.view.self_digest(),
            revision: &value.revision,
            environment: &value.environment,
            source_binding: value.source_binding,
            source_manifest: value.source_manifest.as_ref(),
            freshness: value.freshness,
            source_files: &value.source_files,
            input_failures: &value.input_failures,
            projection_profile: value.reference.schema,
            callable_coverage: value
                .reference
                .view
                .partitions()
                .iter()
                .find(|part| part.id() == wow_reference::native_view::NATIVE_API_PARTITION)
                .map(|part| part.coverage()),
            issues: &value.reference.issues,
        }
    }
}
