use crate::{SearchError, SearchResult};

pub(super) fn authority_band(
    match_class: &str,
    source_band: u32,
) -> SearchResult<(u32, &'static str)> {
    match (match_class, source_band) {
        ("exact_owner_fact_identity", 0) => Ok((0, "original_component_band")),
        ("exact_recorded_source_name", 1) => Ok((4, "recorded_source_string_relation")),
        ("exact_recorded_receiver_member_syntax", 1) => Ok((4, "recorded_source_string_relation")),
        ("recorded_name_prefix_candidate", 2) => Ok((5, "recorded_source_prefix_candidate")),
        ("literal_recorded_name_candidate", 3) => Ok((7, "recorded_source_literal_candidate")),
        ("identifier_similarity_candidate", 4) => Ok((8, "recorded_source_similarity_candidate")),
        ("exact_recorded_global_slot_observation", 1) => {
            Ok((4, "recorded_source_slot_observation"))
        }
        ("possible_alias_global_slot_candidate", 2) => {
            Ok((10, "possible_source_alias_not_exact_slot"))
        }
        _ => Err(SearchError::OwnerEvidenceInvalid),
    }
}

#[cfg(test)]
mod tests {
    use super::authority_band;
    use crate::{SearchError, SearchResult};

    #[test]
    fn authority_band_strict_finite_mapping() -> SearchResult<()> {
        let allowed = [
            (
                "exact_owner_fact_identity",
                0u32,
                0u32,
                "original_component_band",
            ),
            (
                "exact_recorded_source_name",
                1u32,
                4u32,
                "recorded_source_string_relation",
            ),
            (
                "exact_recorded_receiver_member_syntax",
                1u32,
                4u32,
                "recorded_source_string_relation",
            ),
            (
                "recorded_name_prefix_candidate",
                2u32,
                5u32,
                "recorded_source_prefix_candidate",
            ),
            (
                "literal_recorded_name_candidate",
                3u32,
                7u32,
                "recorded_source_literal_candidate",
            ),
            (
                "identifier_similarity_candidate",
                4u32,
                8u32,
                "recorded_source_similarity_candidate",
            ),
        ];

        for (match_class, source_band, expected_band, expected_relation) in allowed {
            match authority_band(match_class, source_band) {
                Ok((band, relation)) if band == expected_band && relation == expected_relation => {}
                _ => return Err(SearchError::OwnerEvidenceInvalid),
            }
        }

        for (match_class, source_band, _, _) in allowed {
            if authority_band(match_class, source_band + 1).is_ok() {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
        }

        if authority_band("exact_canonical_name", 0).is_ok()
            || authority_band("alias_name", 1).is_ok()
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }

        Ok(())
    }
}
