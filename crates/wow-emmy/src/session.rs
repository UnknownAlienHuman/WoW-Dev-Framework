//! Retained physical Main analysis over exact, caller-supplied snapshots.
//!
//! Only changed texts enter the upstream parser. Semantic indexes are rebuilt
//! conservatively from retained VFS trees; no dependency or fact reuse is inferred.
#[cfg(test)]
mod tests;
use std::collections::BTreeSet;
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use emmylua_code_analysis::{EmmyLuaAnalysis, file_path_to_uri};
use wow_core::ProjectGenerationId;

use crate::references::{MemberCallSessionQueryProfile, RegisteredMemberAnalysis};
use crate::{EmmyLocalFlowReport, EmmyMemberCallReport, EmmySyntaxReport, LuaWorkspaceSnapshot};

/// Exact changes to one canonical logical Main path. New bytes come from target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnalyzerFileOperation {
    Add {
        path: Box<str>,
    },
    Update {
        path: Box<str>,
        expected_content_sha256: Box<str>,
    },
    Remove {
        path: Box<str>,
        expected_content_sha256: Box<str>,
    },
}

/// A complete final-state delta with an exact previous snapshot precondition.
#[derive(Debug, Clone)]
pub struct AnalyzerUpdateBatch {
    expected_previous_snapshot_id: Box<str>,
    target_project_generation: ProjectGenerationId,
    target: LuaWorkspaceSnapshot,
    operations: Vec<AnalyzerFileOperation>,
}

impl AnalyzerUpdateBatch {
    /// Derive all and only changed Main paths from two validated snapshots.
    pub fn between(
        previous: &LuaWorkspaceSnapshot,
        target: LuaWorkspaceSnapshot,
        target_project_generation: ProjectGenerationId,
    ) -> Result<Self, AnalyzerSessionError> {
        if previous.backend() != target.backend() || previous.universe() != target.universe() {
            return Err(error(AnalyzerSessionErrorCode::IncompatibleInputs));
        }
        let paths = previous
            .files()
            .iter()
            .chain(target.files())
            .map(|f| f.path())
            .collect::<BTreeSet<_>>();
        let operations = paths
            .into_iter()
            .filter_map(|path| match (previous.file(path), target.file(path)) {
                (None, Some(_)) => Some(AnalyzerFileOperation::Add { path: path.into() }),
                (Some(old), None) => Some(AnalyzerFileOperation::Remove {
                    path: path.into(),
                    expected_content_sha256: old.content_sha256().into(),
                }),
                (Some(old), Some(new)) if old.content_sha256() != new.content_sha256() => {
                    Some(AnalyzerFileOperation::Update {
                        path: path.into(),
                        expected_content_sha256: old.content_sha256().into(),
                    })
                }
                _ => None,
            })
            .collect();
        Ok(Self {
            expected_previous_snapshot_id: previous.snapshot_id().into(),
            target_project_generation,
            target,
            operations,
        })
    }

    #[must_use]
    pub fn expected_previous_snapshot_id(&self) -> &str {
        &self.expected_previous_snapshot_id
    }
    #[must_use]
    pub const fn target_project_generation(&self) -> ProjectGenerationId {
        self.target_project_generation
    }
    #[must_use]
    pub fn operations(&self) -> &[AnalyzerFileOperation] {
        &self.operations
    }
}

/// Observed Main parser work for the latest successful transition. Each changed
/// text is parsed in the separate syntax and semantic engines.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AnalyzerUpdateWork {
    pub added_files: usize,
    pub updated_files: usize,
    pub removed_files: usize,
    pub reused_files: usize,
}

/// Typed session failure; error text never includes upstream panic payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalyzerSessionErrorCode {
    Cancelled,
    StaleSnapshot,
    IncompatibleInputs,
    OutputBudgetExceeded,
    NativeFailure,
    Poisoned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzerSessionError {
    code: AnalyzerSessionErrorCode,
}
impl AnalyzerSessionError {
    #[must_use]
    pub const fn code(&self) -> AnalyzerSessionErrorCode {
        self.code
    }
}
impl fmt::Display for AnalyzerSessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self.code {
            AnalyzerSessionErrorCode::Cancelled => "native analysis cancelled",
            AnalyzerSessionErrorCode::StaleSnapshot => "native analysis previous snapshot differs",
            AnalyzerSessionErrorCode::IncompatibleInputs => {
                "native analysis inputs require a new session"
            }
            AnalyzerSessionErrorCode::NativeFailure => {
                "native analysis failed; session unavailable"
            }
            AnalyzerSessionErrorCode::OutputBudgetExceeded => {
                "native analysis output budget exceeded"
            }
            AnalyzerSessionErrorCode::Poisoned => {
                "native analysis session is unavailable after failure"
            }
        })
    }
}
impl std::error::Error for AnalyzerSessionError {}
fn error(code: AnalyzerSessionErrorCode) -> AnalyzerSessionError {
    AnalyzerSessionError { code }
}
fn checkpoint(stop: &AtomicBool) -> Result<(), AnalyzerSessionError> {
    if stop.load(Ordering::Relaxed) {
        Err(error(AnalyzerSessionErrorCode::Cancelled))
    } else {
        Ok(())
    }
}

fn member_error(source: crate::EmmyMemberCallError) -> AnalyzerSessionError {
    error(match source.code() {
        crate::EmmyMemberCallErrorCode::Cancelled => AnalyzerSessionErrorCode::Cancelled,
        crate::EmmyMemberCallErrorCode::FactBudgetExceeded => {
            AnalyzerSessionErrorCode::OutputBudgetExceeded
        }
        _ => AnalyzerSessionErrorCode::NativeFailure,
    })
}

/// Fresh immutable reports extracted against the target identities.
#[derive(Debug)]
pub struct AnalyzerSessionReports {
    pub syntax: EmmySyntaxReport,
    pub member_calls: EmmyMemberCallReport,
    pub symbol_lookup: Option<crate::bindings::SymbolLookupReport>,
    pub function_calls: Option<crate::function_calls::FunctionCallReport>,
    pub local_flow: EmmyLocalFlowReport,
}

/// Private mutable upstream state, never shared with a published read view.
#[derive(Debug)]
pub struct AnalyzerSession {
    main: LuaWorkspaceSnapshot,
    libraries: Vec<LuaWorkspaceSnapshot>,
    syntax_root: std::path::PathBuf,
    syntax: EmmyLuaAnalysis,
    semantic: RegisteredMemberAnalysis,
    healthy: bool,
    last_work: AnalyzerUpdateWork,
}

impl AnalyzerSession {
    /// Open the existing syntax policy and Main/Library semantic policy separately.
    pub fn open(
        main: LuaWorkspaceSnapshot,
        libraries: &[LuaWorkspaceSnapshot],
        stop: &AtomicBool,
    ) -> Result<Self, AnalyzerSessionError> {
        checkpoint(stop)?;
        let result = catch_unwind(AssertUnwindSafe(|| {
            let syntax_root = crate::syntax::analyzer::virtual_root(main.snapshot_id());
            let syntax = crate::syntax::analyzer::build_registered(&main, &syntax_root)
                .map_err(|_| error(AnalyzerSessionErrorCode::NativeFailure))?;
            checkpoint(stop)?;
            let semantic = RegisteredMemberAnalysis::new(
                &main,
                &libraries.iter().collect::<Vec<_>>(),
                None,
                stop,
            )
            .map_err(member_error)?;
            checkpoint(stop)?;
            let last_work = AnalyzerUpdateWork {
                added_files: main.files().len(),
                ..Default::default()
            };
            Ok(Self {
                main,
                libraries: libraries.to_vec(),
                syntax_root,
                syntax,
                semantic,
                healthy: true,
                last_work,
            })
        }));
        result.unwrap_or_else(|_| Err(error(AnalyzerSessionErrorCode::NativeFailure)))
    }

    #[must_use]
    pub const fn main(&self) -> &LuaWorkspaceSnapshot {
        &self.main
    }
    #[must_use]
    pub const fn last_work(&self) -> AnalyzerUpdateWork {
        self.last_work
    }
    #[must_use]
    pub fn compatible(
        &self,
        main: &LuaWorkspaceSnapshot,
        libraries: &[LuaWorkspaceSnapshot],
    ) -> bool {
        let mut expected = self
            .libraries
            .iter()
            .map(LuaWorkspaceSnapshot::snapshot_id)
            .collect::<Vec<_>>();
        let mut observed = libraries
            .iter()
            .map(LuaWorkspaceSnapshot::snapshot_id)
            .collect::<Vec<_>>();
        expected.sort_unstable();
        observed.sort_unstable();
        self.healthy
            && self.main.backend() == main.backend()
            && self.main.universe() == main.universe()
            && expected == observed
    }

    /// Extract all current reports without parsing supplied files again.
    pub fn reports(
        &mut self,
        queries: MemberCallSessionQueryProfile<'_>,
        function_calls: bool,
        stop: &AtomicBool,
    ) -> Result<AnalyzerSessionReports, AnalyzerSessionError> {
        if !self.healthy {
            return Err(error(AnalyzerSessionErrorCode::Poisoned));
        }
        checkpoint(stop)?;
        self.healthy = false;
        let result = catch_unwind(AssertUnwindSafe(|| {
            self.collect(queries, function_calls, stop)
        }))
        .unwrap_or_else(|_| Err(error(AnalyzerSessionErrorCode::NativeFailure)));
        self.healthy = result.is_ok();
        result
    }

    /// Validate CAS before touching either VFS. A post-mutation failure poisons
    /// the cache; callers retain their last immutable publication and reopen.
    pub fn apply_update(
        &mut self,
        batch: AnalyzerUpdateBatch,
        queries: MemberCallSessionQueryProfile<'_>,
        function_calls: bool,
        stop: &AtomicBool,
    ) -> Result<AnalyzerSessionReports, AnalyzerSessionError> {
        if !self.healthy {
            return Err(error(AnalyzerSessionErrorCode::Poisoned));
        }
        checkpoint(stop)?;
        if batch.expected_previous_snapshot_id() != self.main.snapshot_id() {
            return Err(error(AnalyzerSessionErrorCode::StaleSnapshot));
        }
        if !self.compatible(&batch.target, &self.libraries) {
            return Err(error(AnalyzerSessionErrorCode::IncompatibleInputs));
        }
        if batch.operations.is_empty() {
            self.last_work = AnalyzerUpdateWork {
                reused_files: self.main.files().len(),
                ..Default::default()
            };
            return self.reports(queries, function_calls, stop);
        }
        self.healthy = false;
        let result = catch_unwind(AssertUnwindSafe(|| {
            apply_delta(&mut self.syntax, &self.syntax_root, &batch, stop)?;
            apply_delta(
                &mut self.semantic.analysis,
                &self.semantic.main_root,
                &batch,
                stop,
            )?;
            checkpoint(stop)?;
            self.main = batch.target.clone();
            let reports = self.collect(queries, function_calls, stop)?;
            let mut work = AnalyzerUpdateWork::default();
            for operation in &batch.operations {
                match operation {
                    AnalyzerFileOperation::Add { .. } => work.added_files += 1,
                    AnalyzerFileOperation::Update { .. } => work.updated_files += 1,
                    AnalyzerFileOperation::Remove { .. } => work.removed_files += 1,
                }
            }
            work.reused_files = self.main.files().len() - work.added_files - work.updated_files;
            self.last_work = work;
            Ok(reports)
        }))
        .unwrap_or_else(|_| Err(error(AnalyzerSessionErrorCode::NativeFailure)));
        self.healthy = result.is_ok();
        result
    }

    fn collect(
        &self,
        queries: MemberCallSessionQueryProfile<'_>,
        function_calls: bool,
        stop: &AtomicBool,
    ) -> Result<AnalyzerSessionReports, AnalyzerSessionError> {
        checkpoint(stop)?;
        let syntax = crate::syntax::analyzer::collect_registered(
            &self.syntax,
            &self.main,
            &self.syntax_root,
        )
        .map_err(|_| error(AnalyzerSessionErrorCode::NativeFailure))?;
        checkpoint(stop)?;
        let session = self
            .semantic
            .collect(&self.main, None, None, queries, function_calls, stop)
            .map_err(member_error)?;
        checkpoint(stop)?;
        let local_flow = crate::flow::collect_registered(
            &self.semantic.analysis,
            &self.main,
            &self.semantic.main_root,
            &session.member_calls,
        )
        .map_err(|_| error(AnalyzerSessionErrorCode::NativeFailure))?;
        checkpoint(stop)?;
        Ok(AnalyzerSessionReports {
            syntax,
            member_calls: session.member_calls,
            symbol_lookup: session.symbol_lookup,
            function_calls: session.function_calls,
            local_flow,
        })
    }
}

fn apply_delta(
    analysis: &mut EmmyLuaAnalysis,
    root: &Path,
    batch: &AnalyzerUpdateBatch,
    stop: &AtomicBool,
) -> Result<(), AnalyzerSessionError> {
    for operation in &batch.operations {
        checkpoint(stop)?;
        let path = match operation {
            AnalyzerFileOperation::Add { path }
            | AnalyzerFileOperation::Update { path, .. }
            | AnalyzerFileOperation::Remove { path, .. } => path,
        };
        let uri = file_path_to_uri(&root.join(path.as_ref()))
            .ok_or_else(|| error(AnalyzerSessionErrorCode::NativeFailure))?;
        let vfs = analysis.compilation.get_db_mut().get_vfs_mut();
        match operation {
            AnalyzerFileOperation::Remove { .. } => {
                vfs.remove_file(&uri)
                    .ok_or_else(|| error(AnalyzerSessionErrorCode::NativeFailure))?;
            }
            _ => {
                let file = batch
                    .target
                    .file(path)
                    .ok_or_else(|| error(AnalyzerSessionErrorCode::NativeFailure))?;
                vfs.set_file_content(&uri, Some(file.text().to_owned()));
            }
        }
    }
    checkpoint(stop)?;
    // Recompute unknown semantic dependency closure while preserving VFS trees,
    // roots and configuration. Canonical path order avoids native allocation order.
    let vfs = analysis.compilation.get_db().get_vfs();
    let mut ids = vfs.get_all_file_ids();
    ids.sort_by(|a, b| vfs.get_file_path(a).cmp(&vfs.get_file_path(b)));
    analysis.compilation.clear_index();
    analysis.compilation.update_index(ids);
    checkpoint(stop)
}
