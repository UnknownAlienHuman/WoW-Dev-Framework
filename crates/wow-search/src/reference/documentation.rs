//! Data-only documentation copies and the frozen native FTS row encoding.
use super::*;
use wow_reference::native_view::NativeDocumentation;

pub(super) const INDEX_PROFILE: &str = "original-documentation-whitespace-to-space/1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Documentation {
    NotStated,
    Recorded { lines: Vec<String> },
    Unsupported,
}

impl Documentation {
    pub(super) fn capture(
        input: NativeDocumentation<'_>,
        bounds: SearchBounds,
        stop: &AtomicBool,
    ) -> SearchResult<Self> {
        check(stop)?;
        let data = match input {
            NativeDocumentation::NotStated => Self::NotStated,
            NativeDocumentation::Unsupported => Self::Unsupported,
            NativeDocumentation::Recorded(lines) => {
                let bytes = lines.iter().try_fold(0u64, |total, line| {
                    check(stop)?;
                    total
                        .checked_add(line.len() as u64 + 1)
                        .ok_or(SearchError::FileSetBudgetExceeded)
                })?;
                if bytes > bounds.max_owner_manifest_bytes.get() {
                    return Err(SearchError::FileSetBudgetExceeded);
                }
                Self::Recorded {
                    lines: lines.to_vec(),
                }
            }
        };
        data.index_text(bounds)?;
        Ok(data)
    }

    pub(super) fn index_text(&self, bounds: SearchBounds) -> SearchResult<String> {
        let mut text = String::new();
        if let Self::Recorded { lines } = self {
            for (index, line) in lines.iter().enumerate() {
                if index != 0 {
                    text.push(' ');
                }
                for character in line.chars() {
                    text.push(if character.is_control() || character.is_whitespace() {
                        ' '
                    } else {
                        character
                    });
                    if text.len() > bounds.max_owner_metadata_label_bytes.get() as usize {
                        return Err(SearchError::FileSetBudgetExceeded);
                    }
                }
                if text.len() > bounds.max_owner_metadata_label_bytes.get() as usize {
                    return Err(SearchError::FileSetBudgetExceeded);
                }
            }
        }
        // The existing physical row contract requires nonempty text. A blank
        // encodes an empty token stream; it is never exposed as source metadata.
        if text.is_empty() {
            text.push(' ');
        }
        Ok(text)
    }
}

pub(super) fn row_text(document: &Document, bounds: SearchBounds) -> SearchResult<String> {
    document.documentation.as_ref().map_or_else(
        || Ok(document.canonical_qualified_name.clone()),
        |data| data.index_text(bounds),
    )
}

pub(super) fn origin(document: &Document) -> &'static str {
    if document.declaration.is_some() {
        "NativeDeclarationSignature.documentation()"
    } else if document.event.is_some() {
        "NativeEventSignature.documentation()"
    } else {
        "NativeCallableSignature.documentation()"
    }
}

#[derive(Debug, Serialize)]
pub(super) struct Coverage {
    recorded: usize,
    not_stated: usize,
    unsupported: usize,
    authority_class: &'static str,
}
pub(super) fn coverage(manifest: &Manifest) -> Option<Coverage> {
    manifest.include_documentation.map(|_| {
        let mut coverage = Coverage {
            recorded: 0,
            not_stated: 0,
            unsupported: 0,
            authority_class: "ReferenceContractDocumentation",
        };
        for document in &manifest.documents {
            match &document.documentation {
                Some(Documentation::Recorded { .. }) => coverage.recorded += 1,
                Some(Documentation::NotStated) => coverage.not_stated += 1,
                Some(Documentation::Unsupported) => coverage.unsupported += 1,
                None => {}
            }
        }
        coverage
    })
}
