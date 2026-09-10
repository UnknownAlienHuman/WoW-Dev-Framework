use std::{path::PathBuf, sync::Arc};

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

pub(super) fn analyze(snapshot: &LuaWorkspaceSnapshot) -> EmmySyntaxResult<EmmySyntaxReport> {
    validate_backend(snapshot)?;
    let root = virtual_root(snapshot.snapshot_id());
    let mut analysis = EmmyLuaAnalysis::new();
    let mut configuration = Emmyrc::default();
    configuration.runtime.version = EmmyrcLuaVersion::Lua51;
    configuration.diagnostics.enable = true;
    analysis.update_config(Arc::new(configuration));
    match snapshot.universe() {
        LuaWorkspaceUniverse::Project | LuaWorkspaceUniverse::Fixture => {
            analysis.add_main_workspace(root.clone());
        }
        LuaWorkspaceUniverse::BlizzardUi => {
            analysis.add_library_workspace(&WorkspaceFolder::new(root.clone(), true));
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

    let mut diagnostics = Vec::new();
    let mut files = Vec::with_capacity(snapshot.files().len());
    for file in snapshot.files() {
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
        let upstream = analysis
            .diagnose_file(file_id, CancellationToken::new())
            .ok_or_else(|| {
                EmmySyntaxError::new(
                    EmmySyntaxErrorCode::DiagnosticsUnavailable,
                    "upstream diagnostics are unavailable for a registered file",
                    Some(file.path()),
                )
            })?;
        let before = diagnostics.len();
        for diagnostic in upstream {
            let Some((kind, code)) = syntax_code(diagnostic.code.as_ref()) else {
                continue;
            };
            if diagnostics.len() - before >= MAX_DIAGNOSTICS_PER_FILE
                || diagnostics.len() >= MAX_DIAGNOSTICS
            {
                return Err(EmmySyntaxError::new(
                    EmmySyntaxErrorCode::DiagnosticBudgetExceeded,
                    "syntax diagnostic count exceeds the adapter budget",
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
            diagnostics.push(EmmySyntaxDiagnostic {
                category: FRAMEWORK_CATEGORY,
                upstream_code: code.into(),
                kind,
                upstream_severity: severity(diagnostic.severity),
                normalized_severity: EmmyDiagnosticSeverity::Error,
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
                    "syntax diagnostic count exceeds u64",
                    Some(file.path()),
                )
            })?,
        });
    }
    diagnostics.sort();

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
            "syntax report identity cannot be canonicalized",
            None,
        )
    })?;
    Ok(EmmySyntaxReport {
        schema: REPORT_SCHEMA,
        analysis_id: format!("emmy-syntax:sha256:{:x}", Sha256::digest(canonical)).into_boxed_str(),
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

fn virtual_root(snapshot_id: &str) -> PathBuf {
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

fn syntax_code(code: Option<&NumberOrString>) -> Option<(EmmySyntaxDiagnosticKind, &'static str)> {
    match code? {
        NumberOrString::String(value) if value == "syntax-error" => {
            Some((EmmySyntaxDiagnosticKind::LuaSyntax, "syntax-error"))
        }
        NumberOrString::String(value) if value == "doc-syntax-error" => Some((
            EmmySyntaxDiagnosticKind::DocumentationSyntax,
            "doc-syntax-error",
        )),
        NumberOrString::String(_) | NumberOrString::Number(_) => None,
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
