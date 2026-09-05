//! Source-bound connection of the native Ketho loader model to the Rust emitters.
//!
//! This in-memory projection keeps raw metadata as a sidecar. It is not a
//! certified ReferenceView, exhaustive API inventory or language-server probe.
//! No IO, source execution, automatic correction, or provider lookup occurs here.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use wow_reference::native::{DocumentationDocument, RawKind, RawValue, Span, source_digest};
use wow_reference::native_constants::{ResolvedScalar, ScalarCatalog, ScalarError, ScalarValue};
use wow_reference::native_corrections::{
    self, CorrectionError, CorrectionReport, ValidatedCorrections,
};
use wow_reference::native_model::{
    CallableFact, FieldFact, SystemFacts, SystemOwner, TableFact, ValueFact, normalize_document,
};

use crate::ketho::{Field, Function, Owner, RenderError, Renderer, System, Table};
use crate::literals::{
    ConstantGroup, EnumDeclaration, EventLiteral, IntegerFormat, LiteralMember, LiteralRenderer,
    LiteralValue, MemberOrder,
};

const MAX_FILES: usize = 4096;
const MAX_UNITS: usize = 65_536;
const MAX_LIBRARY_BYTES: usize = 64 * 1024 * 1024;
const MAX_FILE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, Serialize)]
pub struct SourceLink {
    pub path: String,
    pub sha256: String,
    pub span: Span,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProjectionIssue {
    pub code: String,
    pub source: SourceLink,
}

#[derive(Clone, Debug, Serialize)]
pub struct SourceMapping {
    /// Declaration or whole literal-file range, never a fabricated member range.
    pub granularity: &'static str,
    pub generated: Span,
    pub source: SourceLink,
}

#[derive(Clone, Debug, Serialize)]
pub struct AnnotationFile {
    /// Renderer-owned name, independent of source-controlled paths/identifiers.
    pub path: String,
    pub sha256: String,
    pub text: String,
    pub mappings: Vec<SourceMapping>,
}

/// A source field not encoded in LuaCATS. The field value stays in the exact raw
/// document; this record is an explicit loss/sidecar link, not an inferred rule.
#[derive(Clone, Debug, Serialize)]
pub struct MetadataSidecar {
    pub field: String,
    pub source: SourceLink,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScalarResolutionRecord {
    pub source: SourceLink,
    pub result: Result<ResolvedScalar, ScalarError>,
}

/// An explicit consumer-specific label change, not a change to the source API.
#[derive(Clone, Debug, Serialize)]
pub struct NameProjection {
    pub source: SourceLink,
    pub original: String,
    pub rendered: String,
    pub rule: &'static str,
}

#[derive(Debug, Serialize)]
pub struct NativeLibrary<'a> {
    pub schema: &'static str,
    pub revision: &'a str,
    pub projection: &'static str,
    pub negative_authority: bool,
    /// All raw registration fields, including unknown/restriction metadata.
    pub sources: Vec<&'a DocumentationDocument>,
    pub files: Vec<AnnotationFile>,
    pub issues: Vec<ProjectionIssue>,
    pub metadata_sidecars: Vec<MetadataSidecar>,
    pub scalar_resolutions: Vec<ScalarResolutionRecord>,
    pub name_projections: Vec<NameProjection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corrections: Option<CorrectionReport>,
    pub limitations: Vec<&'static str>,
}

fn link(document: &DocumentationDocument, raw: &RawValue) -> SourceLink {
    SourceLink {
        path: document.path().to_owned(),
        sha256: document.sha256().to_owned(),
        span: raw.span,
    }
}
fn issue(
    document: &DocumentationDocument,
    raw: &RawValue,
    code: impl Into<String>,
) -> ProjectionIssue {
    ProjectionIssue {
        code: code.into(),
        source: link(document, raw),
    }
}

/// Project an explicitly selected source corpus. All files must belong to one
/// revision. Duplicate source paths and cross-generation inputs reject globally.
/// Unsupported declarations remain in the raw sidecar and the issue inventory.
/// `environment` is an exact source environment (e.g. Mainline); All declarations
/// and unnamed table groups remain eligible. No flavor is guessed from a build.
pub fn project<'a>(
    documents: &'a [DocumentationDocument],
    environment: &str,
    cancelled: &AtomicBool,
) -> Result<NativeLibrary<'a>, RenderError> {
    project_with_corrections(documents, environment, None, cancelled)
}

/// Apply an explicitly reviewed, source-guarded Ketho correction set in the
/// reference owner before rendering. No pack is discovered or enabled by default.
pub fn project_with_corrections<'a>(
    documents: &'a [DocumentationDocument],
    environment: &str,
    corrections: Option<&'a ValidatedCorrections>,
    cancelled: &AtomicBool,
) -> Result<NativeLibrary<'a>, RenderError> {
    if documents.is_empty() || documents.len() > MAX_FILES || environment.is_empty() {
        return Err(RenderError::InputLimit);
    }
    if cancelled.load(Ordering::Relaxed) {
        return Err(RenderError::Cancelled);
    }
    if documents
        .iter()
        .map(DocumentationDocument::source_bytes)
        .fold(0usize, usize::saturating_add)
        > MAX_LIBRARY_BYTES
    {
        return Err(RenderError::InputLimit);
    }
    let revision = documents[0].revision();
    let mut sorted = documents.iter().collect::<Vec<_>>();
    sorted.sort_by_key(|doc| doc.path());
    if sorted.windows(2).any(|p| p[0].path() == p[1].path())
        || sorted.iter().any(|doc| doc.revision() != revision)
    {
        return Err(RenderError::InvalidSource);
    }
    let corrected = corrections
        .map(|set| native_corrections::apply_to_documents(documents, environment, set, cancelled))
        .transpose()
        .map_err(|error| match error {
            CorrectionError::Cancelled => RenderError::Cancelled,
            CorrectionError::Limit => RenderError::InputLimit,
            _ => RenderError::InvalidSource,
        })?;
    let normalized = sorted
        .iter()
        .map(|doc| normalize_document(doc))
        .collect::<Vec<_>>();
    let mut issues = Vec::new();
    let mut metadata_sidecars = Vec::new();
    let mut scalar_resolutions = Vec::new();
    let mut name_projections = Vec::new();
    let mut resolution_budget = MAX_LIBRARY_BYTES;
    let mut systems = Vec::new();
    for document in &normalized {
        for system in &document.systems {
            match system {
                Ok(system)
                    if system
                        .environment
                        .is_none_or(|e| e == "All" || e == environment) =>
                {
                    systems.push((document.source, system))
                }
                Ok(system) => issues.push(issue(
                    document.source,
                    system.raw,
                    "environment_not_selected",
                )),
                Err(error) => issues.push(ProjectionIssue {
                    code: format!("normalization_{:?}", error.code),
                    source: SourceLink {
                        path: document.source.path().into(),
                        sha256: document.source.sha256().into(),
                        span: error.span,
                    },
                }),
            }
        }
    }
    if let Some(corpus) = &corrected {
        systems = corpus
            .systems()
            .iter()
            .filter(|(_, s)| s.environment.is_none_or(|e| e == "All" || e == environment))
            .map(|(d, s)| (*d, s))
            .collect();
    }
    let mut identities = BTreeMap::<String, usize>::new();
    let mut enum_names = BTreeSet::new();
    let mut units = 0usize;
    for (_, system) in &systems {
        for function in &system.functions {
            *identities
                .entry(function_key(system, function.name))
                .or_default() += 1;
        }
        for table in &system.tables {
            *identities.entry(table_key(table)).or_default() += 1;
            if let TableFact::Enumeration { name, .. } = table {
                enum_names.insert((*name).to_owned());
            }
        }
        for event in &system.events {
            *identities
                .entry(format!("event:{}", event.literal_name))
                .or_default() += 1;
        }
        units = units
            .saturating_add(system.functions.len() + system.tables.len() + system.events.len());
        if units > MAX_UNITS {
            return Err(RenderError::InputLimit);
        }
    }
    let catalog = ScalarCatalog::new(revision, &systems).map_err(|error| match error {
        ScalarError::InvalidSource => RenderError::InvalidSource,
        _ => RenderError::InputLimit,
    })?;
    let renderer = Renderer::new(enum_names, MAX_FILE_BYTES)?;
    let literals = LiteralRenderer::new(MAX_FILE_BYTES)?;
    let mut files = Vec::new();
    let mut total_bytes = 0usize;
    let mut all_enums = Vec::new();
    let mut all_constants = Vec::new();
    let mut all_events = Vec::new();
    let mut all_literal_sources = Vec::new();
    let mut all_event_sources = Vec::new();
    for (document, system) in systems {
        if cancelled.load(Ordering::Relaxed) {
            return Err(RenderError::Cancelled);
        }
        let mut scalars = ScalarProjection {
            catalog: &catalog,
            document,
            cancelled,
            records: &mut scalar_resolutions,
            budget: &mut resolution_budget,
        };
        metadata(
            document,
            system.raw,
            &[
                "Name",
                "Type",
                "Namespace",
                "Environment",
                "Functions",
                "Events",
                "Tables",
            ],
            &mut metadata_sidecars,
        );
        for function in &system.functions {
            metadata(
                document,
                function.raw,
                &["Name", "Type", "Documentation", "Arguments", "Returns"],
                &mut metadata_sidecars,
            );
            for field in function.arguments.iter().chain(&function.returns) {
                field_metadata(document, field, &mut metadata_sidecars);
            }
        }
        for table in &system.tables {
            metadata(
                document,
                table_raw(table),
                &["Name", "Type", "Fields", "Values", "Arguments", "Returns"],
                &mut metadata_sidecars,
            );
            match table {
                TableFact::Structure { fields, .. } => {
                    for field in fields {
                        field_metadata(document, field, &mut metadata_sidecars);
                    }
                }
                TableFact::Callback {
                    arguments, returns, ..
                } => {
                    for field in arguments.iter().chain(returns) {
                        field_metadata(document, field, &mut metadata_sidecars);
                    }
                }
                TableFact::Enumeration { values, .. } | TableFact::Constants { values, .. } => {
                    for value in values {
                        metadata(
                            document,
                            value.raw,
                            &["Name", "Type", "EnumValue", "Value"],
                            &mut metadata_sidecars,
                        );
                    }
                }
                _ => {}
            }
        }
        for event in &system.events {
            metadata(
                document,
                event.raw,
                &["Name", "Type", "LiteralName", "Payload"],
                &mut metadata_sidecars,
            );
            for field in &event.payload {
                metadata(document, field.raw, &["Name"], &mut metadata_sidecars);
            }
        }
        let owner = match system.owner {
            SystemOwner::Global => Owner::Global,
            SystemOwner::Namespace(n) => Owner::Namespace(n.into()),
            SystemOwner::ScriptObject(n) => Owner::ScriptObject {
                system_name: n.into(),
                annotation_name: None,
            },
        };
        let mut input = System {
            owner,
            functions: Vec::new(),
            tables: Vec::new(),
        };
        let mut function_sources = Vec::new();
        let mut table_sources = Vec::new();
        let mut enums = Vec::new();
        let mut constants = Vec::new();
        let mut literal_sources = Vec::new();
        let mut events = Vec::new();
        let mut event_sources = Vec::new();
        for function in &system.functions {
            if identities[&function_key(system, function.name)] > 1 {
                issues.push(issue(document, function.raw, "duplicate_callable"));
                continue;
            }
            match callable(function, &mut scalars) {
                Ok(projected) => {
                    let candidate = System {
                        owner: input.owner.clone(),
                        functions: vec![projected.function],
                        tables: Vec::new(),
                    };
                    match renderer.render(&candidate) {
                        Ok(_) => {
                            name_projections.extend(projected.names);
                            if projected.escaped_documentation {
                                metadata_sidecars.push(MetadataSidecar {
                                    field: "Documentation:escaped_control_characters".into(),
                                    source: link(document, function.raw),
                                });
                            }
                            input.functions.extend(candidate.functions);
                            function_sources.push(link(document, function.raw));
                        }
                        Err(error) => issues.push(issue(
                            document,
                            function.raw,
                            format!("renderer_{error:?}"),
                        )),
                    }
                }
                Err(error) => {
                    issues.push(issue(document, function.raw, format!("callable_{error:?}")))
                }
            }
        }
        for table in &system.tables {
            let raw = table_raw(table);
            if identities[&table_key(table)] > 1 {
                issues.push(issue(document, raw, "duplicate_type"));
                continue;
            }
            let converted = match table {
                TableFact::Structure { name, fields, .. } => convert_fields(fields, &mut scalars)
                    .map(|fields| {
                        Some(Table::Structure {
                            name: (*name).into(),
                            fields,
                        })
                    }),
                TableFact::Callback {
                    name,
                    arguments,
                    returns,
                    ..
                } if returns.is_empty() => {
                    convert_fields(arguments, &mut scalars).map(|arguments| {
                        Some(Table::Callback {
                            name: (*name).into(),
                            arguments,
                        })
                    })
                }
                TableFact::Enumeration { name, values, .. } => {
                    let converted = convert_values(values, false, &mut scalars, &mut issues);
                    if values.is_empty() || !converted.is_empty() {
                        let declaration = EnumDeclaration {
                            name: (*name).into(),
                            values: converted,
                            integer_format: IntegerFormat::Decimal,
                        };
                        match literals.render_enums(std::slice::from_ref(&declaration), &[]) {
                            Ok(_) => {
                                enums.push(declaration);
                                literal_sources.push(link(document, raw));
                            }
                            Err(error) => {
                                issues.push(issue(document, raw, format!("renderer_{error:?}")))
                            }
                        }
                    }
                    Ok(None)
                }
                TableFact::Constants { name, values, .. } => {
                    let converted = convert_values(values, true, &mut scalars, &mut issues);
                    if values.is_empty() || !converted.is_empty() {
                        let declaration = ConstantGroup {
                            name: (*name).into(),
                            values: converted,
                            order: MemberOrder::Name,
                        };
                        match literals.render_enums(&[], std::slice::from_ref(&declaration)) {
                            Ok(_) => {
                                constants.push(declaration);
                                literal_sources.push(link(document, raw));
                            }
                            Err(error) => {
                                issues.push(issue(document, raw, format!("renderer_{error:?}")))
                            }
                        }
                    }
                    Ok(None)
                }
                _ => Err(RenderError::UnsupportedType),
            };
            match converted {
                Ok(Some(table)) => {
                    let candidate = System {
                        owner: input.owner.clone(),
                        functions: Vec::new(),
                        tables: vec![table],
                    };
                    match renderer.render(&candidate) {
                        Ok(_) => {
                            input.tables.extend(candidate.tables);
                            table_sources.push(link(document, raw));
                        }
                        Err(error) => {
                            issues.push(issue(document, raw, format!("renderer_{error:?}")))
                        }
                    }
                }
                Ok(None) => {}
                Err(error) => issues.push(issue(document, raw, format!("table_{error:?}"))),
            }
        }
        for event in &system.events {
            if identities[&format!("event:{}", event.literal_name)] > 1 {
                issues.push(issue(document, event.raw, "duplicate_event"));
                continue;
            }
            events.push(EventLiteral {
                name: event.literal_name.into(),
                payload: event
                    .payload
                    .iter()
                    .map(|f| f.name)
                    .collect::<Vec<_>>()
                    .join(", "),
            });
            event_sources.push(link(document, event.raw));
        }
        if !input.functions.is_empty() || !input.tables.is_empty() {
            match renderer.render_mapped(&input) {
                Ok(rendered) => {
                    let mappings = rendered
                        .declarations
                        .iter()
                        .map(|d| SourceMapping {
                            granularity: "declaration",
                            generated: Span {
                                start: d.start,
                                end: d.end,
                            },
                            source: if d.table {
                                table_sources[d.index].clone()
                            } else {
                                function_sources[d.index].clone()
                            },
                        })
                        .collect();
                    push_file(&mut files, &mut total_bytes, "api", rendered.text, mappings)?;
                }
                Err(error) => {
                    issues.push(issue(document, system.raw, format!("renderer_{error:?}")))
                }
            }
        }
        if !enums.is_empty() || !constants.is_empty() {
            match literals.render_enums(&enums, &constants) {
                Ok(text) => {
                    let _ = text; // validate this registration before aggregation
                    all_enums.extend(enums);
                    all_constants.extend(constants);
                    all_literal_sources.extend(literal_sources);
                }
                Err(error) => {
                    issues.push(issue(document, system.raw, format!("literals_{error:?}")))
                }
            }
        }
        if !events.is_empty() {
            match literals.render_events(&events) {
                Ok(text) => {
                    let _ = text;
                    all_events.extend(events);
                    all_event_sources.extend(event_sources);
                }
                Err(error) => issues.push(issue(document, system.raw, format!("events_{error:?}"))),
            }
        }
    }
    if resolution_budget == 0 {
        return Err(RenderError::OutputLimit);
    }
    if !all_enums.is_empty() || !all_constants.is_empty() {
        let text = literals.render_enums(&all_enums, &all_constants)?;
        let maps = whole_file_maps(&text, all_literal_sources);
        push_file(&mut files, &mut total_bytes, "values", text, maps)?;
    }
    if !all_events.is_empty() {
        let text = literals.render_events(&all_events)?;
        let maps = whole_file_maps(&text, all_event_sources);
        push_file(&mut files, &mut total_bytes, "events", text, maps)?;
    }
    if cancelled.load(Ordering::Relaxed) {
        return Err(RenderError::Cancelled);
    }
    Ok(NativeLibrary {
        schema: if corrected.is_some() {
            "wow-native-annotation-library/4"
        } else {
            "wow-native-annotation-library/3"
        },
        revision,
        projection: if issues.is_empty()
            && corrected.as_ref().is_none_or(|c| !c.report.has_blockers())
        {
            "projected_with_sidecars"
        } else {
            "partial"
        },
        negative_authority: false,
        sources: sorted,
        files,
        issues,
        metadata_sidecars,
        scalar_resolutions,
        name_projections,
        corrections: corrected.map(|c| c.report),
        limitations: vec![
            "raw restriction and unknown metadata are retained, not interpreted as runtime safety",
            "named type closure and source-owned widget aliases require the correction/type mapping lane",
            "parameter nilability and default rendering follow the Ketho compatibility profile",
            "declaration source maps; literal maps are file-level; fine-grained E1 maps remain incomplete",
            "CVars and extracted runtime resources are not inferred from documentation absence",
            "EmmyLua and LuaLS semantic consumer compatibility is not established by rendering",
        ],
    })
}

fn push_file(
    files: &mut Vec<AnnotationFile>,
    total: &mut usize,
    lane: &str,
    text: String,
    mappings: Vec<SourceMapping>,
) -> Result<(), RenderError> {
    *total = total
        .checked_add(text.len())
        .ok_or(RenderError::OutputLimit)?;
    if *total > MAX_LIBRARY_BYTES || files.len() >= MAX_FILES {
        return Err(RenderError::OutputLimit);
    }
    files.push(AnnotationFile {
        path: format!("{lane}-{:04}.lua", files.len()),
        sha256: source_digest(text.as_bytes()),
        text,
        mappings,
    });
    Ok(())
}
fn whole_file_maps(text: &str, sources: Vec<SourceLink>) -> Vec<SourceMapping> {
    sources
        .into_iter()
        .map(|source| SourceMapping {
            granularity: "literal_file",
            generated: Span {
                start: 0,
                end: text.len(),
            },
            source,
        })
        .collect()
}
fn function_key(system: &SystemFacts<'_>, name: &str) -> String {
    match system.owner {
        SystemOwner::Global => format!("function:global:{name}"),
        SystemOwner::Namespace(n) => format!("function:namespace:{n}:{name}"),
        SystemOwner::ScriptObject(n) => format!("function:object:{n}:{name}"),
    }
}
fn table_key(table: &TableFact<'_>) -> String {
    match table {
        TableFact::Enumeration { name, .. } => format!("enum:{name}"),
        TableFact::Constants { name, .. } => format!("constants:{name}"),
        TableFact::Structure { name, .. }
        | TableFact::Callback { name, .. }
        | TableFact::Unsupported { name, .. } => format!("type:{name}"),
    }
}
fn table_raw<'a>(table: &TableFact<'a>) -> &'a RawValue {
    match table {
        TableFact::Structure { raw, .. }
        | TableFact::Callback { raw, .. }
        | TableFact::Enumeration { raw, .. }
        | TableFact::Constants { raw, .. }
        | TableFact::Unsupported { raw, .. } => raw,
    }
}
struct ScalarProjection<'a, 'c> {
    catalog: &'c ScalarCatalog<'a>,
    document: &'a DocumentationDocument,
    cancelled: &'c AtomicBool,
    records: &'c mut Vec<ScalarResolutionRecord>,
    budget: &'c mut usize,
}
impl ScalarProjection<'_, '_> {
    fn resolve(
        &mut self,
        raw: &RawValue,
        declared_type: Option<&str>,
    ) -> Result<ScalarValue, RenderError> {
        if *self.budget == 0 {
            return Err(RenderError::OutputLimit);
        }
        let result = self
            .catalog
            .resolve(self.document, raw, declared_type, self.cancelled);
        let value =
            result
                .as_ref()
                .map(|result| result.value.clone())
                .map_err(|error| match error {
                    ScalarError::Cancelled => RenderError::Cancelled,
                    _ => RenderError::UnsupportedType,
                });
        let cost = 512
            + self.document.path().len()
            + result.as_ref().map_or(0, |resolved| {
                let value_bytes = match &resolved.value {
                    ScalarValue::Number(s) | ScalarValue::String(s) => s.len(),
                    ScalarValue::Boolean(_) => 8,
                };
                value_bytes
                    + resolved
                        .evidence
                        .iter()
                        .map(|e| 256 + e.path.len() + e.sha256.len())
                        .sum::<usize>()
            });
        let Some(remaining) = self.budget.checked_sub(cost) else {
            *self.budget = 0;
            return Err(RenderError::OutputLimit);
        };
        *self.budget = remaining;
        self.records.push(ScalarResolutionRecord {
            source: link(self.document, raw),
            result,
        });
        value
    }
}
struct CallableProjection {
    function: Function,
    names: Vec<NameProjection>,
    escaped_documentation: bool,
}

fn callable(
    value: &CallableFact<'_>,
    scalars: &mut ScalarProjection<'_, '_>,
) -> Result<CallableProjection, RenderError> {
    let mut escaped_documentation = false;
    let documentation = value
        .documentation
        .as_ref()
        .map(|parts| {
            parts
                .iter()
                .map(|part| {
                    let (text, changed) = sanitize_documentation(part)?;
                    escaped_documentation |= changed;
                    Ok(text)
                })
                .collect::<Result<Vec<_>, RenderError>>()
        })
        .transpose()?;
    let arguments = convert_fields(&value.arguments, scalars)?;
    let mut returns = convert_fields(&value.returns, scalars)?;
    let mut occupied = returns
        .iter()
        .map(|f| f.name.clone())
        .collect::<BTreeSet<_>>();
    if occupied.len() != returns.len() {
        return Err(RenderError::DuplicateName);
    }
    let mut names = Vec::new();
    for (index, field) in returns.iter_mut().enumerate() {
        if crate::ketho::is_keyword(&field.name) {
            let mut rendered = format!("__wow_return_{}", index + 1);
            while occupied.contains(&rendered) {
                rendered.push('_');
            }
            occupied.insert(rendered.clone());
            names.push(NameProjection {
                source: link(scalars.document, value.returns[index].raw),
                original: field.name.clone(),
                rendered: rendered.clone(),
                rule: "reserved-return-label/1",
            });
            field.name = rendered;
        }
    }
    Ok(CallableProjection {
        function: Function {
            name: value.name.into(),
            documentation,
            arguments,
            returns,
        },
        names,
        escaped_documentation,
    })
}

// Preserve prose as one inert comment line. Do not turn source-provided
// annotation directives into trusted tags, even when preceded by a newline.
fn sanitize_documentation(text: &str) -> Result<(String, bool), RenderError> {
    if text.len() > crate::ketho::MAX_TEXT_BYTES {
        return Err(RenderError::InputLimit);
    }
    if text
        .split(['\n', '\r', '\u{2028}', '\u{2029}'])
        .any(|line| {
            line.trim_start()
                .trim_start_matches('-')
                .trim_start()
                .starts_with('@')
        })
    {
        return Err(RenderError::UnsafeDocumentation);
    }
    let mut result = String::new();
    let mut changed = false;
    for character in text.chars() {
        match character {
            '\n' => {
                result.push_str("\\n");
                changed = true;
            }
            '\r' => {
                result.push_str("\\r");
                changed = true;
            }
            '\t' => {
                result.push_str("\\t");
                changed = true;
            }
            c if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}') => {
                result.push_str(&format!("\\u{{{:x}}}", c as u32));
                changed = true;
            }
            c => result.push(c),
        }
        if result.len() > crate::ketho::MAX_TEXT_BYTES {
            return Err(RenderError::InputLimit);
        }
    }
    Ok((result, changed))
}

fn convert_fields(
    fields: &[FieldFact<'_>],
    scalars: &mut ScalarProjection<'_, '_>,
) -> Result<Vec<Field>, RenderError> {
    fields
        .iter()
        .map(|f| {
            let default_text = match f.default {
                None
                | Some(RawValue {
                    kind: RawKind::Nil, ..
                }) => None,
                Some(raw) => Some(match scalars.resolve(raw, None)? {
                    ScalarValue::Boolean(value) => value.to_string(),
                    ScalarValue::Number(value) | ScalarValue::String(value) => value,
                }),
            };
            let variadic = match f.stride_index.map(|v| &v.kind) {
                None | Some(RawKind::Nil | RawKind::Boolean(false)) => false,
                Some(RawKind::Number(n)) if n.parse::<u64>().is_ok_and(|v| v > 0) => true,
                _ => return Err(RenderError::InvalidVariadic),
            };
            Ok(Field {
                name: f.name.into(),
                type_name: f.type_name.into(),
                inner_type: f.inner_type.map(Into::into),
                nilable: f.nilable.unwrap_or(false),
                default_text,
                variadic,
            })
        })
        .collect()
}
fn convert_values(
    values: &[ValueFact<'_>],
    typed_constants: bool,
    scalars: &mut ScalarProjection<'_, '_>,
    issues: &mut Vec<ProjectionIssue>,
) -> Vec<LiteralMember> {
    let mut result = Vec::new();
    let mut names = BTreeMap::new();
    for value in values {
        *names.entry(value.name).or_insert(0usize) += 1;
    }
    for value in values {
        if names[value.name] > 1 {
            issues.push(issue(scalars.document, value.raw, "value_DuplicateName"));
            continue;
        }
        let resolved = scalars.resolve(
            value.value,
            if typed_constants {
                value.type_name
            } else {
                None
            },
        );
        match resolved.and_then(scalar) {
            Ok(literal) => result.push(LiteralMember {
                name: value.name.into(),
                value: literal,
            }),
            Err(error) => issues.push(issue(
                scalars.document,
                value.raw,
                format!("value_{error:?}"),
            )),
        }
    }
    result
}
fn scalar(value: ScalarValue) -> Result<LiteralValue, RenderError> {
    match value {
        ScalarValue::Boolean(b) => Ok(LiteralValue::Boolean(b)),
        ScalarValue::String(s) => Ok(LiteralValue::String(s)),
        ScalarValue::Number(s) => {
            let (negative, magnitude) = s
                .strip_prefix('-')
                .map_or((false, s.as_str()), |v| (true, v));
            let integer = if let Some(hex) = magnitude
                .strip_prefix("0x")
                .or_else(|| magnitude.strip_prefix("0X"))
            {
                i64::from_str_radix(hex, 16).ok()
            } else {
                magnitude.parse::<i64>().ok()
            };
            match integer.and_then(|v| if negative { v.checked_neg() } else { Some(v) }) {
                Some(value) => Ok(LiteralValue::Integer(value)),
                None => Err(RenderError::UnsupportedType),
            }
        }
    }
}

fn metadata(
    document: &DocumentationDocument,
    value: &RawValue,
    projected: &[&str],
    out: &mut Vec<MetadataSidecar>,
) {
    if let Some(fields) = value.fields() {
        for field in fields {
            if let wow_reference::native::RawKey::Name(name) = &field.key
                && !projected.contains(&name.as_str())
            {
                out.push(MetadataSidecar {
                    field: name.clone(),
                    source: link(document, &field.value),
                });
            }
        }
    }
}
fn field_metadata(
    document: &DocumentationDocument,
    value: &FieldFact<'_>,
    out: &mut Vec<MetadataSidecar>,
) {
    metadata(
        document,
        value.raw,
        &[
            "Name",
            "Type",
            "InnerType",
            "Nilable",
            "Default",
            "StrideIndex",
        ],
        out,
    );
}
