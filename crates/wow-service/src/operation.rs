use serde::Serialize;

use crate::ServiceResult;
use crate::identity::validate_identifier;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct OperationId(Box<str>);

impl OperationId {
    pub fn new(value: impl Into<Box<str>>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier(&value, "operation_id")?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
