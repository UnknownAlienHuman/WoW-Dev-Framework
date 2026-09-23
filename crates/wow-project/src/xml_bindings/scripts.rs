//! Enumerate inherited handler sources, not effective callback dispatch. The
//! declaring script and the consuming declaration have separate identities.
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_core::{ContentDigest, SourceContent};

use super::{MAX_BINDINGS, exhausted, invalid, receivers};
use crate::ProjectResult;
use crate::load::{ProjectLoadPlan, XmlElementRecord, XmlElementRole, XmlScriptSource};

/// One source occurrence reachable through the consumer's captured `inherits`
/// graph. Multiple same-named handlers remain candidates; no winner is selected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlInheritedScriptSource {
    /// Both IDs address the load plan's XML declaration table.
    pub consumer_id: String,
    pub declaring_owner_id: String,
    /// Exact element in the declaring owner's XML document, including its flags,
    /// source map and original body. Bodies are not copied or parsed again.
    pub script_id: String,
    pub source_kind: XmlScriptSource,
    /// The shared receiver-source graph is keyed by consumer_id. This flag only
    /// attests source enumeration and script representation, never dispatch.
    pub source_complete: bool,
    /// Indices in ProjectXmlLuaBindings::bindings. Methods use the consumer's
    /// mixins; functions reuse their global query. Inline bodies have no lookup.
    pub binding_indices: Vec<usize>,
}

#[derive(Clone, Copy)]
pub(super) struct BindingSite<'a> {
    pub document: &'a str,
    pub digest: ContentDigest<SourceContent>,
    pub element: &'a XmlElementRecord,
    pub owner: Option<&'a XmlElementRecord>,
    pub consumer_id: Option<&'a str>,
    pub inherited_index: Option<usize>,
}

pub(super) struct BindingSites<'a> {
    pub sites: Vec<BindingSite<'a>>,
    pub inherited: Vec<XmlInheritedScriptSource>,
}

pub(super) fn collect<'a>(
    plan: &'a ProjectLoadPlan,
    receivers: &mut receivers::Resolver<'a>,
    stop: &AtomicBool,
) -> ProjectResult<BindingSites<'a>> {
    let mut elements = BTreeMap::new();
    for index in plan.xml_documents().values() {
        for element in index.elements() {
            crate::analyzer::checkpoint(stop)?;
            if elements
                .insert(element.occurrence_id.as_str(), element)
                .is_some()
            {
                return Err(invalid());
            }
        }
    }
    let mut sites = Vec::new();
    let mut by_owner: BTreeMap<&str, Vec<BindingSite<'a>>> = BTreeMap::new();
    for (document, index) in plan.xml_documents() {
        for element in index.elements() {
            crate::analyzer::checkpoint(stop)?;
            if element.declaration.is_none() && element.script.is_none() {
                continue;
            }
            let owner_id = element
                .script
                .as_ref()
                .and_then(|script| script.owner_occurrence_id.as_deref());
            let owner = owner_id.and_then(|id| elements.get(id).copied());
            let site = BindingSite {
                document,
                digest: index.source_digest(),
                element,
                owner,
                consumer_id: None,
                inherited_index: None,
            };
            sites.push(site);
            // Only a direct Scripts child can be a handler source. Do not
            // inherit top-level chunks or handlers of nested child objects.
            if element.ui_namespace
                && element.role == XmlElementRole::ScriptBinding
                && let Some(id) = owner_id
            {
                by_owner.entry(id).or_default().push(site);
            }
        }
    }
    let mut inherited = Vec::new();
    if by_owner.is_empty() {
        return Ok(BindingSites { sites, inherited });
    }
    let mut visits = 0usize;
    let mut text_bytes = 0usize;
    for index in plan.xml_documents().values() {
        for consumer in index.declarations() {
            crate::analyzer::checkpoint(stop)?;
            if consumer
                .declaration
                .as_ref()
                .is_none_or(|d| d.inherits.is_empty())
            {
                continue;
            }
            let sources = receivers.resolve(&consumer.occurrence_id, stop)?;
            for declaration_id in &sources.declarations {
                crate::analyzer::checkpoint(stop)?;
                visits = visits.checked_add(1).ok_or_else(exhausted)?;
                if visits > 262_144 {
                    return Err(exhausted());
                }
                if declaration_id == &consumer.occurrence_id {
                    continue;
                }
                let Some(scripts) = by_owner.get(declaration_id.as_str()) else {
                    continue;
                };
                for source in scripts {
                    crate::analyzer::checkpoint(stop)?;
                    if inherited.len() >= MAX_BINDINGS {
                        return Err(exhausted());
                    }
                    let script = source.element.script.as_ref().ok_or_else(invalid)?;
                    text_bytes = text_bytes
                        .checked_add(
                            consumer.occurrence_id.len()
                                + declaration_id.len()
                                + source.element.occurrence_id.len(),
                        )
                        .ok_or_else(exhausted)?;
                    if text_bytes > 16 * 1024 * 1024 {
                        return Err(exhausted());
                    }
                    let inherited_index = inherited.len();
                    inherited.push(XmlInheritedScriptSource {
                        consumer_id: consumer.occurrence_id.clone(),
                        declaring_owner_id: declaration_id.clone(),
                        script_id: source.element.occurrence_id.clone(),
                        source_kind: script.source_kind,
                        source_complete: sources.complete
                            && source.element.issues.is_empty()
                            && script.source_kind != XmlScriptSource::Unresolved,
                        binding_indices: Vec::new(),
                    });
                    if script.method_reference.is_some() || script.function_reference.is_some() {
                        sites.push(BindingSite {
                            owner: Some(consumer),
                            consumer_id: Some(&consumer.occurrence_id),
                            inherited_index: Some(inherited_index),
                            ..*source
                        });
                    }
                }
            }
        }
    }
    Ok(BindingSites { sites, inherited })
}
