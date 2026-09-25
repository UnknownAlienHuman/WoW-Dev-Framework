//! Transport-only JSON admission. Domain validation remains with wow-graph.
use std::collections::BTreeSet;
use std::fmt;
use std::sync::atomic::AtomicBool;

use serde::de::{self, DeserializeOwned, DeserializeSeed, MapAccess, SeqAccess, Visitor};

use super::{GraphReadFailure, GraphReadStage, checkpoint};
use crate::ServiceErrorCode;

const MAX_DEPTH: usize = 64;
const MAX_STRING_BYTES: usize = 16 * 1024;

pub(super) fn decode<T: DeserializeOwned>(
    bytes: &[u8],
    max_bytes: usize,
    max_tokens: usize,
    stage: GraphReadStage,
    stop: &AtomicBool,
) -> Result<T, GraphReadFailure> {
    decode_profile(bytes, max_bytes, max_tokens, MAX_STRING_BYTES, stage, stop)
}

pub(super) fn decode_bundle<T: DeserializeOwned>(
    bytes: &[u8],
    max_bytes: usize,
    stop: &AtomicBool,
) -> Result<T, GraphReadFailure> {
    // Build receipts can retain inline Lua text larger than a graph identifier.
    // Do not widen the independent bare-snapshot profile.
    decode_profile(
        bytes,
        max_bytes,
        2_000_000,
        max_bytes,
        GraphReadStage::Bundle,
        stop,
    )
}

fn decode_profile<T: DeserializeOwned>(
    bytes: &[u8],
    max_bytes: usize,
    max_tokens: usize,
    max_string_bytes: usize,
    stage: GraphReadStage,
    stop: &AtomicBool,
) -> Result<T, GraphReadFailure> {
    checkpoint(stop)?;
    if bytes.len() > max_bytes {
        return Err(GraphReadFailure::service(
            stage,
            ServiceErrorCode::BudgetExceeded,
        ));
    }
    // This first pass retains only bounded object keys, not a duplicate DOM.
    // Decoded keys catch equivalent escape spellings before maps can overwrite.
    let mut budget = ScanBudget {
        remaining: max_tokens,
        max_string_bytes,
        failure: None,
    };
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    Scan {
        budget: &mut budget,
        depth: 0,
        stop,
    }
    .deserialize(&mut decoder)
    .and_then(|()| decoder.end())
    .map_err(|_| {
        GraphReadFailure::service(
            stage,
            budget.failure.unwrap_or(ServiceErrorCode::InvalidRequest),
        )
    })?;
    checkpoint(stop)?;
    let value = serde_json::from_slice(bytes)
        .map_err(|_| GraphReadFailure::service(stage, ServiceErrorCode::InvalidRequest))?;
    checkpoint(stop)?;
    Ok(value)
}

struct ScanBudget {
    remaining: usize,
    max_string_bytes: usize,
    failure: Option<ServiceErrorCode>,
}

struct Scan<'a> {
    budget: &'a mut ScanBudget,
    depth: usize,
    stop: &'a AtomicBool,
}
impl<'de> DeserializeSeed<'de> for Scan<'_> {
    type Value = ();
    fn deserialize<D: de::Deserializer<'de>>(self, decoder: D) -> Result<(), D::Error> {
        if checkpoint(self.stop).is_err() {
            self.budget.failure = Some(ServiceErrorCode::Cancelled);
            return Err(de::Error::custom("graph JSON admission stopped"));
        }
        if self.depth > MAX_DEPTH || self.budget.remaining == 0 {
            self.budget.failure = Some(ServiceErrorCode::BudgetExceeded);
            return Err(de::Error::custom("graph JSON admission limit"));
        }
        self.budget.remaining -= 1;
        decoder.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Scan<'_> {
    type Value = ();
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("bounded graph JSON")
    }
    fn visit_bool<E: de::Error>(self, _: bool) -> Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: de::Error>(self, _: u64) -> Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: de::Error>(self, _: i64) -> Result<(), E> {
        Ok(())
    }
    fn visit_str<E: de::Error>(self, value: &str) -> Result<(), E> {
        if value.len() > self.budget.max_string_bytes {
            self.budget.failure = Some(ServiceErrorCode::BudgetExceeded);
            Err(de::Error::custom("graph JSON string limit"))
        } else {
            Ok(())
        }
    }
    // Null and floating-point values are not in these graph wire profiles.
    fn visit_seq<A: SeqAccess<'de>>(self, mut items: A) -> Result<(), A::Error> {
        while items
            .next_element_seed(Scan {
                budget: &mut *self.budget,
                depth: self.depth + 1,
                stop: self.stop,
            })?
            .is_some()
        {}
        Ok(())
    }
    fn visit_map<A: MapAccess<'de>>(self, mut members: A) -> Result<(), A::Error> {
        let mut keys = BTreeSet::new();
        while let Some(key) = members.next_key::<String>()? {
            if key.len() > MAX_STRING_BYTES || self.budget.remaining == 0 {
                self.budget.failure = Some(ServiceErrorCode::BudgetExceeded);
                return Err(de::Error::custom("graph JSON key limit"));
            }
            if !keys.insert(key) {
                return Err(de::Error::custom("duplicate graph JSON key"));
            }
            self.budget.remaining -= 1;
            members.next_value_seed(Scan {
                budget: &mut *self.budget,
                depth: self.depth + 1,
                stop: self.stop,
            })?;
        }
        Ok(())
    }
}
