use std::error::Error;

use serde_json::json;

use super::{OperationCommitBoundary, Store};
use crate::{
    CatalogExpectation, CatalogMutation, CatalogName, CatalogPath, OperationBegin, OperationId,
    OperationResultMutation, OperationState, PendingObject, RequestDigest, StoreConfiguration,
    StoreError, StoreErrorCode, StoreLimits, WriteBatch,
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

#[test]
fn completed_result_retarget_and_catalog_prune_share_one_transaction() -> TestResult {
    let mut store = Store::open_in_memory(StoreConfiguration::new(
        "atomic-result-retarget-fixture",
        StoreLimits::default(),
    )?)?;
    let history_catalog = CatalogName::new("context.history")?;
    let history_path = CatalogPath::new("history:one")?;
    let old_history = pending("context.history", json!({"state":"terminal"}))?;
    let old_history_id = old_history.object_id().clone();
    let old_result = pending("context.outcome", json!({"response":"old"}))?;
    let old_result_id = old_result.object_id().clone();
    let mut seed = WriteBatch::new();
    seed.add_object(old_history)?;
    seed.add_object(old_result)?;
    seed.add_catalog_mutation(CatalogMutation::set(
        history_catalog.clone(),
        history_path.clone(),
        CatalogExpectation::Absent,
        old_history_id.clone(),
    ))?;
    store.commit(seed)?;

    let issued_operation_id = OperationId::new("operation:context:issued:1")?;
    let issued_digest = RequestDigest::new(format!("sha256:{}", "b".repeat(64)))?;
    assert!(matches!(
        store.begin_operation(issued_operation_id.clone(), issued_digest.clone())?,
        OperationBegin::Started(_)
    ));
    store.commit_operation(
        WriteBatch::new(),
        &issued_operation_id,
        &issued_digest,
        &old_result_id,
    )?;

    let prune_operation_id = OperationId::new("operation:context:prune:one")?;
    let prune_digest = RequestDigest::new(format!("sha256:{}", "c".repeat(64)))?;
    assert!(matches!(
        store.begin_operation(prune_operation_id.clone(), prune_digest.clone())?,
        OperationBegin::Started(_)
    ));
    let marker = pending("context.history-pruned", json!({"state":"pruned"}))?;
    let marker_id = marker.object_id().clone();
    let mut batch = WriteBatch::new();
    batch.add_object(marker)?;
    batch.add_catalog_mutation(CatalogMutation::set(
        history_catalog.clone(),
        history_path.clone(),
        CatalogExpectation::Exact(old_history_id.clone()),
        marker_id.clone(),
    ))?;
    batch.add_operation_result_mutation(OperationResultMutation::replace_completed(
        issued_operation_id.clone(),
        issued_digest.clone(),
        old_result_id.clone(),
        marker_id.clone(),
    )?)?;
    assert_eq!(
        store
            .commit(batch.clone())
            .err()
            .ok_or("operation result retarget must require commit_operation")?
            .code(),
        StoreErrorCode::OperationStateInvalid
    );

    let before_commit = store.commit_operation_with_boundary(
        batch.clone(),
        &prune_operation_id,
        &prune_digest,
        &marker_id,
        |boundary| match boundary {
            OperationCommitBoundary::BeforeCommit => Err(StoreError::new(
                StoreErrorCode::OutcomeUnknown,
                "injected prune response loss before transaction commit",
            )),
            OperationCommitBoundary::AfterCommit => Ok(()),
        },
    );
    assert_eq!(
        before_commit
            .err()
            .ok_or("expected injected precommit prune failure")?
            .code(),
        StoreErrorCode::OutcomeUnknown
    );
    assert_eq!(
        store
            .operation(&issued_operation_id)?
            .ok_or("issued operation after prune rollback")?
            .result_object_id(),
        Some(&old_result_id)
    );
    assert_eq!(
        store
            .catalog_entry(&history_catalog, &history_path)?
            .ok_or("history catalog after prune rollback")?
            .object_id(),
        &old_history_id
    );
    assert!(store.object(&marker_id)?.is_none());
    assert_eq!(
        store
            .operation(&prune_operation_id)?
            .ok_or("prepared prune operation")?
            .state(),
        OperationState::Prepared
    );

    let after_commit = store.commit_operation_with_boundary(
        batch.clone(),
        &prune_operation_id,
        &prune_digest,
        &marker_id,
        |boundary| match boundary {
            OperationCommitBoundary::BeforeCommit => Ok(()),
            OperationCommitBoundary::AfterCommit => Err(StoreError::new(
                StoreErrorCode::OutcomeUnknown,
                "injected prune response loss after transaction commit",
            )),
        },
    );
    assert_eq!(
        after_commit
            .err()
            .ok_or("expected injected postcommit prune response loss")?
            .code(),
        StoreErrorCode::OutcomeUnknown
    );
    assert_eq!(
        store
            .operation(&issued_operation_id)?
            .ok_or("issued operation after prune commit")?
            .result_object_id(),
        Some(&marker_id)
    );
    assert_eq!(
        store
            .catalog_entry(&history_catalog, &history_path)?
            .ok_or("history catalog after prune commit")?
            .object_id(),
        &marker_id
    );

    let collected = store.collect_garbage(crate::LogicalEpoch::new(0), 8)?;
    assert!(collected.deleted().contains(&old_result_id));
    assert!(collected.deleted().contains(&old_history_id));
    assert!(store.object(&marker_id)?.is_some());
    assert_eq!(
        store.commit_operation(batch, &prune_operation_id, &prune_digest, &marker_id)?,
        store
            .operation(&prune_operation_id)?
            .ok_or("exact prune retry result")?,
        "completed prune retry must not replay result retargets"
    );
    assert_eq!(
        store
            .operation(&issued_operation_id)?
            .ok_or("issued tombstone after exact retry")?
            .result_object_id(),
        Some(&marker_id)
    );
    Ok(())
}
