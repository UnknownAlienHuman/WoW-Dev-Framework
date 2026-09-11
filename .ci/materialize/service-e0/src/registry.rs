use serde::Serialize;

pub(crate) const RULES_EVALUATE_ID: &str = "rules.evaluate";
pub(crate) const RULES_EVALUATE_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationDescriptor {
    pub operation_id: &'static str,
    pub version: u32,
    pub input_schema: &'static str,
    pub output_schema: &'static str,
    pub deterministic: bool,
    pub read_only: bool,
}

const DESCRIPTORS: [OperationDescriptor; 1] = [OperationDescriptor {
    operation_id: RULES_EVALUATE_ID,
    version: RULES_EVALUATE_VERSION,
    input_schema: "wow-service/rules-evaluate-request/1",
    output_schema: "wow-service/rules-evaluate-response/1",
    deterministic: true,
    read_only: true,
}];

pub struct OperationRegistry;

impl OperationRegistry {
    #[must_use]
    pub const fn descriptors() -> &'static [OperationDescriptor] {
        &DESCRIPTORS
    }

    #[must_use]
    pub fn resolve(operation_id: &str, version: u32) -> Option<&'static OperationDescriptor> {
        DESCRIPTORS.iter().find(|descriptor| {
            descriptor.operation_id == operation_id && descriptor.version == version
        })
    }

    #[must_use]
    pub fn contains_id(operation_id: &str) -> bool {
        DESCRIPTORS
            .iter()
            .any(|descriptor| descriptor.operation_id == operation_id)
    }
}
