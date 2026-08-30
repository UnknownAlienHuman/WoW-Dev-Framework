use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    CoreError, CoreErrorCode, CoreResult, ExternalGenerationId, ProducerId, ProfileIdentity,
    ProjectGenerationId, ReferenceGenerationId, SchemaVersion, ToolVersion,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ExternalGeneration {
    pub producer: ProducerId,
    pub generation: ExternalGenerationId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationContext {
    profile: ProfileIdentity,
    reference: ReferenceGenerationId,
    project: Option<ProjectGenerationId>,
    external: BTreeMap<ProducerId, ExternalGenerationId>,
    schemas: BTreeMap<ProducerId, SchemaVersion>,
    tools: BTreeMap<ProducerId, ToolVersion>,
}

impl GenerationContext {
    pub fn new(
        profile: ProfileIdentity,
        reference: ReferenceGenerationId,
        project: Option<ProjectGenerationId>,
    ) -> CoreResult<Self> {
        profile.validate()?;
        Ok(Self {
            profile,
            reference,
            project,
            external: BTreeMap::new(),
            schemas: BTreeMap::new(),
            tools: BTreeMap::new(),
        })
    }

    pub fn with_external(
        mut self,
        producer: ProducerId,
        generation: ExternalGenerationId,
    ) -> CoreResult<Self> {
        insert_exact(
            &mut self.external,
            producer,
            generation,
            "external generation",
        )?;
        Ok(self)
    }

    pub fn with_schema(
        mut self,
        producer: ProducerId,
        version: SchemaVersion,
    ) -> CoreResult<Self> {
        insert_exact(&mut self.schemas, producer, version, "schema version")?;
        Ok(self)
    }

    pub fn with_tool(
        mut self,
        producer: ProducerId,
        version: ToolVersion,
    ) -> CoreResult<Self> {
        insert_exact(&mut self.tools, producer, version, "tool version")?;
        Ok(self)
    }

    #[must_use]
    pub fn profile(&self) -> &ProfileIdentity {
        &self.profile
    }

    #[must_use]
    pub fn reference(&self) -> &ReferenceGenerationId {
        &self.reference
    }

    #[must_use]
    pub fn project(&self) -> Option<&ProjectGenerationId> {
        self.project.as_ref()
    }

    #[must_use]
    pub fn external(&self) -> &BTreeMap<ProducerId, ExternalGenerationId> {
        &self.external
    }

    #[must_use]
    pub fn schemas(&self) -> &BTreeMap<ProducerId, SchemaVersion> {
        &self.schemas
    }

    #[must_use]
    pub fn tools(&self) -> &BTreeMap<ProducerId, ToolVersion> {
        &self.tools
    }
}

fn insert_exact<K, V>(
    map: &mut BTreeMap<K, V>,
    key: K,
    value: V,
    component: &'static str,
) -> CoreResult<()>
where
    K: Ord + Clone + std::fmt::Display,
    V: PartialEq + std::fmt::Display,
{
    if let Some(existing) = map.get(&key) {
        if existing != &value {
            return Err(CoreError::generation_mismatch(component, existing, value));
        }
        return Ok(());
    }
    map.insert(key, value);
    Ok(())
}

pub fn merge_generation_context(
    left: &GenerationContext,
    right: &GenerationContext,
) -> CoreResult<GenerationContext> {
    if left.profile != right.profile {
        return Err(CoreError::new(
            CoreErrorCode::GenerationMismatch,
            "merge_generation_context",
            format!(
                "profile identity differs for {}",
                left.profile.id()
            ),
        ));
    }
    if left.reference != right.reference {
        return Err(CoreError::generation_mismatch(
            "reference generation",
            &left.reference,
            &right.reference,
        ));
    }

    let project = match (&left.project, &right.project) {
        (Some(left), Some(right)) if left != right => {
            return Err(CoreError::generation_mismatch(
                "project generation",
                left,
                right,
            ));
        }
        (Some(project), _) | (_, Some(project)) => Some(project.clone()),
        (None, None) => None,
    };

    let mut merged = GenerationContext::new(left.profile.clone(), left.reference.clone(), project)?;
    for (producer, generation) in left.external.iter().chain(&right.external) {
        merged = merged.with_external(producer.clone(), generation.clone())?;
    }
    for (producer, version) in left.schemas.iter().chain(&right.schemas) {
        merged = merged.with_schema(producer.clone(), version.clone())?;
    }
    for (producer, version) in left.tools.iter().chain(&right.tools) {
        merged = merged.with_tool(producer.clone(), version.clone())?;
    }
    Ok(merged)
}

pub fn require_same_generation(
    expected: &GenerationContext,
    actual: &GenerationContext,
) -> CoreResult<()> {
    if expected == actual {
        return Ok(());
    }
    Err(CoreError::new(
        CoreErrorCode::GenerationMismatch,
        "require_same_generation",
        format!(
            "contexts differ for profile {} and reference {}",
            expected.profile.id(),
            expected.reference
        ),
    ))
}
