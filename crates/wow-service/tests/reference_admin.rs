use std::error::Error;

use wow_service::reference_admin::{
    PublishReferenceRequest, ReferenceAdminConfiguration, ReferenceAdminErrorCode,
    ReferenceAdminExpectation, ReferenceAdminOperationId, ReferenceAdminPublicationKey,
    ReferenceAdminService, ReferenceAdminStoreLimits,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn configuration()
-> Result<ReferenceAdminConfiguration, wow_service::reference_admin::ReferenceAdminError> {
    ReferenceAdminConfiguration::new(
        "reference-admin-test",
        ReferenceAdminStoreLimits::default(),
        1024 * 1024,
    )
}

#[test]
fn empty_store_status_is_explicit_and_deterministic() -> TestResult {
    let key = ReferenceAdminPublicationKey::new("retail-12.1", "stable")?;
    let mut left = ReferenceAdminService::open_in_memory(configuration()?)?;
    let mut right = ReferenceAdminService::open_in_memory(configuration()?)?;
    let left_status = left.status(&key, 100)?;
    let right_status = right.status(&key, 100)?;
    assert!(left_status.current_reference_object_id().is_none());
    assert!(left_status.integrity_complete());
    assert_eq!(
        serde_json::to_vec(&left_status)?,
        serde_json::to_vec(&right_status)?
    );
    assert_eq!(
        left_status.logical_manifest_id(),
        right_status.logical_manifest_id()
    );
    Ok(())
}

#[test]
fn malformed_view_fails_before_any_publication() -> TestResult {
    let key = ReferenceAdminPublicationKey::new("retail-12.1", "stable")?;
    let mut service = ReferenceAdminService::open_in_memory(configuration()?)?;
    let request = PublishReferenceRequest::new(
        ReferenceAdminOperationId::new("reference-publish:test:1")?,
        key.clone(),
        ReferenceAdminExpectation::Absent,
        br#"{"not":"a-reference-view"}"#.to_vec(),
    );
    let error = service
        .publish(request)
        .err()
        .ok_or("expected invalid view")?;
    assert_eq!(error.code(), ReferenceAdminErrorCode::ReferenceViewInvalid);
    assert!(
        service
            .status(&key, 100)?
            .current_reference_object_id()
            .is_none()
    );
    Ok(())
}

#[test]
fn input_budget_is_checked_before_deserialization() -> TestResult {
    let configuration = ReferenceAdminConfiguration::new(
        "reference-admin-small",
        ReferenceAdminStoreLimits::default(),
        8,
    )?;
    let key = ReferenceAdminPublicationKey::new("retail", "stable")?;
    let mut service = ReferenceAdminService::open_in_memory(configuration)?;
    let request = PublishReferenceRequest::new(
        ReferenceAdminOperationId::new("reference-publish:test:2")?,
        key,
        ReferenceAdminExpectation::Absent,
        vec![b'x'; 9],
    );
    assert_eq!(
        service
            .publish(request)
            .err()
            .ok_or("expected input limit")?
            .code(),
        ReferenceAdminErrorCode::InputTooLarge
    );
    Ok(())
}
