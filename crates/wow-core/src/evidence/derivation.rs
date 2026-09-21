//! Registry-local validation only: no project graph, source acquisition or inference.

use std::collections::BTreeMap;

use super::{ClaimScope, EvidenceRecord, ProvenanceClass};
use crate::{CoreError, CoreErrorCode, CoreResult, EvidenceId};

const OPERATION: &str = "validate_evidence_derivation_graph";
const INPUTS: &str = "evidence_records.derivation_input_ids";

#[derive(Clone, Copy, PartialEq, Eq)]
enum VisitState {
    Unseen,
    Visiting,
    Complete,
}

pub(super) fn validate(input: &[EvidenceRecord]) -> CoreResult<()> {
    // Sorting references, not records, keeps caller bytes unchanged and makes
    // traversal/error selection independent of producer or insertion order.
    let mut records: Vec<_> = input.iter().collect();
    records.sort_unstable_by_key(|record| record.evidence_id());
    if records
        .windows(2)
        .any(|pair| pair[0].evidence_id() == pair[1].evidence_id())
    {
        return Err(error(
            CoreErrorCode::DuplicateEvidenceReference,
            "evidence_records.evidence_id",
        ));
    }
    if let Some(first) = records.first()
        && records
            .iter()
            .any(|record| record.context_id() != first.context_id())
    {
        return Err(error(
            CoreErrorCode::EvidenceContextMismatch,
            "evidence_records.context_id",
        ));
    }
    for record in &records {
        record.validate_fields()?;
    }
    let index: BTreeMap<_, _> = records
        .iter()
        .enumerate()
        .map(|(position, record)| (record.evidence_id(), position))
        .collect();
    for record in &records {
        for input_id in record.derivation_input_ids() {
            let parent = records[resolve(&index, input_id)?];
            // The enum's documented order is strongest to weakest. No edge may
            // promote Possible to Derived, or Candidate to Possible/Derived.
            if record.confidence() < parent.confidence() {
                return Err(error(CoreErrorCode::EvidenceAuthorityViolation, INPUTS));
            }
        }
    }

    validate_ancestry(&records, &index)?;
    // Cycles can be diagnosed on hostile decoded IDs without requiring a hash
    // fixed point. This ordering never admits them: every ID is checked on the
    // successful path, after structural and authority validation.
    for record in records {
        record.validate()?;
    }
    Ok(())
}

fn validate_ancestry(
    records: &[&EvidenceRecord],
    index: &BTreeMap<EvidenceId, usize>,
) -> CoreResult<()> {
    let mut states = vec![VisitState::Unseen; records.len()];
    let mut scenario_ancestry = vec![false; records.len()];
    // One active frame per record, not per path: shared inputs are never
    // re-expanded. Depth consumes heap entries rather than the process stack.
    let mut stack = Vec::new();
    for root in 0..records.len() {
        if states[root] == VisitState::Complete {
            continue;
        }
        states[root] = VisitState::Visiting;
        stack.push((root, 0));
        while let Some((position, next_input)) = stack.pop() {
            let record = records[position];
            if let Some(input_id) = record.derivation_input_ids().get(next_input) {
                let parent = resolve(index, input_id)?;
                if states[parent] == VisitState::Visiting {
                    return Err(error(CoreErrorCode::EvidenceDerivationCycle, INPUTS));
                }
                stack.push((position, next_input + 1));
                if states[parent] == VisitState::Unseen {
                    states[parent] = VisitState::Visiting;
                    stack.push((parent, 0));
                }
            } else {
                let mut restricted = record.provenance() == ProvenanceClass::RuntimeProbe
                    || record.claim_scope() == ClaimScope::RuntimeScenario;
                for input_id in record.derivation_input_ids() {
                    restricted |= scenario_ancestry[resolve(index, input_id)?];
                }
                if restricted && record.claim_scope() == ClaimScope::PlatformContract {
                    return Err(error(
                        CoreErrorCode::EvidenceAuthorityViolation,
                        "evidence_records.claim_scope",
                    ));
                }
                scenario_ancestry[position] = restricted;
                states[position] = VisitState::Complete;
            }
        }
    }
    Ok(())
}

fn resolve(index: &BTreeMap<EvidenceId, usize>, id: &EvidenceId) -> CoreResult<usize> {
    index
        .get(id)
        .copied()
        .ok_or_else(|| error(CoreErrorCode::MissingEvidenceReference, INPUTS))
}

fn error(code: CoreErrorCode, field: &'static str) -> CoreError {
    crate::error::validation_error(OPERATION, code, field)
}
