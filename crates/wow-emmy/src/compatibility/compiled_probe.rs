//! A scoped executable probe of the already compiled adapter, not pin activation.
//! Source bytes are explicit data; filesystem/Git acquisition belongs to the caller.
use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::{Value, json};
use wow_core::{ProjectGenerationId, canonical_json_bytes};

use super::{EmmyCompatibilityErrorCode, EmmyCompatibilityResult, report_error, sha256};
use crate::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    EmmyReferenceResolution, EmmySyntaxDiagnosticKind, LuaWorkspaceFileInput, LuaWorkspaceLimits,
    LuaWorkspaceSnapshot, LuaWorkspaceUniverse, MemberCallSessionQueryProfile,
    session::{AnalyzerSession, AnalyzerSessionErrorCode, AnalyzerUpdateBatch},
};

pub const COMPILED_PROBE_PROFILE: &str = "wow-emmy/compiled-backend-probe/1";
/// Fixed source members for this compiled adapter probe; not an upstream inventory.
pub const COMPILED_PROBE_MEMBERS: &[&str] = &[
    "Cargo.toml",
    "crates/emmylua_code_analysis/Cargo.toml",
    "crates/emmylua_code_analysis/src/compilation/mod.rs",
    "crates/emmylua_code_analysis/src/config/mod.rs",
    "crates/emmylua_code_analysis/src/lib.rs",
    "crates/emmylua_code_analysis/src/semantic/mod.rs",
    "crates/emmylua_parser/Cargo.toml",
    "crates/emmylua_parser/src/lib.rs",
];
pub const COMPILED_PROBE_MEMBER_MAX_BYTES: usize = 1024 * 1024;
// Evidence for the exact EMMYLUA_REVISION/EMMYLUA_TREE compiled above. A pin
// update must remeasure this selected source surface; moving upstream selectors
// are never resolved from this value. This also rejects fabricated caller bytes.
const COMPILED_PIN_SURFACE_DIGEST: &str =
    "sha256:912392a68a2f4bdfcc3e6d8a2884f3b17af3cd96c52d1f3840fbef7c0463ae25";
const LIBRARY: &str = concat!(
    "---@meta _\n---@class ProbeApi\n",
    "---@field Known fun(value: string): string\n",
    "---@field Produce fun(): string\nProbeApi = {}\n",
    "---@param value any\n---@return boolean\nfunction canaccessvalue(value) end\n",
);
const CLEAN: &str = "return ProbeApi.Known(\"ok\")\n";
const GENERIC: &str = "---@type string\nlocal value = 42\nreturn value\n";
const MISSING: &str = "return ProbeApi.Removed()\n";
const FLOW: &str = concat!(
    "local function guarded()\n local value = ProbeApi.Produce()\n",
    " if canaccessvalue(value) then return value .. \"!\" end\nend\nreturn guarded\n",
);

/// Produce the original compatibility-report wire from exact source fingerprints
/// and successful native observations. The caller must acquire these ordered
/// members from the specified exact Git revision/tree, not working-tree files.
/// This report does not establish freshness, license acceptance, P1-P12 acceptance
/// or activation of a new dependency pin.
pub fn probe_compiled_backend(
    revision: &str,
    tree: &str,
    members: &[Vec<u8>],
    stop: &AtomicBool,
) -> EmmyCompatibilityResult<Vec<u8>> {
    checkpoint(stop)?;
    if revision != EMMYLUA_REVISION || tree != EMMYLUA_TREE {
        return Err(failed("probe source differs from the compiled backend"));
    }
    if members.len() != COMPILED_PROBE_MEMBERS.len()
        || members
            .iter()
            .any(|bytes| bytes.is_empty() || bytes.len() > COMPILED_PROBE_MEMBER_MAX_BYTES)
    {
        return Err(failed(
            "probe source members are missing or exceed their limit",
        ));
    }
    let files = COMPILED_PROBE_MEMBERS
        .iter()
        .zip(members)
        .map(|(path, bytes)| json!({"path":path,"byte_length":bytes.len(),"sha256":sha256(bytes)}))
        .collect::<Vec<_>>();
    // These public operations actually execute through the existing adapter below.
    // Compilation, rather than source-text symbol scraping, verifies accessibility.
    let symbols = [
        "EmmyLuaAnalysis::add_library_workspace",
        "EmmyLuaAnalysis::add_main_workspace",
        "EmmyLuaAnalysis::diagnose_file",
        "EmmyLuaAnalysis::get_file_id",
        "EmmyLuaAnalysis::new",
        "EmmyLuaAnalysis::update_config",
        "EmmyLuaAnalysis::update_files_by_path",
        "file_path_to_uri",
    ];
    let surface_digest =
        digest(&json!({"profile":COMPILED_PROBE_PROFILE,"files":files,"symbols":symbols}))?;
    if surface_digest != COMPILED_PIN_SURFACE_DIGEST {
        return Err(failed(
            "probe source fingerprint differs from the measured compiled pin",
        ));
    }
    let candidate_digest = digest(&json!({
        "profile":COMPILED_PROBE_PROFILE,"revision":revision,"tree":tree,
        "source":surface_digest,"corpus":[LIBRARY,CLEAN,GENERIC,MISSING,FLOW]
    }))?;
    // A measured candidate identity bootstraps the probe. It is never published
    // as an accepted production backend; the final report digest replaces it.
    let candidate = EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(EMMYLUA_CODE_ANALYSIS_VERSION),
        revision,
        tree,
        surface_digest.clone(),
        candidate_digest,
    )
    .map_err(|_| failed("compiled probe candidate was rejected"))?;
    let limits = LuaWorkspaceLimits::new(8, 1024, 16 * 1024, 64 * 1024)
        .map_err(|_| failed("compiled probe limits were rejected"))?;
    let build = |universe, inputs| {
        LuaWorkspaceSnapshot::build(candidate.clone(), universe, inputs, limits)
            .map_err(|_| failed("compiled probe workspace was rejected"))
    };
    let main = build(
        LuaWorkspaceUniverse::Fixture,
        vec![
            LuaWorkspaceFileInput::new("main/clean.lua", CLEAN),
            LuaWorkspaceFileInput::new("main/generic.lua", GENERIC),
            LuaWorkspaceFileInput::new("main/missing.lua", MISSING),
            LuaWorkspaceFileInput::new("main/flow.lua", FLOW),
        ],
    )?;
    let library = build(
        LuaWorkspaceUniverse::BlizzardUi,
        vec![LuaWorkspaceFileInput::new("library/ProbeApi.lua", LIBRARY)],
    )?;
    checkpoint(stop)?;
    let mut session = AnalyzerSession::open(main.clone(), std::slice::from_ref(&library), stop)
        .map_err(|error| session_failure(error.code(), "native probe session failed"))?;
    let queries = MemberCallSessionQueryProfile::new(&[], &[]);
    let reports = session
        .reports(queries, false, stop)
        .map_err(|error| session_failure(error.code(), "native probe collection failed"))?;
    checkpoint(stop)?;
    if reports.syntax.files().len() != 4
        || !reports.syntax.diagnostics().iter().any(|d| {
            d.path() == "main/generic.lua"
                && d.kind() == EmmySyntaxDiagnosticKind::AssignmentTypeMismatch
        })
        || reports
            .syntax
            .diagnostics()
            .iter()
            .any(|d| d.path() == "main/clean.lua" || d.path().starts_with("library/"))
        || !reports.member_calls.references().iter().any(|r| {
            r.path() == "main/clean.lua"
                && r.member() == "Known"
                && r.resolution() == EmmyReferenceResolution::Resolved
        })
        || !reports.member_calls.references().iter().any(|r| {
            r.path() == "main/missing.lua"
                && r.member() == "Removed"
                && r.resolution() != EmmyReferenceResolution::Resolved
        })
        || reports.member_calls.calls().is_empty()
        || reports.local_flow.bindings().is_empty()
        || reports.local_flow.operations().is_empty()
        || reports.local_flow.guards().is_empty()
        || reports.local_flow.control_flow().is_empty()
    {
        return Err(failed(
            "native probe observations do not satisfy the selected corpus",
        ));
    }
    let observations = json!({
        "profile":COMPILED_PROBE_PROFILE,
        "main_snapshot":main.snapshot_id(),"library_snapshot":library.snapshot_id(),
        "syntax":reports.syntax,"member_calls":reports.member_calls,"local_flow":reports.local_flow,
    });
    let target = build(
        LuaWorkspaceUniverse::Fixture,
        vec![
            LuaWorkspaceFileInput::new("main/clean.lua", CLEAN),
            LuaWorkspaceFileInput::new("main/generic.lua", "local value = 42\nreturn value\n"),
            LuaWorkspaceFileInput::new("main/flow.lua", FLOW),
        ],
    )?;
    let generation = ProjectGenerationId::derive(COMPILED_PROBE_PROFILE)
        .map_err(|_| failed("probe generation identity failed"))?;
    let batch = AnalyzerUpdateBatch::between(&main, target.clone(), generation)
        .map_err(|_| failed("probe update was rejected"))?;
    let changed = session
        .apply_update(batch, queries, false, stop)
        .map_err(|error| session_failure(error.code(), "native probe update failed"))?;
    checkpoint(stop)?;
    if session.main().snapshot_id() != target.snapshot_id()
        || changed.syntax.files().len() != 3
        || !changed.syntax.diagnostics().is_empty()
        || changed
            .member_calls
            .references()
            .iter()
            .any(|r| r.path() == "main/missing.lua")
    {
        return Err(failed(
            "native probe update/removal observations are incoherent",
        ));
    }
    let work = session.last_work();
    let mut report = json!({
        "schema":super::REPORT_SCHEMA,"schema_version":super::REPORT_SCHEMA_VERSION,"producer_version":2,
        "source":{"branch":"compiled-pin","revision":revision,"tree":tree,"relation":"unverified_current","network_checked":false},
        "workspace":{"compiled_probe":observations,"updated_probe":{"syntax":changed.syntax,"member_calls":changed.member_calls,"work":{"added_files":work.added_files,"updated_files":work.updated_files,"removed_files":work.removed_files,"reused_files":work.reused_files}}},
        "analysis_crate":{"manifest_path":"crates/emmylua_code_analysis/Cargo.toml","name":"emmylua_code_analysis","version":EMMYLUA_CODE_ANALYSIS_VERSION,"features":[]},
        "surface":{"files":files,"symbols":symbols,"surface_sha256":surface_digest},
        "compatibility":{"required_symbols":symbols,"missing_symbols":[],"status":"compatible"},
        "limitations":["Exact compiled pin; current upstream selector is NotEvaluated.","Scoped compiled probe only; full E0-C P1-P12, pin activation, rollback, diagnostic-family matrix, coordinate matrix, performance and runtime acceptance are NotEvaluated."]
    });
    report["report_sha256"] = Value::String(digest(&report)?);
    let bytes =
        canonical_json_bytes(&report).map_err(|_| failed("probe report serialization failed"))?;
    checkpoint(stop)?;
    super::backend_identity_from_report(&bytes)?;
    Ok(bytes)
}

fn digest(value: &Value) -> EmmyCompatibilityResult<String> {
    canonical_json_bytes(value)
        .map(|bytes| sha256(&bytes))
        .map_err(|_| failed("probe evidence cannot be canonicalized"))
}
fn failed(message: &str) -> super::EmmyCompatibilityError {
    report_error(EmmyCompatibilityErrorCode::ProbeFailed, message)
}
fn session_failure(code: AnalyzerSessionErrorCode, message: &str) -> super::EmmyCompatibilityError {
    if code == AnalyzerSessionErrorCode::Cancelled {
        report_error(
            EmmyCompatibilityErrorCode::Cancelled,
            "compiled probe was cancelled",
        )
    } else {
        failed(message)
    }
}
fn checkpoint(stop: &AtomicBool) -> EmmyCompatibilityResult<()> {
    if stop.load(Ordering::Relaxed) {
        return Err(report_error(
            EmmyCompatibilityErrorCode::Cancelled,
            "compiled probe was cancelled",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_fabricated_source_before_executing_the_backend() {
        let stop = AtomicBool::new(false);
        let mut members = vec![b"fabricated donor member".to_vec(); COMPILED_PROBE_MEMBERS.len()];
        let result = probe_compiled_backend(EMMYLUA_REVISION, EMMYLUA_TREE, &members, &stop);
        assert_eq!(
            result.err().map(|error| error.code()),
            Some(EmmyCompatibilityErrorCode::ProbeFailed)
        );
        let missing = probe_compiled_backend(EMMYLUA_REVISION, EMMYLUA_TREE, &[], &stop);
        assert_eq!(
            missing.err().map(|error| error.code()),
            Some(EmmyCompatibilityErrorCode::ProbeFailed)
        );
        members[0] = vec![0; COMPILED_PROBE_MEMBER_MAX_BYTES + 1];
        let result = probe_compiled_backend(EMMYLUA_REVISION, EMMYLUA_TREE, &members, &stop);
        assert_eq!(
            result.err().map(|error| error.code()),
            Some(EmmyCompatibilityErrorCode::ProbeFailed)
        );
        let result = probe_compiled_backend(
            "0000000000000000000000000000000000000000",
            EMMYLUA_TREE,
            &[],
            &stop,
        );
        assert_eq!(
            result.err().map(|error| error.code()),
            Some(EmmyCompatibilityErrorCode::ProbeFailed)
        );
    }

    #[test]
    fn cancelled_probe_never_produces_a_report() {
        let result =
            probe_compiled_backend(EMMYLUA_REVISION, EMMYLUA_TREE, &[], &AtomicBool::new(true));
        assert_eq!(
            result.err().map(|error| error.code()),
            Some(EmmyCompatibilityErrorCode::Cancelled)
        );
    }
}
