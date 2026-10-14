use std::error::Error;

use serde_json::json;

use super::{OperationCommitBoundary, Store};
use crate::{
    CatalogExpectation, CatalogMutation, CatalogName, CatalogPath, OperationBegin, OperationId,
    OperationState, PendingObject, RequestDigest, StoreConfiguration, StoreError, StoreErrorCode,
    StoreLimits, WriteBatch,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn pending(kind: &str, value: serde_json::Value) -> Result<PendingObject, StoreError> {
    PendingObject::from_json(kind, 1, &value, StoreLimits::default())
}

#[test]
fn operation_batch_and_completion_share_one_transaction() -> TestResult {
    let mut store = Store::open_in_memory(StoreConfiguration::new(
        "atomic-operation-fixture",
        StoreLimits::default(),
    )?)?;
    let token_catalog = CatalogName::new("context.continuation")?;
    let token_path = CatalogPath::new("active-token")?;
    let receipt_catalog = CatalogName::new("context.operation")?;
    let receipt_path = CatalogPath::new("continue-1")?;

    let old_token = pending("context.continuation-token", json!({"cursor":"old"}))?;
    let old_token_id = old_token.object_id().clone();
    let mut seed = WriteBatch::new();
    seed.add_object(old_token)?;
    seed.add_catalog_mutation(CatalogMutation::set(
        token_catalog.clone(),
        token_path.clone(),
        CatalogExpectation::Absent,
        old_token_id.clone(),
    ))?;
    store.commit(seed)?;

    let operation_id = OperationId::new("operation:context:continue:1")?;
    let request_digest = RequestDigest::new(format!("sha256:{}", "a".repeat(64)))?;
    let OperationBegin::Started(prepared) =
        store.begin_operation(operation_id.clone(), request_digest.clone())?
    else {
        return Err("first issuer registration was not prepared".into());
    };
    assert_eq!(prepared.state(), OperationState::Prepared);

    let next_token = pending("context.continuation-token", json!({"cursor":"next"}))?;
    let next_token_id = next_token.object_id().clone();
    let result = pending("context.continuation-result", json!({"status":"continued"}))?;
    let result_object_id = result.object_id().clone();
    let mut batch = WriteBatch::new();
    batch.add_object(next_token)?;
    batch.add_object(result)?;
    batch.add_catalog_mutation(CatalogMutation::set(
        token_catalog.clone(),
        token_path.clone(),
        CatalogExpectation::Exact(old_token_id.clone()),
        next_token_id.clone(),
    ))?;
    batch.add_catalog_mutation(CatalogMutation::set(
        receipt_catalog.clone(),
        receipt_path.clone(),
        CatalogExpectation::Absent,
        result_object_id.clone(),
    ))?;

    let before_commit = store.commit_operation_with_boundary(
        batch.clone(),
        &operation_id,
        &request_digest,
        &result_object_id,
        |boundary| match boundary {
            OperationCommitBoundary::BeforeCommit => Err(StoreError::new(
                StoreErrorCode::OutcomeUnknown,
                "injected response loss before transaction commit",
            )),
            OperationCommitBoundary::AfterCommit => Ok(()),
        },
    );
    assert_eq!(
        before_commit
            .err()
            .ok_or("expected injected precommit failure")?
            .code(),
        StoreErrorCode::OutcomeUnknown
    );
    assert_eq!(
        store
            .operation(&operation_id)?
            .ok_or("prepared operation missing after rollback")?
            .state(),
        OperationState::Prepared
    );
    assert_eq!(
        store
            .catalog_entry(&token_catalog, &token_path)?
            .ok_or("old token catalog entry missing after rollback")?
            .object_id(),
        &old_token_id
    );
    assert!(store.object(&next_token_id)?.is_none());
    assert!(
        store
            .catalog_entry(&receipt_catalog, &receipt_path)?
            .is_none()
    );

    let after_commit = store.commit_operation_with_boundary(
        batch.clone(),
        &operation_id,
        &request_digest,
        &result_object_id,
        |boundary| match boundary {
            OperationCommitBoundary::BeforeCommit => Ok(()),
            OperationCommitBoundary::AfterCommit => Err(StoreError::new(
                StoreErrorCode::OutcomeUnknown,
                "injected response loss after transaction commit",
            )),
        },
    );
    assert_eq!(
        after_commit
            .err()
            .ok_or("expected injected postcommit response loss")?
            .code(),
        StoreErrorCode::OutcomeUnknown
    );

    let completed = store
        .operation(&operation_id)?
        .ok_or("completed operation missing after postcommit response loss")?;
    assert_eq!(completed.state(), OperationState::Completed);
    assert_eq!(completed.result_object_id(), Some(&result_object_id));
    assert_eq!(
        store
            .catalog_entry(&token_catalog, &token_path)?
            .ok_or("new token catalog entry missing after commit")?
            .object_id(),
        &next_token_id
    );
    assert_eq!(
        store
            .catalog_entry(&receipt_catalog, &receipt_path)?
            .ok_or("receipt catalog entry missing after commit")?
            .object_id(),
        &result_object_id
    );
    assert!(store.object(&next_token_id)?.is_some());
    assert!(store.object(&result_object_id)?.is_some());

    assert_eq!(
        store.commit_operation(batch, &operation_id, &request_digest, &result_object_id)?,
        completed,
        "exact retry must return the durable record without replaying catalog CAS"
    );
    assert_eq!(
        store
            .catalog_entry(&token_catalog, &token_path)?
            .ok_or("token catalog entry missing after exact retry")?
            .object_id(),
        &next_token_id
    );
    Ok(())
}
