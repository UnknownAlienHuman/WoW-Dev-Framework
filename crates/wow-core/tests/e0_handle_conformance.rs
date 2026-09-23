//! E0-A source-handle construction, retained validation and digest verification.

#[path = "handle/support.rs"]
mod support;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{TestResult, assert_error, builder, field, fixture, rebuild, reseal};
use wow_core::{
    ContentDigest, CoreErrorCode, E0CheckResultEnvelope, E0DecodeLimits, SourceContent,
    SourceHandle, SourceSpan, canonical_json_bytes, verify_source_handle_content,
};

#[test]
fn handle_001_013_020_rebuilds_the_exact_committed_identity() -> TestResult {
    let value = fixture()?;
    let handle = rebuild(&value)?;
    handle.validate()?;
    assert_eq!(
        handle.handle_id().to_string(),
        value["handle_id"].as_str().ok_or("id")?
    );
    assert_eq!(
        canonical_json_bytes(&handle)?,
        canonical_json_bytes(&value)?
    );
    let decoded: SourceHandle = serde_json::from_slice(&serde_json::to_vec(&handle)?)?;
    assert_eq!(decoded, handle);
    let expected = canonical_json_bytes(&handle)?;
    let entries = value
        .as_object()
        .ok_or("object")?
        .iter()
        .collect::<Vec<_>>();
    let mut seed = 0x4841_4e44_4c45_u64;
    for _ in 0..64 {
        let recorded_seed = seed;
        let mut order = (0..entries.len()).collect::<Vec<_>>();
        for index in (1..order.len()).rev() {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let other = (seed % (index as u64 + 1)) as usize;
            order.swap(index, other);
        }
        // Construct wire text directly: Value's map would already sort the keys.
        let members = order
            .into_iter()
            .map(|index| {
                let (key, value) = entries[index];
                Ok(format!(
                    "{}:{}",
                    serde_json::to_string(key)?,
                    serde_json::to_string(value)?
                ))
            })
            .collect::<Result<Vec<_>, serde_json::Error>>()?;
        let wire = format!("{{{}}}", members.join(","));
        let decoded: SourceHandle = serde_json::from_str(&wire)?;
        decoded.validate()?;
        assert_eq!(
            canonical_json_bytes(&decoded)?,
            expected,
            "seed {recorded_seed}"
        );
        assert_eq!(
            rebuild(&serde_json::from_str(&wire)?)?,
            handle,
            "seed {recorded_seed}"
        );
    }
    Ok(())
}

#[test]
fn handle_002_origin_generation_matrix_matches_on_both_entrypaths() -> TestResult {
    let original = fixture()?;
    for origin in [
        "repository",
        "reference_pack",
        "generated_artifact",
        "fixture",
    ] {
        for reference in [false, true] {
            for project in [false, true] {
                let mut value = original.clone();
                value["origin_kind"] = origin.into();
                if !reference {
                    value
                        .as_object_mut()
                        .ok_or("object")?
                        .remove("reference_generation");
                }
                if !project {
                    value
                        .as_object_mut()
                        .ok_or("object")?
                        .remove("project_generation");
                }
                let accepted = match origin {
                    "repository" => !reference && !project,
                    "reference_pack" => reference && !project,
                    "generated_artifact" => reference || project,
                    _ => true,
                };
                let decoded = reseal(value.clone())?;
                if accepted {
                    let constructed = rebuild(&value)?;
                    decoded.validate()?;
                    assert_eq!(constructed, decoded, "{origin}/{reference}/{project}");
                    verify_source_handle_content(&decoded, decoded.content_digest())?;
                } else {
                    assert_error(
                        builder(&value)?.build(),
                        CoreErrorCode::InvalidSourceHandle,
                        "origin_kind",
                    )?;
                    assert_error(
                        decoded.validate(),
                        CoreErrorCode::InvalidSourceHandle,
                        "origin_kind",
                    )?;
                    assert_error(
                        verify_source_handle_content(&decoded, decoded.content_digest()),
                        CoreErrorCode::InvalidSourceHandle,
                        "origin_kind",
                    )?;
                }
            }
        }
    }
    Ok(())
}

#[test]
fn handle_003_rejects_floating_revisions_after_resealing() -> TestResult {
    for revision in [
        "main", "master", "current", "latest", "live", "head", "default", "auto", "MAIN", "HeAd",
    ] {
        let mut value = fixture()?;
        value["revision"] = revision.into();
        assert_error(
            builder(&value)?.build(),
            CoreErrorCode::InvalidSourceHandle,
            "revision",
        )?;
        let decoded = reseal(value)?;
        assert_error(
            decoded.validate(),
            CoreErrorCode::InvalidSourceHandle,
            "revision",
        )?;
        assert_error(
            verify_source_handle_content(&decoded, decoded.content_digest()),
            CoreErrorCode::InvalidSourceHandle,
            "revision",
        )?;
    }
    Ok(())
}

#[test]
fn handle_004_005_006_011_rejects_invalid_path_span_digest_and_host_fields() -> TestResult {
    for (path, code) in [
        ("../PRIVATE_MARKER.lua", CoreErrorCode::PathEscape),
        (
            "./C:/PRIVATE_MARKER.lua",
            CoreErrorCode::AbsolutePathForbidden,
        ),
        ("/PRIVATE_MARKER.lua", CoreErrorCode::AbsolutePathForbidden),
    ] {
        let mut value = fixture()?;
        value["path"] = path.into();
        assert_error(builder(&value)?.build(), code, "candidate")?;
        assert!(serde_json::from_value::<SourceHandle>(value).is_err());
    }
    let mut value = fixture()?;
    value["span"] = json!({"kind":"byte_range","byte_start":9,"byte_end":8});
    assert_error(
        builder(&value)?.build(),
        CoreErrorCode::InvalidSourceSpan,
        "span",
    )?;
    assert_error(
        reseal(value)?.validate(),
        CoreErrorCode::InvalidSourceSpan,
        "span",
    )?;
    for digest in ["sha256:00", "sha1:00", "PRIVATE_MARKER"] {
        let mut value = fixture()?;
        value["content_digest"] = digest.into();
        let error = match digest.parse::<ContentDigest<SourceContent>>() {
            Ok(_) => return Err("invalid digest must not be admitted".into()),
            Err(error) => error,
        };
        assert!(matches!(
            error.code(),
            CoreErrorCode::InvalidDigest | CoreErrorCode::UnsupportedDigestAlgorithm
        ));
        assert!(serde_json::from_value::<SourceHandle>(value).is_err());
    }
    for extra in ["host_root", "checkout_path", "line", "column", "excerpt"] {
        let mut value = fixture()?;
        value[extra] = "PRIVATE_MARKER".into();
        assert!(
            serde_json::from_value::<SourceHandle>(value).is_err(),
            "{extra}"
        );
    }
    Ok(())
}

#[test]
fn handle_012_bounded_origin_and_revision_never_echo_unsafe_text() -> TestResult {
    for key in ["origin_id", "revision"] {
        for invalid in [
            String::new(),
            " PRIVATE_MARKER".into(),
            "PRIVATE_MARKER\n".into(),
            "https://PRIVATE_MARKER@host.invalid/source".into(),
            "a".repeat(1025),
        ] {
            let mut value = fixture()?;
            value[key] = invalid.into();
            assert_error(
                builder(&value)?.build(),
                CoreErrorCode::InvalidSourceHandle,
                key,
            )?;
            assert_error(
                reseal(value)?.validate(),
                CoreErrorCode::InvalidSourceHandle,
                key,
            )?;
        }
        let mut value = fixture()?;
        value[key] = "a".repeat(1024).into();
        let handle = rebuild(&value)?;
        assert_eq!(serde_json::to_value(&handle)?[key], value[key]);
        handle.validate()?;
    }
    Ok(())
}

#[test]
fn handle_008_009_010_018_every_identity_field_is_bound_without_entity_resolution() -> TestResult {
    let original = fixture()?;
    let handle = rebuild(&original)?;
    for (key, changed) in [
        ("origin_kind", json!("generated_artifact")),
        ("origin_id", json!("fixture:other-origin")),
        ("revision", json!("fixture:e0-rev2")),
        ("path", json!("Other/Core.lua")),
        (
            "content_digest",
            json!(format!("sha256:{}", "03".repeat(32))),
        ),
        (
            "entity_key",
            json!("entity:api:C_NeverResolved.Unavailable"),
        ),
        (
            "reference_generation",
            json!(format!("generation:reference:sha256:{}", "05".repeat(32))),
        ),
        (
            "project_generation",
            json!(format!("generation:project:sha256:{}", "06".repeat(32))),
        ),
        ("span", json!({"kind":"unknown"})),
    ] {
        let mut value = original.clone();
        value[key] = changed;
        let changed = rebuild(&value)?;
        changed.validate()?;
        assert_ne!(changed.handle_id(), handle.handle_id(), "{key}");
        let decoded: SourceHandle = serde_json::from_value(value)?;
        assert_error(
            decoded.validate(),
            CoreErrorCode::CanonicalDigestMismatch,
            "handle_id",
        )?;
        assert_error(
            verify_source_handle_content(&decoded, decoded.content_digest()),
            CoreErrorCode::CanonicalDigestMismatch,
            "handle_id",
        )?;
    }
    Ok(())
}

#[test]
fn handle_verify_001_002_and_handle_014_015_match_only_the_bound_content_digest() -> TestResult {
    let handle = rebuild(&fixture()?)?;
    verify_source_handle_content(&handle, handle.content_digest())?;
    assert_error(
        verify_source_handle_content(&handle, &ContentDigest::from_bytes([0; 32])),
        CoreErrorCode::DigestMismatch,
        "digest",
    )?;
    Ok(())
}

#[test]
fn handle_verify_003_rejects_a_forged_id_even_with_matching_content() -> TestResult {
    let mut value = fixture()?;
    value["handle_id"] = format!("handle:sha256:{}", "00".repeat(32)).into();
    let handle: SourceHandle = serde_json::from_value(value)?;
    assert_error(
        verify_source_handle_content(&handle, handle.content_digest()),
        CoreErrorCode::CanonicalDigestMismatch,
        "handle_id",
    )
}

#[test]
fn handle_verify_004_rejects_a_resealed_invalid_origin_generation_matrix() -> TestResult {
    let mut value = fixture()?;
    value["origin_kind"] = "repository".into();
    let handle = reseal(value)?;
    assert_error(
        verify_source_handle_content(&handle, handle.content_digest()),
        CoreErrorCode::InvalidSourceHandle,
        "origin_kind",
    )
}

#[test]
fn handle_verify_005_rejects_a_resealed_invalid_span_before_a_content_mismatch() -> TestResult {
    let mut value = fixture()?;
    value["span"] = json!({"kind":"whole_file","byte_start":0,"byte_end":1});
    let handle = reseal(value)?;
    for digest in [*handle.content_digest(), ContentDigest::from_bytes([0; 32])] {
        assert_error(
            verify_source_handle_content(&handle, &digest),
            CoreErrorCode::SpanStateConflict,
            "span",
        )?;
    }
    Ok(())
}

#[test]
fn handle_verify_006_uses_the_whole_artifact_digest_not_the_selected_span() -> TestResult {
    let text = "α\nlocal привет = 1\n";
    let start = text.find("привет").ok_or("token")?;
    let end = start + "привет".len();
    assert!(end - start > "привет".chars().count(), "SPAN-015");
    let digest = ContentDigest::<SourceContent>::from_bytes(Sha256::digest(text.as_bytes()).into());
    let mut value = fixture()?;
    value["content_digest"] = serde_json::to_value(digest)?;
    value["span"] = serde_json::to_value(SourceSpan::byte_range(start as u64, end as u64)?)?;
    let handle = rebuild(&value)?;
    assert_eq!(handle.span().byte_start(), Some(start as u64));
    assert_eq!(handle.span().byte_end(), Some(end as u64));
    verify_source_handle_content(&handle, &digest)?;
    let span_digest = ContentDigest::<SourceContent>::from_bytes(
        Sha256::digest(&text.as_bytes()[start..end]).into(),
    );
    assert_error(
        verify_source_handle_content(&handle, &span_digest),
        CoreErrorCode::DigestMismatch,
        "digest",
    )?;
    Ok(())
}

#[test]
fn handle_007_and_span_016_presentation_is_not_canonical_identity() -> TestResult {
    let handle = rebuild(&fixture()?)?;
    let before = canonical_json_bytes(&handle)?;
    // A transport projection owns hints; the pure handle does not store them.
    let mut projection = json!({"source":handle,"line":2,"column":8});
    projection["line"] = 200.into();
    projection["column"] = 300.into();
    let retained: SourceHandle = field(&projection, "source")?;
    retained.validate()?;
    assert_eq!(canonical_json_bytes(&retained)?, before);
    assert_eq!(retained.handle_id(), handle.handle_id());
    Ok(())
}

#[test]
fn handle_019_duplicate_registry_is_rejected_before_digest_validation() -> TestResult {
    let mut value: Value =
        serde_json::from_str(include_str!("../examples/e0-findings-result.json"))?;
    let handles = value["source_handles"].as_array_mut().ok_or("handles")?;
    let mut duplicate = handles.first().ok_or("handle")?.clone();
    duplicate["revision"] = "fixture:contradictory-copy".into();
    handles.push(duplicate);
    let envelope: E0CheckResultEnvelope = serde_json::from_value(value)?;
    assert_error(
        envelope.validate(),
        CoreErrorCode::ResultDuplicateId,
        "source_handles",
    )
}

#[test]
fn handle_golden_consumers_keep_the_full_original_bytes_and_digests() -> TestResult {
    let limits = E0DecodeLimits::new(1024 * 1024, 64, 100_000, 64 * 1024)?;
    for golden in [
        include_str!("../examples/e0-clean-result.json"),
        include_str!("../examples/e0-findings-result.json"),
        include_str!("../examples/e0-not-evaluated-result.json"),
        include_str!("../examples/e0-conflict-not-evaluated-result.json"),
    ] {
        let expected: Value = serde_json::from_str(golden)?;
        let envelope = E0CheckResultEnvelope::from_json_slice(golden.as_bytes(), limits)?;
        assert_eq!(
            envelope.canonical_bytes()?,
            canonical_json_bytes(&expected)?
        );
        for value in expected["source_handles"].as_array().ok_or("sources")? {
            let handle = rebuild(value)?;
            assert_eq!(canonical_json_bytes(&handle)?, canonical_json_bytes(value)?);
            verify_source_handle_content(&handle, handle.content_digest())?;
        }
    }
    Ok(())
}
