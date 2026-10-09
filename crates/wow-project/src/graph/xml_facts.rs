//! Generation-bound static XML facts for downstream structural recognizers.
//! Parsing, indexing and local declaration linking stay in the existing load
//! owner. This projection only re-states retained records: no source is
//! reparsed, no document is reopened, and no runtime object, callback, handler
//! or load outcome is inferred.
use super::*;
use crate::load::xml_references::{XmlReferenceKind, XmlReferenceOrder, XmlReferenceResolution};
use crate::load::{
    LoadSource, ProjectLoadPlan, XmlDeclaration, XmlElementRole, XmlScriptRecord, XmlScriptSource,
};
use std::collections::BTreeSet;
use wow_core::{
    CanonicalResult, ContentDigest, GenerationContextId, ProfileIdentity, SourceContent,
};

pub const PROJECT_XML_FACT_PROFILE: &str = "wow-project/xml-recognizer-facts/1";
const MAX_DECLARATION_FACTS: usize = 4096;
const MAX_PARENT_FACTS: usize = 8192;
const MAX_INHERITANCE_FACTS: usize = 8192;
const MAX_SCRIPT_FACTS: usize = 4096;
const MAX_FACTS: usize =
    MAX_DECLARATION_FACTS + MAX_PARENT_FACTS + MAX_INHERITANCE_FACTS + MAX_SCRIPT_FACTS;

/// One generation-bound static XML observation. Exact occurrence identity and
/// content binding come from the retained index and resolved reference report;
/// a named declaration is never an accepted template kind, XSD type or object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectXmlFact {
    pub fact_id: String,
    pub context_id: GenerationContextId,
    pub scope: ProjectXmlFactScope,
    pub document: String,
    pub occurrence_id: String,
    pub element_name: String,
    pub span: SourceSpan,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
    pub content_digest: ContentDigest<SourceContent>,
    pub document_digest: ContentDigest<CanonicalResult>,
    pub kind: ProjectXmlFactKind,
}

/// Scope retained from the owner plan, never inferred from a path spelling.
/// `package` stays `None` for the standalone selected-TOC plan this reads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectXmlFactScope {
    pub selected_toc: String,
    pub flavor: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
}

/// Declared attribute state retained verbatim. Absent means the attribute was
/// never spelled in this document; it is never normalized to a default kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectXmlFactDeclarationState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_template: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intrinsic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_array: Option<String>,
    pub inherited_names: Vec<String>,
    pub mixin_names: Vec<String>,
    pub valid_declaration: bool,
    /// Captured lexical XML containment inside this document. It is not the
    /// explicit `parent` name reference and never a runtime frame parent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_occurrence_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProjectXmlFactKind {
    /// Structural declaration state of one element occurrence.
    Declaration {
        role: XmlElementRole,
        declaration: ProjectXmlFactDeclarationState,
    },
    /// The explicit `parent` attribute resolved by the load owner. A unique
    /// valid local declaration retains its exact target identity; every other
    /// spelling keeps its resolution plus the loader order and cycle verdict.
    /// This is the name reference, distinct from captured lexical containment.
    Parent {
        reference_id: String,
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        target_occurrence_id: Option<String>,
        resolution: XmlReferenceResolution,
        #[serde(skip_serializing_if = "Option::is_none")]
        order: Option<XmlReferenceOrder>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cycle_id: Option<String>,
    },
    /// A resolved local inheritance name, retaining the loader exact order
    /// verdict and any cycle membership.
    Inheritance {
        reference_id: String,
        target_occurrence_id: String,
        ordinal: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        order: Option<XmlReferenceOrder>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cycle_id: Option<String>,
    },
    /// An inheritance name that produced no admissible target. The spelling and
    /// resolution are retained so an unresolved name is never a clean negative.
    InheritanceUnresolved {
        reference_id: String,
        ordinal: u64,
        name: String,
        resolution: XmlReferenceResolution,
        #[serde(skip_serializing_if = "Option::is_none")]
        order: Option<XmlReferenceOrder>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cycle_id: Option<String>,
    },
    /// A captured script binding with its retained source classification.
    Script {
        script_name: String,
        source_kind: XmlScriptSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        owner_occurrence_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        inherit: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        intrinsic_order: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_reference: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        function_reference: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        method_reference: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        inline_unit_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        inline_content_digest: Option<ContentDigest<SourceContent>>,
    },
}

/// Captured lexical containment, kept out of `ProjectXmlFactKind` because it
/// describes document structure rather than a resolved name reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectXmlContainment {
    pub fact_id: String,
    pub context_id: GenerationContextId,
    pub scope: ProjectXmlFactScope,
    pub document: String,
    pub occurrence_id: String,
    pub element_name: String,
    pub span: SourceSpan,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
    pub content_digest: ContentDigest<SourceContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_occurrence_id: Option<String>,
    pub document_digest: ContentDigest<CanonicalResult>,
}

/// Shared exact-source identity of one emitted record.
struct XmlFactInputs<'a> {
    project: &'a ProjectView,
    scope: ProjectXmlFactScope,
    document: &'a str,
    source: &'a LoadSource,
    element_name: &'a str,
    occurrence_id: &'a str,
    span: SourceSpan,
    source_handle_id: StableHandleId,
    evidence_id: EvidenceId,
    document_digest: ContentDigest<CanonicalResult>,
}
impl XmlFactInputs<'_> {
    fn charge(&self, text_bytes: &mut usize) -> ProjectResult<()> {
        charge(
            text_bytes,
            self.document
                .len()
                .saturating_mul(4)
                .saturating_add(self.occurrence_id.len().saturating_mul(6))
                .saturating_add(self.element_name.len().saturating_mul(3))
                .saturating_add(512),
        )
    }
    fn context_id(&self) -> GenerationContextId {
        self.project.snapshot().generation_context().context_id()
    }
    fn finish(
        self,
        document_digest: ContentDigest<CanonicalResult>,
        kind: ProjectXmlFactKind,
        text_bytes: &mut usize,
    ) -> ProjectResult<ProjectXmlFact> {
        self.charge(text_bytes)?;
        let kind_bytes = wow_core::canonical_json_bytes(&kind).map_err(|_| invalid())?;
        charge(text_bytes, kind_bytes.len())?;
        let context_id = self.context_id();
        let identity = (
            PROJECT_XML_FACT_PROFILE,
            context_id,
            &self.scope,
            self.document,
            self.occurrence_id,
            self.span,
            self.source.content_digest,
            document_digest,
            &kind,
        );
        let digest = crate::identity::canonical_digest(
            PROJECT_XML_FACT_PROFILE,
            &identity,
            ProjectPhase::View,
        )?;
        Ok(ProjectXmlFact {
            fact_id: format!("project-xml-fact:{digest}"),
            context_id,
            scope: self.scope,
            document: self.document.to_owned(),
            occurrence_id: self.occurrence_id.to_owned(),
            element_name: self.element_name.to_owned(),
            span: self.span,
            source_handle_id: self.source_handle_id,
            evidence_id: self.evidence_id,
            content_digest: self.source.content_digest,
            document_digest,
            kind,
        })
    }
    /// Captured containment is emitted from the same exact support but keeps its
    /// own record type, so it can never be read as a resolved `parent` name.
    fn containment(self, parent: Option<String>) -> ProjectResult<ProjectXmlContainment> {
        let context_id = self.context_id();
        let source_digest = self.source.content_digest;
        let document_digest = self.document_digest;
        #[derive(Serialize)]
        struct Identity<'a> {
            profile: &'static str,
            context_id: GenerationContextId,
            scope: &'a ProjectXmlFactScope,
            document: &'a str,
            occurrence: &'a str,
            span: SourceSpan,
            source_digest: ContentDigest<SourceContent>,
            document_digest: ContentDigest<CanonicalResult>,
            #[serde(skip_serializing_if = "Option::is_none")]
            parent: Option<&'a str>,
        }
        let identity = Identity {
            profile: PROJECT_XML_FACT_PROFILE,
            context_id,
            scope: &self.scope,
            document: self.document,
            occurrence: self.occurrence_id,
            span: self.span,
            source_digest,
            document_digest,
            parent: parent.as_deref(),
        };
        let digest = crate::identity::canonical_digest(
            PROJECT_XML_FACT_PROFILE,
            &identity,
            ProjectPhase::View,
        )?;
        Ok(ProjectXmlContainment {
            fact_id: format!("project-xml-containment:{digest}"),
            context_id,
            scope: self.scope,
            document: self.document.to_owned(),
            occurrence_id: self.occurrence_id.to_owned(),
            element_name: self.element_name.to_owned(),
            span: self.span,
            source_handle_id: self.source_handle_id,
            evidence_id: self.evidence_id,
            content_digest: source_digest,
            parent_occurrence_id: parent,
            document_digest,
        })
    }
}

fn declaration_state(
    declaration: &XmlDeclaration,
    parent_occurrence_id: Option<String>,
    valid_declaration: bool,
) -> ProjectXmlFactDeclarationState {
    ProjectXmlFactDeclarationState {
        name: declaration.name.clone(),
        virtual_template: declaration.virtual_template,
        intrinsic: declaration.intrinsic,
        parent_reference: declaration.parent_reference.clone(),
        parent_key: declaration.parent_key.clone(),
        parent_array: declaration.parent_array.clone(),
        inherited_names: declaration.inherits.clone(),
        mixin_names: declaration.mixins.clone(),
        valid_declaration,
        parent_occurrence_id,
    }
}

/// One shared scope builder. Every document is read from the same retained
/// standalone plan, so selected TOC and flavor are identical across a run and
/// package stays absent rather than guessed from a path spelling.
fn scope(plan: &ProjectLoadPlan, profile: &ProfileIdentity) -> ProjectXmlFactScope {
    ProjectXmlFactScope {
        selected_toc: plan.selected_toc().to_owned(),
        flavor: profile.flavor_id().to_owned(),
        package: None,
    }
}

fn script_state(script: &XmlScriptRecord, element_name: &str) -> ProjectXmlFactKind {
    ProjectXmlFactKind::Script {
        script_name: element_name.to_owned(),
        source_kind: script.source_kind,
        owner_occurrence_id: script.owner_occurrence_id.clone(),
        inherit: script.inherit.clone(),
        intrinsic_order: script.intrinsic_order.clone(),
        file_reference: script.file_reference.clone(),
        function_reference: script.function_reference.clone(),
        method_reference: script.method_reference.clone(),
        inline_unit_id: script.inline_lua.as_ref().map(|body| body.unit_id.clone()),
        inline_content_digest: script.inline_lua.as_ref().map(|body| body.content_digest),
    }
}

/// Project one retained standalone closure. Name resolution and load order come
/// from the load owner; lexical containment remains a separate record stream.
pub(super) fn project(
    project: &ProjectView,
    provenance: &mut ProjectGraphProvenance,
    text_bytes: &mut usize,
    stop: &AtomicBool,
) -> ProjectResult<(Vec<ProjectXmlFact>, Vec<ProjectXmlContainment>)> {
    const MAX_CONTAINMENT_FACTS: usize = 32_768;
    let mut facts = Vec::new();
    let mut containment = Vec::new();
    let configuration = project.configuration();
    let Some(plan) = configuration.load_plan() else {
        return Ok((facts, containment));
    };
    plan.validate_profile(configuration.selected_profile())?;
    let selected_scope = scope(plan, configuration.selected_profile());
    let report = plan.xml_references();
    if report.declarations().len() > MAX_DECLARATION_FACTS {
        return Err(exhausted());
    }
    let sources = plan
        .sources()
        .iter()
        .map(|source| (source.path.as_str(), source))
        .collect::<BTreeMap<_, _>>();
    let mut occurrences = BTreeSet::new();
    let mut declaration_count = 0usize;
    let mut script_count = 0usize;
    for (document, index) in plan.xml_documents() {
        crate::analyzer::checkpoint(stop)?;
        let source = *sources.get(document.as_str()).ok_or_else(invalid)?;
        let text = plan.document_text(document).ok_or_else(invalid)?;
        if index.document() != document
            || source.byte_length != text.len() as u64
            || crate::identity::source_digest(text.as_bytes()) != index.source_digest()
            || source.content_digest != index.source_digest()
        {
            return Err(invalid());
        }
        for element in index.elements() {
            crate::analyzer::checkpoint(stop)?;
            if containment.len() >= MAX_CONTAINMENT_FACTS {
                return Err(exhausted());
            }
            if !occurrences.insert(element.occurrence_id.as_str()) {
                return Err(invalid());
            }
            if let Some(parent) = element.parent_occurrence_id.as_deref()
                && index.element(parent).is_none()
            {
                return Err(invalid());
            }
            let span = super::xml::source_span(plan, document, &element.span)?;
            let (source_handle_id, evidence_id) = support(project, source, span, provenance)?;
            let inputs = || XmlFactInputs {
                project,
                scope: selected_scope.clone(),
                document,
                source,
                element_name: &element.qualified_name,
                occurrence_id: &element.occurrence_id,
                span,
                source_handle_id,
                evidence_id,
                document_digest: index.digest(),
            };
            inputs().charge(text_bytes)?;
            containment.push(inputs().containment(element.parent_occurrence_id.clone())?);
            if let Some(declaration) = &element.declaration {
                if declaration_count >= MAX_DECLARATION_FACTS || facts.len() >= MAX_FACTS {
                    return Err(exhausted());
                }
                facts.push(inputs().finish(
                    index.digest(),
                    ProjectXmlFactKind::Declaration {
                        role: element.role,
                        declaration: declaration_state(
                            declaration,
                            element.parent_occurrence_id.clone(),
                            element.issues.is_empty(),
                        ),
                    },
                    text_bytes,
                )?);
                declaration_count += 1;
            }
            if let Some(script) = &element.script {
                if script_count >= MAX_SCRIPT_FACTS || facts.len() >= MAX_FACTS {
                    return Err(exhausted());
                }
                facts.push(inputs().finish(
                    index.digest(),
                    script_state(script, &element.qualified_name),
                    text_bytes,
                )?);
                script_count += 1;
            }
        }
    }
    let mut parent_count = 0usize;
    let mut inheritance_count = 0usize;
    for reference in report.references() {
        crate::analyzer::checkpoint(stop)?;
        if facts.len() >= MAX_FACTS {
            return Err(exhausted());
        }
        match reference.kind {
            XmlReferenceKind::Parent => {
                if parent_count >= MAX_PARENT_FACTS {
                    return Err(exhausted());
                }
                parent_count += 1;
            }
            XmlReferenceKind::Inherits => {
                if inheritance_count >= MAX_INHERITANCE_FACTS {
                    return Err(exhausted());
                }
                inheritance_count += 1;
            }
        }
        let site = report
            .declarations()
            .get(&reference.source_id)
            .ok_or_else(invalid)?;
        if site.occurrence_id != reference.source_id {
            return Err(invalid());
        }
        let document = site.document.as_str();
        let source = *sources.get(document).ok_or_else(invalid)?;
        let index = plan.xml_documents().get(document).ok_or_else(invalid)?;
        let element = index.element(&site.occurrence_id).ok_or_else(invalid)?;
        if index.source_digest() != source.content_digest
            || site.content_digest != source.content_digest
        {
            return Err(invalid());
        }
        let target = if let XmlReferenceResolution::UniqueLocalDeclaration { declaration_id } =
            &reference.resolution
        {
            let target = report
                .declarations()
                .get(declaration_id)
                .ok_or_else(invalid)?;
            let target_index = plan
                .xml_documents()
                .get(&target.document)
                .ok_or_else(invalid)?;
            let target_source = *sources.get(target.document.as_str()).ok_or_else(invalid)?;
            if target.occurrence_id != *declaration_id
                || target_index.element(declaration_id).is_none()
                || target_index.source_digest() != target.content_digest
                || target_source.content_digest != target.content_digest
            {
                return Err(invalid());
            }
            Some(target.occurrence_id.clone())
        } else {
            None
        };
        let kind = match reference.kind {
            XmlReferenceKind::Parent => ProjectXmlFactKind::Parent {
                reference_id: reference.reference_id.clone(),
                name: reference.name.clone(),
                target_occurrence_id: target,
                resolution: reference.resolution.clone(),
                order: reference.order,
                cycle_id: reference.cycle_id.clone(),
            },
            XmlReferenceKind::Inherits => match target {
                Some(target_occurrence_id) => ProjectXmlFactKind::Inheritance {
                    reference_id: reference.reference_id.clone(),
                    target_occurrence_id,
                    ordinal: reference.ordinal,
                    order: reference.order,
                    cycle_id: reference.cycle_id.clone(),
                },
                None => ProjectXmlFactKind::InheritanceUnresolved {
                    reference_id: reference.reference_id.clone(),
                    ordinal: reference.ordinal,
                    name: reference.name.clone(),
                    resolution: reference.resolution.clone(),
                    order: reference.order,
                    cycle_id: reference.cycle_id.clone(),
                },
            },
        };
        let span = super::xml::source_span(plan, document, &reference.attribute_span)?;
        let (source_handle_id, evidence_id) = support(project, source, span, provenance)?;
        facts.push(
            XmlFactInputs {
                project,
                scope: selected_scope.clone(),
                document,
                source,
                element_name: &element.qualified_name,
                occurrence_id: &element.occurrence_id,
                span,
                source_handle_id,
                evidence_id,
                document_digest: index.digest(),
            }
            .finish(index.digest(), kind, text_bytes)?,
        );
    }
    crate::analyzer::checkpoint(stop)?;
    facts.sort_by(|a, b| a.fact_id.cmp(&b.fact_id));
    containment.sort_by(|a, b| a.fact_id.cmp(&b.fact_id));
    if facts
        .windows(2)
        .any(|pair| pair[0].fact_id == pair[1].fact_id)
        || containment
            .windows(2)
            .any(|pair| pair[0].fact_id == pair[1].fact_id)
    {
        return Err(invalid());
    }
    Ok((facts, containment))
}
