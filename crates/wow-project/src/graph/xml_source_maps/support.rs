use super::*;

fn evidence(context: &GenerationContext, handle: StableHandleId) -> ProjectResult<EvidenceRecord> {
    EvidenceRecord::new(
        context.context_id(),
        ProvenanceClass::ProjectSource,
        EvidenceConfidence::Proven,
        ClaimScope::SourceObservation,
        "wow.project".parse::<ProducerId>().map_err(|_| invalid())?,
        ToolVersion::parse(env!("CARGO_PKG_VERSION")).map_err(|_| invalid())?,
        vec![handle],
        Vec::new(),
        Vec::new(),
    )
    .map_err(|_| invalid())
}

pub(super) fn fact_support(fact: &ProjectXmlFact) -> Support {
    Support {
        handle: fact.source_handle_id,
        evidence: fact.evidence_id,
    }
}

pub(super) fn validate_support(
    project: &ProjectView,
    source: &ProjectGraphProvenance,
    support: Support,
    document: &str,
    span: SourceSpan,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    let handle = source
        .source_handles()
        .get(&support.handle)
        .ok_or_else(invalid)?;
    let record = source
        .evidence()
        .get(&support.evidence)
        .ok_or_else(invalid)?;
    budget.charge_serialized(&("xml-map-support-validation", handle, record), stop)?;
    if handle.handle_id() != support.handle
        || handle.path().as_str() != document
        || handle.span() != span
        || project.source_handle(document, span, None)? != *handle
        || evidence(source.context(), support.handle)? != *record
        || record.evidence_id() != support.evidence
    {
        return Err(invalid());
    }
    crate::analyzer::checkpoint(stop)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn admit_piece(
    project: &ProjectView,
    source: &mut ProjectGraphProvenance,
    evidence_by_handle: &mut BTreeMap<StableHandleId, EvidenceId>,
    document: &str,
    segment: &XmlLuaMapSegment,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<Support> {
    budget.charge_serialized(
        &(
            "xml-map-piece-support-input",
            document,
            &segment.xml_span,
            source.context(),
        ),
        stop,
    )?;
    let span = SourceSpan::byte_range(segment.xml_span.byte_start, segment.xml_span.byte_end)
        .map_err(|_| invalid())?;
    let handle = project.source_handle(document, span, None)?;
    budget.charge_serialized(&handle, stop)?;
    let record = evidence(source.context(), handle.handle_id())?;
    budget.charge_serialized(&record, stop)?;
    let support = Support {
        handle: handle.handle_id(),
        evidence: record.evidence_id(),
    };
    match (
        source.source_handles.get(&support.handle),
        evidence_by_handle.get(&support.handle),
    ) {
        (Some(existing), Some(id)) => {
            if existing != &handle
                || *id != support.evidence
                || source.evidence.get(id) != Some(&record)
            {
                return Err(invalid());
            }
        }
        (None, None) => {
            if source.source_handles.len() >= MAX_PIECES || source.evidence.len() >= MAX_PIECES {
                return Err(exhausted());
            }
            if source.evidence.contains_key(&support.evidence) {
                return Err(invalid());
            }
            budget.charge_serialized(&("xml-map-new-support", support), stop)?;
            crate::analyzer::checkpoint(stop)?;
            source.source_handles.insert(support.handle, handle);
            source.evidence.insert(support.evidence, record);
            evidence_by_handle.insert(support.handle, support.evidence);
        }
        _ => return Err(invalid()),
    }
    crate::analyzer::checkpoint(stop)?;
    Ok(support)
}
