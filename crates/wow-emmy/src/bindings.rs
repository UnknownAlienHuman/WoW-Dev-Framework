//! Read-only symbol queries against an already populated Emmy Main/Library session.
//! Query paths are data, never generated Lua. Targets are analyzer observations,
//! not runtime bindings or proof that a symbol is absent from a WoW client.
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use emmylua_code_analysis::{
    EmmyLuaAnalysis, FileId, LuaMemberKey, LuaSemanticDeclId, LuaSignatureId, LuaType, LuaTypeOwner,
};
use serde::Serialize;
use wow_core::SourceSpan;

use crate::references::{
    EmmyMemberCallError, EmmyMemberCallErrorCode, EmmyMemberCallResult, ast_span, canonical_id,
    semantic_model,
};
use crate::{LuaWorkspaceFile, LuaWorkspaceSnapshot};

pub const SYMBOL_LOOKUP_PROFILE: &str = "wow-emmy/symbol-lookup/1";
const MAX_QUERIES: usize = 4096;
const MAX_CANDIDATES: usize = 256;
const MAX_TOTAL_TARGETS: usize = 65_536;
const MAX_RETAINED_TEXT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolLookupState {
    UniqueAnalyzerDeclaration,
    Ambiguous,
    NotObserved,
    Indeterminate,
    SourceParseFailed,
    UnsupportedPath,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct SymbolTarget {
    pub workspace_id: String,
    pub role: &'static str,
    pub path: String,
    pub content_digest: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SymbolLookup {
    pub state: SymbolLookupState,
    /// Number of leading path components processed; an ambiguous prefix is not
    /// misrepresented as a successful lookup of the final member.
    pub resolved_components: usize,
    pub targets: Vec<SymbolTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SymbolLookupReport {
    profile: &'static str,
    main_snapshot_id: String,
    library_snapshot_ids: Vec<String>,
    source_health_complete: bool,
    lookups: BTreeMap<String, SymbolLookup>,
    analysis_id: String,
}
impl SymbolLookupReport {
    #[must_use]
    pub fn analysis_id(&self) -> &str {
        &self.analysis_id
    }
    #[must_use]
    pub fn lookups(&self) -> &BTreeMap<String, SymbolLookup> {
        &self.lookups
    }
    #[must_use]
    pub const fn source_health_complete(&self) -> bool {
        self.source_health_complete
    }
}

/// A closed ASCII dotted-name dialect, not an expression parser. Empty segments,
/// calls, indexing, colon invocation and dynamic expressions are not evaluated.
#[must_use]
pub fn supported_path(path: &str) -> bool {
    if path.is_empty() || path.len() > 4096 || path.split('.').count() > 16 {
        return false;
    }
    path.split('.').all(|part| {
        let mut bytes = part.bytes();
        bytes
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
            && bytes.all(|c| c.is_ascii_alphanumeric() || c == b'_')
    })
}

pub(crate) fn validate_queries(queries: &[String]) -> EmmyMemberCallResult<()> {
    if queries.len() > MAX_QUERIES || queries.iter().any(|q| q.len() > 4096) {
        return Err(error(EmmyMemberCallErrorCode::FactBudgetExceeded));
    }
    Ok(())
}

pub(crate) fn checkpoint(stop: &AtomicBool) -> EmmyMemberCallResult<()> {
    if stop.load(Ordering::Acquire) {
        return Err(error(EmmyMemberCallErrorCode::Cancelled));
    }
    Ok(())
}
fn error(code: EmmyMemberCallErrorCode) -> EmmyMemberCallError {
    EmmyMemberCallError::new(
        code,
        "symbol lookup could not retain a coherent bounded result",
        None,
    )
}

struct Source<'a> {
    workspace: &'a str,
    role: &'static str,
    file: &'a LuaWorkspaceFile,
    healthy: bool,
}
struct Candidate {
    owner: LuaSemanticDeclId,
    typ: LuaType,
}

pub(crate) fn resolve(
    analysis: &EmmyLuaAnalysis,
    main: &LuaWorkspaceSnapshot,
    main_root: &Path,
    libraries: &[(&LuaWorkspaceSnapshot, PathBuf)],
    queries: &[String],
    mut callable_signatures: Option<&mut BTreeMap<String, LuaSignatureId>>,
    stop: &AtomicBool,
) -> EmmyMemberCallResult<SymbolLookupReport> {
    checkpoint(stop)?;
    validate_queries(queries)?;
    let mut sources = HashMap::new();
    let mut healthy = true;
    for (workspace, root, role) in std::iter::once((main, main_root, "main"))
        .chain(libraries.iter().map(|(w, r)| (*w, r.as_path(), "library")))
    {
        for file in workspace.files() {
            checkpoint(stop)?;
            let model = semantic_model(analysis, root, file)?;
            let parsed = model
                .get_file_parse_error()
                .is_none_or(|errors| errors.is_empty());
            healthy &= parsed;
            if sources
                .insert(
                    model.get_file_id(),
                    Source {
                        workspace: workspace.snapshot_id(),
                        role,
                        file,
                        healthy: parsed,
                    },
                )
                .is_some()
            {
                return Err(error(
                    EmmyMemberCallErrorCode::AnalyzerFileRegistrationFailed,
                ));
            }
        }
    }
    let main_file = main
        .files()
        .first()
        .ok_or_else(|| error(EmmyMemberCallErrorCode::InvalidMainWorkspace))?;
    let model = semantic_model(analysis, main_root, main_file)?;
    let mut lookups = BTreeMap::new();
    let mut target_count = 0usize;
    let mut text_bytes = 0usize;
    for query in queries.iter().collect::<BTreeSet<_>>() {
        checkpoint(stop)?;
        let (lookup, signature) = if supported_path(query) {
            resolve_one(analysis, model.get_db(), &sources, query, stop)?
        } else {
            (
                SymbolLookup {
                    state: SymbolLookupState::UnsupportedPath,
                    resolved_components: 0,
                    targets: Vec::new(),
                },
                None,
            )
        };
        // The optional graph sidecar retains a concrete callable identity from
        // this very lookup. Do not rerun name resolution or infer it from text.
        if let (Some(sink), Some(signature)) = (callable_signatures.as_deref_mut(), signature) {
            sink.insert(query.clone(), signature);
        }
        target_count = target_count
            .checked_add(lookup.targets.len())
            .ok_or_else(|| error(EmmyMemberCallErrorCode::FactBudgetExceeded))?;
        if target_count > MAX_TOTAL_TARGETS {
            return Err(error(EmmyMemberCallErrorCode::FactBudgetExceeded));
        }
        let retained = query.len()
            + lookup
                .targets
                .iter()
                .map(|t| t.workspace_id.len() + t.path.len() + t.content_digest.len())
                .sum::<usize>();
        text_bytes = text_bytes
            .checked_add(retained)
            .ok_or_else(|| error(EmmyMemberCallErrorCode::FactBudgetExceeded))?;
        if text_bytes > MAX_RETAINED_TEXT_BYTES {
            return Err(error(EmmyMemberCallErrorCode::FactBudgetExceeded));
        }
        lookups.insert(query.clone(), lookup);
    }
    let library_snapshot_ids: Vec<_> = libraries
        .iter()
        .map(|(w, _)| w.snapshot_id().to_owned())
        .collect();
    let analysis_id = canonical_id(
        "emmy-symbol-lookup:sha256:",
        &(
            SYMBOL_LOOKUP_PROFILE,
            main.backend(),
            main.snapshot_id(),
            &library_snapshot_ids,
            healthy,
            &lookups,
        ),
    )?;
    Ok(SymbolLookupReport {
        profile: SYMBOL_LOOKUP_PROFILE,
        main_snapshot_id: main.snapshot_id().into(),
        library_snapshot_ids,
        source_health_complete: healthy,
        lookups,
        analysis_id,
    })
}

fn resolve_one(
    analysis: &EmmyLuaAnalysis,
    db: &emmylua_code_analysis::DbIndex,
    sources: &HashMap<FileId, Source<'_>>,
    query: &str,
    stop: &AtomicBool,
) -> EmmyMemberCallResult<(SymbolLookup, Option<LuaSignatureId>)> {
    let parts: Vec<_> = query.split('.').collect();
    let Some(ids) = db.get_global_index().get_global_decl_ids(parts[0]) else {
        return Ok((
            SymbolLookup {
                state: SymbolLookupState::NotObserved,
                resolved_components: 0,
                targets: Vec::new(),
            },
            None,
        ));
    };
    if ids.len() > MAX_CANDIDATES {
        return Err(error(EmmyMemberCallErrorCode::FactBudgetExceeded));
    }
    let mut candidates = Vec::new();
    for id in ids {
        checkpoint(stop)?;
        if candidates
            .iter()
            .any(|c: &Candidate| c.owner == LuaSemanticDeclId::LuaDecl(*id))
        {
            continue;
        }
        let model = analysis
            .compilation
            .get_semantic_model(id.file_id)
            .ok_or_else(|| error(EmmyMemberCallErrorCode::SemanticModelUnavailable))?;
        candidates.push(Candidate {
            owner: LuaSemanticDeclId::LuaDecl(*id),
            typ: model.get_type(LuaTypeOwner::Decl(*id)),
        });
    }
    let mut resolved_components = 1;
    let mut incomplete = false;
    for part in parts.iter().skip(1) {
        checkpoint(stop)?;
        if candidates.len() != 1 {
            break;
        }
        let candidate = &candidates[0];
        let Some(file_id) = candidate.owner.get_file_id() else {
            incomplete = true;
            break;
        };
        if !sources.get(&file_id).is_some_and(|s| s.healthy) {
            incomplete = true;
            break;
        }
        if matches!(candidate.typ, LuaType::Unknown | LuaType::Any) {
            incomplete = true;
            break;
        }
        let model = analysis
            .compilation
            .get_semantic_model(file_id)
            .ok_or_else(|| error(EmmyMemberCallErrorCode::SemanticModelUnavailable))?;
        let Some(members) =
            model.get_member_info_with_key(&candidate.typ, LuaMemberKey::from(*part), true)
        else {
            incomplete = true;
            break;
        };
        if members.len() > MAX_CANDIDATES {
            return Err(error(EmmyMemberCallErrorCode::FactBudgetExceeded));
        }
        let mut next: Vec<Candidate> = Vec::new();
        for member in members {
            checkpoint(stop)?;
            let Some(owner) = member.property_owner_id else {
                incomplete = true;
                continue;
            };
            if let Some(previous) = next.iter().find(|c| c.owner == owner) {
                // Multiple overloads may share one declaration. A disagreement
                // cannot be collapsed to an arbitrary type for a further lookup.
                incomplete |= previous.typ != member.typ;
            } else {
                next.push(Candidate {
                    owner,
                    typ: member.typ,
                });
            }
        }
        candidates = next;
        resolved_components += 1;
        if incomplete {
            break;
        }
    }
    let mut targets = BTreeSet::new();
    let mut failed_source = false;
    for candidate in &candidates {
        checkpoint(stop)?;
        let range = match &candidate.owner {
            LuaSemanticDeclId::LuaDecl(id) => db
                .get_decl_index()
                .get_decl(id)
                .map(|d| (id.file_id, d.get_range())),
            LuaSemanticDeclId::Member(id) => db
                .get_member_index()
                .get_member(id)
                .map(|m| (id.file_id, m.get_range())),
            _ => None,
        };
        let Some((file_id, range)) = range else {
            incomplete = true;
            continue;
        };
        let Some(source) = sources.get(&file_id) else {
            incomplete = true;
            continue;
        };
        failed_source |= !source.healthy;
        targets.insert(SymbolTarget {
            workspace_id: source.workspace.into(),
            role: source.role,
            path: source.file.path().into(),
            content_digest: source.file.content_sha256().into(),
            span: ast_span(source.file, range)?,
        });
    }
    let state = if failed_source {
        SymbolLookupState::SourceParseFailed
    } else if incomplete {
        SymbolLookupState::Indeterminate
    } else if candidates.len() > 1 {
        SymbolLookupState::Ambiguous
    } else if targets.len() == 1 && resolved_components == parts.len() {
        SymbolLookupState::UniqueAnalyzerDeclaration
    } else if candidates.is_empty() {
        SymbolLookupState::NotObserved
    } else {
        SymbolLookupState::Indeterminate
    };
    let signature = if state == SymbolLookupState::UniqueAnalyzerDeclaration {
        match candidates.as_slice() {
            [
                Candidate {
                    typ: LuaType::Signature(id),
                    ..
                },
            ] => Some(*id),
            _ => None,
        }
    } else {
        None
    };
    Ok((
        SymbolLookup {
            state,
            resolved_components,
            targets: targets.into_iter().collect(),
        },
        signature,
    ))
}
