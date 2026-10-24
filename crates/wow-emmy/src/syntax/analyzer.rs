use std::{
    path::{Path, PathBuf},
    sync::Arc,
    sync::atomic::{AtomicBool, Ordering},
};

use emmylua_code_analysis::{
    EmmyLuaAnalysis, Emmyrc, EmmyrcLuaVersion, WorkspaceFolder, file_path_to_uri,
};
use lsp_types::{DiagnosticSeverity, NumberOrString};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;
use wow_core::canonical_json_bytes;

use crate::{LuaWorkspaceSnapshot, LuaWorkspaceUniverse};

use super::{
    CAPABILITY_ID, EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE,
    EmmyDiagnosticClassification, EmmyDiagnosticRollout, EmmyDiagnosticSeverity,
    EmmySyntaxDiagnostic, EmmySyntaxDiagnosticKind, EmmySyntaxError, EmmySyntaxErrorCode,
    EmmySyntaxFileReport, EmmySyntaxReport, EmmySyntaxResult, FRAMEWORK_CATEGORY, MAX_DIAGNOSTICS,
    MAX_DIAGNOSTICS_PER_FILE, REPORT_SCHEMA, coordinates,
};

const MAX_DIAGNOSTIC_CODE_BYTES: usize = 1_024;
const MAX_DIAGNOSTIC_MESSAGE_BYTES: usize = 16 * 1_024;

pub(super) fn analyze(snapshot: &LuaWorkspaceSnapshot) -> EmmySyntaxResult<EmmySyntaxReport> {
    validate_backend(snapshot)?;
    let root = virtual_root(snapshot.snapshot_id());
    let analysis = build_registered(snapshot, &root)?;
    collect_registered(&analysis, snapshot, &root, &AtomicBool::new(false))
}

/// Same collection under an operation stop state.
pub(super) fn analyze_with_stop(
    snapshot: &LuaWorkspaceSnapshot,
    stop: &AtomicBool,
) -> EmmySyntaxResult<EmmySyntaxReport> {
    validate_backend(snapshot)?;
    let root = virtual_root(snapshot.snapshot_id());
    let analysis = build_registered(snapshot, &root)?;
    collect_registered(&analysis, snapshot, &root, stop)
}

// The counter and the seam exist only in test builds, so production carries
// no per-call bookkeeping and never observes a test-only stop arming.
#[cfg(test)]
thread_local! {
    static DIAGNOSTIC_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static STOP_AFTER_CALL: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Test-only reset of the observed upstream diagnostic call count.
///
/// A test resets before its own run so a parallel test cannot make the
/// expected call count ambiguous, and clears the armed bound so no
/// unrelated run observes a stale seam.
#[cfg(test)]
pub(crate) fn reset_diagnostic_calls() {
    DIAGNOSTIC_CALLS.set(0);
    STOP_AFTER_CALL.set(0);
}

/// Test-only observed upstream diagnostic call count.
#[cfg(test)]
pub(crate) fn diagnostic_calls() -> usize {
    DIAGNOSTIC_CALLS.get()
}

/// Report one completed upstream diagnostic call and apply any armed stop.
///
/// The seam runs after a native diagnostic call has completed, so the first
/// file is fully diagnosed and only a later file is prevented from starting.
/// A test arms a finite one-based bound. Without a test arming the seam this
/// only reads a counter, so production never polls, spins, waits, or observes
/// an extra source callback.
#[cfg(test)]
fn observe_diagnostic_call(stop: &AtomicBool) -> usize {
    let calls = DIAGNOSTIC_CALLS.get() + 1;
    DIAGNOSTIC_CALLS.set(calls);
    let armed = STOP_AFTER_CALL.get();
    if armed != 0 && calls >= armed {
        stop.store(true, Ordering::Release);
    }
    calls
}

/// Test-only arming of the deterministic after-first-native-call seam.
///
/// `calls` is the one-based number of completed native diagnostic calls after
/// which the stop state must become visible. Zero disarms the seam. The bound
/// is finite and chosen by the test, and `reset_diagnostic_calls` clears it, so
/// no unrelated run can observe a stale arming.
#[cfg(test)]
pub(crate) fn arm_stop_after_diagnostic_call(calls: usize) {
    STOP_AFTER_CALL.set(calls);
}

pub(crate) fn build_registered(
    snapshot: &LuaWorkspaceSnapshot,
    root: &Path,
) -> EmmySyntaxResult<EmmyLuaAnalysis> {
    validate_backend(snapshot)?;
    let mut analysis = EmmyLuaAnalysis::new();
    let mut configuration = Emmyrc::default();
    configuration.runtime.version = EmmyrcLuaVersion::Lua51;
    configuration.diagnostics.enable = true;
    analysis.update_config(Arc::new(configuration));
    analysis.init_std_lib(None);
    match snapshot.universe() {
        LuaWorkspaceUniverse::Project
        | LuaWorkspaceUniverse::Fixture
        | LuaWorkspaceUniverse::BlizzardUiMain => {
            analysis.add_main_workspace(root.to_path_buf());
        }
        LuaWorkspaceUniverse::BlizzardUi => {
            analysis.add_library_workspace(&WorkspaceFolder::new(root.to_path_buf(), true));
        }
    }

    let exact_files = snapshot
        .files()
        .iter()
        .map(|file| (root.join(file.path()), Some(file.text().to_owned())))
        .collect::<Vec<_>>();
    if analysis.update_files_by_path(exact_files).len() != snapshot.files().len() {
        return Err(EmmySyntaxError::new(
            EmmySyntaxErrorCode::AnalyzerFileRegistrationFailed,
            "upstream did not register every supplied file",
            None,
        ));
    }
    Ok(analysis)
}

/// Collect configured diagnostics from an already populated exact analyzer
/// session. The caller owns workspace registration; this function never opens a
/// URI or creates a second parser/session.
pub(crate) fn collect_registered(
    analysis: &EmmyLuaAnalysis,
    snapshot: &LuaWorkspaceSnapshot,
    root: &Path,
    stop: &AtomicBool,
) -> EmmySyntaxResult<EmmySyntaxReport> {
    validate_backend(snapshot)?;
    let mut diagnostics = Vec::new();
    let mut files = Vec::with_capacity(snapshot.files().len());
    for file in snapshot.files() {
        cancel(stop, file.path())?;
        let uri = file_path_to_uri(&root.join(file.path())).ok_or_else(|| {
            EmmySyntaxError::new(
                EmmySyntaxErrorCode::AnalyzerFileRegistrationFailed,
                "upstream URI construction failed",
                Some(file.path()),
            )
        })?;
        let file_id = analysis.get_file_id(&uri).ok_or_else(|| {
            EmmySyntaxError::new(
                EmmySyntaxErrorCode::AnalyzerFileRegistrationFailed,
                "upstream did not retain a supplied file",
                Some(file.path()),
            )
        })?;
        // One upstream diagnostic call per file. The call receives the shared
        // stop token so an already cancelled entry is refused on entry
        // instead of running checkers under a dead operation.
        let upstream = analysis.diagnose_file(file_id, shared_token(stop));
        // Deterministic after-first-native-call boundary. This call has
        // completed, so the stop state can now become visible before any later
        // file enters the native collector. The seam is test-only, so
        // production never polls, spins, or observes an extra source callback.
        #[cfg(test)]
        observe_diagnostic_call(stop);
        // The between-file checkpoint follows the completed native call. An
        // armed stop is visible here, so the next file never enters upstream.
        cancel(stop, file.path())?;
        let before = diagnostics.len();
        // Upstream returns `None` both for a refused cancelled entry and for
        // a file it cannot diagnose. Only the shared stop state names this
        // operation's cancellation; any other refusal keeps the
        // unavailable-diagnostics reading, so a healthy analyzer is never
        // reported as cancelled.
        let upstream = upstream.ok_or_else(|| {
            EmmySyntaxError::new(
                if stop.load(Ordering::Acquire) {
                    EmmySyntaxErrorCode::Cancelled
                } else {
                    EmmySyntaxErrorCode::DiagnosticsUnavailable
                },
                "upstream diagnostics are unavailable for a registered file",
                Some(file.path()),
            )
        })?;
        for diagnostic in upstream {
            cancel(stop, file.path())?;
            let (kind, code) = diagnostic_code(diagnostic.code.as_ref());
            if code.len() > MAX_DIAGNOSTIC_CODE_BYTES
                || diagnostic.message.len() > MAX_DIAGNOSTIC_MESSAGE_BYTES
            {
                return Err(EmmySyntaxError::new(
                    EmmySyntaxErrorCode::DiagnosticBudgetExceeded,
                    "an upstream diagnostic code or message exceeds the adapter budget",
                    Some(file.path()),
                ));
            }
            if diagnostics.len() - before >= MAX_DIAGNOSTICS_PER_FILE
                || diagnostics.len() >= MAX_DIAGNOSTICS
            {
                return Err(EmmySyntaxError::new(
                    EmmySyntaxErrorCode::DiagnosticBudgetExceeded,
                    "diagnostic count exceeds the adapter budget",
                    Some(file.path()),
                ));
            }
            let span = coordinates::convert(file.text(), diagnostic.range).map_err(|code| {
                EmmySyntaxError::new(
                    code,
                    "upstream diagnostic range cannot bind to exact source bytes",
                    Some(file.path()),
                )
            })?;
            let upstream_severity = severity(diagnostic.severity);
            diagnostics.push(EmmySyntaxDiagnostic {
                category: FRAMEWORK_CATEGORY,
                upstream_code: code,
                message: diagnostic.message.into_boxed_str(),
                kind,
                upstream_severity,
                normalized_severity: upstream_severity,
                classification: EmmyDiagnosticClassification::Accepted,
                rollout: EmmyDiagnosticRollout::Advisory,
                path: file.path().into(),
                content_sha256: file.content_sha256().into(),
                span,
            });
        }
        files.push(EmmySyntaxFileReport {
            capability_id: CAPABILITY_ID,
            path: file.path().into(),
            content_sha256: file.content_sha256().into(),
            status: "complete",
            diagnostic_count: u64::try_from(diagnostics.len() - before).map_err(|_| {
                EmmySyntaxError::new(
                    EmmySyntaxErrorCode::DiagnosticBudgetExceeded,
                    "diagnostic count exceeds u64",
                    Some(file.path()),
                )
            })?,
        });
    }
    cancel(stop, "")?;
    diagnostics.sort();
    cancel(stop, "")?;

    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        upstream_revision: &'static str,
        upstream_tree: &'static str,
        upstream_crate_version: &'static str,
        upstream_column_encoding: &'static str,
        workspace_snapshot_id: &'a str,
        universe: LuaWorkspaceUniverse,
        files: &'a [EmmySyntaxFileReport],
        diagnostics: &'a [EmmySyntaxDiagnostic],
    }
    let identity = Identity {
        schema: REPORT_SCHEMA,
        upstream_revision: EMMYLUA_REVISION,
        upstream_tree: EMMYLUA_TREE,
        upstream_crate_version: EMMYLUA_CODE_ANALYSIS_VERSION,
        upstream_column_encoding: "unicode_scalar_value",
        workspace_snapshot_id: snapshot.snapshot_id(),
        universe: snapshot.universe(),
        files: &files,
        diagnostics: &diagnostics,
    };
    let canonical = canonical_json_bytes(&identity).map_err(|_| {
        EmmySyntaxError::new(
            EmmySyntaxErrorCode::CanonicalizationFailed,
            "diagnostic report identity cannot be canonicalized",
            None,
        )
    })?;
    cancel(stop, "")?;
    Ok(EmmySyntaxReport {
        schema: REPORT_SCHEMA,
        analysis_id: format!("emmy-diagnostics:sha256:{:x}", Sha256::digest(canonical))
            .into_boxed_str(),
        upstream_revision: EMMYLUA_REVISION,
        upstream_tree: EMMYLUA_TREE,
        upstream_crate_version: EMMYLUA_CODE_ANALYSIS_VERSION,
        upstream_column_encoding: "unicode_scalar_value",
        workspace_snapshot_id: snapshot.snapshot_id().into(),
        universe: snapshot.universe(),
        files,
        diagnostics,
    })
}
fn validate_backend(snapshot: &LuaWorkspaceSnapshot) -> EmmySyntaxResult<()> {
    let backend = snapshot.backend();
    if backend.crate_name() != "emmylua_code_analysis"
        || backend.crate_version() != Some(EMMYLUA_CODE_ANALYSIS_VERSION)
        || backend.revision() != EMMYLUA_REVISION
        || backend.tree() != EMMYLUA_TREE
    {
        return Err(EmmySyntaxError::new(
            EmmySyntaxErrorCode::IncompatibleBackend,
            "workspace backend does not match the compiled analyzer pin",
            None,
        ));
    }
    Ok(())
}

/// Stop this collection when the caller asked to stop.
///
/// Typed cancellation stays distinct from `DiagnosticsUnavailable` and from a
/// native failure, so the session keeps an operator cancellation separate from
/// a lost analyzer.
fn cancel(stop: &AtomicBool, path: &str) -> EmmySyntaxResult<()> {
    if stop.load(Ordering::Acquire) {
        return Err(EmmySyntaxError::new(
            EmmySyntaxErrorCode::Cancelled,
            "diagnostic collection cancelled by the caller",
            Some(path),
        ));
    }
    Ok(())
}

/// Carry the operation stop state into the upstream diagnostic call.
///
/// Upstream `diagnose_file` checks its token on entry and then runs `check_file`
/// synchronously without polling it. One shared token therefore decides
/// whether the next file is diagnosed at all, and the between-file checkpoint
/// above returns a typed cancellation before that file starts. Sharing the
/// token does not make one file of synchronous work preemptible, and no
/// polling thread, spin loop, or second parser is introduced here.
fn shared_token(stop: &AtomicBool) -> CancellationToken {
    let token = CancellationToken::new();
    if stop.load(Ordering::Acquire) {
        token.cancel();
    }
    token
}

pub(crate) fn virtual_root(snapshot_id: &str) -> PathBuf {
    let stable = snapshot_id
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() {
                char::from(byte)
            } else {
                '_'
            }
        })
        .collect::<String>();
    std::env::temp_dir().join("wow-emmy-v1").join(stable)
}

fn diagnostic_code(code: Option<&NumberOrString>) -> (EmmySyntaxDiagnosticKind, Box<str>) {
    match code {
        Some(NumberOrString::String(value)) => (diagnostic_kind(value), value.as_str().into()),
        Some(NumberOrString::Number(value)) => (
            EmmySyntaxDiagnosticKind::Other,
            value.to_string().into_boxed_str(),
        ),
        None => (EmmySyntaxDiagnosticKind::Other, "uncoded".into()),
    }
}

fn diagnostic_kind(code: &str) -> EmmySyntaxDiagnosticKind {
    match code {
        "syntax-error" => EmmySyntaxDiagnosticKind::LuaSyntax,
        "doc-syntax-error" => EmmySyntaxDiagnosticKind::DocumentationSyntax,
        "assign-type-mismatch" => EmmySyntaxDiagnosticKind::AssignmentTypeMismatch,
        "param-type-mismatch" => EmmySyntaxDiagnosticKind::ParameterTypeMismatch,
        _ => EmmySyntaxDiagnosticKind::Other,
    }
}

fn severity(value: Option<DiagnosticSeverity>) -> EmmyDiagnosticSeverity {
    if value == Some(DiagnosticSeverity::ERROR) {
        EmmyDiagnosticSeverity::Error
    } else if value == Some(DiagnosticSeverity::WARNING) {
        EmmyDiagnosticSeverity::Warning
    } else if value == Some(DiagnosticSeverity::INFORMATION) {
        EmmyDiagnosticSeverity::Information
    } else if value == Some(DiagnosticSeverity::HINT) {
        EmmyDiagnosticSeverity::Hint
    } else {
        EmmyDiagnosticSeverity::Unknown
    }
}
