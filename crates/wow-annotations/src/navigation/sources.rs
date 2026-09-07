//! Separate immutable source universes for generated-to-source navigation.
use super::{LookupError, MAX_FILES, check_cancelled};
use crate::native::{NativeLibrary, SourceLink};
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

struct Identity<'a> {
    revision: &'a str,
    sha256: &'a str,
    bytes: usize,
}

pub(super) struct Sources<'a> {
    blizzard: BTreeMap<&'a str, Identity<'a>>,
    aliases: BTreeMap<&'a str, Identity<'a>>,
}

impl<'a> Sources<'a> {
    pub(super) fn new(
        library: &'a NativeLibrary<'_>,
        cancelled: &AtomicBool,
    ) -> Result<Self, LookupError> {
        if library.sources.is_empty() {
            return Err(LookupError::InvalidMapping);
        }
        if library.sources.len() > MAX_FILES {
            return Err(LookupError::InputLimit);
        }
        let mut result = Self {
            blizzard: BTreeMap::new(),
            aliases: BTreeMap::new(),
        };
        for document in &library.sources {
            check_cancelled(cancelled)?;
            if document.revision() != library.revision {
                return Err(LookupError::InvalidMapping);
            }
            let identity = Identity {
                revision: document.revision(),
                sha256: document.sha256(),
                bytes: document.source_bytes(),
            };
            if result.blizzard.insert(document.path(), identity).is_some() {
                return Err(LookupError::InvalidMapping);
            }
        }
        if let Some(report) = &library.aliases {
            if !matches!(
                library.schema,
                "wow-native-annotation-library/5" | "wow-native-annotation-library/6"
            ) || !matches!(
                report.schema,
                "wow-native-alias-projection/1"
                    | "wow-native-alias-projection/2"
                    | "wow-native-alias-projection/3"
                    | "wow-native-alias-projection/4"
            ) {
                return Err(LookupError::UnsupportedProfile);
            }
            if report.authority != "external_annotation_overlay"
                || report.additional_sources.len() >= crate::aliases::MAX_CATALOG_FILES
            {
                return Err(LookupError::InvalidMapping);
            }
            for document in
                std::iter::once(report.source).chain(report.additional_sources.iter().copied())
            {
                check_cancelled(cancelled)?;
                if document.revision() != report.source.revision() {
                    return Err(LookupError::InvalidMapping);
                }
                let identity = Identity {
                    revision: document.revision(),
                    sha256: document.sha256(),
                    bytes: document.text().len(),
                };
                if result.aliases.insert(document.path(), identity).is_some() {
                    return Err(LookupError::InvalidMapping);
                }
            }
        }
        Ok(result)
    }

    pub(super) fn revision(&self, link: &SourceLink) -> Result<&'a str, LookupError> {
        let sources = match link.scope {
            None => &self.blizzard,
            Some("annotation_alias_catalog") => &self.aliases,
            _ => return Err(LookupError::UnsupportedProfile),
        };
        let identity = sources
            .get(link.path.as_str())
            .ok_or(LookupError::InvalidMapping)?;
        if link.sha256 != identity.sha256
            || link.span.start >= link.span.end
            || link.span.end > identity.bytes
        {
            return Err(LookupError::InvalidMapping);
        }
        Ok(identity.revision)
    }
}
