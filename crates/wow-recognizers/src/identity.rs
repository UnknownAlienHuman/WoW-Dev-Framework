use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{RecognizerError, RecognizerErrorCode, RecognizerResult};

macro_rules! text_id {
    ($name:ident, $label:literal, $max:expr, $validator:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Box<str>);

        impl $name {
            pub fn new(value: impl Into<Box<str>>) -> RecognizerResult<Self> {
                let value = value.into();
                if value.is_empty() || value.len() > $max || !$validator(&value) {
                    return Err(RecognizerError::new(
                        RecognizerErrorCode::IdentifierInvalid,
                        concat!("invalid ", $label),
                    ));
                }
                Ok(Self(value))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

fn component(value: &str) -> bool {
    value.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'@')
    })
}

fn observation_id(value: &str) -> bool {
    digest_id(value, "recognizer-observation:sha256:")
}

fn assertion_id(value: &str) -> bool {
    digest_id(value, "recognizer-assertion:sha256:")
}

fn report_id(value: &str) -> bool {
    digest_id(value, "recognition-report:sha256:")
}

fn registry_id(value: &str) -> bool {
    digest_id(value, "recognizer-registry:sha256:")
}

fn emmy_adapter_id(value: &str) -> bool {
    digest_id(value, "recognizer-emmy-adapter:sha256:")
}

fn digest_id(value: &str, prefix: &str) -> bool {
    let Some(hex) = value.strip_prefix(prefix) else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

text_id!(RecognizerId, "recognizer id", 256, component);
text_id!(RecognizerVersion, "recognizer version", 128, component);
text_id!(
    StructuredObservationId,
    "observation id",
    128,
    observation_id
);
text_id!(RecognitionAssertionId, "assertion id", 128, assertion_id);
text_id!(RecognitionReportId, "report id", 128, report_id);
text_id!(RecognizerRegistryId, "registry id", 128, registry_id);
text_id!(
    EmmyDirectCallAdapterId,
    "Emmy adapter id",
    128,
    emmy_adapter_id
);
