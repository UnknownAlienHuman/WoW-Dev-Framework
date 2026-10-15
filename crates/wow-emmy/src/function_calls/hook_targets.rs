//! Closed W5 literal recipe; source expressions are read in their original session.
use super::*;
use emmylua_code_analysis::FileId;

/// Opt-in consumer: Project/Service W5 secure-hook endpoint handoff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookTargetRecipe {
    HookTargetLiteralsV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceHookTargetForm {
    GlobalName,
    TableMember,
}

/// A proven Main closure attached to the original call, not a spelling endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceHookTarget {
    form: SourceHookTargetForm,
    query: String,
    function_id: String,
    lookup_analysis_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    table_declaration: Option<SymbolTarget>,
}
impl SourceHookTarget {
    pub const fn form(&self) -> SourceHookTargetForm {
        self.form
    }
    pub fn query(&self) -> &str {
        &self.query
    }
    pub fn function_id(&self) -> &str {
        &self.function_id
    }
}

struct Request {
    call_index: usize,
    query: String,
    form: SourceHookTargetForm,
    table_declaration: Option<SymbolTarget>,
}

#[derive(Default)]
pub(super) struct Requests {
    pending: Vec<Request>,
    text_bytes: u64,
}

fn simple_name(name: &str) -> bool {
    !name.contains('.') && crate::bindings::supported_path(name)
}
fn literal_name(expr: LuaExpr) -> Option<String> {
    match call_literal(expr)? {
        SourceCallLiteral::String(name) if simple_name(&name) => Some(name),
        _ => None,
    }
}

impl Requests {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn observe(
        &mut self,
        call: &LuaCallExpr,
        target: &SourceCallTarget,
        model: &SemanticModel<'_>,
        sources: &HashMap<FileId, crate::global_access::AccessSource<'_>>,
        callables: &BTreeMap<String, LuaSignatureId>,
        signatures: &HashMap<LuaSignatureId, CapturedFunction>,
        call_index: usize,
        limits: FunctionCallBudget,
        query_limit: usize,
        stop: &AtomicBool,
    ) -> EmmyMemberCallResult<()> {
        checkpoint(stop)?;
        if call.is_colon_call() {
            return Ok(());
        }
        let Some(api) = callables.get("hooksecurefunc") else {
            return Ok(());
        };
        if !matches!((signatures.get(api), target),
            (Some(CapturedFunction::Library(selected)), SourceCallTarget::LibraryFunction { target: actual })
                if actual == selected)
        {
            return Ok(());
        }
        let Some(args) = call.get_args_list() else {
            return Ok(());
        };
        // Iterate at most four arguments; no unbounded collection before admission.
        let mut args = args.get_args();
        let (Some(first), Some(second)) = (args.next(), args.next()) else {
            return Ok(());
        };
        let third = args.next();
        if args.next().is_some() {
            return Ok(());
        }
        let (query, form, table_declaration) = if third.is_none() {
            let Some(name) = literal_name(first) else {
                return Ok(());
            };
            if call_literal(second).is_some() {
                return Ok(());
            }
            (name, SourceHookTargetForm::GlobalName, None)
        } else {
            let Some(member) = literal_name(second) else {
                return Ok(());
            };
            if third.is_some_and(|callback| call_literal(callback).is_some()) {
                return Ok(());
            }
            let LuaExpr::NameExpr(table) = first else {
                return Ok(());
            };
            let Some(name) = table.get_name_text() else {
                return Ok(());
            };
            if !simple_name(&name) {
                return Ok(());
            }
            // Local aliases/parameters are deliberately unsupported. Prove the
            // lexical cell and unique global owner in this exact argument scope.
            let Some(LuaSemanticDeclId::LuaDecl(id)) =
                crate::global_access::source_name_decl(model, &table)?
            else {
                return Ok(());
            };
            let db = model.get_db();
            if !db
                .get_global_index()
                .get_global_decl_ids(&name)
                .is_some_and(|ids| ids.as_slice() == [id])
            {
                return Ok(());
            }
            let Some(decl) = db.get_decl_index().get_decl(&id) else {
                return Ok(());
            };
            let Some(source) = sources.get(&id.file_id) else {
                return Ok(());
            };
            if !decl.is_global() || !source.is_main {
                return Ok(());
            }
            let target = SymbolTarget {
                workspace_id: source.workspace_id.to_owned(),
                role: "main",
                path: source.file.path().to_owned(),
                content_digest: source.file.content_sha256().to_owned(),
                span: ast_span(source.file, decl.get_range())?,
            };
            (
                format!("{name}.{member}"),
                SourceHookTargetForm::TableMember,
                Some(target),
            )
        };
        // Continuing lane: static callable inputs and every call request count.
        let attempted = callables.len() as u128 + self.pending.len() as u128 + 1;
        if attempted > u128::from(limits.max_named_targets()) {
            return Err(resource_budget(
                "hook named targets",
                attempted,
                limits.max_named_targets(),
                None,
            ));
        }
        if self.pending.len() >= query_limit {
            return Err(resource_budget(
                "hook target queries",
                self.pending.len() as u128 + 1,
                query_limit as u64,
                None,
            ));
        }
        self.text_bytes = self
            .text_bytes
            .checked_add(query.len() as u64 + 256)
            .ok_or_else(budget)?;
        if self.text_bytes > limits.max_report_bytes() {
            return Err(resource_budget(
                "hook target text",
                self.text_bytes as u128,
                limits.max_report_bytes(),
                None,
            ));
        }
        self.pending.push(Request {
            call_index,
            query,
            form,
            table_declaration,
        });
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn resolve(
        self,
        analysis: &EmmyLuaAnalysis,
        main: &LuaWorkspaceSnapshot,
        root: &Path,
        libraries: &[(&LuaWorkspaceSnapshot, PathBuf)],
        captured: &HashMap<LuaSignatureId, CapturedFunction>,
        accesses: &[crate::global_access::SourceGlobalAccess],
        calls: &mut [SourceCallFact],
        stop: &AtomicBool,
    ) -> EmmyMemberCallResult<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let writes = Writes::collect(accesses, stop)?;
        let queries = self
            .pending
            .iter()
            .map(|request| request.query.clone())
            .collect::<Vec<_>>();
        let mut signatures = BTreeMap::new();
        let lookup = crate::bindings::resolve(
            analysis,
            main,
            root,
            libraries,
            crate::bindings::QueryInput {
                names: &queries,
                limit: queries.len(),
            },
            Some(&mut signatures),
            stop,
        )?;
        if !lookup.source_health_complete() {
            return Ok(());
        }
        for request in self.pending {
            checkpoint(stop)?;
            if !writes.stable(&request.query) {
                continue;
            }
            let Some(CapturedFunction::Main(function_id)) = signatures
                .get(&request.query)
                .and_then(|signature| captured.get(signature))
            else {
                continue;
            };
            let call = calls.get_mut(request.call_index).ok_or_else(invalid)?;
            if let Some(callback_key) = call
                .arguments
                .last()
                .and_then(SourceCallArgument::reference_key)
                && !writes.stable(callback_key)
            {
                continue;
            }
            if let Some(table) = &request.table_declaration
                && call
                    .arguments
                    .first()
                    .and_then(SourceCallArgument::reference_target)
                    != Some(table)
            {
                continue;
            }
            call.hook_target = Some(SourceHookTarget {
                form: request.form,
                query: request.query,
                table_declaration: request.table_declaration,
                function_id: function_id.clone(),
                lookup_analysis_id: lookup.analysis_id().to_owned(),
            });
        }
        Ok(())
    }
}

/// Reuse the existing bounded traversal's writes. An inferred first signature
/// does not prove the global cell/member was never reassigned.
struct Writes {
    counts: BTreeMap<String, usize>,
    blocked_roots: BTreeSet<String>,
}
impl Writes {
    fn collect(
        accesses: &[crate::global_access::SourceGlobalAccess],
        stop: &AtomicBool,
    ) -> EmmyMemberCallResult<Self> {
        use crate::global_access::{GlobalAccessKey, GlobalAccessKind};
        let mut result = Self {
            counts: BTreeMap::new(),
            blocked_roots: BTreeSet::new(),
        };
        for access in accesses {
            checkpoint(stop)?;
            if access.kind() == GlobalAccessKind::Read {
                continue;
            }
            let mut key = access.root_name().to_owned();
            let mut complete = access.path_complete() && access.kind() == GlobalAccessKind::Write;
            for component in access.keys() {
                checkpoint(stop)?;
                let GlobalAccessKey::String(component) = component else {
                    complete = false;
                    break;
                };
                if !simple_name(component) {
                    complete = false;
                    break;
                }
                key.push('.');
                key.push_str(component);
            }
            if complete {
                *result.counts.entry(key).or_default() += 1;
            } else {
                result.blocked_roots.insert(access.root_name().to_owned());
            }
        }
        Ok(result)
    }
    fn stable(&self, query: &str) -> bool {
        let root = query.split('.').next().unwrap_or(query);
        !self.blocked_roots.contains(root)
            && self.counts.get(root) == Some(&1)
            && self.counts.get(query) == Some(&1)
    }
}

pub(super) fn validate(
    report: &FunctionCallReport,
    stop: Option<&AtomicBool>,
) -> EmmyMemberCallResult<()> {
    let functions = report
        .functions
        .iter()
        .map(|function| function.fact_id.as_str())
        .collect::<BTreeSet<_>>();
    for call in &report.calls {
        if let Some(stop) = stop {
            checkpoint(stop)?;
        }
        let Some(target) = &call.hook_target else {
            continue;
        };
        if report.hook_target_recipe.is_none()
            || !crate::bindings::supported_path(&target.query)
            || target.lookup_analysis_id.is_empty()
            || call.colon_call
            || !matches!((&call.target, report.named_targets.get("hooksecurefunc")),
                (SourceCallTarget::LibraryFunction { target: actual },
                 Some(SourceCallTarget::LibraryFunction { target: selected })) if actual == selected)
            || !functions.contains(target.function_id.as_str())
        {
            return Err(invalid());
        }
        match (target.form, call.arguments.as_slice()) {
            (SourceHookTargetForm::GlobalName, [name, _])
                if matches!(name.literal(), Some(SourceCallLiteral::String(name)) if name == &target.query)
                    && target.table_declaration.is_none() => {}
            (SourceHookTargetForm::TableMember, [table, member, _])
                if table.reference_target() == target.table_declaration.as_ref()
                    && target.table_declaration.is_some()
                    && matches!((table.reference_key(), member.literal()),
                        (Some(table), Some(SourceCallLiteral::String(member))) if target.query == format!("{table}.{member}")) =>
                {}
            _ => return Err(invalid()),
        }
    }
    Ok(())
}
