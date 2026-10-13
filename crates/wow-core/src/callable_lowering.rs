//! One admitted callable display-lowering observation shared by concrete owners.
//! Admission checks data shape only. It supplies no producer, consumer or runtime proof.
use serde::Serialize;
use std::collections::BTreeSet;

use crate::error::validation_error;
use crate::{
    CanonicalResult, ContentDigest, CoreErrorCode, CoreResult, ProfileId, ReferenceGenerationId,
    SourceContent, SourceSpan,
};

const OPERATION: &str = "admit_callable_lowering";
const MAX_FIELDS: usize = 65_536;
const MAX_TYPE_BYTES: usize = 64 * 1024;
const MAX_TYPES_BYTES: usize = 16 * 1024 * 1024;
const MAX_ALIAS_DEFINITIONS: usize = 4096;
const MAX_ALIAS_ATOMS: usize = 16;
const MAX_ALIAS_DEPENDENCIES: usize = 16;
const MAX_ALIAS_DEFINITION_BYTES: usize = 256 * 1024;
const MAX_ALIAS_NAME_BYTES: usize = 1024;
const MAX_ALIAS_PATH_BYTES: usize = 4096;
pub const CALLABLE_ALIAS_DEFINITION_SCHEMA: &str = "wow-core/callable-alias-definition/1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CallableLoweringLane {
    Parameter,
    Return,
}

/// Immutable, validated ranges and an owner-supplied emitted type token.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CallableFieldLowering {
    lane: CallableLoweringLane,
    ordinal: usize,
    original_source_span: SourceSpan,
    generated_member_span: SourceSpan,
    type_token_span: SourceSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    optional_marker_span: Option<SourceSpan>,
    lowered_type: Box<str>,
    optional: bool,
    #[serde(skip_serializing_if = "slice_is_empty")]
    type_atoms: Box<[CallableTypeAtom]>,
}

impl CallableFieldLowering {
    #[allow(clippy::too_many_arguments)]
    pub fn admit(
        lane: CallableLoweringLane,
        ordinal: usize,
        original_source_span: SourceSpan,
        generated_member_span: SourceSpan,
        type_token_span: SourceSpan,
        optional_marker_span: Option<SourceSpan>,
        lowered_type: Box<str>,
        optional: bool,
    ) -> CoreResult<Self> {
        range(original_source_span)?;
        let (member_start, member_end) = range(generated_member_span)?;
        let (type_start, type_end) = range(type_token_span)?;
        let marker_is_valid = match optional_marker_span {
            Some(span) => {
                let (marker_start, marker_end) = range(span)?;
                marker_end - marker_start == 1
                    && marker_start >= member_start
                    && marker_end <= member_end
                    && (marker_end <= type_start || marker_start >= type_end)
            }
            None => true,
        };
        if ordinal >= MAX_FIELDS
            || lowered_type.is_empty()
            || lowered_type.len() > MAX_TYPE_BYTES
            || lowered_type.chars().any(char::is_control)
            || type_start < member_start
            || type_end > member_end
            || type_end - type_start != lowered_type.len() as u64
            || optional != optional_marker_span.is_some()
            || !marker_is_valid
        {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::ContractViolation,
                "field",
            ));
        }
        Ok(Self {
            lane,
            ordinal,
            original_source_span,
            generated_member_span,
            type_token_span,
            optional_marker_span,
            lowered_type,
            optional,
            type_atoms: Box::default(),
        })
    }

    pub const fn lane(&self) -> CallableLoweringLane {
        self.lane
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn original_source_span(&self) -> SourceSpan {
        self.original_source_span
    }
    pub const fn generated_member_span(&self) -> SourceSpan {
        self.generated_member_span
    }
    pub const fn type_token_span(&self) -> SourceSpan {
        self.type_token_span
    }
    pub const fn optional_marker_span(&self) -> Option<SourceSpan> {
        self.optional_marker_span
    }
    pub fn lowered_type(&self) -> &str {
        &self.lowered_type
    }
    pub const fn optional(&self) -> bool {
        self.optional
    }
    pub fn type_atoms(&self) -> &[CallableTypeAtom] {
        &self.type_atoms
    }

    /// Attach exact qualified-name associations within this already-emitted token.
    /// This validates data geometry only; it neither parses nor lowers type syntax.
    pub fn with_type_atoms(mut self, atoms: Box<[CallableTypeAtom]>) -> CoreResult<Self> {
        if atoms.len() > MAX_ALIAS_ATOMS {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::BudgetExceeded,
                "type_atoms",
            ));
        }
        let (token_start, token_end) = range(self.type_token_span)?;
        let mut previous_end = token_start;
        for atom in atoms.iter() {
            let (start, end) = range(atom.generated_span)?;
            if start < token_start || end > token_end || start < previous_end {
                return Err(validation_error(
                    OPERATION,
                    CoreErrorCode::ContractViolation,
                    "type_atom_range",
                ));
            }
            let expected_len = usize::try_from(end - start).map_err(|_| {
                validation_error(OPERATION, CoreErrorCode::UsageOverflow, "type_atom")
            })?;
            let relative_start = usize::try_from(start - token_start).map_err(|_| {
                validation_error(OPERATION, CoreErrorCode::UsageOverflow, "type_atom")
            })?;
            let relative_end = usize::try_from(end - token_start).map_err(|_| {
                validation_error(OPERATION, CoreErrorCode::UsageOverflow, "type_atom")
            })?;
            if expected_len != atom.rendered_name.len()
                || self.lowered_type.get(relative_start..relative_end) != Some(atom.rendered_name())
            {
                return Err(validation_error(
                    OPERATION,
                    CoreErrorCode::ContractViolation,
                    "type_atom_range",
                ));
            }
            previous_end = end;
        }
        self.type_atoms = atoms;
        Ok(self)
    }
}

/// One exact qualified input name and its already-emitted name range.
/// This is immutable data only; Core performs no source lookup or lowering.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CallableTypeAtom {
    source_name: Box<str>,
    rendered_name: Box<str>,
    generated_span: SourceSpan,
}

impl CallableTypeAtom {
    pub fn admit(
        source_name: impl Into<Box<str>>,
        rendered_name: impl Into<Box<str>>,
        generated_span: SourceSpan,
    ) -> CoreResult<Self> {
        let source_name = source_name.into();
        let rendered_name = rendered_name.into();
        validate_qualified_name(&source_name, "source_name")?;
        validate_qualified_name(&rendered_name, "rendered_name")?;
        let (start, end) = range(generated_span)?;
        if end - start != rendered_name.len() as u64 {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::ContractViolation,
                "type_atom_length",
            ));
        }
        Ok(Self {
            source_name,
            rendered_name,
            generated_span,
        })
    }
    pub fn source_name(&self) -> &str {
        &self.source_name
    }
    pub fn rendered_name(&self) -> &str {
        &self.rendered_name
    }
    pub const fn generated_span(&self) -> SourceSpan {
        self.generated_span
    }
}

/// Exact emitted alias declaration and its source/generated byte custody.
/// The fixed metadata labels state the proof ceiling; they are not caller claims.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CallableAliasDefinition {
    schema: &'static str,
    source_scope: &'static str,
    consumer_semantics: &'static str,
    custody: &'static str,
    name: Box<str>,
    source_revision: Box<str>,
    source_path: Box<str>,
    source_sha256: ContentDigest<SourceContent>,
    source_span: SourceSpan,
    generated_path: Box<str>,
    generated_sha256: ContentDigest<SourceContent>,
    declaration_span: SourceSpan,
    type_token_span: SourceSpan,
    lowered_definition: Box<str>,
    dependency_names: Box<[Box<str>]>,
}

impl CallableAliasDefinition {
    #[allow(clippy::too_many_arguments)]
    pub fn admit(
        name: impl Into<Box<str>>,
        source_revision: &str,
        source_path: &str,
        source_sha256: &str,
        source_span: SourceSpan,
        generated_path: &str,
        generated_sha256: &str,
        declaration_span: SourceSpan,
        type_token_span: SourceSpan,
        lowered_definition: impl Into<Box<str>>,
        dependency_names: Vec<String>,
    ) -> CoreResult<Self> {
        let name = name.into();
        let lowered_definition = lowered_definition.into();
        validate_qualified_name(&name, "name")?;
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
        if source_revision.len() > MAX_ALIAS_NAME_BYTES
            || source_path.len() > MAX_ALIAS_PATH_BYTES
            || generated_path.len() > MAX_ALIAS_PATH_BYTES
        {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::BudgetExceeded,
                "alias_identity",
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
                "alias_digest",
            ));
        }
        let (source_start, source_end) = range(source_span)?;
        let (declaration_start, declaration_end) = range(declaration_span)?;
        let (token_start, token_end) = range(type_token_span)?;
        let definition_len = usize::try_from(token_end - token_start).map_err(|_| {
            validation_error(
                OPERATION,
                CoreErrorCode::UsageOverflow,
                "lowered_definition",
            )
        })?;
        if source_end <= source_start
            || token_start < declaration_start
            || token_end > declaration_end
            || lowered_definition.is_empty()
            || lowered_definition.len() > MAX_ALIAS_DEFINITION_BYTES
            || lowered_definition.len() != definition_len
            || lowered_definition.chars().any(char::is_control)
        {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::ContractViolation,
                "alias_ranges",
            ));
        }
        if dependency_names.len() > MAX_ALIAS_DEPENDENCIES {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::BudgetExceeded,
                "dependency_names",
            ));
        }
        let mut dependencies = Vec::with_capacity(dependency_names.len());
        for dependency in dependency_names {
            validate_qualified_name(&dependency, "dependency_name")?;
            dependencies.push(dependency.into_boxed_str());
        }
        if dependencies.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::ContractViolation,
                "dependency_order",
            ));
        }
        Ok(Self {
            schema: CALLABLE_ALIAS_DEFINITION_SCHEMA,
            source_scope: "annotation_alias_catalog",
            consumer_semantics: "NotEvaluated",
            custody: "NotEvaluated",
            name,
            source_revision: source_revision.into(),
            source_path: source_path.into(),
            source_sha256: source_digest.into_value(),
            source_span,
            generated_path: generated_path.into(),
            generated_sha256: generated_digest.into_value(),
            declaration_span,
            type_token_span,
            lowered_definition,
            dependency_names: dependencies.into_boxed_slice(),
        })
    }
    pub const fn schema(&self) -> &'static str {
        self.schema
    }
    pub const fn source_scope(&self) -> &'static str {
        self.source_scope
    }
    pub const fn consumer_semantics(&self) -> &'static str {
        self.consumer_semantics
    }
    pub const fn custody(&self) -> &'static str {
        self.custody
    }
    pub fn name(&self) -> &str {
        &self.name
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
    pub const fn type_token_span(&self) -> SourceSpan {
        self.type_token_span
    }
    pub fn lowered_definition(&self) -> &str {
        &self.lowered_definition
    }
    pub fn dependency_names(&self) -> &[Box<str>] {
        &self.dependency_names
    }
}

/// Borrowed data only. Exact Library membership/bytes must be verified by its consumer.
/// No public fields or deserialization can bypass admission. Owner IDs are never reminted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CallableLoweringObservation<'a> {
    schema: &'static str,
    profile_id: &'a str,
    reference_generation: &'a str,
    reference_view_digest: &'a str,
    record_key: &'a str,
    record_digest: &'a str,
    source_id: &'a str,
    source_path: &'a str,
    source_sha256: &'a str,
    annotation_artifact_id: &'a str,
    source_map_id: &'a str,
    generated_path: &'a str,
    generated_sha256: &'a str,
    declaration_span: SourceSpan,
    fields: &'a [CallableFieldLowering],
    lowering_syntax: &'static str,
    consumer_semantics: &'static str,
    #[serde(skip_serializing_if = "slice_is_empty")]
    alias_definitions: &'a [CallableAliasDefinition],
}

impl<'a> CallableLoweringObservation<'a> {
    /// Validate bounded exact borrowed owner descriptors without normalization.
    /// Existing purpose/family parsers may use bounded identity scratch; nothing is retained.
    /// Existing native source/artifact/map descriptors stay in their owner families.
    #[allow(clippy::too_many_arguments)]
    pub fn admit(
        profile_id: &'a str,
        reference_generation: &'a str,
        reference_view_digest: &'a str,
        record_key: &'a str,
        record_digest: &'a str,
        source_id: &'a str,
        source_path: &'a str,
        source_sha256: &'a str,
        annotation_artifact_id: &'a str,
        source_map_id: &'a str,
        generated_path: &'a str,
        generated_sha256: &'a str,
        declaration_span: SourceSpan,
        fields: &'a [CallableFieldLowering],
    ) -> CoreResult<Self> {
        for value in [
            profile_id,
            reference_generation,
            record_key,
            source_id,
            annotation_artifact_id,
            source_map_id,
        ] {
            if value.is_empty()
                || value.len() > 1024
                || value.split([':', '/']).any(str::is_empty)
                || !value.bytes().any(|b| b.is_ascii_alphanumeric())
                || !value.bytes().all(|b| {
                    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b':' | b'/')
                })
            {
                return Err(validation_error(
                    OPERATION,
                    CoreErrorCode::InvalidIdentifier,
                    "identity",
                ));
            }
        }
        let _: ProfileId = profile_id.parse()?;
        let _: ReferenceGenerationId = reference_generation.parse()?;
        for value in [reference_view_digest, record_digest] {
            if !ContentDigest::<CanonicalResult>::parse(value)?.was_canonical() {
                return Err(validation_error(
                    OPERATION,
                    CoreErrorCode::InvalidDigest,
                    "digest",
                ));
            }
        }
        for value in [source_sha256, generated_sha256] {
            if !ContentDigest::<SourceContent>::parse(value)?.was_canonical() {
                return Err(validation_error(
                    OPERATION,
                    CoreErrorCode::InvalidDigest,
                    "digest",
                ));
            }
        }
        for value in [source_path, generated_path] {
            if value.is_empty()
                || value.len() > 4096
                || value.starts_with('/')
                || value.contains(['\\', ':'])
                || value.chars().any(char::is_control)
                || value
                    .split('/')
                    .any(|part| part.is_empty() || part == "." || part == "..")
            {
                return Err(validation_error(
                    OPERATION,
                    CoreErrorCode::InvalidSourcePath,
                    "path",
                ));
            }
        }
        let (start, end) = range(declaration_span)?;
        if fields.len() > MAX_FIELDS {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::BudgetExceeded,
                "fields",
            ));
        }
        let mut parameters = 0;
        let mut returns = 0;
        let mut previous_end = start;
        let mut bytes = 0usize;
        for field in fields {
            let next = match field.lane {
                CallableLoweringLane::Parameter if returns == 0 => &mut parameters,
                CallableLoweringLane::Return => &mut returns,
                _ => {
                    return Err(validation_error(
                        OPERATION,
                        CoreErrorCode::ContractViolation,
                        "lane_order",
                    ));
                }
            };
            let (member_start, member_end) = range(field.generated_member_span)?;
            bytes = bytes.checked_add(field.lowered_type.len()).ok_or_else(|| {
                validation_error(OPERATION, CoreErrorCode::UsageOverflow, "types")
            })?;
            for atom in field.type_atoms.iter() {
                bytes = bytes
                    .checked_add(atom.source_name.len())
                    .and_then(|total| total.checked_add(atom.rendered_name.len()))
                    .ok_or_else(|| {
                        validation_error(OPERATION, CoreErrorCode::UsageOverflow, "type_atoms")
                    })?;
            }
            if field.ordinal != *next
                || member_start < previous_end
                || member_end > end
                || bytes > MAX_TYPES_BYTES
            {
                return Err(validation_error(
                    OPERATION,
                    CoreErrorCode::ContractViolation,
                    "field_order",
                ));
            }
            *next += 1;
            previous_end = member_end;
        }
        Ok(Self {
            schema: "wow-core/callable-lowering-observation/1",
            profile_id,
            reference_generation,
            reference_view_digest,
            record_key,
            record_digest,
            source_id,
            source_path,
            source_sha256,
            annotation_artifact_id,
            source_map_id,
            generated_path,
            generated_sha256,
            declaration_span,
            fields,
            lowering_syntax: "emitted_analysis_only",
            consumer_semantics: "NotEvaluated",
            alias_definitions: &[],
        })
    }

    /// Attach a canonically name-ordered selected subgraph. Core validates membership
    /// and byte bounds only; dependency meaning, cycles and type syntax stay owner-owned.
    pub fn with_alias_definitions(
        mut self,
        definitions: &'a [CallableAliasDefinition],
    ) -> CoreResult<Self> {
        if definitions.len() > MAX_ALIAS_DEFINITIONS
            || definitions
                .windows(2)
                .any(|pair| pair[0].name >= pair[1].name)
        {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::ContractViolation,
                "alias_definition_order",
            ));
        }
        let names = definitions
            .iter()
            .map(|definition| definition.name.as_ref())
            .collect::<BTreeSet<_>>();
        if names.len() != definitions.len() {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::ContractViolation,
                "alias_definition_duplicate",
            ));
        }
        if let Some(first) = definitions.first()
            && definitions.iter().any(|definition| {
                definition.generated_path != first.generated_path
                    || definition.generated_sha256 != first.generated_sha256
            })
        {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::ContractViolation,
                "alias_generated_file",
            ));
        }
        let mut bytes = 0usize;
        for field in self.fields {
            add_alias_bytes(&mut bytes, field.lowered_type.len())?;
            for atom in field.type_atoms.iter() {
                add_alias_bytes(&mut bytes, atom.source_name.len())?;
                add_alias_bytes(&mut bytes, atom.rendered_name.len())?;
            }
        }
        for definition in definitions {
            for dependency in definition.dependency_names.iter() {
                if !names.contains(dependency.as_ref()) {
                    return Err(validation_error(
                        OPERATION,
                        CoreErrorCode::ContractViolation,
                        "dependency_missing",
                    ));
                }
                add_alias_bytes(&mut bytes, dependency.len())?;
            }
            for value in [
                definition.schema,
                definition.source_scope,
                definition.consumer_semantics,
                definition.custody,
                definition.name.as_ref(),
                definition.source_revision.as_ref(),
                definition.source_path.as_ref(),
                definition.generated_path.as_ref(),
                definition.lowered_definition.as_ref(),
            ] {
                add_alias_bytes(&mut bytes, value.len())?;
            }
            add_alias_bytes(&mut bytes, 71)?;
            add_alias_bytes(&mut bytes, 71)?;
        }
        self.alias_definitions = definitions;
        Ok(self)
    }

    pub const fn alias_definitions(&self) -> &'a [CallableAliasDefinition] {
        self.alias_definitions
    }

    pub const fn schema(&self) -> &'static str {
        self.schema
    }
    pub const fn profile_id(&self) -> &'a str {
        self.profile_id
    }
    pub const fn reference_generation(&self) -> &'a str {
        self.reference_generation
    }
    pub const fn reference_view_digest(&self) -> &'a str {
        self.reference_view_digest
    }
    pub const fn record_key(&self) -> &'a str {
        self.record_key
    }
    pub const fn record_digest(&self) -> &'a str {
        self.record_digest
    }
    pub const fn source_id(&self) -> &'a str {
        self.source_id
    }
    pub const fn source_path(&self) -> &'a str {
        self.source_path
    }
    pub const fn source_sha256(&self) -> &'a str {
        self.source_sha256
    }
    pub const fn annotation_artifact_id(&self) -> &'a str {
        self.annotation_artifact_id
    }
    pub const fn source_map_id(&self) -> &'a str {
        self.source_map_id
    }
    pub const fn generated_path(&self) -> &'a str {
        self.generated_path
    }
    pub const fn generated_sha256(&self) -> &'a str {
        self.generated_sha256
    }
    pub const fn declaration_span(&self) -> SourceSpan {
        self.declaration_span
    }
    pub const fn fields(&self) -> &'a [CallableFieldLowering] {
        self.fields
    }
    pub const fn lowering_syntax(&self) -> &'static str {
        self.lowering_syntax
    }
    pub const fn consumer_semantics(&self) -> &'static str {
        self.consumer_semantics
    }
}

fn slice_is_empty<T>(value: &[T]) -> bool {
    value.is_empty()
}

fn validate_qualified_name(value: &str, field: &'static str) -> CoreResult<()> {
    let valid_segment = |segment: &str| {
        let mut bytes = segment.bytes();
        bytes
            .next()
            .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphabetic())
            && bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
    };
    if value.is_empty()
        || value.len() > MAX_ALIAS_NAME_BYTES
        || !value.split('.').all(valid_segment)
    {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::InvalidIdentifier,
            field,
        ));
    }
    Ok(())
}

fn canonical_relative_path(value: &str) -> CoreResult<String> {
    if value.len() > MAX_ALIAS_PATH_BYTES {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::BudgetExceeded,
            "path",
        ));
    }
    let parsed = crate::NormalizedSourcePath::parse(value)?;
    if !parsed.was_canonical() {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::InvalidSourcePath,
            "noncanonical_path",
        ));
    }
    Ok(parsed.into_value().as_str().to_owned())
}

fn add_alias_bytes(total: &mut usize, amount: usize) -> CoreResult<()> {
    *total = total
        .checked_add(amount)
        .ok_or_else(|| validation_error(OPERATION, CoreErrorCode::UsageOverflow, "alias_bytes"))?;
    if *total > MAX_TYPES_BYTES {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::BudgetExceeded,
            "alias_bytes",
        ));
    }
    Ok(())
}

fn range(span: SourceSpan) -> CoreResult<(u64, u64)> {
    span.validate()?;
    match (span.byte_start(), span.byte_end()) {
        (Some(start), Some(end)) if start < end => Ok((start, end)),
        _ => Err(validation_error(
            OPERATION,
            CoreErrorCode::InvalidSourceSpan,
            "span",
        )),
    }
}
