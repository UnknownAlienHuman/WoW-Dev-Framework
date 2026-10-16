//! Bounded original source field filters; no type resolution or widening.
use super::*;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Nilability {
    Unspecified,
    AllowsNil,
    DisallowsNil,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum InnerType {
    Unspecified,
    Equals { original_type: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FieldFilter {
    pub(super) position: u32,
    pub(super) original_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) nilability: Option<Nilability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) inner_type: Option<InnerType>,
}

pub(super) fn validate_fields(filters: &[FieldFilter]) -> SearchResult<()> {
    if filters.len() > 128 {
        return Err(SearchError::InvalidProfile);
    }

    let mut positions = BTreeSet::new();
    for filter in filters {
        if !positions.insert(filter.position) {
            return Err(SearchError::InvalidProfile);
        }
        valid_text(&filter.original_type)?;
        if let Some(InnerType::Equals { original_type }) = &filter.inner_type {
            valid_text(original_type)?;
        }
    }
    Ok(())
}

pub(super) fn fields_match(fields: &[Field], filters: &[FieldFilter]) -> bool {
    filters.iter().all(|filter| {
        let Some(field) = fields.get(filter.position as usize) else {
            return false;
        };
        if field.original_type != filter.original_type {
            return false;
        }

        if let Some(nilability) = &filter.nilability {
            let ok = match nilability {
                Nilability::Unspecified => field.nilable.is_none(),
                Nilability::AllowsNil => field.nilable == Some(true),
                Nilability::DisallowsNil => field.nilable == Some(false),
            };
            if !ok {
                return false;
            }
        }

        if let Some(inner_type) = &filter.inner_type {
            let ok = match inner_type {
                InnerType::Unspecified => field.original_inner_type.is_none(),
                InnerType::Equals { original_type } => {
                    field.original_inner_type.as_deref() == Some(original_type.as_str())
                }
            };
            if !ok {
                return false;
            }
        }

        true
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_field(original_type: &str, inner_type: Option<&str>, nilable: Option<bool>) -> Field {
        Field {
            name: "field".to_string(),
            original_type: original_type.to_string(),
            original_inner_type: inner_type.map(str::to_string),
            nilable,
            byte_range: [0, 1],
        }
    }

    fn make_filter(
        position: u32,
        original_type: &str,
        nilability: Option<Nilability>,
        inner_type: Option<InnerType>,
    ) -> FieldFilter {
        FieldFilter {
            position,
            original_type: original_type.to_string(),
            nilability,
            inner_type,
        }
    }

    #[test]
    fn absent_inner_type_is_unconstrained() {
        let field = make_field("number", Some("string"), Some(true));
        let filter = make_filter(0, "number", None, None);
        assert!(fields_match(
            std::slice::from_ref(&field),
            std::slice::from_ref(&filter)
        ));
    }

    #[test]
    fn unspecified_inner_type_matches_none() {
        let field = make_field("number", None, Some(true));
        let filter = make_filter(0, "number", None, Some(InnerType::Unspecified));
        assert!(fields_match(
            std::slice::from_ref(&field),
            std::slice::from_ref(&filter)
        ));

        let field = make_field("number", Some("string"), Some(true));
        assert!(!fields_match(
            std::slice::from_ref(&field),
            std::slice::from_ref(&filter)
        ));
    }

    #[test]
    fn equals_inner_type_matches_exact_and_is_case_sensitive() {
        let field = make_field("number", Some("string"), Some(true));
        let filter = make_filter(
            0,
            "number",
            None,
            Some(InnerType::Equals {
                original_type: "string".to_string(),
            }),
        );
        assert!(fields_match(
            std::slice::from_ref(&field),
            std::slice::from_ref(&filter)
        ));
        assert!(!fields_match(
            &[make_field("number", Some("String"), Some(true))],
            std::slice::from_ref(&filter)
        ));
    }

    #[test]
    fn nilability_absent_unspecified_allows_and_disallows() {
        let any = make_field("number", None, Some(true));
        assert!(fields_match(
            std::slice::from_ref(&any),
            &[make_filter(0, "number", None, None)]
        ));

        let unspecified_field = make_field("number", None, None);
        assert!(fields_match(
            std::slice::from_ref(&unspecified_field),
            &[make_filter(
                0,
                "number",
                Some(Nilability::Unspecified),
                None
            )]
        ));
        assert!(!fields_match(
            std::slice::from_ref(&any),
            &[make_filter(
                0,
                "number",
                Some(Nilability::Unspecified),
                None
            )]
        ));

        assert!(fields_match(
            std::slice::from_ref(&any),
            &[make_filter(0, "number", Some(Nilability::AllowsNil), None)]
        ));
        assert!(!fields_match(
            std::slice::from_ref(&any),
            &[make_filter(
                0,
                "number",
                Some(Nilability::DisallowsNil),
                None
            )]
        ));
    }

    #[test]
    fn wrong_position_or_type_does_not_match() {
        let field = make_field("number", None, Some(false));
        assert!(!fields_match(
            std::slice::from_ref(&field),
            &[make_filter(1, "number", None, None)]
        ));
        assert!(!fields_match(
            std::slice::from_ref(&field),
            &[make_filter(0, "string", None, None)]
        ));
    }

    #[test]
    fn strict_unknown_fields_are_rejected() {
        let result = serde_json::from_str::<FieldFilter>(
            r#"{"position":0,"original_type":"number","unknown":true}"#,
        );
        assert!(result.is_err());

        let result = serde_json::from_str::<InnerType>(
            r#"{"kind":"equals","original_type":"number","unknown":true}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn invalid_filters_are_rejected() {
        assert!(
            validate_fields(&[
                make_filter(0, "number", None, None),
                make_filter(0, "number", None, None)
            ])
            .is_err()
        );

        let too_many: Vec<_> = (0..129)
            .map(|i| make_filter(i, "number", None, None))
            .collect();
        assert!(validate_fields(&too_many).is_err());

        assert!(validate_fields(&[make_filter(0, "", None, None)]).is_err());
        assert!(
            validate_fields(&[make_filter(
                0,
                "number",
                None,
                Some(InnerType::Equals {
                    original_type: String::new()
                })
            )])
            .is_err()
        );
    }

    #[test]
    fn canonical_bytes_preserve_absent_inner_identity() -> SearchResult<()> {
        let filter = make_filter(7, "number", Some(Nilability::AllowsNil), None);
        let bytes =
            wow_core::canonical_json_bytes(&filter).map_err(|_| SearchError::InvalidProfile)?;
        let restored: FieldFilter =
            serde_json::from_slice(&bytes).map_err(|_| SearchError::InvalidProfile)?;
        assert_eq!(restored, filter);
        let legacy =
            serde_json::json!({"position":7,"original_type":"number","nilability":"allows_nil"});
        assert_eq!(
            bytes,
            wow_core::canonical_json_bytes(&legacy).map_err(|_| SearchError::InvalidProfile)?
        );
        Ok(())
    }
}
