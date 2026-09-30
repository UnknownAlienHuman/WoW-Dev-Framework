#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_service::reference_pack::{
    PackImageFile, PackMaterializationPlan, RebuildComparisonStatus, ReferencePackBudgets,
    ReferencePackBuildRequest, ReferencePackBuildStatus, ReferencePackEligibilityTarget,
    ReferencePackError, ReferencePackErrorCode, ReferencePackImage, ReferencePackManifest,
    ReferencePackRebuildComparisonReport, ReferencePackRebuildComparisonRequest,
    ReferencePackService, ReferencePackValidationReport, ReferencePackValidationRequest,
};
use wow_service::{LocalProjectInput, ServiceError, ServiceErrorCode};

const BUILD_REQUEST_SCHEMA: &str = "wow-reference-builder/build-request/1";
const VALIDATION_EXPECTATION_SCHEMA: &str = "wow-reference-builder/validation-expectation/1";
const REBUILD_REQUEST_SCHEMA: &str = "wow-reference-builder/rebuild-request/1";
const BUILD_RESULT_SCHEMA: &str = "wow-reference-builder/build-result/1";
const VALIDATE_RESULT_SCHEMA: &str = "wow-reference-builder/validate-result/1";
const REBUILD_RESULT_SCHEMA: &str = "wow-reference-builder/rebuild-result/1";
const MAX_REQUEST_BYTES: u64 = 16 * 1024 * 1024;
const MAX_EXPECTATION_BYTES: u64 = 1024 * 1024;
const MAX_STAGING_ATTEMPTS: u32 = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExitClass {
    Success,
    Usage,
    Blocked,
    Validation,
    Build,
    Cancelled,
    Security,
    Unavailable,
}

impl ExitClass {
    const fn code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::Usage => 2,
            Self::Blocked => 3,
            Self::Validation => 4,
            Self::Build => 5,
            Self::Cancelled => 6,
            Self::Security => 7,
            Self::Unavailable => 8,
        }
    }
}

#[derive(Debug)]
struct CliError {
    class: ExitClass,
    code: &'static str,
    message: Box<str>,
}

impl CliError {
    fn new(class: ExitClass, code: &'static str, message: impl Into<Box<str>>) -> Self {
        Self {
            class,
            code,
            message: message.into(),
        }
    }

    fn usage(code: &'static str, message: impl Into<Box<str>>) -> Self {
        Self::new(ExitClass::Usage, code, message)
    }

    fn security(code: &'static str, message: impl Into<Box<str>>) -> Self {
        Self::new(ExitClass::Security, code, message)
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl From<ReferencePackError> for CliError {
    fn from(source: ReferencePackError) -> Self {
        let class = match source.code() {
            ReferencePackErrorCode::InvalidRequest => ExitClass::Usage,
            ReferencePackErrorCode::UnsupportedLayout => ExitClass::Unavailable,
            ReferencePackErrorCode::Cancelled => ExitClass::Cancelled,
            ReferencePackErrorCode::IdentityMismatch
            | ReferencePackErrorCode::MemberInvalid
            | ReferencePackErrorCode::ManifestInvalid
            | ReferencePackErrorCode::BudgetExceeded => ExitClass::Security,
            ReferencePackErrorCode::ValidationFailed => ExitClass::Validation,
            ReferencePackErrorCode::SourceInputUnavailable
            | ReferencePackErrorCode::SerializationFailed => ExitClass::Build,
        };
        Self::new(class, "reference_pack_error", source.message())
    }
}

impl From<ServiceError> for CliError {
    fn from(source: ServiceError) -> Self {
        let class = match source.code() {
            ServiceErrorCode::InvalidConfiguration
            | ServiceErrorCode::InvalidRequest
            | ServiceErrorCode::InvalidContext
            | ServiceErrorCode::IdentityMismatch => ExitClass::Usage,
            ServiceErrorCode::Cancelled => ExitClass::Cancelled,
            ServiceErrorCode::BudgetExceeded
            | ServiceErrorCode::CanonicalizationFailed
            | ServiceErrorCode::InternalContractViolation => ExitClass::Security,
            ServiceErrorCode::OperationNotImplementedForMilestone
            | ServiceErrorCode::ComponentUnavailable => ExitClass::Unavailable,
            ServiceErrorCode::ExactGenerationUnavailable
            | ServiceErrorCode::CurrentGenerationUnavailable
            | ServiceErrorCode::ProjectTargetExcluded
            | ServiceErrorCode::ProjectTargetUnresolved
            | ServiceErrorCode::OperationConflict
            | ServiceErrorCode::OperationBusy
            | ServiceErrorCode::StoreCurrentConflict
            | ServiceErrorCode::StoreOutcomeUnknown => ExitClass::Build,
        };
        Self::new(class, "service_error", source.message())
    }
}

#[derive(Debug, Default)]
struct Options {
    values: BTreeMap<Box<str>, Box<str>>,
    flags: BTreeSet<Box<str>>,
}

impl Options {
    fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self, CliError> {
        let mut arguments = arguments.into_iter();
        let mut options = Self::default();
        while let Some(argument) = arguments.next() {
            if argument == "--json" {
                if !options.flags.insert(argument.into()) {
                    return Err(CliError::usage("duplicate_flag", "duplicate --json"));
                }
                continue;
            }
            if !argument.starts_with("--") {
                return Err(CliError::usage(
                    "unexpected_positional_argument",
                    "unexpected positional argument",
                ));
            }
            let value = arguments.next().ok_or_else(|| {
                CliError::usage("missing_option_value", "option is missing its value")
            })?;
            if value.starts_with("--") {
                return Err(CliError::usage(
                    "missing_option_value",
                    "option is missing its value",
                ));
            }
            if options
                .values
                .insert(argument.into(), value.into())
                .is_some()
            {
                return Err(CliError::usage("duplicate_option", "duplicate option"));
            }
        }
        Ok(options)
    }

    fn required(&self, name: &str) -> Result<&str, CliError> {
        self.values
            .get(name)
            .map(AsRef::as_ref)
            .ok_or_else(|| CliError::usage("missing_required_option", format!("missing {name}")))
    }

    fn optional(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(AsRef::as_ref)
    }

    fn flag(&self, name: &str) -> bool {
        self.flags.contains(name)
    }

    fn reject_unknown(
        &self,
        allowed_values: &[&str],
        allowed_flags: &[&str],
    ) -> Result<(), CliError> {
        if self
            .values
            .keys()
            .any(|name| !allowed_values.contains(&name.as_ref()))
            || self
                .flags
                .iter()
                .any(|name| !allowed_flags.contains(&name.as_ref()))
        {
            return Err(CliError::usage("unknown_option", "unknown option"));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildCommandRequest {
    schema: Box<str>,
    source_config: Box<str>,
    request: ReferencePackBuildRequest,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidationExpectation {
    schema: Box<str>,
    request: ReferencePackValidationRequest,
    eligibility_target: ReferencePackEligibilityTarget,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RebuildCommandRequest {
    schema: Box<str>,
    source_config: Box<str>,
    request: ReferencePackRebuildComparisonRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum DestinationState {
    NotFinalized,
    Created,
    Replaced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum CleanupState {
    NotRequired,
    Completed,
    Pending,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct BuildCommandResult<'a> {
    schema: &'static str,
    status: ReferencePackBuildStatus,
    eligibility_target: ReferencePackEligibilityTarget,
    pack_id: &'a str,
    plan_id: &'a str,
    staging_validation_report_id: &'a str,
    final_validation_report_id: &'a str,
    candidate_eligible: bool,
    validated_local_eligible: bool,
    member_count: usize,
    total_bytes: u64,
    destination_state: DestinationState,
    prior_destination_pack_id: Option<Box<str>>,
    backup_cleanup: CleanupState,
    #[serde(skip_serializing_if = "Option::is_none")]
    backup_cleanup_path: Option<Box<str>>,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct ValidateCommandResult<'a> {
    schema: &'static str,
    pack_id: &'a str,
    validation_report_id: &'a str,
    eligibility_target: ReferencePackEligibilityTarget,
    candidate_eligible: bool,
    validated_local_eligible: bool,
    member_count: usize,
    total_bytes: u64,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct RebuildCommandResult<'a> {
    schema: &'static str,
    report_id: &'a str,
    status: RebuildComparisonStatus,
    left_execution_profile_id: &'a str,
    right_execution_profile_id: &'a str,
    difference_count: usize,
    scratch_read_back: &'static str,
}

pub fn run(
    arguments: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let stop = AtomicBool::new(false);
    run_with_stop(arguments, stdout, stderr, &stop)
}

pub fn run_with_stop(
    arguments: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    stop: &AtomicBool,
) -> i32 {
    match execute(arguments, stdout, stop) {
        Ok(class) => class.code(),
        Err(error) => {
            let _ = writeln!(stderr, "{}: {}", error.code, error.message);
            error.class.code()
        }
    }
}

fn execute(
    arguments: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stop: &AtomicBool,
) -> Result<ExitClass, CliError> {
    checkpoint(stop)?;
    let mut arguments = arguments.into_iter();
    let command = arguments
        .next()
        .ok_or_else(|| CliError::usage("missing_command", "missing command"))?;
    let options = Options::parse(arguments)?;
    match command.as_str() {
        "build" => build(&options, stdout, stop),
        "validate" => validate(&options, stdout, stop),
        "rebuild-compare" => rebuild_compare(&options, stdout, stop),
        _ => Err(CliError::usage("unknown_command", "unknown command")),
    }
}

fn build(
    options: &Options,
    stdout: &mut dyn Write,
    stop: &AtomicBool,
) -> Result<ExitClass, CliError> {
    options.reject_unknown(&["--request", "--source-root", "--output"], &["--json"])?;
    let request_bytes = read_explicit_file(
        Path::new(options.required("--request")?),
        MAX_REQUEST_BYTES,
        "build request",
    )?;
    let command: BuildCommandRequest = serde_json::from_slice(&request_bytes)
        .map_err(|_| CliError::usage("invalid_build_request", "invalid build request JSON"))?;
    if command.schema.as_ref() != BUILD_REQUEST_SCHEMA {
        return Err(CliError::usage(
            "unsupported_build_request_schema",
            "unsupported build request schema",
        ));
    }

    let source_root =
        canonical_existing_directory(Path::new(options.required("--source-root")?), "source root")?;
    let source_config =
        confined_existing_file(&source_root, &command.source_config, "source config")?;
    let output = destination_path(Path::new(options.required("--output")?))?;
    reject_overlap(&source_root, &output, "source and output roots overlap")?;
    checkpoint(stop)?;

    let input = LocalProjectInput::from_config_path(&source_config, stop)?;
    let outcome = ReferencePackService::reference_pack_build(&command.request, &input, stop)?;
    let materialized = materialize_and_finalize(
        outcome.plan(),
        outcome.validation_report(),
        &command.request,
        &output,
        stop,
    )?;
    let result = BuildCommandResult {
        schema: BUILD_RESULT_SCHEMA,
        status: outcome.status(),
        eligibility_target: command.request.eligibility_target(),
        pack_id: outcome.plan().pack_id(),
        plan_id: outcome.plan().plan_id(),
        staging_validation_report_id: materialized.staging_report.report_id(),
        final_validation_report_id: materialized.final_report.report_id(),
        candidate_eligible: materialized.final_report.candidate_eligible(),
        validated_local_eligible: materialized.final_report.validated_local_eligible(),
        member_count: materialized.member_count,
        total_bytes: materialized.total_bytes,
        destination_state: materialized.destination_state,
        prior_destination_pack_id: materialized.prior_destination_pack_id,
        backup_cleanup: materialized.backup_cleanup,
        backup_cleanup_path: materialized.backup_cleanup_path,
    };
    emit_build(stdout, &result, options.flag("--json"))?;
    Ok(eligibility_exit(
        command.request.eligibility_target(),
        &materialized.final_report,
    ))
}

fn validate(
    options: &Options,
    stdout: &mut dyn Write,
    stop: &AtomicBool,
) -> Result<ExitClass, CliError> {
    options.reject_unknown(&["--pack", "--expect"], &["--json"])?;
    let root = canonical_existing_directory(Path::new(options.required("--pack")?), "pack root")?;
    checkpoint(stop)?;

    let default_budgets = ReferencePackBudgets::default();
    let expectation = match options.optional("--expect") {
        Some(path) => {
            let bytes = read_explicit_file(Path::new(path), MAX_EXPECTATION_BYTES, "expectation")?;
            let expectation: ValidationExpectation =
                serde_json::from_slice(&bytes).map_err(|_| {
                    CliError::usage(
                        "invalid_validation_expectation",
                        "invalid validation expectation JSON",
                    )
                })?;
            if expectation.schema.as_ref() != VALIDATION_EXPECTATION_SCHEMA {
                return Err(CliError::usage(
                    "unsupported_validation_expectation_schema",
                    "unsupported validation expectation schema",
                ));
            }
            Some(expectation)
        }
        None => None,
    };
    let budgets = expectation
        .as_ref()
        .map_or(default_budgets, |value| value.request.budgets());
    let loaded = load_pack_image(&root, budgets, stop)?;
    let manifest = manifest_from_image(&loaded.image)?;
    let (request, eligibility_target) = match expectation {
        Some(value) => (value.request, value.eligibility_target),
        None => (
            ReferencePackValidationRequest::new(
                manifest.pack_id(),
                manifest.profile().profile_id().to_string(),
                manifest.reference_generation_id(),
                default_budgets,
            )?,
            ReferencePackEligibilityTarget::Candidate,
        ),
    };
    let report = ReferencePackService::reference_pack_validate(&request, &loaded.image, stop)?;
    let result = ValidateCommandResult {
        schema: VALIDATE_RESULT_SCHEMA,
        pack_id: report.pack_id(),
        validation_report_id: report.report_id(),
        eligibility_target,
        candidate_eligible: report.candidate_eligible(),
        validated_local_eligible: report.validated_local_eligible(),
        member_count: loaded.member_count,
        total_bytes: loaded.total_bytes,
    };
    emit_validate(stdout, &result, options.flag("--json"))?;
    Ok(eligibility_exit(eligibility_target, &report))
}

fn rebuild_compare(
    options: &Options,
    stdout: &mut dyn Write,
    stop: &AtomicBool,
) -> Result<ExitClass, CliError> {
    options.reject_unknown(
        &["--request", "--source-root", "--scratch-root"],
        &["--json"],
    )?;
    let request_bytes = read_explicit_file(
        Path::new(options.required("--request")?),
        MAX_REQUEST_BYTES,
        "rebuild request",
    )?;
    let command: RebuildCommandRequest = serde_json::from_slice(&request_bytes)
        .map_err(|_| CliError::usage("invalid_rebuild_request", "invalid rebuild request JSON"))?;
    if command.schema.as_ref() != REBUILD_REQUEST_SCHEMA {
        return Err(CliError::usage(
            "unsupported_rebuild_request_schema",
            "unsupported rebuild request schema",
        ));
    }
    let source_root =
        canonical_existing_directory(Path::new(options.required("--source-root")?), "source root")?;
    let scratch_root = canonical_existing_directory(
        Path::new(options.required("--scratch-root")?),
        "scratch root",
    )?;
    reject_overlap(
        &source_root,
        &scratch_root,
        "source and scratch roots overlap",
    )?;
    let source_config =
        confined_existing_file(&source_root, &command.source_config, "source config")?;
    let input = LocalProjectInput::from_config_path(&source_config, stop)?;
    let report =
        ReferencePackService::reference_pack_rebuild_compare(&command.request, &input, stop)?;
    let bytes = report.canonical_bytes()?;
    let scratch = create_isolated_directory(&scratch_root, "rebuild", report.report_id())?;
    let report_path = scratch.path.join("rebuild-comparison.json");
    let write_result = write_new_file(&report_path, &bytes);
    if let Err(error) = write_result {
        let _ = scratch.cleanup();
        return Err(error);
    }
    let read_back = read_explicit_file(&report_path, bytes.len() as u64, "rebuild report")?;
    let reopened = ReferencePackRebuildComparisonReport::from_canonical_slice(&read_back)?;
    if reopened.report_id() != report.report_id() || reopened.status() != report.status() {
        let _ = scratch.cleanup();
        return Err(CliError::security(
            "rebuild_report_read_back_mismatch",
            "rebuild report read-back differs from the service result",
        ));
    }
    scratch.cleanup()?;
    let result = RebuildCommandResult {
        schema: REBUILD_RESULT_SCHEMA,
        report_id: report.report_id(),
        status: report.status(),
        left_execution_profile_id: report.left_execution_profile_id(),
        right_execution_profile_id: report.right_execution_profile_id(),
        difference_count: report.differences().len(),
        scratch_read_back: "passed",
    };
    emit_rebuild(stdout, &result, options.flag("--json"))?;
    Ok(match report.status() {
        RebuildComparisonStatus::Passed => ExitClass::Success,
        RebuildComparisonStatus::Failed => ExitClass::Validation,
    })
}

fn eligibility_exit(
    target: ReferencePackEligibilityTarget,
    report: &ReferencePackValidationReport,
) -> ExitClass {
    let eligible = match target {
        ReferencePackEligibilityTarget::Candidate => report.candidate_eligible(),
        ReferencePackEligibilityTarget::ValidatedLocal => report.validated_local_eligible(),
    };
    if eligible {
        ExitClass::Success
    } else {
        ExitClass::Blocked
    }
}

struct MaterializationOutcome {
    staging_report: ReferencePackValidationReport,
    final_report: ReferencePackValidationReport,
    member_count: usize,
    total_bytes: u64,
    destination_state: DestinationState,
    prior_destination_pack_id: Option<Box<str>>,
    backup_cleanup: CleanupState,
    backup_cleanup_path: Option<Box<str>>,
}

fn materialize_and_finalize(
    plan: &PackMaterializationPlan,
    original_report: &ReferencePackValidationReport,
    request: &ReferencePackBuildRequest,
    output: &Path,
    stop: &AtomicBool,
) -> Result<MaterializationOutcome, CliError> {
    checkpoint(stop)?;
    let parent = output
        .parent()
        .ok_or_else(|| CliError::security("invalid_output_parent", "output has no parent"))?;
    let mut staging = create_isolated_directory(parent, "staging", plan.pack_id())?;
    if let Err(error) = write_materialization_plan(&staging.path, plan, stop) {
        let _ = staging.cleanup();
        return Err(error);
    }
    sync_directory(&staging.path)?;
    let staged = load_pack_image(&staging.path, request.budgets(), stop)?;
    let validation_request = ReferencePackValidationRequest::new(
        plan.pack_id(),
        request.expected_profile_id(),
        request.expected_reference_generation_id(),
        request.budgets(),
    )?;
    let staging_report =
        ReferencePackService::reference_pack_validate(&validation_request, &staged.image, stop)?;
    if staging_report.report_id() != original_report.report_id() {
        let _ = staging.cleanup();
        return Err(CliError::security(
            "staging_validation_mismatch",
            "staging validation differs from the service build result",
        ));
    }
    if !staging_report.candidate_eligible() {
        let member_count = staged.member_count;
        let total_bytes = staged.total_bytes;
        staging.cleanup()?;
        return Ok(MaterializationOutcome {
            final_report: staging_report.clone(),
            staging_report,
            member_count,
            total_bytes,
            destination_state: DestinationState::NotFinalized,
            prior_destination_pack_id: None,
            backup_cleanup: CleanupState::NotRequired,
            backup_cleanup_path: None,
        });
    }
    checkpoint(stop)?;

    let prior_destination = observe_prior_destination(output, request.budgets(), stop)?;
    let destination_state = if prior_destination.is_some() {
        DestinationState::Replaced
    } else {
        DestinationState::Created
    };
    let prior_destination_pack_id = prior_destination
        .as_ref()
        .map(|value| value.pack_id.clone());
    let backup = if prior_destination.is_some() {
        Some(unique_sibling_path(parent, "backup", plan.pack_id())?)
    } else {
        None
    };

    if let Some(path) = &backup {
        fs::rename(output, path).map_err(|_| {
            CliError::security(
                "destination_backup_failed",
                "existing destination could not be moved to an isolated backup",
            )
        })?;
        sync_directory(parent)?;
    }

    if fs::rename(&staging.path, output).is_err() {
        if let Some(path) = &backup {
            let _ = fs::rename(path, output);
            let _ = sync_directory(parent);
        }
        return Err(CliError::security(
            "destination_finalization_failed",
            "staged pack could not be atomically installed",
        ));
    }
    staging.disarm();
    sync_directory(parent)?;

    let final_result = (|| {
        checkpoint(stop)?;
        let final_image = load_pack_image(output, request.budgets(), stop)?;
        let final_report = ReferencePackService::reference_pack_validate(
            &validation_request,
            &final_image.image,
            stop,
        )?;
        if final_report.report_id() != staging_report.report_id()
            || final_report.pack_id() != plan.pack_id()
        {
            return Err(CliError::security(
                "final_read_back_mismatch",
                "final pack read-back differs from validated staging",
            ));
        }
        Ok((final_image, final_report))
    })();

    let (final_image, final_report) = match final_result {
        Ok(value) => value,
        Err(error) => {
            rollback_finalization(output, backup.as_deref(), parent, plan.pack_id());
            return Err(error);
        }
    };

    let (backup_cleanup, backup_cleanup_path) = match backup {
        Some(path) => match remove_tree(&path) {
            Ok(()) => {
                sync_directory(parent)?;
                (CleanupState::Completed, None)
            }
            Err(_) => (
                CleanupState::Pending,
                Some(path.to_string_lossy().into_owned().into_boxed_str()),
            ),
        },
        None => (CleanupState::NotRequired, None),
    };

    Ok(MaterializationOutcome {
        staging_report,
        final_report,
        member_count: final_image.member_count,
        total_bytes: final_image.total_bytes,
        destination_state,
        prior_destination_pack_id,
        backup_cleanup,
        backup_cleanup_path,
    })
}

struct PriorDestination {
    pack_id: Box<str>,
}

fn observe_prior_destination(
    output: &Path,
    budgets: ReferencePackBudgets,
    stop: &AtomicBool,
) -> Result<Option<PriorDestination>, CliError> {
    let metadata = match fs::symlink_metadata(output) {
        Ok(value) => value,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(_) => {
            return Err(CliError::security(
                "destination_metadata_unavailable",
                "destination metadata is unavailable",
            ));
        }
    };
    if is_reparse_or_symlink(&metadata) || !metadata.is_dir() {
        return Err(CliError::security(
            "unsafe_existing_destination",
            "existing destination is not a safe directory",
        ));
    }
    let loaded = load_pack_image(output, budgets, stop)?;
    let manifest = manifest_from_image(&loaded.image)?;
    let request = ReferencePackValidationRequest::new(
        manifest.pack_id(),
        manifest.profile().profile_id().to_string(),
        manifest.reference_generation_id(),
        budgets,
    )?;
    let report = ReferencePackService::reference_pack_validate(&request, &loaded.image, stop)?;
    if !report.candidate_eligible() {
        return Err(CliError::new(
            ExitClass::Validation,
            "existing_destination_not_candidate_eligible",
            "existing destination is not a valid candidate pack",
        ));
    }
    Ok(Some(PriorDestination {
        pack_id: manifest.pack_id().into(),
    }))
}

fn rollback_finalization(output: &Path, backup: Option<&Path>, parent: &Path, token: &str) {
    let quarantine = unique_sibling_path(parent, "quarantine", token).ok();
    if output.exists()
        && let Some(path) = quarantine.as_deref()
    {
        let _ = fs::rename(output, path);
    }
    if let Some(path) = backup {
        let _ = fs::rename(path, output);
    }
    let _ = sync_directory(parent);
}

fn write_materialization_plan(
    root: &Path,
    plan: &PackMaterializationPlan,
    stop: &AtomicBool,
) -> Result<(), CliError> {
    for directory in plan.directories() {
        checkpoint(stop)?;
        create_relative_directory(root, directory)?;
    }
    for entry in plan.entries() {
        checkpoint(stop)?;
        let relative = safe_relative_path(entry.member().path())?;
        let target = root.join(&relative);
        if let Some(parent) = target.parent() {
            create_directory_chain(root, parent)?;
        }
        write_new_file(&target, entry.bytes())?;
        let bytes =
            read_explicit_file(&target, entry.member().byte_length(), "staged pack member")?;
        if bytes.len() as u64 != entry.member().byte_length()
            || sha256_identifier(&bytes) != entry.member().sha256()
        {
            return Err(CliError::security(
                "staged_member_digest_mismatch",
                "staged pack member does not match the service plan",
            ));
        }
    }
    Ok(())
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), CliError> {
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|_| {
        CliError::security(
            "file_create_failed",
            "pack member could not be created safely",
        )
    })?;
    file.write_all(bytes).map_err(|_| {
        CliError::new(
            ExitClass::Build,
            "file_write_failed",
            "pack member write failed",
        )
    })?;
    file.sync_all().map_err(|_| {
        CliError::new(
            ExitClass::Build,
            "file_sync_failed",
            "pack member sync failed",
        )
    })?;
    Ok(())
}

struct LoadedPack {
    image: ReferencePackImage,
    member_count: usize,
    total_bytes: u64,
}

fn load_pack_image(
    root: &Path,
    budgets: ReferencePackBudgets,
    stop: &AtomicBool,
) -> Result<LoadedPack, CliError> {
    let root = canonical_existing_directory(root, "pack root")?;
    let mut directories = vec![root.clone()];
    let mut files = Vec::new();
    let mut total_bytes = 0_u64;
    let mut casefolded = BTreeSet::new();

    while let Some(directory) = directories.pop() {
        checkpoint(stop)?;
        let mut entries = fs::read_dir(&directory)
            .map_err(|_| {
                CliError::security("pack_read_dir_failed", "pack directory cannot be listed")
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| {
                CliError::security("pack_read_dir_failed", "pack directory cannot be listed")
            })?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            checkpoint(stop)?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|_| {
                CliError::security(
                    "pack_metadata_failed",
                    "pack member metadata is unavailable",
                )
            })?;
            if is_reparse_or_symlink(&metadata) {
                return Err(CliError::security(
                    "pack_symlink_or_reparse",
                    "pack contains a symlink or reparse point",
                ));
            }
            if metadata.is_dir() {
                directories.push(path);
                continue;
            }
            if !metadata.is_file() {
                return Err(CliError::security(
                    "pack_special_file",
                    "pack contains an unsupported special file",
                ));
            }
            if files.len() >= budgets.max_members() as usize {
                return Err(CliError::security(
                    "pack_member_budget_exceeded",
                    "pack contains too many files",
                ));
            }
            if metadata.len() > budgets.max_member_bytes() {
                return Err(CliError::security(
                    "pack_member_size_exceeded",
                    "pack member exceeds its byte budget",
                ));
            }
            total_bytes = total_bytes.checked_add(metadata.len()).ok_or_else(|| {
                CliError::security("pack_total_size_overflow", "pack byte count overflowed")
            })?;
            if total_bytes > budgets.max_total_bytes() {
                return Err(CliError::security(
                    "pack_total_budget_exceeded",
                    "pack exceeds its total byte budget",
                ));
            }
            let relative = path.strip_prefix(&root).map_err(|_| {
                CliError::security("pack_path_escape", "pack member escaped its root")
            })?;
            let relative = normalized_relative_string(relative)?;
            if !casefolded.insert(relative.to_ascii_lowercase()) {
                return Err(CliError::security(
                    "pack_case_collision",
                    "pack contains a case-insensitive path collision",
                ));
            }
            let bytes = read_explicit_file(&path, budgets.max_member_bytes(), "pack member")?;
            files.push(PackImageFile::new(relative, bytes));
        }
    }
    files.sort_by(|left, right| left.path().cmp(right.path()));
    if files.is_empty() {
        return Err(CliError::security("empty_pack", "pack directory is empty"));
    }
    let member_count = files.len();
    Ok(LoadedPack {
        image: ReferencePackImage::new(files),
        member_count,
        total_bytes,
    })
}

fn manifest_from_image(image: &ReferencePackImage) -> Result<ReferencePackManifest, CliError> {
    let bytes = image
        .files()
        .iter()
        .find(|file| file.path() == "manifest.json")
        .map(PackImageFile::bytes)
        .ok_or_else(|| CliError::security("manifest_missing", "pack manifest is missing"))?;
    serde_json::from_slice(bytes)
        .map_err(|_| CliError::security("manifest_invalid", "pack manifest JSON is invalid"))
}

fn read_explicit_file(path: &Path, max_bytes: u64, label: &str) -> Result<Vec<u8>, CliError> {
    reject_existing_reparse_components(path)?;
    let metadata = fs::symlink_metadata(path).map_err(|_| {
        CliError::security(
            "input_metadata_unavailable",
            format!("{label} metadata is unavailable"),
        )
    })?;
    if is_reparse_or_symlink(&metadata) || !metadata.is_file() || metadata.len() > max_bytes {
        return Err(CliError::security(
            "unsafe_or_oversized_input",
            format!("{label} is not a safe bounded regular file"),
        ));
    }
    let file = File::open(path).map_err(|_| {
        CliError::security("input_open_failed", format!("{label} could not be opened"))
    })?;
    let mut bytes = Vec::new();
    file.take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| CliError::security("input_read_failed", format!("{label} read failed")))?;
    if bytes.len() as u64 > max_bytes || bytes.len() as u64 != metadata.len() {
        return Err(CliError::security(
            "input_changed_or_oversized",
            format!("{label} changed while being read or exceeded its budget"),
        ));
    }
    Ok(bytes)
}

fn canonical_existing_directory(path: &Path, label: &str) -> Result<PathBuf, CliError> {
    reject_existing_reparse_components(path)?;
    let canonical = fs::canonicalize(path).map_err(|_| {
        CliError::security("directory_unavailable", format!("{label} is unavailable"))
    })?;
    let metadata = fs::symlink_metadata(&canonical).map_err(|_| {
        CliError::security(
            "directory_metadata_unavailable",
            format!("{label} metadata is unavailable"),
        )
    })?;
    if is_reparse_or_symlink(&metadata) || !metadata.is_dir() {
        return Err(CliError::security(
            "unsafe_directory",
            format!("{label} is not a safe directory"),
        ));
    }
    Ok(canonical)
}

fn confined_existing_file(root: &Path, relative: &str, label: &str) -> Result<PathBuf, CliError> {
    let relative = safe_relative_path(relative)?;
    let joined = root.join(relative);
    reject_existing_reparse_components(&joined)?;
    let canonical = fs::canonicalize(&joined).map_err(|_| {
        CliError::security(
            "confined_file_unavailable",
            format!("{label} is unavailable"),
        )
    })?;
    if !canonical.starts_with(root) {
        return Err(CliError::security(
            "confined_path_escape",
            format!("{label} escaped its root"),
        ));
    }
    let metadata = fs::symlink_metadata(&canonical).map_err(|_| {
        CliError::security(
            "confined_file_metadata_unavailable",
            format!("{label} metadata is unavailable"),
        )
    })?;
    if is_reparse_or_symlink(&metadata) || !metadata.is_file() {
        return Err(CliError::security(
            "unsafe_confined_file",
            format!("{label} is not a safe regular file"),
        ));
    }
    Ok(canonical)
}

fn destination_path(path: &Path) -> Result<PathBuf, CliError> {
    let file_name = path
        .file_name()
        .filter(|name| !name.is_empty())
        .ok_or_else(|| CliError::security("invalid_output_path", "output must name a directory"))?;
    if file_name == OsStr::new(".") || file_name == OsStr::new("..") {
        return Err(CliError::security(
            "invalid_output_path",
            "output must name a normal directory",
        ));
    }
    let parent = path.parent().unwrap_or(Path::new("."));
    let parent = canonical_existing_directory(parent, "output parent")?;
    let output = parent.join(file_name);
    if let Ok(metadata) = fs::symlink_metadata(&output)
        && is_reparse_or_symlink(&metadata)
    {
        return Err(CliError::security(
            "unsafe_output_path",
            "output is a symlink or reparse point",
        ));
    }
    Ok(output)
}

fn reject_overlap(left: &Path, right: &Path, message: &'static str) -> Result<(), CliError> {
    if left == right || left.starts_with(right) || right.starts_with(left) {
        return Err(CliError::security("root_overlap", message));
    }
    Ok(())
}

fn safe_relative_path(value: &str) -> Result<PathBuf, CliError> {
    if value.is_empty() || value.contains('\\') {
        return Err(CliError::security(
            "invalid_relative_path",
            "relative path is empty or noncanonical",
        ));
    }
    let path = Path::new(value);
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(CliError::security(
            "invalid_relative_path",
            "relative path contains traversal or an absolute prefix",
        ));
    }
    Ok(path.to_path_buf())
}

fn normalized_relative_string(path: &Path) -> Result<String, CliError> {
    let mut parts = Vec::new();
    for component in path.components() {
        let Component::Normal(value) = component else {
            return Err(CliError::security(
                "invalid_pack_path",
                "pack path is not a canonical relative path",
            ));
        };
        let value = value
            .to_str()
            .ok_or_else(|| CliError::security("non_utf8_pack_path", "pack path is not UTF-8"))?;
        if value.is_empty() || value == "." || value == ".." {
            return Err(CliError::security(
                "invalid_pack_path",
                "pack path contains an invalid segment",
            ));
        }
        parts.push(value);
    }
    if parts.is_empty() {
        return Err(CliError::security("empty_pack_path", "pack path is empty"));
    }
    Ok(parts.join("/"))
}

fn reject_existing_reparse_components(path: &Path) -> Result<(), CliError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|_| {
                CliError::security(
                    "current_directory_unavailable",
                    "current directory is unavailable",
                )
            })?
            .join(path)
    };
    let mut current = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => current.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(CliError::security(
                    "parent_traversal",
                    "path contains parent traversal",
                ));
            }
            Component::Normal(value) => {
                current.push(value);
                match fs::symlink_metadata(&current) {
                    Ok(metadata) if is_reparse_or_symlink(&metadata) => {
                        return Err(CliError::security(
                            "symlink_or_reparse_component",
                            "path contains a symlink or reparse point",
                        ));
                    }
                    Ok(_) => {}
                    Err(error) if error.kind() == io::ErrorKind::NotFound => break,
                    Err(_) => {
                        return Err(CliError::security(
                            "path_component_unavailable",
                            "path component metadata is unavailable",
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

fn is_reparse_or_symlink(metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
        return metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0;
    }
    #[cfg(not(windows))]
    {
        false
    }
}

fn create_relative_directory(root: &Path, relative: &str) -> Result<(), CliError> {
    let relative = safe_relative_path(relative)?;
    create_directory_chain(root, &root.join(relative))
}

fn create_directory_chain(root: &Path, target: &Path) -> Result<(), CliError> {
    let relative = target
        .strip_prefix(root)
        .map_err(|_| CliError::security("directory_escape", "directory escaped staging root"))?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(value) = component else {
            return Err(CliError::security(
                "invalid_directory_path",
                "directory path is not canonical",
            ));
        };
        current.push(value);
        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if is_reparse_or_symlink(&metadata) || !metadata.is_dir() {
                    return Err(CliError::security(
                        "unsafe_staging_directory",
                        "staging path collides with a non-directory or link",
                    ));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                create_private_directory(&current)?;
            }
            Err(_) => {
                return Err(CliError::security(
                    "staging_directory_metadata_failed",
                    "staging directory metadata is unavailable",
                ));
            }
        }
    }
    Ok(())
}

fn create_private_directory(path: &Path) -> Result<(), CliError> {
    fs::create_dir(path).map_err(|_| {
        CliError::security(
            "directory_create_failed",
            "isolated directory could not be created",
        )
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|_| {
            CliError::security(
                "directory_mode_failed",
                "isolated directory mode could not be set",
            )
        })?;
    }
    Ok(())
}

struct IsolatedDirectory {
    path: PathBuf,
    armed: bool,
}

impl IsolatedDirectory {
    fn disarm(&mut self) {
        self.armed = false;
    }

    fn cleanup(mut self) -> Result<(), CliError> {
        self.armed = false;
        remove_tree(&self.path)
    }
}

impl Drop for IsolatedDirectory {
    fn drop(&mut self) {
        if self.armed {
            let _ = remove_tree(&self.path);
        }
    }
}

fn create_isolated_directory(
    parent: &Path,
    purpose: &str,
    token: &str,
) -> Result<IsolatedDirectory, CliError> {
    for attempt in 0..MAX_STAGING_ATTEMPTS {
        let name = format!(
            ".wow-reference-builder-{purpose}-{}-{}-{attempt}",
            std::process::id(),
            short_token(token)
        );
        let path = parent.join(name);
        match create_private_directory(&path) {
            Ok(()) => return Ok(IsolatedDirectory { path, armed: true }),
            Err(error) if path.exists() => {
                let _ = error;
            }
            Err(error) => return Err(error),
        }
    }
    Err(CliError::security(
        "isolated_directory_exhausted",
        "could not allocate an isolated directory name",
    ))
}

fn unique_sibling_path(parent: &Path, purpose: &str, token: &str) -> Result<PathBuf, CliError> {
    for attempt in 0..MAX_STAGING_ATTEMPTS {
        let name = format!(
            ".wow-reference-builder-{purpose}-{}-{}-{attempt}",
            std::process::id(),
            short_token(token)
        );
        let path = parent.join(name);
        if !path.exists() {
            return Ok(path);
        }
    }
    Err(CliError::security(
        "sibling_path_exhausted",
        "could not allocate an isolated sibling path",
    ))
}

fn remove_tree(path: &Path) -> Result<(), CliError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(value) => value,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(_) => {
            return Err(CliError::security(
                "cleanup_metadata_failed",
                "cleanup target metadata is unavailable",
            ));
        }
    };
    if is_reparse_or_symlink(&metadata) || !metadata.is_dir() {
        return Err(CliError::security(
            "unsafe_cleanup_target",
            "cleanup target is not a safe directory",
        ));
    }
    fs::remove_dir_all(path)
        .map_err(|_| CliError::new(ExitClass::Build, "cleanup_failed", "cleanup failed"))
}

fn sync_directory(path: &Path) -> Result<(), CliError> {
    #[cfg(unix)]
    {
        File::open(path)
            .and_then(|file| file.sync_all())
            .map_err(|_| {
                CliError::new(
                    ExitClass::Build,
                    "directory_sync_failed",
                    "directory sync failed",
                )
            })?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

fn checkpoint(stop: &AtomicBool) -> Result<(), CliError> {
    if stop.load(Ordering::Acquire) {
        return Err(CliError::new(
            ExitClass::Cancelled,
            "cancelled",
            "operation was cancelled",
        ));
    }
    Ok(())
}

fn short_token(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    hex(&digest)[..16].to_owned()
}

fn sha256_identifier(bytes: &[u8]) -> String {
    format!("sha256:{}", hex(&Sha256::digest(bytes)))
}

fn hex(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(TABLE[(byte >> 4) as usize] as char);
        output.push(TABLE[(byte & 0x0f) as usize] as char);
    }
    output
}

fn emit_json(writer: &mut dyn Write, value: &impl Serialize) -> Result<(), CliError> {
    let mut bytes = serde_json::to_vec(value).map_err(|_| {
        CliError::new(
            ExitClass::Build,
            "result_serialization_failed",
            "result serialization failed",
        )
    })?;
    bytes.push(b'\n');
    write_output(writer, &bytes)
}

fn emit_build(
    writer: &mut dyn Write,
    value: &BuildCommandResult<'_>,
    json: bool,
) -> Result<(), CliError> {
    if json {
        return emit_json(writer, value);
    }
    let text = format!(
        "build {}\npack_id={}\nplan_id={}\nfinal_validation_report_id={}\ncandidate_eligible={}\nvalidated_local_eligible={}\ndestination_state={:?}\nbackup_cleanup={:?}\n",
        match value.status {
            ReferencePackBuildStatus::CandidateReady => "candidate_ready",
            ReferencePackBuildStatus::Blocked => "blocked",
        },
        value.pack_id,
        value.plan_id,
        value.final_validation_report_id,
        value.candidate_eligible,
        value.validated_local_eligible,
        value.destination_state,
        value.backup_cleanup,
    );
    write_output(writer, text.as_bytes())
}

fn emit_validate(
    writer: &mut dyn Write,
    value: &ValidateCommandResult<'_>,
    json: bool,
) -> Result<(), CliError> {
    if json {
        return emit_json(writer, value);
    }
    let text = format!(
        "validate\npack_id={}\nvalidation_report_id={}\ncandidate_eligible={}\nvalidated_local_eligible={}\n",
        value.pack_id,
        value.validation_report_id,
        value.candidate_eligible,
        value.validated_local_eligible,
    );
    write_output(writer, text.as_bytes())
}

fn emit_rebuild(
    writer: &mut dyn Write,
    value: &RebuildCommandResult<'_>,
    json: bool,
) -> Result<(), CliError> {
    if json {
        return emit_json(writer, value);
    }
    let text = format!(
        "rebuild_compare {:?}\nreport_id={}\nleft_execution_profile_id={}\nright_execution_profile_id={}\ndifference_count={}\nscratch_read_back={}\n",
        value.status,
        value.report_id,
        value.left_execution_profile_id,
        value.right_execution_profile_id,
        value.difference_count,
        value.scratch_read_back,
    );
    write_output(writer, text.as_bytes())
}

fn write_output(writer: &mut dyn Write, bytes: &[u8]) -> Result<(), CliError> {
    match writer.write_all(bytes) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(_) => Err(CliError::new(
            ExitClass::Build,
            "output_write_failed",
            "output write failed",
        )),
    }
}
