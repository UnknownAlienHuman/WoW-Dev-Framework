//! Source-owned callable occurrences and call-site evidence, not semantic Calls edges.
use super::*;
use wow_emmy::function_calls::SourceFunctionKind;

pub(super) const MAX_FUNCTIONS: usize = 8192;
pub(super) const MAX_CALLS: usize = 8192;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphFunction {
    pub function_id: String,
    pub path: String,
    pub kind: SourceFunctionKind,
    pub span: SourceSpan,
    pub proposal_id: String,
    pub ownership_proposal_id: String,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphCallSite {
    pub call_id: String,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
}

pub(super) struct FunctionProposals {
    pub entities: Vec<GraphEntityProposal>,
    pub relations: Vec<GraphRelationProposal>,
}

pub(super) fn project(
    project: &ProjectView,
    file_ids: &BTreeMap<&str, String>,
    provenance: &mut ProjectGraphProvenance,
    text_bytes: &mut usize,
    stop: &AtomicBool,
) -> ProjectResult<FunctionProposals> {
    let analyzer = project.snapshot().analyzer_binding();
    let Some(report) = analyzer.function_call_report() else {
        return Ok(FunctionProposals {
            entities: Vec::new(),
            relations: Vec::new(),
        });
    };
    report.validate().map_err(|_| invalid())?;
    if report.main_snapshot_id() != analyzer.main_workspace().snapshot_id()
        || report
            .library_snapshot_ids()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            != analyzer.library_snapshot_ids().collect::<Vec<_>>()
        || report.files().len() != project.file_manifest().len()
    {
        return Err(invalid());
    }
    if report.functions().len() > MAX_FUNCTIONS || report.calls().len() > MAX_CALLS {
        return Err(exhausted());
    }
    for file in report.files() {
        crate::analyzer::checkpoint(stop)?;
        let captured = project.file_by_path(&file.path)?.ok_or_else(invalid)?;
        if captured.content_digest()
            != crate::identity::parse_source_digest(&file.content_digest, ProjectPhase::View)?
        {
            return Err(invalid());
        }
    }
    let sources = analyzer
        .main_workspace()
        .files()
        .iter()
        .map(|file| {
            let captured = project.file_by_path(file.path())?.ok_or_else(invalid)?;
            Ok((
                file.path(),
                (
                    file,
                    LoadSource {
                        path: file.path().into(),
                        content_digest: captured.content_digest(),
                        byte_length: captured.byte_length(),
                    },
                ),
            ))
        })
        .collect::<ProjectResult<BTreeMap<_, _>>>()?;
    let mut result = FunctionProposals {
        entities: Vec::new(),
        relations: Vec::new(),
    };
    for function in report.functions() {
        crate::analyzer::checkpoint(stop)?;
        let source = source(
            &sources,
            function.path(),
            function.content_digest(),
            function.span(),
        )?;
        charge(
            text_bytes,
            function.path().len().saturating_mul(4).saturating_add(512),
        )?;
        let file_id = file_ids.get(function.path()).ok_or_else(invalid)?;
        let proposal_id = format!("function:{}", function.fact_id());
        let ownership_id = format!("function-owner:{}", function.fact_id());
        let (handle, evidence) = support(project, source, function.span(), provenance)?;
        result.entities.push(
            GraphEntityProposal::new(
                proposal_id.as_str(),
                "lua_source_function",
                BTreeMap::from([
                    (
                        "document".into(),
                        GraphProposalValue::String(function.path().into()),
                    ),
                    (
                        "function".into(),
                        GraphProposalValue::String(function.fact_id().into()),
                    ),
                ]),
                GraphConfidence::Derived,
                vec![handle],
                vec![evidence],
                Vec::new(),
            )
            .map_err(|_| invalid())?,
        );
        result.relations.push(
            GraphRelationProposal::new(
                ownership_id.as_str(),
                "source_declaration_owns",
                GraphRelationProposalInput {
                    source: GraphProposalEndpoint::Proposed(file_id.clone().into()),
                    target: GraphProposalEndpoint::Proposed(proposal_id.clone().into()),
                    confidence: GraphConfidence::Derived,
                    source_handle_ids: vec![handle],
                    evidence_ids: vec![evidence],
                    coverage_ids: Vec::new(),
                },
            )
            .map_err(|_| invalid())?,
        );
        provenance.functions.push(ProjectGraphFunction {
            function_id: function.fact_id().into(),
            path: function.path().into(),
            kind: function.kind(),
            span: function.span(),
            proposal_id,
            ownership_proposal_id: ownership_id,
            source_handle_id: handle,
            evidence_id: evidence,
        });
    }
    for call in report.calls() {
        crate::analyzer::checkpoint(stop)?;
        let source = source(
            &sources,
            call.path(),
            call.content_digest(),
            call.call_span(),
        )?;
        charge(
            text_bytes,
            call.path().len().saturating_mul(2).saturating_add(256),
        )?;
        let (handle, evidence) = support(project, source, call.call_span(), provenance)?;
        provenance.call_sites.push(ProjectGraphCallSite {
            call_id: call.fact_id().into(),
            source_handle_id: handle,
            evidence_id: evidence,
        });
    }
    provenance.function_call_report = Some(report.clone());
    Ok(result)
}

fn source<'a>(
    sources: &'a BTreeMap<&str, (&wow_emmy::LuaWorkspaceFile, LoadSource)>,
    path: &str,
    digest: &str,
    span: SourceSpan,
) -> ProjectResult<&'a LoadSource> {
    let (file, source) = sources.get(path).ok_or_else(invalid)?;
    if file.content_sha256() != digest {
        return Err(invalid());
    }
    if span != SourceSpan::whole_file() {
        let (Some(start), Some(end)) = (span.byte_start(), span.byte_end()) else {
            return Err(invalid());
        };
        let (start, end) = (
            usize::try_from(start).map_err(|_| invalid())?,
            usize::try_from(end).map_err(|_| invalid())?,
        );
        // Validate exact captured UTF-8 boundaries, without reopening source.
        if file.text().get(start..end).is_none() {
            return Err(invalid());
        }
    }
    Ok(source)
}
