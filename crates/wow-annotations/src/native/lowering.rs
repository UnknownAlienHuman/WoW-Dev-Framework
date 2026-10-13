//! Private final-emission custody. Neither this catalog nor its fields deserialize.
use super::RenderError;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use wow_core::{CallableAliasDefinition, CallableFieldLowering, SourceSpan};

pub(crate) const MAX_CAPTURE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CapturedCallable {
    pub(crate) file_index: usize,
    pub(crate) declaration_span: SourceSpan,
    pub(crate) source_path: Box<str>,
    pub(crate) source_sha256: Box<str>,
    pub(crate) source_span: SourceSpan,
    pub(crate) fields: Arc<[CallableFieldLowering]>,
    /// One name-ordered, shared used-alias closure for this callable.
    pub(crate) alias_definitions: Arc<[CallableAliasDefinition]>,
}

pub(crate) struct CaptureBudget {
    callables: usize,
    fields: usize,
    bytes: usize,
}

impl CaptureBudget {
    pub(crate) const fn new() -> Self {
        Self {
            callables: 0,
            fields: 0,
            bytes: 0,
        }
    }
    pub(crate) const fn metadata_bytes(&self) -> usize {
        self.bytes
    }

    pub(crate) fn reserve(
        &mut self,
        callables: usize,
        fields: usize,
        bytes: usize,
        stop: &AtomicBool,
    ) -> Result<(), RenderError> {
        if stop.load(Ordering::Acquire) {
            return Err(RenderError::Cancelled);
        }
        self.callables = self
            .callables
            .checked_add(callables)
            .ok_or(RenderError::OutputLimit)?;
        self.fields = self
            .fields
            .checked_add(fields)
            .ok_or(RenderError::OutputLimit)?;
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or(RenderError::OutputLimit)?;
        if self.callables > super::MAX_UNITS
            || self.fields > super::maps::MAX_MEMBERS
            || self.bytes > MAX_CAPTURE_BYTES
        {
            return Err(RenderError::OutputLimit);
        }
        Ok(())
    }
}

pub(super) fn span(start: usize, end: usize) -> Result<SourceSpan, RenderError> {
    SourceSpan::byte_range(
        u64::try_from(start).map_err(|_| RenderError::OutputLimit)?,
        u64::try_from(end).map_err(|_| RenderError::OutputLimit)?,
    )
    .map_err(|_| RenderError::InvalidSource)
}
