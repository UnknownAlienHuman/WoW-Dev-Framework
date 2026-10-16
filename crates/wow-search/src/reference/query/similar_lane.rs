//! Explicit approximate identifier signals over captured native owner fields.
use super::similarity::{DistanceResult, bounded_distance, normalize};
use super::*;
const ALGORITHM_PROFILE: &str = "wow-search/scalar-banded-levenshtein-k4/1";

#[derive(Serialize)]
pub(super) struct SimilarityEvidence {
    algorithm_profile: &'static str,
    pub(super) normalization_profile: &'static str,
    distance: u32,
    max_distance: u32,
    distance_cells: u64,
    original_query_digest: Box<str>,
    normalized_query_digest: Box<str>,
    original_identifier_digest: Box<str>,
    normalized_identifier_digest: Box<str>,
    ascii_case_only_difference: bool,
}
#[derive(Serialize)]
pub(super) struct SimilarityWork {
    algorithm_profile: &'static str,
    normalization_profile: &'static str,
    original_query_digest: Box<str>,
    normalized_query_digest: Box<str>,
    prefiltered_documents: u32,
    compared_fields: u32,
    length_rejected_fields: u32,
    distance_cells: u64,
    matched_signals: u32,
    signals_omitted_by_bound: u32,
}
struct Hit<'a> {
    document: &'a Document,
    material: MatchMaterial,
    proof: SimilarityEvidence,
    normalized: String,
    field_order: u8,
}
impl Hit<'_> {
    fn key(&self) -> (u32, &str, &str, u8) {
        (
            self.proof.distance,
            &self.normalized,
            &self.document.entity_key,
            self.field_order,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run<'a>(
    documents: &'a [Document],
    request: &ReferenceQueryRequest,
    universe: &str,
    query_id: &str,
    candidates: &mut BTreeMap<&'a str, Candidate<'a>>,
    scanned: &mut u32,
    signals: &mut u32,
    omitted: &mut u32,
    stop: &AtomicBool,
) -> SearchResult<(&'static str, SimilarityWork)> {
    let selector = request
        .selectors
        .identifier_similarity
        .as_ref()
        .ok_or(SearchError::InvalidProfile)?;
    let fold = matches!(selector.case_policy, SimilarityCase::AsciiFold);
    let normalization = if fold {
        "ascii-fold-scalar-identity/1"
    } else {
        "utf8-scalar-identity/1"
    };
    let query = normalize(&selector.identifier, fold);
    let normalized_query = query.iter().collect::<String>();
    let mut work = SimilarityWork {
        algorithm_profile: ALGORITHM_PROFILE,
        normalization_profile: normalization,
        original_query_digest: identity("", &selector.identifier)?,
        normalized_query_digest: identity("", &normalized_query)?,
        prefiltered_documents: 0,
        compared_fields: 0,
        length_rejected_fields: 0,
        distance_cells: 0,
        matched_signals: 0,
        signals_omitted_by_bound: 0,
    };
    let limit = request.max_signals.get().saturating_sub(*signals) as usize;
    if limit == 0 {
        return Ok(("truncated", work));
    }
    let mut remaining = selector.max_distance_cells.get();
    let mut hits = Vec::<Hit<'_>>::new();
    let mut state = "complete";
    'documents: for document in documents {
        check(stop)?;
        if *scanned >= request.max_scanned_documents.get() {
            state = "truncated";
            break;
        }
        *scanned += 1;
        let fields = [
            (
                0u8,
                name_origin(document, true),
                &document.canonical_qualified_name,
            ),
            (
                1u8,
                name_origin(document, false),
                &document.canonical_short_name,
            ),
        ];
        let mut admitted = false;
        for (field_order, field, original) in fields {
            check(stop)?;
            if (field_order == 0 && matches!(selector.field, SimilarityField::ShortName))
                || (field_order == 1 && matches!(selector.field, SimilarityField::QualifiedName))
            {
                continue;
            }
            let length = original.chars().count();
            if query.len().abs_diff(length) > selector.max_distance as usize {
                work.length_rejected_fields = work
                    .length_rejected_fields
                    .checked_add(1)
                    .ok_or(SearchError::FileSetBudgetExceeded)?;
                continue;
            }
            if !admitted {
                if work.prefiltered_documents >= selector.max_prefiltered_documents.get() {
                    state = "truncated";
                    break 'documents;
                }
                work.prefiltered_documents += 1;
                admitted = true;
            }
            work.compared_fields = work
                .compared_fields
                .checked_add(1)
                .ok_or(SearchError::FileSetBudgetExceeded)?;
            let normalized = normalize(original, fold);
            let before = remaining;
            let result = bounded_distance(
                &query,
                &normalized,
                selector.max_distance,
                &mut remaining,
                stop,
            )?;
            match result {
                DistanceResult::Outside => {}
                DistanceResult::Exhausted => {
                    state = "truncated";
                    break 'documents;
                }
                DistanceResult::Within(distance) => {
                    work.matched_signals = work
                        .matched_signals
                        .checked_add(1)
                        .ok_or(SearchError::FileSetBudgetExceeded)?;
                    let normalized = normalized.iter().collect::<String>();
                    let original_digest = if field_order == 0 {
                        name_digest(document)?
                    } else {
                        identity("", original)?
                    };
                    let normalized_digest = identity("", &normalized)?;
                    let proof = SimilarityEvidence {
                        algorithm_profile: ALGORITHM_PROFILE,
                        normalization_profile: normalization,
                        distance,
                        max_distance: selector.max_distance,
                        distance_cells: before - remaining,
                        original_query_digest: work.original_query_digest.clone(),
                        normalized_query_digest: work.normalized_query_digest.clone(),
                        original_identifier_digest: identity("", original)?,
                        normalized_identifier_digest: normalized_digest.clone(),
                        ascii_case_only_difference: fold
                            && distance == 0
                            && original != &selector.identifier,
                    };
                    let hit = Hit {
                        document,
                        material: (
                            8,
                            "bounded_identifier_similarity_candidate",
                            field,
                            original_digest,
                            normalized_digest,
                        ),
                        proof,
                        normalized,
                        field_order,
                    };
                    // Keep a bounded best ordinal set, not the first matching
                    // witnesses. Exhaustion remains visible even if all retained hits fit.
                    let position = hits
                        .binary_search_by(|old| old.key().cmp(&hit.key()))
                        .unwrap_or_else(|position| position);
                    if position < limit {
                        hits.insert(position, hit);
                    }
                    if hits.len() > limit {
                        hits.pop();
                    }
                    if work.matched_signals as usize > limit {
                        state = "truncated";
                        work.signals_omitted_by_bound = work
                            .signals_omitted_by_bound
                            .checked_add(1)
                            .ok_or(SearchError::FileSetBudgetExceeded)?;
                    }
                }
            }
        }
    }
    work.distance_cells = selector.max_distance_cells.get() - remaining;
    for (index, hit) in hits.into_iter().enumerate() {
        check(stop)?;
        let rank = u32::try_from(index + 1).map_err(|_| SearchError::FileSetBudgetExceeded)?;
        if !include_signal(
            candidates,
            hit.document,
            Lane::IdentifierSimilarity,
            rank,
            hit.material,
            request,
            universe,
            query_id,
            signals,
            omitted,
            Some(hit.proof),
            None,
        )? {
            state = "truncated";
        }
    }
    Ok((state, work))
}
