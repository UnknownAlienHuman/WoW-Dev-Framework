//! Direct source-selection queries reuse the prepared source-buffer implementation.
use super::{
    GeneratedLookup, LookupError, NavigationIndex, PositionEncoding, SourceFile, TextRange,
};
use std::sync::atomic::AtomicBool;
use wow_reference::native::Span;

/// Complete caller-supplied source text and its independently selected identity.
///
/// The view is borrowed and is never copied into serialized navigation results.
/// Its identity and bytes are validated before source coordinates are interpreted.
/// For repeated queries, prefer `NavigationIndex::bind_source` to binding again.
#[derive(Clone, Copy)]
pub struct SourceText<'a> {
    pub scope: Option<&'a str>,
    pub revision: &'a str,
    pub path: &'a str,
    pub sha256: &'a str,
    pub text: &'a str,
}

impl SourceText<'_> {
    fn identity(&self) -> SourceFile<'_> {
        SourceFile {
            scope: self.scope,
            revision: self.revision,
            path: self.path,
            sha256: self.sha256,
        }
    }
}

impl<'generation> NavigationIndex<'generation> {
    /// Resolve a whole UTF-8 source selection without retaining a source view.
    ///
    /// This uses the same binding, boundaries, ranking, ties and budgets as
    /// `bind_source(...).generated_for_range(...)`. Each call validates the
    /// supplied text and prepares only that source file's interval/line indexes.
    /// The result borrows the generation, not the temporary source view or text.
    /// Empty ranges use cursor semantics; Unmapped never proves API absence.
    pub fn generated_for_range(
        &self,
        source: SourceText<'_>,
        range: Span,
        cancelled: &AtomicBool,
    ) -> Result<GeneratedLookup<'generation>, LookupError> {
        self.bind_source(source.identity(), source.text, cancelled)?
            .generated_for_range(range, cancelled)
    }

    /// Resolve explicitly encoded source endpoints against one validated buffer.
    ///
    /// Both endpoints share source scope, revision, path, digest and exact text.
    /// Reversed or mid-codepoint endpoints reject without clamping. Generated
    /// ranges remain stored UTF-8 bytes; no source-to-output interpolation occurs.
    /// Retain a bound source view instead when querying the same buffer repeatedly.
    pub fn generated_for_text_range(
        &self,
        source: SourceText<'_>,
        range: TextRange,
        encoding: PositionEncoding,
        cancelled: &AtomicBool,
    ) -> Result<GeneratedLookup<'generation>, LookupError> {
        self.bind_source(source.identity(), source.text, cancelled)?
            .generated_for_text_range(range, encoding, cancelled)
    }
}
