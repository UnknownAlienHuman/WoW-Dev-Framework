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
use wow_service::reference_pack_materialization::{
    ReferencePackMaterializationAction, ReferencePackMaterializationConfiguration,
    ReferencePackMaterializationDestinationState, ReferencePackMaterializationError,
    ReferencePackMaterializationErrorCode, ReferencePackMaterializationObservation,
    ReferencePackMaterializationOperationId, ReferencePackMaterializationRequest,
    ReferencePackMaterializationService, ReferencePackMaterializationStage,
    ReferencePackMaterializationStoreLimits, ReferencePackPathObservation,
};
use wow_service::{LocalProjectInput, ServiceError, ServiceErrorCode};

const BUILD_REQUEST_SCHEMA: &str = "wow-reference-builder/build-request/1";
const VALIDATION_EXPECTATION_SCHEMA: &str = "wow-reference-builder/validation-expectation/1";
const REBUILD_REQUEST_SCHEMA: &str = "wow-reference-builder/rebuild-request/1";
const BUILD_RESULT_SCHEMA: &str = "wow-reference-builder/build-result/2";
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

impl From<ReferencePackMaterializationError> for CliError {
    fn from(source: ReferencePackMaterializationError) -> Self {
        let class = match source.code() {
            ReferencePackMaterializationErrorCode::ConfigurationInvalid
            | ReferencePackMaterializationErrorCode::InvalidRequest => ExitClass::Usage,
            ReferencePackMaterializationErrorCode::StateInvalid
            | ReferencePackMaterializationErrorCode::ReceiptInvalid => ExitClass::Security,
            ReferencePackMaterializationErrorCode::OperationConflict
            | ReferencePackMaterializationErrorCode::OperationIncomplete
            | ReferencePackMaterializationErrorCode::OutcomeUnknown
            | ReferencePackMaterializationErrorCode::StoreFailure => ExitClass::Build,
        };
        Self::new(class, "materialization_error", source.message())
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
    #[serde(skip_serializing_if = "Option::is_none")]
    materialization_operation_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    materialization_receipt_id: Option<&'a str>,
    materialization_replayed: bool,
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
        materialization_operation_id: materialized.materialization_operation_id.as_deref(),
        materialization_receipt_id: materialized.materialization_receipt_id.as_deref(),
        materialization_replayed: materialized.materialization_replayed,
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
    materialization_operation_id: Option<Box<str>>,
    materialization_receipt_id: Option<Box<str>>,
    materialization_replayed: bool,
}

fn materialize_and_finalize(
    plan: &PackMaterializationPlan,
    original_report: &ReferencePackValidationReport,
    request: &ReferencePackBuildRequest,
    output: &Path,
    stop: &AtomicBool,
) -> Result<MaterializationOutcome, CliError> {
    if !original_report.candidate_eligible() {
        let member_count = plan.entries().len();
        let total_bytes = plan.entries().iter().try_fold(0_u64, |total, entry| {
            total
                .checked_add(entry.bytes().len() as u64)
                .ok_or_else(|| {
                    CliError::security("pack_total_size_overflow", "pack byte count overflowed")
                })
        })?;
        return Ok(MaterializationOutcome {
            staging_report: original_report.clone(),
            final_report: original_report.clone(),
            member_count,
            total_bytes,
            destination_state: DestinationState::NotFinalized,
            prior_destination_pack_id: None,
            backup_cleanup: CleanupState::NotRequired,
            backup_cleanup_path: None,
            materialization_operation_id: None,
            materialization_receipt_id: None,
            materialization_replayed: false,
        });
    }

    checkpoint(stop)?;
    let parent = output
        .parent()
        .ok_or_else(|| CliError::security("invalid_output_parent", "output has no parent"))?;
    let paths = materialization_paths(output, plan.plan_id())?;
    ensure_private_state_file(&paths.state_file)?;
    let configuration = ReferencePackMaterializationConfiguration::new(
        "wow-reference-builder-materialization/1",
        ReferencePackMaterializationStoreLimits::default(),
    )?;
    let mut journal = ReferencePackMaterializationService::open(&paths.state_file, configuration)?;
    let (materialization_request, prior_destination) =
        match journal.stored_request(&paths.operation_id)? {
            Some(stored) => {
                validate_stored_materialization_request(
                    &stored,
                    &paths,
                    output,
                    plan,
                    original_report,
                )?;
                let prior = stored
                    .prior_destination_pack_id()
                    .map(|pack_id| PriorDestination {
                        pack_id: pack_id.to_owned().into_boxed_str(),
                    });
                (stored, prior)
            }
            None => {
                let prior = observe_prior_destination(output, request.budgets(), stop)?;
                let materialization_request = ReferencePackMaterializationRequest::new(
                    paths.operation_id.clone(),
                    path_text(output)?,
                    path_text(&paths.staging)?,
                    path_text(&paths.backup)?,
                    path_text(&paths.quarantine)?,
                    plan.pack_id(),
                    plan.plan_id(),
                    original_report.report_id(),
                    prior.as_ref().map(|value| value.pack_id.as_ref()),
                )?;
                (materialization_request, prior)
            }
        };
    let validation_request = ReferencePackValidationRequest::new(
        plan.pack_id(),
        request.expected_profile_id(),
        request.expected_reference_generation_id(),
        request.budgets(),
    )?;
    let mut materialization_replayed = None;

    loop {
        checkpoint(stop)?;
        let observation = observe_materialization(&paths, request.budgets(), stop)?;
        let reconciliation = journal.reconcile(&materialization_request, observation.clone())?;
        materialization_replayed.get_or_insert(!reconciliation.started());

        match reconciliation.action() {
            ReferencePackMaterializationAction::CreateStaging => {
                create_private_directory(&paths.staging)?;
                write_materialization_plan(&paths.staging, plan, stop)?;
                sync_directory(&paths.staging)?;
                let (loaded, _) = validate_expected_pack(
                    &paths.staging,
                    &validation_request,
                    plan.pack_id(),
                    original_report.report_id(),
                    request.budgets(),
                    stop,
                )?;
                if loaded.member_count == 0 || loaded.total_bytes == 0 {
                    return Err(CliError::security(
                        "staging_empty_after_write",
                        "staged pack is empty after materialization",
                    ));
                }
                let next = observe_materialization(&paths, request.budgets(), stop)?;
                journal.checkpoint(
                    &materialization_request,
                    ReferencePackMaterializationStage::StagingValidated,
                    next,
                )?;
            }
            ReferencePackMaterializationAction::ResetStaging => {
                remove_tree(&paths.staging)?;
                sync_directory(parent)?;
                let next = observe_materialization(&paths, request.budgets(), stop)?;
                journal.checkpoint(
                    &materialization_request,
                    ReferencePackMaterializationStage::RolledBack,
                    next,
                )?;
            }
            ReferencePackMaterializationAction::AdoptStaging => {
                journal.checkpoint(
                    &materialization_request,
                    ReferencePackMaterializationStage::StagingValidated,
                    observation,
                )?;
            }
            ReferencePackMaterializationAction::MovePriorToBackup => {
                if reconciliation.stage() != Some(ReferencePackMaterializationStage::BackupIntent) {
                    journal.checkpoint(
                        &materialization_request,
                        ReferencePackMaterializationStage::BackupIntent,
                        observation.clone(),
                    )?;
                }
                fs::rename(output, &paths.backup).map_err(|_| {
                    CliError::new(
                        ExitClass::Build,
                        "destination_backup_failed",
                        "existing destination could not be moved to the durable backup path; rerun the same command to reconcile",
                    )
                })?;
                sync_directory(parent)?;
                let next = observe_materialization(&paths, request.budgets(), stop)?;
                journal.checkpoint(
                    &materialization_request,
                    ReferencePackMaterializationStage::BackupMoved,
                    next,
                )?;
            }
            ReferencePackMaterializationAction::AdoptMovedBackup => {
                journal.checkpoint(
                    &materialization_request,
                    ReferencePackMaterializationStage::BackupMoved,
                    observation,
                )?;
            }
            ReferencePackMaterializationAction::InstallStaging => {
                if reconciliation.stage() != Some(ReferencePackMaterializationStage::InstallIntent)
                {
                    journal.checkpoint(
                        &materialization_request,
                        ReferencePackMaterializationStage::InstallIntent,
                        observation.clone(),
                    )?;
                }
                fs::rename(&paths.staging, output).map_err(|_| {
                    CliError::new(
                        ExitClass::Build,
                        "destination_finalization_failed",
                        "staged pack could not be installed; rerun the same command to reconcile",
                    )
                })?;
                sync_directory(parent)?;
                let next = observe_materialization(&paths, request.budgets(), stop)?;
                journal.checkpoint(
                    &materialization_request,
                    ReferencePackMaterializationStage::Installed,
                    next,
                )?;
            }
            ReferencePackMaterializationAction::AdoptInstalledDestination => {
                journal.checkpoint(
                    &materialization_request,
                    ReferencePackMaterializationStage::Installed,
                    observation,
                )?;
            }
            ReferencePackMaterializationAction::ValidateDestination => {
                let validation = validate_expected_pack(
                    output,
                    &validation_request,
                    plan.pack_id(),
                    original_report.report_id(),
                    request.budgets(),
                    stop,
                );
                match validation {
                    Ok(_) => {
                        let next = observe_materialization(&paths, request.budgets(), stop)?;
                        journal.checkpoint(
                            &materialization_request,
                            ReferencePackMaterializationStage::FinalValidated,
                            next,
                        )?;
                    }
                    Err(error) => {
                        journal.checkpoint(
                            &materialization_request,
                            ReferencePackMaterializationStage::RollbackIntent,
                            observation,
                        )?;
                        perform_rollback(&paths, output, prior_destination.as_ref(), parent)?;
                        let next = observe_materialization(&paths, request.budgets(), stop)?;
                        journal.checkpoint(
                            &materialization_request,
                            ReferencePackMaterializationStage::RolledBack,
                            next,
                        )?;
                        return Err(error);
                    }
                }
            }
            ReferencePackMaterializationAction::RemoveBackup => {
                if reconciliation.stage() != Some(ReferencePackMaterializationStage::CleanupIntent)
                {
                    journal.checkpoint(
                        &materialization_request,
                        ReferencePackMaterializationStage::CleanupIntent,
                        observation,
                    )?;
                }
                remove_tree(&paths.backup)?;
                sync_directory(parent)?;
            }
            ReferencePackMaterializationAction::Complete => {
                if paths.quarantine.exists() {
                    remove_tree(&paths.quarantine)?;
                    sync_directory(parent)?;
                }
                let (final_image, final_report) = validate_expected_pack(
                    output,
                    &validation_request,
                    plan.pack_id(),
                    original_report.report_id(),
                    request.budgets(),
                    stop,
                )?;
                let final_observation = observe_materialization(&paths, request.budgets(), stop)?;
                let member_count = u32::try_from(final_image.member_count).map_err(|_| {
                    CliError::security(
                        "pack_member_count_overflow",
                        "final pack member count exceeds receipt range",
                    )
                })?;
                let receipt = journal.complete(
                    &materialization_request,
                    &final_observation,
                    member_count,
                    final_image.total_bytes,
                )?;
                return Ok(materialization_outcome_from_receipt(
                    original_report,
                    final_report,
                    final_image,
                    &receipt,
                    materialization_replayed.unwrap_or(false),
                ));
            }
            ReferencePackMaterializationAction::PerformRollback => {
                if reconciliation.stage() != Some(ReferencePackMaterializationStage::RollbackIntent)
                {
                    journal.checkpoint(
                        &materialization_request,
                        ReferencePackMaterializationStage::RollbackIntent,
                        observation,
                    )?;
                }
                perform_rollback(&paths, output, prior_destination.as_ref(), parent)?;
                let next = observe_materialization(&paths, request.budgets(), stop)?;
                journal.checkpoint(
                    &materialization_request,
                    ReferencePackMaterializationStage::RolledBack,
                    next,
                )?;
            }
            ReferencePackMaterializationAction::AdoptRollback => {
                journal.checkpoint(
                    &materialization_request,
                    ReferencePackMaterializationStage::RolledBack,
                    observation,
                )?;
            }
            ReferencePackMaterializationAction::ReturnCompleted => {
                let receipt = reconciliation.receipt().ok_or_else(|| {
                    CliError::security(
                        "materialization_receipt_missing",
                        "completed materialization is missing its durable receipt",
                    )
                })?;
                let (final_image, final_report) = validate_expected_pack(
                    output,
                    &validation_request,
                    plan.pack_id(),
                    original_report.report_id(),
                    request.budgets(),
                    stop,
                )?;
                return Ok(materialization_outcome_from_receipt(
                    original_report,
                    final_report,
                    final_image,
                    receipt,
                    true,
                ));
            }
            ReferencePackMaterializationAction::TerminalNoEffect => {
                return Err(CliError::new(
                    ExitClass::Build,
                    "materialization_no_effect_terminal",
                    "materialization operation is terminal without installation; use a new build request identity",
                ));
            }
            ReferencePackMaterializationAction::TerminalFailed => {
                return Err(CliError::new(
                    ExitClass::Build,
                    "materialization_failed_terminal",
                    "materialization operation is terminal after failure; inspect durable state before starting a new operation",
                ));
            }
            ReferencePackMaterializationAction::OperatorReview => {
                return Err(CliError::new(
                    ExitClass::Build,
                    "materialization_outcome_unknown",
                    "materialization filesystem state is ambiguous; blind retry is prohibited and operator reconciliation is required",
                ));
            }
        }
    }
}

struct MaterializationPaths {
    operation_id: ReferencePackMaterializationOperationId,
    output: PathBuf,
    state_file: PathBuf,
    staging: PathBuf,
    backup: PathBuf,
    quarantine: PathBuf,
}

fn materialization_paths(output: &Path, plan_id: &str) -> Result<MaterializationPaths, CliError> {
    let parent = output
        .parent()
        .ok_or_else(|| CliError::security("invalid_output_parent", "output has no parent"))?;
    let output_text = path_text(output)?;
    let token = hex(&Sha256::digest(
        format!("{output_text}\n{plan_id}").as_bytes(),
    ));
    let state_token = hex(&Sha256::digest(output_text.as_bytes()));
    let operation_id =
        ReferencePackMaterializationOperationId::new(format!("reference-pack-materialize:{token}"))
            .map_err(|_| {
                CliError::security(
                    "materialization_operation_id_invalid",
                    "materialization operation identity is invalid",
                )
            })?;
    Ok(MaterializationPaths {
        operation_id,
        output: output.to_path_buf(),
        state_file: parent.join(format!(
            ".wow-reference-builder-state-{state_token}.sqlite3"
        )),
        staging: parent.join(format!(".wow-reference-builder-staging-{token}")),
        backup: parent.join(format!(".wow-reference-builder-backup-{token}")),
        quarantine: parent.join(format!(".wow-reference-builder-quarantine-{token}")),
    })
}

fn validate_stored_materialization_request(
    stored: &ReferencePackMaterializationRequest,
    paths: &MaterializationPaths,
    output: &Path,
    plan: &PackMaterializationPlan,
    report: &ReferencePackValidationReport,
) -> Result<(), CliError> {
    if stored.operation_id() != &paths.operation_id
        || stored.output_path() != path_text(output)?.as_ref()
        || stored.staging_path() != path_text(&paths.staging)?.as_ref()
        || stored.backup_path() != path_text(&paths.backup)?.as_ref()
        || stored.quarantine_path() != path_text(&paths.quarantine)?.as_ref()
        || stored.pack_id() != plan.pack_id()
        || stored.plan_id() != plan.plan_id()
        || stored.validation_report_id() != report.report_id()
    {
        return Err(CliError::new(
            ExitClass::Build,
            "materialization_operation_conflict",
            "durable materialization operation is bound to another output or pack plan",
        ));
    }
    Ok(())
}

fn path_text(path: &Path) -> Result<Box<str>, CliError> {
    path.to_str()
        .map(|value| value.to_owned().into_boxed_str())
        .ok_or_else(|| {
            CliError::security(
                "non_utf8_materialization_path",
                "materialization paths must be valid UTF-8",
            )
        })
}

fn ensure_private_state_file(path: &Path) -> Result<(), CliError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if is_reparse_or_symlink(&metadata) || !metadata.is_file() {
                return Err(CliError::security(
                    "unsafe_materialization_state_file",
                    "materialization state path is not a safe regular file",
                ));
            }
            Ok(())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let mut options = OpenOptions::new();
            options.create_new(true).read(true).write(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            options
                .open(path)
                .and_then(|file| file.sync_all())
                .map_err(|_| {
                    CliError::security(
                        "materialization_state_create_failed",
                        "materialization state file could not be created safely",
                    )
                })
        }
        Err(_) => Err(CliError::security(
            "materialization_state_metadata_failed",
            "materialization state metadata is unavailable",
        )),
    }
}

fn observe_materialization(
    paths: &MaterializationPaths,
    budgets: ReferencePackBudgets,
    stop: &AtomicBool,
) -> Result<ReferencePackMaterializationObservation, CliError> {
    Ok(ReferencePackMaterializationObservation::new(
        observe_pack_path(&paths.staging, budgets, stop)?,
        observe_pack_path_from_output(paths, budgets, stop)?,
        observe_pack_path(&paths.backup, budgets, stop)?,
    )?)
}

fn observe_pack_path_from_output(
    paths: &MaterializationPaths,
    budgets: ReferencePackBudgets,
    stop: &AtomicBool,
) -> Result<ReferencePackPathObservation, CliError> {
    observe_pack_path(&paths.output, budgets, stop)
}

fn observe_pack_path(
    path: &Path,
    budgets: ReferencePackBudgets,
    stop: &AtomicBool,
) -> Result<ReferencePackPathObservation, CliError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(value) => value,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(ReferencePackPathObservation::absent());
        }
        Err(_) => {
            return Ok(ReferencePackPathObservation::invalid(
                "metadata_unavailable",
            )?);
        }
    };
    if is_reparse_or_symlink(&metadata) || !metadata.is_dir() {
        return Ok(ReferencePackPathObservation::invalid(
            "not_a_safe_directory",
        )?);
    }
    let observed = (|| {
        let loaded = load_pack_image(path, budgets, stop)?;
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
                "observed_pack_not_candidate",
                "observed pack is not candidate eligible",
            ));
        }
        Ok(ReferencePackPathObservation::pack(
            report.pack_id(),
            report.report_id(),
        )?)
    })();
    match observed {
        Ok(value) => Ok(value),
        Err(error) if error.class == ExitClass::Cancelled => Err(error),
        Err(_) => Ok(ReferencePackPathObservation::invalid(
            "pack_validation_failed",
        )?),
    }
}

fn validate_expected_pack(
    path: &Path,
    validation_request: &ReferencePackValidationRequest,
    expected_pack_id: &str,
    expected_report_id: &str,
    budgets: ReferencePackBudgets,
    stop: &AtomicBool,
) -> Result<(LoadedPack, ReferencePackValidationReport), CliError> {
    let loaded = load_pack_image(path, budgets, stop)?;
    let report =
        ReferencePackService::reference_pack_validate(validation_request, &loaded.image, stop)?;
    if report.pack_id() != expected_pack_id || report.report_id() != expected_report_id {
        return Err(CliError::security(
            "materialization_read_back_mismatch",
            "materialized pack differs from the exact service result",
        ));
    }
    Ok((loaded, report))
}

fn perform_rollback(
    paths: &MaterializationPaths,
    output: &Path,
    prior_destination: Option<&PriorDestination>,
    parent: &Path,
) -> Result<(), CliError> {
    if output.exists() {
        if paths.quarantine.exists() {
            return Err(CliError::security(
                "quarantine_path_occupied",
                "failed destination cannot be quarantined because the durable quarantine path is occupied",
            ));
        }
        fs::rename(output, &paths.quarantine).map_err(|_| {
            CliError::new(
                ExitClass::Build,
                "destination_quarantine_failed",
                "failed destination could not be moved to quarantine",
            )
        })?;
        sync_directory(parent)?;
    }
    if prior_destination.is_some() && paths.backup.exists() {
        fs::rename(&paths.backup, output).map_err(|_| {
            CliError::new(
                ExitClass::Build,
                "destination_restore_failed",
                "prior destination could not be restored from durable backup",
            )
        })?;
        sync_directory(parent)?;
    }
    Ok(())
}

fn materialization_outcome_from_receipt(
    staging_report: &ReferencePackValidationReport,
    final_report: ReferencePackValidationReport,
    final_image: LoadedPack,
    receipt: &wow_service::reference_pack_materialization::ReferencePackMaterializationReceipt,
    replayed: bool,
) -> MaterializationOutcome {
    MaterializationOutcome {
        staging_report: staging_report.clone(),
        final_report,
        member_count: final_image.member_count,
        total_bytes: final_image.total_bytes,
        destination_state: match receipt.destination_state() {
            ReferencePackMaterializationDestinationState::Created => DestinationState::Created,
            ReferencePackMaterializationDestinationState::Replaced => DestinationState::Replaced,
        },
        prior_destination_pack_id: receipt
            .prior_destination_pack_id()
            .map(|value| value.to_owned().into_boxed_str()),
        backup_cleanup: if receipt.prior_destination_pack_id().is_some() {
            CleanupState::Completed
        } else {
            CleanupState::NotRequired
        },
        backup_cleanup_path: None,
        materialization_operation_id: Some(
            receipt.operation_id().as_str().to_owned().into_boxed_str(),
        ),
        materialization_receipt_id: Some(receipt.receipt_id().to_owned().into_boxed_str()),
        materialization_replayed: replayed,
    }
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
        metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
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
        "build {}\npack_id={}\nplan_id={}\nfinal_validation_report_id={}\ncandidate_eligible={}\nvalidated_local_eligible={}\ndestination_state={:?}\nbackup_cleanup={:?}\nmaterialization_operation_id={}\nmaterialization_receipt_id={}\nmaterialization_replayed={}\n",
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
        value
            .materialization_operation_id
            .unwrap_or("not_applicable"),
        value.materialization_receipt_id.unwrap_or("not_applicable"),
        value.materialization_replayed,
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
