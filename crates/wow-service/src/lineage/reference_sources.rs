//! Two actual Reference carrier epochs. Paths are operational inputs; exact
//! epoch/generation and consumer identities bind durable retention plans.
use super::*;
use crate::live_project::{LiveProjectRead, LiveProjectStore};
use wow_store::StoreInstanceId;
use wow_store::project::{ReadSelector, RetentionRoot, RetentionRootId, RetentionRootKind};

#[derive(Clone, Copy)]
pub struct ReferenceLineageRoots<'a> {
    pub before: &'a Path,
    pub after: &'a Path,
}
pub(super) struct ReferenceSources {
    pub before: LiveProjectStore,
    pub after: LiveProjectStore,
}
impl ReferenceSources {
    pub(super) fn open(roots: ReferenceLineageRoots<'_>) -> ServiceResult<Self> {
        for path in [roots.before, roots.after] {
            if !path.is_absolute() || path.as_os_str().len() > 32768 {
                return Err(failure(ServiceErrorCode::InvalidRequest));
            }
        }
        let before = LiveProjectStore::open(roots.before)?;
        match LiveProjectStore::open(roots.after) {
            Ok(after) => {
                if before.retained_owner().epoch().namespace().is_some()
                    || after.retained_owner().epoch().namespace().is_some()
                {
                    let a = after.close();
                    let b = before.close();
                    return if a.is_err() || b.is_err() {
                        Err(unknown())
                    } else {
                        Err(failure(ServiceErrorCode::InvalidRequest))
                    };
                }
                Ok(Self { before, after })
            }
            Err(error) => match before.close() {
                Ok(()) => Err(error),
                Err(_) => Err(unknown()),
            },
        }
    }
    pub(super) fn close(self) -> ServiceResult<()> {
        let a = self.after.close();
        let b = self.before.close();
        if a.is_err() || b.is_err() {
            Err(unknown())
        } else {
            Ok(())
        }
    }
    pub(super) fn with_pair<T>(
        &self,
        request: &LineageBuildRequest,
        stop: &AtomicBool,
        run: impl FnOnce(&LiveProjectRead, &LiveProjectRead) -> ServiceResult<T>,
    ) -> ServiceResult<T> {
        checkpoint(stop)?;
        let before_id =
            StoreGenerationId::parse(request.before_generation.to_string()).map_err(store_error)?;
        let after_id =
            StoreGenerationId::parse(request.after_generation.to_string()).map_err(store_error)?;
        let refusal = |error: crate::live_project::LiveProjectReadFailure| match error.closure {
            crate::live_project::ReadFailureClosure::Failed => unknown(),
            _ => error.error,
        };
        let before = self
            .before
            .read_checked(&ReadSelector::Exact(before_id), stop)
            .map_err(refusal)?;
        let draft = match self
            .after
            .read_checked(&ReadSelector::Exact(after_id), stop)
        {
            Ok(after) => {
                let draft = run(&before, &after);
                match after.close() {
                    Ok(()) => draft,
                    Err(_) => Err(unknown()),
                }
            }
            Err(error) => Err(refusal(error)),
        };
        match before.close() {
            Ok(()) => draft,
            Err(_) => Err(unknown()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReferenceHoldPlan {
    schema: Box<str>,
    consumer: Box<str>,
    operation: Box<str>,
    request_digest: RequestDigest,
    before: RetentionRoot,
    after: RetentionRoot,
}
impl ReferenceHoldPlan {
    pub(super) fn prepare(
        sources: &ReferenceSources,
        consumer: &StoreInstanceId,
        request: &LineageBuildRequest,
        digest: &RequestDigest,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        checkpoint(stop)?;
        let make = |role: &str,
                    source: &LiveProjectStore,
                    generation: &str|
         -> ServiceResult<RetentionRoot> {
            let generation =
                StoreGenerationId::parse(generation.to_owned()).map_err(store_error)?;
            let hash = crate::identity::canonical_digest(
                "reference-lineage-source-hold:sha256:",
                &(
                    consumer,
                    &request.operation_id,
                    digest,
                    role,
                    source.retained_owner().epoch().epoch_id(),
                    &generation,
                ),
            )?;
            RetentionRoot::new(
                source.retained_owner().epoch().epoch_id().clone(),
                RetentionRootId::new(hash).map_err(crate::live_project::store_error)?,
                RetentionRootKind::Policy,
                generation,
                "wow-graph.lineage-reference-callable-key",
            )
            .map_err(crate::live_project::store_error)
        };
        Ok(Self {
            schema: "wow-service/reference-lineage-source-holds/1".into(),
            consumer: consumer.as_str().into(),
            operation: request.operation_id.clone(),
            request_digest: digest.clone(),
            before: make("before", &sources.before, &request.before_generation)?,
            after: make("after", &sources.after, &request.after_generation)?,
        })
    }
    fn validate(
        &self,
        sources: &ReferenceSources,
        consumer: &StoreInstanceId,
        request: &LineageBuildRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<()> {
        if self != &Self::prepare(sources, consumer, request, &self.request_digest, stop)? {
            return Err(failure(ServiceErrorCode::IdentityMismatch));
        }
        Ok(())
    }
    pub(super) fn install(
        &self,
        sources: &mut ReferenceSources,
        consumer: &StoreInstanceId,
        request: &LineageBuildRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<()> {
        self.validate(sources, consumer, request, stop)?;
        for (root, store) in [
            (&self.before, &mut sources.before),
            (&self.after, &mut sources.after),
        ] {
            checkpoint(stop)?;
            if store
                .retained_owner_mut()
                .put_retention_root(root, stop)
                .map_err(crate::live_project::store_error)?
                != *root
            {
                return Err(unknown());
            }
        }
        self.verify(sources, consumer, request, stop)
    }
    pub(super) fn verify(
        &self,
        sources: &ReferenceSources,
        consumer: &StoreInstanceId,
        request: &LineageBuildRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<()> {
        self.validate(sources, consumer, request, stop)?;
        for (root, store) in [
            (&self.before, &sources.before),
            (&self.after, &sources.after),
        ] {
            checkpoint(stop)?;
            if store
                .retained_owner()
                .retention_roots(stop)
                .map_err(crate::live_project::store_error)?
                .iter()
                .find(|r| r.root_id() == root.root_id())
                != Some(root)
            {
                return Err(failure(ServiceErrorCode::OperationReleased));
            }
        }
        Ok(())
    }
    pub(super) fn release(
        &self,
        sources: &mut ReferenceSources,
        consumer: &StoreInstanceId,
        request: &LineageBuildRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<()> {
        self.validate(sources, consumer, request, stop)?;
        for (root, store) in [
            (&self.after, &mut sources.after),
            (&self.before, &mut sources.before),
        ] {
            checkpoint(stop)?;
            store
                .retained_owner_mut()
                .remove_retention_root(root.root_id(), root.pin_digest(), stop)
                .map_err(crate::live_project::store_error)?;
        }
        Ok(())
    }
}
