//! Exact inert declarations captured by the native annotation emitter.
use super::{
    MAX_ALIAS_NAME_BYTES, MAX_ALIAS_PATH_BYTES, OPERATION, canonical_relative_path, range,
    validate_qualified_name,
};
use crate::error::validation_error;
use crate::{ContentDigest, CoreErrorCode, CoreResult, SourceContent, SourceSpan};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CallableTypeDeclarationKind {
    Structure,
    Callback,
    ScriptObject,
    Enum,
}

/// An emitted name link, not complete type closure or semantic consumer proof.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CallableTypeDeclaration {
    schema: &'static str,
    source_scope: &'static str,
    consumer_semantics: &'static str,
    custody: &'static str,
    kind: CallableTypeDeclarationKind,
    source_name: Box<str>,
    rendered_name: Box<str>,
    source_revision: Box<str>,
    source_path: Box<str>,
    source_sha256: ContentDigest<SourceContent>,
    source_span: SourceSpan,
    generated_path: Box<str>,
    generated_sha256: ContentDigest<SourceContent>,
    declaration_span: SourceSpan,
    name_span: SourceSpan,
}

impl CallableTypeDeclaration {
    #[allow(clippy::too_many_arguments)]
    pub fn admit(
        kind: CallableTypeDeclarationKind,
        source_name: &str,
        rendered_name: &str,
        source_revision: &str,
        source_path: &str,
        source_sha256: &str,
        source_span: SourceSpan,
        generated_path: &str,
        generated_sha256: &str,
        declaration_span: SourceSpan,
        name_span: SourceSpan,
    ) -> CoreResult<Self> {
        if source_name.len() > MAX_ALIAS_NAME_BYTES
            || rendered_name.len() > MAX_ALIAS_NAME_BYTES
            || source_path.len() > MAX_ALIAS_PATH_BYTES
            || generated_path.len() > MAX_ALIAS_PATH_BYTES
        {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::BudgetExceeded,
                "type_declaration_identity",
            ));
        }
        validate_qualified_name(source_name, "source_name")?;
        validate_qualified_name(rendered_name, "rendered_name")?;
        if !matches!(source_revision.len(), 40 | 64)
            || !source_revision
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::InvalidIdentifier,
                "source_revision",
            ));
        }
        let source_path = canonical_relative_path(source_path)?;
        let generated_path = canonical_relative_path(generated_path)?;
        let source_digest = ContentDigest::<SourceContent>::parse(source_sha256)?;
        let generated_digest = ContentDigest::<SourceContent>::parse(generated_sha256)?;
        if !source_digest.was_canonical() || !generated_digest.was_canonical() {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::InvalidDigest,
                "type_declaration_digest",
            ));
        }
        range(source_span)?;
        let (declaration_start, declaration_end) = range(declaration_span)?;
        let (name_start, name_end) = range(name_span)?;
        if name_start < declaration_start
            || name_end > declaration_end
            || name_end - name_start != rendered_name.len() as u64
        {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::ContractViolation,
                "type_declaration_ranges",
            ));
        }
        Ok(Self {
            schema: "wow-core/callable-type-declaration/1",
            source_scope: "blizzard_generated_documentation",
            consumer_semantics: "NotEvaluated",
            custody: "NotEvaluated",
            kind,
            source_name: source_name.into(),
            rendered_name: rendered_name.into(),
            source_revision: source_revision.into(),
            source_path: source_path.into_boxed_str(),
            source_sha256: source_digest.into_value(),
            source_span,
            generated_path: generated_path.into_boxed_str(),
            generated_sha256: generated_digest.into_value(),
            declaration_span,
            name_span,
        })
    }

    pub const fn kind(&self) -> CallableTypeDeclarationKind {
        self.kind
    }
    pub fn source_name(&self) -> &str {
        &self.source_name
    }
    pub fn rendered_name(&self) -> &str {
        &self.rendered_name
    }
    pub fn source_revision(&self) -> &str {
        &self.source_revision
    }
    pub fn source_path(&self) -> &str {
        &self.source_path
    }
    pub const fn source_sha256(&self) -> &ContentDigest<SourceContent> {
        &self.source_sha256
    }
    pub const fn source_span(&self) -> SourceSpan {
        self.source_span
    }
    pub fn generated_path(&self) -> &str {
        &self.generated_path
    }
    pub const fn generated_sha256(&self) -> &ContentDigest<SourceContent> {
        &self.generated_sha256
    }
    pub const fn declaration_span(&self) -> SourceSpan {
        self.declaration_span
    }
    pub const fn name_span(&self) -> SourceSpan {
        self.name_span
    }

    pub fn metadata_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.source_name.len()
            + self.rendered_name.len()
            + self.source_revision.len()
            + self.source_path.len()
            + self.generated_path.len()
    }
}
