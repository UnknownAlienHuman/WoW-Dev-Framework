use crate::{SearchError, SearchResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub(super) enum SlotKey {
    String(String),
    Integer(String),
    Boolean(bool),
}

impl SlotKey {
    pub(super) fn validate(&self) -> SearchResult<()> {
        match self {
            SlotKey::String(value) => {
                if value.len() > 1024 {
                    Err(SearchError::InvalidProfile)
                } else {
                    Ok(())
                }
            }
            SlotKey::Boolean(_) => Ok(()),
            SlotKey::Integer(value) => {
                let parsed = match value.parse::<i64>() {
                    Ok(parsed) => parsed,
                    Err(_) => return Err(SearchError::InvalidProfile),
                };

                if parsed.to_string().as_str() != value.as_str() {
                    return Err(SearchError::InvalidProfile);
                }

                if !(-9007199254740991..=9007199254740991).contains(&parsed) {
                    return Err(SearchError::InvalidProfile);
                }

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_key_keeps_typed_slots_and_serializes_negative_integers() -> SearchResult<()> {
        let string_key = SlotKey::String("1".to_string());
        let integer_key = SlotKey::Integer("1".to_string());
        let boolean_key = SlotKey::Boolean(true);

        if string_key == integer_key || string_key == boolean_key || integer_key == boolean_key {
            return Err(SearchError::InvalidProfile);
        }

        let negative_json =
            match wow_core::canonical_json_string(&SlotKey::Integer("-1".to_string())) {
                Ok(json) => json,
                Err(_) => return Err(SearchError::InvalidProfile),
            };

        if negative_json != r#"{"kind":"integer","value":"-1"}"# {
            return Err(SearchError::InvalidProfile);
        }

        SlotKey::String(String::new()).validate()?;
        SlotKey::String("a".repeat(1024)).validate()?;
        SlotKey::Boolean(true).validate()?;
        SlotKey::Boolean(false).validate()?;

        SlotKey::Integer("-9007199254740991".to_string()).validate()?;
        SlotKey::Integer("9007199254740991".to_string()).validate()?;

        if SlotKey::String("a".repeat(1025)).validate().is_ok() {
            return Err(SearchError::InvalidProfile);
        }

        if SlotKey::Integer("-9007199254740992".to_string())
            .validate()
            .is_ok()
        {
            return Err(SearchError::InvalidProfile);
        }

        if SlotKey::Integer("9007199254740992".to_string())
            .validate()
            .is_ok()
        {
            return Err(SearchError::InvalidProfile);
        }

        if SlotKey::Integer("+1".to_string()).validate().is_ok() {
            return Err(SearchError::InvalidProfile);
        }

        if SlotKey::Integer("-0".to_string()).validate().is_ok() {
            return Err(SearchError::InvalidProfile);
        }

        if SlotKey::Integer("01".to_string()).validate().is_ok() {
            return Err(SearchError::InvalidProfile);
        }

        if SlotKey::Integer(" 1".to_string()).validate().is_ok() {
            return Err(SearchError::InvalidProfile);
        }

        if SlotKey::Integer("1.0".to_string()).validate().is_ok() {
            return Err(SearchError::InvalidProfile);
        }

        if SlotKey::Integer("1e5".to_string()).validate().is_ok() {
            return Err(SearchError::InvalidProfile);
        }

        if SlotKey::Integer("9223372036854775808".to_string())
            .validate()
            .is_ok()
        {
            return Err(SearchError::InvalidProfile);
        }

        if serde_json::from_str::<SlotKey>(r#"{"kind":"integer","value":"1","extra":0}"#).is_ok() {
            return Err(SearchError::InvalidProfile);
        }

        if serde_json::from_str::<SlotKey>(r#"{"kind":"float","value":"1"}"#).is_ok() {
            return Err(SearchError::InvalidProfile);
        }

        if serde_json::from_str::<SlotKey>(r#"{"kind":"integer","value":1}"#).is_ok() {
            return Err(SearchError::InvalidProfile);
        }

        Ok(())
    }
}
