//! Exact configured declarations over actual Reference-owner projections.
use super::*;
use wow_reference::compatibility_aliases::AliasState;
#[derive(Serialize, Default)]
pub(super) struct AliasMatch {
    matched_records: Vec<String>,
    pub resolved: usize,
    pub unresolved: usize,
    pub conflicted: usize,
}
#[allow(clippy::too_many_arguments)]
pub(super) fn run<'a>(
    manifest: &'a Manifest,
    request: &ReferenceQueryRequest,
    universe: &str,
    query: &str,
    candidates: &mut BTreeMap<&'a str, Candidate<'a>>,
    scanned: &mut u32,
    signals: &mut u32,
    omitted: &mut u32,
    stop: &AtomicBool,
) -> SearchResult<(&'static str, AliasMatch)> {
    let data = manifest
        .compatibility_aliases
        .as_ref()
        .ok_or(SearchError::OwnerEvidenceInvalid)?;
    let value = request
        .selectors
        .explicit_alias
        .as_ref()
        .ok_or(SearchError::InvalidProfile)?;
    let mut report = AliasMatch::default();
    let mut rank = 0u32;
    let mut state = "complete";
    for record in &data.records {
        check(stop)?;
        if *scanned >= request.max_scanned_documents.get() {
            state = "truncated";
            break;
        }
        *scanned += 1;
        if record.value != *value {
            continue;
        }
        report.matched_records.push(record.record_key.clone());
        match record.state {
            AliasState::AliasConflict | AliasState::TargetConflict => {
                report.conflicted += 1;
                continue;
            }
            AliasState::Resolved => report.resolved += 1,
            _ => {
                report.unresolved += 1;
                continue;
            }
        }
        if *signals >= request.max_signals.get() {
            state = "truncated";
            break;
        }
        let index = manifest
            .documents
            .binary_search_by(|document| document.entity_key.cmp(&record.target_key))
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        let document = &manifest.documents[index];
        if !record.target_matches(document) {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        rank = rank
            .checked_add(1)
            .ok_or(SearchError::FileSetBudgetExceeded)?;
        let digest = identity("", &record.value)?;
        if !include_signal(
            candidates,
            document,
            Lane::ExactExplicitAlias,
            rank,
            (
                3,
                "exact_configured_owner_compatibility_alias",
                "CompatibilityAliasRecord.value()",
                digest.clone(),
                digest,
            ),
            request,
            universe,
            query,
            signals,
            omitted,
            None,
            Some(record.evidence()),
        )? {
            state = "truncated";
        }
    }
    Ok((state, report))
}
