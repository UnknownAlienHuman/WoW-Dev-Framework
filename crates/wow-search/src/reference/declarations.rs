//! Data-only Search copies from the sealed native declaration owner port.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DeclarationDocument {
    pub kind: NativeDeclarationKind,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition_namespace: Option<String>,
    pub fields: Vec<Field>,
    pub arguments: Vec<Field>,
    pub returns: Vec<Field>,
    pub values: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_base: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Value {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_type: Option<String>,
    pub value: NativeDeclarationScalar,
    pub byte_range: [usize; 2],
}

fn copy_fields(fields: &[NativeCallableField], stop: &AtomicBool) -> SearchResult<Vec<Field>> {
    fields
        .iter()
        .map(|field| {
            check(stop)?;
            let span = field.source_span();
            Ok(Field {
                name: field.name().into(),
                original_type: field.type_name().into(),
                original_inner_type: field.inner_type().map(str::to_owned),
                nilable: field.nilable(),
                byte_range: [span.start, span.end],
            })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn capture(
    native: &NativeViewProjection,
    partition: &wow_reference::ReferencePartition,
    origins: &BTreeMap<&str, SourceOrigin>,
    manifest: &mut Manifest,
    bounds: SearchBounds,
    charged: &mut u64,
    stop: &AtomicBool,
) -> SearchResult<()> {
    for record in partition.records() {
        check(stop)?;
        let declaration = native
            .declaration_signature(record, stop)
            .map_err(native_error)?;
        if declaration.reference_generation() != manifest.reference_generation
            || declaration.reference_view_digest() != manifest.reference_view_digest
            || declaration.record_key() != record.key()
            || declaration.kind().record_kind() != record.kind()
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let source = origins
            .get(declaration.source_id())
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        if wow_reference::native_view::declaration_origin_id(
            native.schema,
            &manifest.binding.source_revision,
            &source.path,
            &source.sha256,
            wow_reference::native::Span {
                start: source.byte_range[0],
                end: source.byte_range[1],
            },
        )
        .map_err(native_error)?
            != source.source_id
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        if record.key().len() > bounds.max_document_id_bytes.get() as usize
            || declaration.name().len() > bounds.max_owner_metadata_label_bytes.get() as usize
        {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        let mut values = Vec::new();
        for value in declaration.values() {
            check(stop)?;
            values.push(Value {
                name: value.name.clone(),
                original_type: value.original_type.clone(),
                value: value.value.clone(),
                byte_range: [value.source_span.start, value.source_span.end],
            });
        }
        let document = Document {
            restriction: None,
            documentation: manifest
                .include_documentation
                .map(|_| {
                    documentation::Documentation::capture(declaration.documentation(), bounds, stop)
                })
                .transpose()?,
            entity_key: record.key().into(),
            owner_record_digest: declaration.record_digest().into(),
            source: source.clone(),
            canonical_short_name: declaration.name().into(),
            namespace: None,
            canonical_qualified_name: declaration.name().into(),
            arguments: Vec::new(),
            returns: Vec::new(),
            type_authority: "original_source_type_identities_unresolved".into(),
            event: None,
            declaration: Some(DeclarationDocument {
                kind: declaration.kind(),
                name: declaration.name().into(),
                definition_namespace: declaration.definition_namespace().map(str::to_owned),
                fields: copy_fields(declaration.fields(), stop)?,
                arguments: copy_fields(declaration.arguments(), stop)?,
                returns: copy_fields(declaration.returns(), stop)?,
                values,
                original_base: declaration.original_base().map(str::to_owned),
            }),
        };
        *charged = charged
            .checked_add(canonical_size(&document)? + u64::from(!manifest.documents.is_empty()))
            .ok_or(SearchError::FileSetBudgetExceeded)?;
        if *charged > bounds.max_owner_manifest_bytes.get() {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        manifest.documents.push(document);
    }
    Ok(())
}

pub(super) fn valid_document(
    document: &Document,
    selection: Option<NativeReferenceDocuments>,
) -> bool {
    let Some(declaration) = &document.declaration else {
        return true;
    };
    if selection != Some(NativeReferenceDocuments::SourceDeclarationsV1)
        || document.event.is_some()
        || document.namespace.is_some()
        || !document.arguments.is_empty()
        || !document.returns.is_empty()
        || declaration.name != document.canonical_short_name
        || document.entity_key
            != format!(
                "declaration:{}:{}",
                declaration.kind.key_prefix(),
                declaration.name
            )
    {
        return false;
    }
    let fields = !declaration.fields.is_empty();
    let signature = !declaration.arguments.is_empty() || !declaration.returns.is_empty();
    let values = !declaration.values.is_empty();
    match declaration.kind {
        NativeDeclarationKind::Structure => {
            !signature && !values && declaration.original_base.is_none()
        }
        NativeDeclarationKind::Callback => {
            !fields && !values && declaration.original_base.is_none()
        }
        NativeDeclarationKind::Enumeration | NativeDeclarationKind::Constants => {
            !fields && !signature && declaration.original_base.is_none()
        }
        NativeDeclarationKind::ScriptObject => !fields && !signature && !values,
    }
}
