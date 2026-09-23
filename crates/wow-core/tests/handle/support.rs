use std::error::Error;

use serde::de::DeserializeOwned;
use serde_json::Value;
use wow_core::{
    CoreErrorCode, CoreResult, SourceHandle, SourceHandleBuilder, StableHandleId,
    build_source_handle,
};

pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;

pub fn fixture() -> TestResult<Value> {
    let vectors: Value = serde_json::from_str(include_str!("../../examples/HASH_VECTORS.json"))?;
    let vector = vectors["vectors"]
        .as_array()
        .ok_or("vectors array")?
        .iter()
        .find(|value| value["vector_id"] == "fixture-source-handle")
        .ok_or("source handle vector")?;
    let mut handle = vector["value"].clone();
    handle["handle_id"] = vector["typed_id"].clone();
    Ok(handle)
}

pub fn field<T: DeserializeOwned>(value: &Value, key: &str) -> TestResult<T> {
    Ok(serde_json::from_value(
        value.get(key).ok_or("missing field")?.clone(),
    )?)
}

pub fn builder(value: &Value) -> TestResult<SourceHandleBuilder> {
    let mut builder = SourceHandleBuilder::new(
        field(value, "origin_kind")?,
        field::<String>(value, "origin_id")?,
        field::<String>(value, "revision")?,
        field::<String>(value, "path")?,
        field(value, "span")?,
        field(value, "content_digest")?,
    );
    if value.get("reference_generation").is_some() {
        builder = builder.reference_generation(field(value, "reference_generation")?);
    }
    if value.get("project_generation").is_some() {
        builder = builder.project_generation(field(value, "project_generation")?);
    }
    if value.get("entity_key").is_some() {
        builder = builder.entity_key(field(value, "entity_key")?);
    }
    Ok(builder)
}

pub fn rebuild(value: &Value) -> TestResult<SourceHandle> {
    Ok(build_source_handle(builder(value)?)?)
}

pub fn reseal(mut value: Value) -> TestResult<SourceHandle> {
    value
        .as_object_mut()
        .ok_or("handle object")?
        .remove("handle_id");
    value["handle_id"] = StableHandleId::derive(&value)?.to_string().into();
    Ok(serde_json::from_value(value)?)
}

pub fn assert_error<T>(result: CoreResult<T>, expected: CoreErrorCode, field: &str) -> TestResult {
    let error = match result {
        Ok(_) => return Err("invalid source handle must not be admitted".into()),
        Err(error) => error,
    };
    assert_eq!(error.code(), expected);
    assert_eq!(error.field_path(), Some(field));
    error.validate()?;
    assert!(!serde_json::to_string(&error)?.contains("PRIVATE_MARKER"));
    Ok(())
}
