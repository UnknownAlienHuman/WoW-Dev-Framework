use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{
    CheckContext, CheckScope, ComponentSnapshot, ContextIdentity, GenerationSelector, ServiceError,
    ServiceErrorCode, ServiceResult,
};

/// Exact configured state returned by the service's narrow owner adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceBackendStatus {
    current_context: Option<ContextIdentity>,
    components: Vec<ComponentSnapshot>,
}

impl ServiceBackendStatus {
    pub fn new(
        current_context: Option<ContextIdentity>,
        mut components: Vec<ComponentSnapshot>,
    ) -> ServiceResult<Self> {
        components.sort_by(|left, right| left.component_id().cmp(right.component_id()));
        if components.is_empty()
            || components
                .iter()
                .map(ComponentSnapshot::component_id)
                .collect::<BTreeSet<_>>()
                .len()
                != components.len()
        {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidContext,
                "backend component inventory must be non-empty and unique",
            ));
        }
        Ok(Self {
            current_context,
            components,
        })
    }

    #[must_use]
    pub const fn current_context(&self) -> Option<&ContextIdentity> {
        self.current_context.as_ref()
    }

    #[must_use]
    pub fn components(&self) -> &[ComponentSnapshot] {
        &self.components
    }
}

/// Minimal E0 owner port. Implementations return immutable, already normalized
/// owner output and must not switch generation after selector resolution.
pub trait ServiceBackend: Send + Sync {
    fn status(&self) -> ServiceResult<ServiceBackendStatus>;

    fn acquire_context(
        &self,
        selector: &GenerationSelector,
        scope: &CheckScope,
    ) -> ServiceResult<CheckContext>;
}

/// In-memory exact backend used by the closed E0 fixture and embedders that
/// already own immutable project/rule publication. It performs no discovery.
#[derive(Debug, Clone)]
pub struct OwnedServiceBackend {
    status: ServiceBackendStatus,
    contexts: BTreeMap<Box<str>, CheckContext>,
}

impl OwnedServiceBackend {
    pub fn new(
        status: ServiceBackendStatus,
        contexts: Vec<CheckContext>,
    ) -> ServiceResult<Self> {
        let mut by_generation = BTreeMap::new();
        for context in contexts {
            let generation = context.identity().project_generation_id().into();
            if by_generation.insert(generation, context).is_some() {
                return Err(ServiceError::new(
                    ServiceErrorCode::InvalidContext,
                    "duplicate project generation in backend",
                ));
            }
        }
        if let Some(current) = status.current_context()
            && !by_generation.contains_key(current.project_generation_id())
        {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidContext,
                "current project generation has no retained context",
            ));
        }
        Ok(Self {
            status,
            contexts: by_generation,
        })
    }
}

impl ServiceBackend for OwnedServiceBackend {
    fn status(&self) -> ServiceResult<ServiceBackendStatus> {
        Ok(self.status.clone())
    }

    fn acquire_context(
        &self,
        selector: &GenerationSelector,
        scope: &CheckScope,
    ) -> ServiceResult<CheckContext> {
        let generation = match selector {
            GenerationSelector::Exact(generation) => generation.as_ref(),
            GenerationSelector::CurrentPublished { project_id } => {
                let current = self.status.current_context().ok_or_else(|| {
                    ServiceError::new(
                        ServiceErrorCode::CurrentGenerationUnavailable,
                        "no project generation is currently published",
                    )
                })?;
                if current.project_id() != project_id.as_ref() {
                    return Err(ServiceError::new(
                        ServiceErrorCode::IdentityMismatch,
                        "current-generation selector targets another project",
                    ));
                }
                current.project_generation_id()
            }
        };
        let context = self.contexts.get(generation).ok_or_else(|| {
            ServiceError::new(
                ServiceErrorCode::ExactGenerationUnavailable,
                "selected project generation is not retained",
            )
        })?;
        if context.selected_scope() != scope {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidContext,
                "backend context does not cover the requested exact scope",
            ));
        }
        Ok(context.clone())
    }
}
