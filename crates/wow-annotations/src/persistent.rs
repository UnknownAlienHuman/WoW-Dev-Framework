//! Compatibility exports for the former annotation persistence module.
//!
//! Durable storage moved to `wow-service::annotation_admin`. This module retains
//! only store-neutral artifact identities and selectors so existing artifact
//! callers do not need an immediate import-path migration.

pub use crate::artifact::{
    ANNOTATION_ARTIFACT_SCHEMA, AnnotationArtifact, AnnotationArtifactError,
    AnnotationArtifactErrorCode, AnnotationArtifactResult, AnnotationPublicationKey,
};

pub type AnnotationStoreError = AnnotationArtifactError;
pub type AnnotationStoreErrorCode = AnnotationArtifactErrorCode;
pub type AnnotationStoreResult<T> = AnnotationArtifactResult<T>;
