//! Exact context admission: constructor, decoded wire, merge and equality guard.
//! These regressions preserve the committed E0 identity/serialization fixtures.

use std::error::Error;

use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use wow_core::{
    CoreErrorCode, CoreResult, ExternalGeneration, GenerationContext, GenerationContextBuilder,
    GenerationContextId, MergeMode, canonical_json_bytes, derive_generation_context_id,
    merge_generation_context, require_same_generation, validate_generation_context,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
const MODES: [MergeMode; 3] = [
    MergeMode::Strict,
    MergeMode::ExtendMissingOptional,
    MergeMode::ExternalUnion,
];

fn fixture() -> TestResult<Value> {
    let envelope: Value = serde_json::from_str(include_str!("../examples/e0-clean-result.json"))?;
    Ok(envelope
        .get("context")
        .ok_or("missing fixture context")?
        .clone())
}

fn field<T: DeserializeOwned>(value: &Value, key: &str) -> TestResult<T> {
    Ok(serde_json::from_value(
        value
            .get(key)
            .ok_or_else(|| format!("missing {key}"))?
            .clone(),
    )?)
}

// The builder is exercised independently from wire validation. In particular,
// ExternalGeneration's derived Deserialize is not its validated constructor.
fn rebuild(value: &Value) -> TestResult<CoreResult<GenerationContext>> {
    let mut builder = GenerationContextBuilder::new(
        field(value, "profile")?,
        field(value, "reference_generation")?,
    )
    .external_generations(field(value, "external_generations")?)
    .schema_versions(field(value, "schema_versions")?)
    .producer_versions(field(value, "producer_versions")?);
    if value.get("project_generation").is_some() {
        builder = builder.project_generation(field(value, "project_generation")?);
    }
    Ok(builder.build())
}

// Recompute only the documented identity projection. Do not run a context
// constructor/validator: those are exactly the admission paths under test.
fn reseal(value: &mut Value) -> TestResult {
    let mut projection = value.clone();
    projection
        .as_object_mut()
        .ok_or("context is not an object")?
        .remove("context_id");
    value["context_id"] = GenerationContextId::derive(&projection)?.to_string().into();
    Ok(())
}

fn assert_error<T>(result: CoreResult<T>, code: CoreErrorCode, field: &str) -> TestResult {
    let error = match result {
        Err(error) => error,
        Ok(_) => return Err(format!("expected {code:?} at {field}").into()),
    };
    assert_eq!(error.code(), code);
    assert_eq!(error.field_path(), Some(field));
    error.validate()?;
    Ok(())
}

fn external(provider: &str, scope: &str, seed: u8) -> TestResult<Value> {
    Ok(serde_json::to_value(ExternalGeneration::new(
        provider,
        scope,
        format!(
            "generation:external:{provider}:sha256:{}",
            format!("{seed:02x}").repeat(32)
        )
        .parse()?,
        Some("fixture:external-revision".into()),
    )?)?)
}

#[test]
fn context_001_002_and_same_001_preserve_the_normative_identity() -> TestResult {
    let mut wire = fixture()?;
    let golden = canonical_json_bytes(&wire)?;
    reseal(&mut wire)?;
    assert_eq!(canonical_json_bytes(&wire)?, golden);
    let decoded: GenerationContext = serde_json::from_value(wire.clone())?;
    let rebuilt = rebuild(&wire)??;
    decoded.validate()?;
    assert_eq!(decoded, rebuilt);
    assert_eq!(canonical_json_bytes(&rebuilt)?, golden);
    assert_eq!(
        derive_generation_context_id(&rebuilt)?,
        decoded.context_id()
    );
    decoded.require_same_generation(&rebuilt)?;
    require_same_generation(&rebuilt, &decoded)?;
    for mode in MODES {
        assert_eq!(merge_generation_context(&decoded, &rebuilt, mode)?, rebuilt);
    }
    wire.as_object_mut()
        .ok_or("context object")?
        .remove("project_generation");
    let reference_only = rebuild(&wire)??;
    reference_only.validate()?;
    assert!(reference_only.project_generation().is_none());
    Ok(())
}

#[test]
fn context_same_002_003_004_reject_stale_ids_on_either_operand_or_both() -> TestResult {
    let base = fixture()?;
    let original: GenerationContext = serde_json::from_value(base.clone())?;
    let mut mutations = Vec::new();
    let mut value = base.clone();
    value["reference_generation"] =
        format!("generation:reference:sha256:{}", "12".repeat(32)).into();
    mutations.push(value);
    let mut value = base.clone();
    value["project_generation"] = format!("generation:project:sha256:{}", "34".repeat(32)).into();
    mutations.push(value);
    let mut value = base.clone();
    value["profile"]["fixture_scope"] = "another fixture scope".into();
    mutations.push(value);
    for key in ["schema_versions", "producer_versions"] {
        let mut value = base.clone();
        value[key].as_array_mut().ok_or("version array")?.pop();
        mutations.push(value);
    }
    let mut value = base;
    value["external_generations"] = json!([external("fixture", "repo/one", 1)?]);
    mutations.push(value);

    for value in mutations {
        let changed: GenerationContext = serde_json::from_value(value)?;
        assert_eq!(changed.context_id(), original.context_id());
        assert_ne!(changed, original);
        assert_error(
            changed.validate(),
            CoreErrorCode::GenerationMismatch,
            "context_id",
        )?;
        for (left, right) in [
            (&original, &changed),
            (&changed, &original),
            (&changed, &changed),
        ] {
            assert_error(
                left.require_same_generation(right),
                CoreErrorCode::GenerationMismatch,
                "context_id",
            )?;
            assert_error(
                require_same_generation(left, right),
                CoreErrorCode::GenerationMismatch,
                "context_id",
            )?;
        }
    }
    Ok(())
}

#[test]
fn context_same_005_valid_different_contexts_still_fail_the_strict_guard() -> TestResult {
    let base = fixture()?;
    let left = rebuild(&base)??;
    let mut value = base;
    value["project_generation"] = format!("generation:project:sha256:{}", "56".repeat(32)).into();
    let right = rebuild(&value)??;
    assert_ne!(left.context_id(), right.context_id());
    for (left, right) in [(&left, &right), (&right, &left)] {
        assert_error(
            require_same_generation(left, right),
            CoreErrorCode::GenerationMismatch,
            "context_id",
        )?;
    }
    Ok(())
}

#[test]
fn context_merge_013_014_015_forbid_missing_version_entries_in_every_mode() -> TestResult {
    let base = fixture()?;
    let complete = rebuild(&base)??;
    for key in ["schema_versions", "producer_versions"] {
        let entries = base[key].as_array().ok_or("version array")?;
        assert!(entries.len() > 1);
        for retained in 0..entries.len() {
            let mut value = base.clone();
            value[key] = json!(&entries[..retained]);
            let subset = rebuild(&value)??;
            assert_ne!(subset.context_id(), complete.context_id());
            for mode in MODES {
                assert_error(
                    subset.merge(&complete, mode),
                    CoreErrorCode::MergeModeViolation,
                    key,
                )?;
                assert_error(
                    merge_generation_context(&complete, &subset, mode),
                    CoreErrorCode::MergeModeViolation,
                    key,
                )?;
            }
        }
    }
    Ok(())
}

#[test]
fn context_merge_010_011_forbid_conflicting_schema_and_producer_versions() -> TestResult {
    let base = fixture()?;
    let left = rebuild(&base)??;
    for key in ["schema_versions", "producer_versions"] {
        let mut value = base.clone();
        value[key][0]["version"] = "2.0.0".into();
        let right = rebuild(&value)??;
        assert_ne!(left.context_id(), right.context_id());
        for mode in MODES {
            assert_error(
                left.merge(&right, mode),
                CoreErrorCode::MergeModeViolation,
                key,
            )?;
            assert_error(
                right.merge(&left, mode),
                CoreErrorCode::MergeModeViolation,
                key,
            )?;
        }
    }
    Ok(())
}

#[test]
fn context_merge_016_rejects_disjoint_version_inventories() -> TestResult {
    let base = fixture()?;
    let left = rebuild(&base)??;
    for (key, replacement) in [
        (
            "schema_versions",
            json!([{"schema_id":"schema:fixture:isolated", "version":"0.1.0"}]),
        ),
        (
            "producer_versions",
            json!([{"producer_id":"fixture.isolated", "version":"0.1.0"}]),
        ),
    ] {
        let mut value = base.clone();
        value[key] = replacement;
        let right = rebuild(&value)??;
        for mode in MODES {
            assert_error(
                left.merge(&right, mode),
                CoreErrorCode::MergeModeViolation,
                key,
            )?;
            assert_error(
                right.merge(&left, mode),
                CoreErrorCode::MergeModeViolation,
                key,
            )?;
        }
    }
    Ok(())
}

#[test]
fn context_merge_002_003_004_only_extension_may_fill_the_project_generation() -> TestResult {
    let mut value = fixture()?;
    let complete = rebuild(&value)??;
    value
        .as_object_mut()
        .ok_or("context object")?
        .remove("project_generation");
    let without = rebuild(&value)??;
    for (left, right) in [(&without, &complete), (&complete, &without)] {
        assert_eq!(
            left.merge(right, MergeMode::ExtendMissingOptional)?,
            complete
        );
        for mode in [MergeMode::Strict, MergeMode::ExternalUnion] {
            assert_error(
                left.merge(right, mode),
                CoreErrorCode::MergeModeViolation,
                "project_generation",
            )?;
        }
    }
    assert_eq!(
        complete.merge(&complete, MergeMode::ExtendMissingOptional)?,
        complete
    );
    Ok(())
}

#[test]
fn context_merge_005_006_007_never_replace_existing_owner_identities() -> TestResult {
    let base = fixture()?;
    let left = rebuild(&base)??;
    for (key, replacement, code, field) in [
        (
            "reference_generation",
            json!(format!("generation:reference:sha256:{}", "78".repeat(32))),
            CoreErrorCode::GenerationMismatch,
            "reference_generation",
        ),
        (
            "project_generation",
            json!(format!("generation:project:sha256:{}", "90".repeat(32))),
            CoreErrorCode::GenerationMismatch,
            "project_generation",
        ),
        (
            "profile",
            {
                let mut profile = base["profile"].clone();
                profile["fixture_scope"] = "distinct fixture profile".into();
                profile
            },
            CoreErrorCode::ProfileMismatch,
            "profile",
        ),
    ] {
        let mut value = base.clone();
        value[key] = replacement;
        let right = rebuild(&value)??;
        for mode in MODES {
            assert_error(left.merge(&right, mode), code, field)?;
            assert_error(right.merge(&left, mode), code, field)?;
        }
    }
    Ok(())
}

#[test]
fn context_merge_008_017_external_union_is_explicit_and_deterministic() -> TestResult {
    let mut a = fixture()?;
    a["external_generations"] = json!([external("fixture", "repo/a", 1)?]);
    let left = rebuild(&a)??;
    let mut b = fixture()?;
    b["external_generations"] = json!([external("fixture", "repo/b", 2)?]);
    let right = rebuild(&b)??;
    let merged = left.merge(&right, MergeMode::ExternalUnion)?;
    assert_eq!(merged, right.merge(&left, MergeMode::ExternalUnion)?);
    assert_eq!(merged.external_generations().len(), 2);
    assert_eq!(merged.external_generations()[0].scope_id(), "repo/a");
    assert_eq!(merged.external_generations()[1].scope_id(), "repo/b");
    assert_eq!(merged.reference_generation(), left.reference_generation());
    assert_eq!(merged.project_generation(), left.project_generation());
    assert_eq!(merged.schema_versions(), left.schema_versions());
    assert_eq!(merged.producer_versions(), left.producer_versions());
    merged.validate()?;
    assert_eq!(merged.merge(&left, MergeMode::ExternalUnion)?, merged);
    for mode in [MergeMode::Strict, MergeMode::ExtendMissingOptional] {
        assert_error(
            left.merge(&right, mode),
            CoreErrorCode::MergeModeViolation,
            "external_generations",
        )?;
        assert_error(
            right.merge(&left, mode),
            CoreErrorCode::MergeModeViolation,
            "external_generations",
        )?;
    }
    Ok(())
}

#[test]
fn context_merge_009_018_preserves_external_generation_and_revision_conflicts() -> TestResult {
    let mut value = fixture()?;
    let entry = external("fixture", "repo/a", 1)?;
    value["external_generations"] = json!([entry.clone()]);
    let left = rebuild(&value)??;
    let mut changed_revision = entry;
    changed_revision["source_revision"] = "fixture:other-revision".into();
    for changed in [external("fixture", "repo/a", 2)?, changed_revision] {
        value["external_generations"] = json!([changed]);
        let right = rebuild(&value)??;
        for (a, b) in [(&left, &right), (&right, &left)] {
            assert_error(
                a.merge(b, MergeMode::ExternalUnion),
                CoreErrorCode::DuplicateExternalGenerationScope,
                "external_generations",
            )?;
        }
    }
    Ok(())
}

#[test]
fn context_003_004_duplicates_fail_even_with_matching_context_digests() -> TestResult {
    for (key, code) in [
        ("schema_versions", CoreErrorCode::DuplicateSchemaId),
        ("producer_versions", CoreErrorCode::DuplicateProducerId),
    ] {
        for change_version in [false, true] {
            let mut value = fixture()?;
            let entries = value[key].as_array_mut().ok_or("version array")?;
            let mut duplicate = entries.first().ok_or("empty versions")?.clone();
            if change_version {
                duplicate["version"] = "2.0.0".into();
            }
            entries.insert(1, duplicate);
            assert_error(rebuild(&value)?, code, key)?;
            reseal(&mut value)?;
            let decoded: GenerationContext = serde_json::from_value(value)?;
            assert_error(decoded.validate(), code, key)?;
            assert_error(require_same_generation(&decoded, &decoded), code, key)?;
        }
    }
    Ok(())
}

#[test]
fn context_005_006_external_dedup_requires_the_entire_record_to_match() -> TestResult {
    let mut value = fixture()?;
    let entry = external("fixture", "repo/a", 1)?;
    value["external_generations"] = json!([entry.clone(), entry.clone()]);
    let normalized = rebuild(&value)??;
    assert_eq!(normalized.external_generations().len(), 1);
    reseal(&mut value)?;
    let decoded: GenerationContext = serde_json::from_value(value.clone())?;
    assert_error(
        decoded.validate(),
        CoreErrorCode::DuplicateExternalGenerationScope,
        "external_generations",
    )?;
    for (key, changed) in [
        ("source_revision", json!("fixture:other-revision")),
        (
            "external_generation_id",
            external("fixture", "repo/a", 2)?["external_generation_id"].clone(),
        ),
    ] {
        value["external_generations"][1] = entry.clone();
        value["external_generations"][1][key] = changed;
        assert_error(
            rebuild(&value)?,
            CoreErrorCode::DuplicateExternalGenerationScope,
            "external_generations",
        )?;
        reseal(&mut value)?;
        let decoded: GenerationContext = serde_json::from_value(value.clone())?;
        assert_error(
            decoded.validate(),
            CoreErrorCode::DuplicateExternalGenerationScope,
            "external_generations",
        )?;
    }
    Ok(())
}

#[test]
fn context_external_001_provider_binding_is_checked_after_deserialization() -> TestResult {
    let mut entry = external("fixture", "repo/a", 1)?;
    entry["provider_id"] = "other".into();
    assert_invalid_external(
        entry,
        CoreErrorCode::GenerationMismatch,
        "external_generations.external_generation_id",
    )
}

fn assert_invalid_external(entry: Value, code: CoreErrorCode, field_name: &str) -> TestResult {
    let mut value = fixture()?;
    value["external_generations"] = json!([entry.clone()]);
    assert_error(rebuild(&value)?, code, field_name)?;
    reseal(&mut value)?;
    let decoded: GenerationContext = serde_json::from_value(value)?;
    assert_error(validate_generation_context(&decoded), code, field_name)?;
    assert_error(derive_generation_context_id(&decoded), code, field_name)?;
    assert_error(
        require_same_generation(&decoded, &decoded),
        code,
        field_name,
    )?;
    for mode in MODES {
        assert_error(decoded.merge(&decoded, mode), code, field_name)?;
    }
    assert_error(
        ExternalGeneration::new(
            field::<String>(&entry, "provider_id")?,
            field::<String>(&entry, "scope_id")?,
            field(&entry, "external_generation_id")?,
            entry
                .get("source_revision")
                .map(|v| serde_json::from_value(v.clone()))
                .transpose()?,
        ),
        code,
        field_name,
    )?;
    Ok(())
}

#[test]
fn context_external_002_rejects_invalid_provider_names() -> TestResult {
    for (provider, code) in [
        ("".to_owned(), CoreErrorCode::IdentifierTooLong),
        ("x".repeat(64), CoreErrorCode::IdentifierTooLong),
        ("Fixture".to_owned(), CoreErrorCode::InvalidIdentifier),
        ("fixture\n".to_owned(), CoreErrorCode::InvalidIdentifier),
        (
            "latest".to_owned(),
            CoreErrorCode::ReservedIdentifierSegment,
        ),
    ] {
        let mut entry = external("fixture", "repo/a", 1)?;
        entry["provider_id"] = provider.into();
        assert_invalid_external(entry, code, "external_generations.provider_id")?;
    }
    Ok(())
}

#[test]
fn context_external_003_004_enforce_text_bounds_on_all_admission_paths() -> TestResult {
    for (key, field_name) in [
        ("scope_id", "external_generations.scope_id"),
        ("source_revision", "external_generations.source_revision"),
    ] {
        for text in [
            "".to_owned(),
            " padded".to_owned(),
            "padded ".to_owned(),
            "bad\0text".to_owned(),
            "bad\ntext".to_owned(),
            "x".repeat(1_025),
            "é".repeat(513),
        ] {
            let mut entry = external("fixture", "repo/a", 1)?;
            entry[key] = text.into();
            assert_invalid_external(entry, CoreErrorCode::InvalidIdentifier, field_name)?;
        }
    }
    Ok(())
}

#[test]
fn context_external_005_accepts_boundary_lengths_unicode_and_absent_revision() -> TestResult {
    for scope in ["x".repeat(1_024), "é".repeat(512), "repo/猫".to_owned()] {
        let mut value = fixture()?;
        let mut entry = external("fixture", &scope, 1)?;
        entry["source_revision"] = "x".repeat(1_024).into();
        value["external_generations"] = json!([entry.clone()]);
        rebuild(&value)??.validate()?;
        entry
            .as_object_mut()
            .ok_or("external object")?
            .remove("source_revision");
        value["external_generations"] = json!([entry]);
        let context = rebuild(&value)??;
        context.validate()?;
        let decoded: GenerationContext = serde_json::from_slice(&canonical_json_bytes(&context)?)?;
        decoded.require_same_generation(&context)?;
    }
    Ok(())
}

#[test]
fn context_008_merge_012_normalize_only_builder_inputs_not_decoded_contexts() -> TestResult {
    let mut value = fixture()?;
    value["external_generations"] = json!([
        external("fixture", "repo/a", 1)?,
        external("fixture", "repo/b", 2)?,
        external("other", "repo/a", 3)?,
    ]);
    let expected = rebuild(&value)??;
    for key in [
        "schema_versions",
        "producer_versions",
        "external_generations",
    ] {
        let mut changed = value.clone();
        changed[key]
            .as_array_mut()
            .ok_or("context array")?
            .reverse();
        let rebuilt = rebuild(&changed)??;
        assert_eq!(
            canonical_json_bytes(&rebuilt)?,
            canonical_json_bytes(&expected)?
        );
        assert_eq!(rebuilt.merge(&expected, MergeMode::Strict)?, expected);
        reseal(&mut changed)?;
        let decoded: GenerationContext = serde_json::from_value(changed)?;
        assert_error(decoded.validate(), CoreErrorCode::GenerationMismatch, key)?;
        assert_error(
            require_same_generation(&decoded, &decoded),
            CoreErrorCode::GenerationMismatch,
            key,
        )?;
    }
    Ok(())
}

#[test]
fn context_008_seeded_input_permutations_preserve_context_and_merge_bytes() -> TestResult {
    let mut value = fixture()?;
    value["external_generations"] = json!([
        external("fixture", "repo/a", 1)?,
        external("fixture", "repo/b", 2)?,
        external("other", "repo/a", 3)?,
    ]);
    let expected = rebuild(&value)??;
    let golden = canonical_json_bytes(&expected)?;
    // Fixed seeds make a failing permutation reproducible without OS randomness.
    for seed in 0_u64..64 {
        let mut changed = value.clone();
        let mut state = seed;
        for key in [
            "schema_versions",
            "producer_versions",
            "external_generations",
        ] {
            let entries = changed[key].as_array_mut().ok_or("context array")?;
            for index in (1..entries.len()).rev() {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                let other = usize::try_from(state % u64::try_from(index + 1)?)?;
                entries.swap(index, other);
            }
        }
        let rebuilt = rebuild(&changed)??;
        assert_eq!(canonical_json_bytes(&rebuilt)?, golden, "seed {seed}");
        assert_eq!(
            canonical_json_bytes(&rebuilt.merge(&expected, MergeMode::Strict)?)?,
            golden,
            "seed {seed}"
        );
    }
    Ok(())
}

#[test]
fn context_same_006_propagates_profile_admission_errors_before_id_comparison() -> TestResult {
    let mut value = fixture()?;
    let valid: GenerationContext = serde_json::from_value(value.clone())?;
    value["profile"]["interface"] = 0.into();
    let malformed: GenerationContext = serde_json::from_value(value)?;
    assert_eq!(valid.context_id(), malformed.context_id());
    for (left, right) in [
        (&valid, &malformed),
        (&malformed, &valid),
        (&malformed, &malformed),
    ] {
        assert_error(
            require_same_generation(left, right),
            CoreErrorCode::InvalidProfileIdentity,
            "interface",
        )?;
    }
    Ok(())
}
