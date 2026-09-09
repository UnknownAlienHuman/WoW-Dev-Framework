//! Separate immutable source universes for generated-to-source navigation.
use super::{LookupError, MAX_FILES, check_cancelled};
use crate::native::{NativeLibrary, SourceLink};
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
use wow_reference::native::source_digest;

/// A viewed source file, independent of any particular descriptor range.
/// `None` identifies Blizzard input; external aliases use their explicit scope.
#[derive(Clone, Copy, Debug)]
pub struct SourceFile<'a> {
    pub scope: Option<&'a str>,
    pub revision: &'a str,
    pub path: &'a str,
    pub sha256: &'a str,
}

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
                    | "wow-native-alias-projection/5"
                    | "wow-native-alias-projection/6"
                    | "wow-native-alias-projection/7"
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

    pub(super) fn validate_text(
        &self,
        file: SourceFile<'_>,
        text: &str,
        cancelled: &AtomicBool,
    ) -> Result<(), LookupError> {
        check_cancelled(cancelled)?;
        let identity = self.identity(file.scope, file.path)?;
        // Check byte bounds and the recorded length before hashing caller input.
        if text.len() > crate::ketho::MAX_OUTPUT_BYTES {
            return Err(LookupError::InputLimit);
        }
        if file.revision != identity.revision
            || file.sha256 != identity.sha256
            || text.len() != identity.bytes
            || source_digest(text.as_bytes()) != identity.sha256
        {
            return Err(LookupError::StaleArtifact);
        }
        check_cancelled(cancelled)
    }

    fn identity(&self, scope: Option<&str>, path: &str) -> Result<&Identity<'a>, LookupError> {
        let sources = match scope {
            None => &self.blizzard,
            Some("annotation_alias_catalog") => &self.aliases,
            _ => return Err(LookupError::UnsupportedProfile),
        };
        sources.get(path).ok_or(LookupError::InvalidMapping)
    }

    pub(super) fn revision(&self, link: &SourceLink) -> Result<&'a str, LookupError> {
        let identity = self.identity(link.scope, &link.path)?;
        if link.sha256 != identity.sha256
            || link.span.start >= link.span.end
            || link.span.end > identity.bytes
        {
            return Err(LookupError::InvalidMapping);
        }
        Ok(identity.revision)
    }
}
