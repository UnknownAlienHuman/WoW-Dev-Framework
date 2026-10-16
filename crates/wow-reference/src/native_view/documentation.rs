use super::*;
use crate::native::RawValue;
use crate::native_model::{array, object};

/// Sealed documentation view over captured source metadata.
///
/// This value borrows the captured strings and does not by itself attest to
/// record identity; the parent signature row seals that identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeDocumentation<'a> {
    NotStated,
    Recorded(&'a [String]),
    Unsupported,
}

/// Owned source-data capture for native documentation metadata.
///
/// Values are retained exactly as supplied by native raw records, in original
/// order, without joining, sanitizing, trimming, or semantic interpretation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CapturedDocumentation {
    NotStated,
    Recorded(Vec<String>),
    Unsupported,
}

impl CapturedDocumentation {
    pub(super) fn capture(raw: &RawValue, stop: &AtomicBool) -> Result<Self, NativeError> {
        checkpoint(stop)?;

        let fields = object(raw).map_err(|_| error(NativeErrorCode::InvalidIdentity))?;

        let Some(field) = fields.get("Documentation") else {
            return Ok(Self::NotStated);
        };

        let items = match array(field) {
            Ok(items) => items,
            Err(_) => return Ok(Self::Unsupported),
        };

        let mut recorded = Vec::new();
        for item in items {
            checkpoint(stop)?;

            match &item.kind {
                RawKind::String(value) => recorded.push(value.clone()),
                _ => return Ok(Self::Unsupported),
            }
        }

        Ok(Self::Recorded(recorded))
    }

    pub(super) fn view(&self) -> NativeDocumentation<'_> {
        match self {
            Self::NotStated => NativeDocumentation::NotStated,
            Self::Recorded(values) => NativeDocumentation::Recorded(values.as_slice()),
            Self::Unsupported => NativeDocumentation::Unsupported,
        }
    }
}
