//! Independent original-generation policy roots. Intent must be durably
//! registered by the caller before install/release can be dispatched.
use crate::{OperationId, ServiceError, ServiceErrorCode, ServiceResult};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_store::project::{
    EpochId, ProjectStore, RetentionRoot, RetentionRootId, RetentionRootKind, StoreGenerationId,
};
use wow_store::{RequestDigest, StoreInstanceId};
const HOLDER: &str = "wow-graph.lineage-project-span";
const SCHEMA: &str = "wow-service/lineage-project-source-holds/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceHoldPlan {
    schema: Box<str>,
    source_epoch_id: EpochId,
    // Persist only a comparison binding. Store mints the actual opaque identity;
    // decoded intent must be compared with the currently opened consumer.
    consumer_store_instance_id: Box<str>,
    operation_id: Box<str>,
    request_digest: RequestDigest,
    before: RetentionRoot,
    after: RetentionRoot,
}
impl SourceHoldPlan {
    pub(super) fn prepare(
        source: &ProjectStore,
        consumer: &StoreInstanceId,
        operation: &OperationId,
        digest: &RequestDigest,
        before: &StoreGenerationId,
        after: &StoreGenerationId,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        checkpoint(stop)?;
        if source.epoch().namespace().is_some() {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        let epoch = source.epoch().epoch_id();
        let root = |role: &str, generation: &StoreGenerationId| -> ServiceResult<RetentionRoot> {
            let hash = wow_core::domain_separated_digest(
                "wow-service/lineage-source-generation-hold/1",
                &(
                    consumer,
                    operation.as_str(),
                    digest,
                    epoch,
                    role,
                    generation,
                ),
            )
            .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
            let root_id = RetentionRootId::new(format!(
                "wow-service.lineage-source-hold:{}",
                wow_core::ContentDigest::<wow_core::CanonicalResult>::from_bytes(hash)
            ))
            .map_err(crate::live_project::store_error)?;
            RetentionRoot::new(
                epoch.clone(),
                root_id,
                RetentionRootKind::Policy,
                generation.clone(),
                HOLDER,
            )
            .map_err(crate::live_project::store_error)
        };
        Ok(Self {
            schema: SCHEMA.into(),
            source_epoch_id: epoch.clone(),
            consumer_store_instance_id: consumer.as_str().into(),
            operation_id: operation.as_str().into(),
            request_digest: digest.clone(),
            before: root("before", before)?,
            after: root("after", after)?,
        })
    }
    pub(super) fn validate(
        &self,
        source: &ProjectStore,
        consumer: &StoreInstanceId,
        stop: &AtomicBool,
    ) -> ServiceResult<()> {
        if self.consumer_store_instance_id.as_ref() != consumer.as_str() {
            return Err(failure(ServiceErrorCode::IdentityMismatch));
        }
        let operation = OperationId::new(self.operation_id.clone())?;
        let expected = Self::prepare(
            source,
            consumer,
            &operation,
            &self.request_digest,
            self.before.generation_id(),
            self.after.generation_id(),
            stop,
        )?;
        if self != &expected {
            return Err(failure(ServiceErrorCode::IdentityMismatch));
        }
        Ok(())
    }
    pub(super) fn install(
        &self,
        source: &mut ProjectStore,
        consumer: &StoreInstanceId,
        stop: &AtomicBool,
    ) -> ServiceResult<()> {
        self.validate(source, consumer, stop)?;
        for root in [&self.before, &self.after] {
            checkpoint(stop)?;
            let actual = source
                .put_retention_root(root, stop)
                .map_err(crate::live_project::store_error)?;
            if &actual != root {
                return Err(failure(ServiceErrorCode::StoreOutcomeUnknown));
            }
        }
        self.verify(source, consumer, stop)
    }
    pub(super) fn verify(
        &self,
        source: &ProjectStore,
        consumer: &StoreInstanceId,
        stop: &AtomicBool,
    ) -> ServiceResult<()> {
        self.validate(source, consumer, stop)?;
        let roots = source
            .retention_roots(stop)
            .map_err(crate::live_project::store_error)?;
        for expected in [&self.before, &self.after] {
            checkpoint(stop)?;
            if roots.iter().find(|r| r.root_id() == expected.root_id()) != Some(expected) {
                return Err(failure(ServiceErrorCode::OperationReleased));
            }
        }
        Ok(())
    }
    pub(super) fn release(
        &self,
        source: &mut ProjectStore,
        consumer: &StoreInstanceId,
        stop: &AtomicBool,
    ) -> ServiceResult<()> {
        self.validate(source, consumer, stop)?;
        // Reverse roles; missing exact roots are an idempotent release, never a
        // permission to remove a different holder or digest under the same ID.
        for root in [&self.after, &self.before] {
            checkpoint(stop)?;
            source
                .remove_retention_root(root.root_id(), root.pin_digest(), stop)
                .map_err(crate::live_project::store_error)?;
        }
        Ok(())
    }
    pub(super) fn source_epoch_id(&self) -> &EpochId {
        &self.source_epoch_id
    }
    pub(super) fn before_generation(&self) -> &StoreGenerationId {
        self.before.generation_id()
    }
    pub(super) fn after_generation(&self) -> &StoreGenerationId {
        self.after.generation_id()
    }
}
fn checkpoint(stop: &AtomicBool) -> ServiceResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(failure(ServiceErrorCode::Cancelled))
    } else {
        Ok(())
    }
}
fn failure(code: ServiceErrorCode) -> ServiceError {
    ServiceError::new(
        code,
        "lineage source retention is unavailable or inconsistent",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::live_project::LiveProjectStore;
    use std::error::Error;
    use wow_store::{Store, StoreConfiguration, StoreLimits};
    #[test]
    fn native_lineage_source_holds_are_independent_guarded_and_released()
    -> Result<(), Box<dyn Error>> {
        let stop = AtomicBool::new(false);
        let root = std::env::temp_dir().join(format!(
            "wow-lineage-source-holds-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        let (first, first_graph) = crate::live_project::tests::owners("return 1\n")?;
        let (second, second_graph) = crate::live_project::tests::owners("return 2\n")?;
        let mut live = LiveProjectStore::create(&root, first_graph.snapshot().universe().as_str())?;
        let a = live.publish(&first, &first_graph, "source-hold-a", None, &stop)?;
        let expected = live.current()?.ok_or("first Current")?.record_id;
        let b = live.publish(
            &second,
            &second_graph,
            "source-hold-b",
            Some(expected),
            &stop,
        )?;
        live.close()?;
        let consumer = Store::open_in_memory(StoreConfiguration::new_with_instance_identity(
            "wow-graph.lineage-project-span-store.v1",
            StoreLimits::default(),
        )?)?;
        let mut source = LiveProjectStore::open_retained_owner_store(&root)?;
        let op = OperationId::new("lineage-hold-test")?;
        let digest = RequestDigest::new(format!("sha256:{}", "1".repeat(64)))?;
        let plan = SourceHoldPlan::prepare(
            &source,
            consumer.store_instance_id()?,
            &op,
            &digest,
            &a.generation_id,
            &b.generation_id,
            &stop,
        )?;
        let encoded = wow_core::canonical_json_bytes(&plan)?;
        let plan: SourceHoldPlan = serde_json::from_slice(&encoded)?;
        let unrelated_consumer =
            Store::open_in_memory(StoreConfiguration::new_with_instance_identity(
                "wow-graph.lineage-project-span-store.v1",
                StoreLimits::default(),
            )?)?;
        assert_eq!(
            plan.install(&mut source, unrelated_consumer.store_instance_id()?, &stop)
                .err()
                .map(|e| e.code()),
            Some(ServiceErrorCode::IdentityMismatch)
        );
        assert!(source.retention_roots(&stop)?.is_empty());
        unrelated_consumer.close()?;
        plan.install(&mut source, consumer.store_instance_id()?, &stop)?;
        plan.install(&mut source, consumer.store_instance_id()?, &stop)?;
        assert_ne!(plan.before.root_id(), plan.after.root_id());
        assert_eq!(source.retention_roots(&stop)?.len(), 2);
        let mut altered = plan.clone();
        altered.operation_id = "different-operation".into();
        assert_eq!(
            altered
                .release(&mut source, consumer.store_instance_id()?, &stop)
                .err()
                .map(|e| e.code()),
            Some(ServiceErrorCode::IdentityMismatch)
        );
        plan.verify(&source, consumer.store_instance_id()?, &stop)?;
        let same = SourceHoldPlan::prepare(
            &source,
            consumer.store_instance_id()?,
            &OperationId::new("same-generation-holds")?,
            &digest,
            &a.generation_id,
            &a.generation_id,
            &stop,
        )?;
        assert_ne!(same.before.root_id(), same.after.root_id());
        same.install(&mut source, consumer.store_instance_id()?, &stop)?;
        assert_eq!(source.retention_roots(&stop)?.len(), 4);
        same.release(&mut source, consumer.store_instance_id()?, &stop)?;
        plan.release(&mut source, consumer.store_instance_id()?, &stop)?;
        plan.release(&mut source, consumer.store_instance_id()?, &stop)?;
        assert!(source.retention_roots(&stop)?.is_empty());
        assert_eq!(
            plan.verify(&source, consumer.store_instance_id()?, &stop)
                .err()
                .map(|e| e.code()),
            Some(ServiceErrorCode::OperationReleased)
        );
        source.close()?;
        consumer.close()?;
        std::fs::remove_dir_all(root)?;
        Ok(())
    }
}
