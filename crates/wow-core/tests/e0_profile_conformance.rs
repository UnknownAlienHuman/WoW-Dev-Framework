//! PROFILE-001..026 and PROFILE-COMPARE-001..008 through public boundaries.

use std::error::Error;

use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use wow_core::{
    CoreErrorCode, CoreResult, E0CheckResultEnvelope, E0DecodeLimits, GenerationContext,
    GenerationContextId, ProfileComparison, ProfileIdentity, ProfileIdentityBuilder,
    canonical_json_bytes, compare_profile_identity, require_profile_identity_match,
    require_same_generation, validate_profile_identity,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
const GOLDEN: &str = include_str!("../examples/e0-clean-result.json");

fn fixture() -> TestResult<Value> {
    let result: Value = serde_json::from_str(GOLDEN)?;
    Ok(result["context"]["profile"].clone())
}

fn field<T: DeserializeOwned>(value: &Value, key: &str) -> TestResult<T> {
    Ok(serde_json::from_value(
        value.get(key).ok_or("missing field")?.clone(),
    )?)
}

fn rebuild(value: &Value) -> TestResult<ProfileIdentityBuilder> {
    let mut builder = ProfileIdentityBuilder::new(
        field(value, "profile_id")?,
        field(value, "profile_kind")?,
        field::<String>(value, "flavor_id")?,
        field(value, "interface")?,
        field(value, "source_kind")?,
        field::<String>(value, "source_revision")?,
        field(value, "source_logical_digest")?,
    )
    .schema_versions(field(value, "schema_versions")?);
    if value.get("edition_id").is_some() {
        builder = builder.edition_id(field::<String>(value, "edition_id")?);
    }
    if value.get("client_version").is_some() {
        builder = builder.client_version(field(value, "client_version")?);
    }
    if value.get("client_build").is_some() {
        builder = builder.client_build(field(value, "client_build")?);
    }
    if value.get("builder_id").is_some() {
        builder = builder.builder(
            field(value, "builder_id")?,
            field(value, "builder_version")?,
        );
    }
    if value.get("correction_set_digest").is_some() {
        builder = builder.correction_set_digest(field(value, "correction_set_digest")?);
    }
    if value.get("fixture_scope").is_some() {
        builder = builder.fixture_scope(field::<String>(value, "fixture_scope")?);
    }
    Ok(builder)
}

fn release() -> TestResult<Value> {
    // Synthetic test material, not a claim about a released WoW build.
    let mut value = fixture()?;
    value["profile_id"] = "profile:wow:profile-admission-test".into();
    value["profile_kind"] = "release".into();
    value["source_kind"] = "blizzard_snapshot".into();
    value["source_revision"] = "0123456789abcdef0123456789abcdef01234567".into();
    value["client_version"] = "1.2.3".into();
    value["client_build"] = 1.into();
    value["builder_id"] = "fixture.profile_builder".into();
    value["builder_version"] = "0.1.0".into();
    value["correction_set_digest"] = format!("sha256:{}", "12".repeat(32)).into();
    value
        .as_object_mut()
        .ok_or("profile object")?
        .remove("fixture_scope");
    Ok(value)
}

fn assert_error<T>(result: CoreResult<T>, code: CoreErrorCode, field: &str) -> TestResult {
    let error = match result {
        Err(error) => error,
        Ok(_) => return Err(format!("expected {code:?} at {field}").into()),
    };
    assert_eq!(error.code(), code);
    assert_eq!(error.field_path(), Some(field));
    error.validate()?;
    assert!(!serde_json::to_string(&error)?.contains("PRIVATE_MARKER"));
    Ok(())
}

fn reject_profile(value: Value, code: CoreErrorCode, field: &str) -> TestResult {
    let profile: ProfileIdentity = serde_json::from_value(value)?;
    let valid: ProfileIdentity = serde_json::from_value(fixture()?)?;
    assert_error(profile.validate(), code, field)?;
    assert_error(validate_profile_identity(&profile), code, field)?;
    for (left, right) in [(&profile, &valid), (&valid, &profile), (&profile, &profile)] {
        assert_error(compare_profile_identity(left, right), code, field)?;
        assert_error(require_profile_identity_match(left, right), code, field)?;
    }
    Ok(())
}

#[test]
fn profile_001_012_golden_identity_survives_construction_and_round_trip() -> TestResult {
    let value = fixture()?;
    let expected = canonical_json_bytes(&value)?;
    let profile = rebuild(&value)?.build()?;
    profile.validate()?;
    assert_eq!(canonical_json_bytes(&profile)?, expected);
    let decoded: ProfileIdentity = serde_json::from_slice(&expected)?;
    assert_eq!(profile, decoded);
    assert_eq!(
        compare_profile_identity(&profile, &decoded)?,
        ProfileComparison::Identical
    );
    require_profile_identity_match(&profile, &decoded)?;
    Ok(())
}

#[test]
fn profile_002_016_fixture_and_release_namespaces_cannot_be_relabelled() -> TestResult {
    for mut value in [fixture()?, release()?] {
        let wrong_id = if value["profile_kind"] == "fixture" {
            "profile:wow:wrong-kind"
        } else {
            "profile:fixture:wrong-kind"
        };
        for id in [wrong_id, "profile:other:unsupported"] {
            value["profile_id"] = id.into();
            reject_profile(
                value.clone(),
                CoreErrorCode::ProfileKindViolation,
                "profile_id.namespace",
            )?;
        }
    }
    let mut value = fixture()?;
    value["profile_kind"] = "release".into();
    reject_profile(
        value,
        CoreErrorCode::ProfileKindViolation,
        "profile_id.namespace",
    )?;
    let mut value = fixture()?;
    value
        .as_object_mut()
        .ok_or("profile object")?
        .remove("fixture_scope");
    assert_error(
        rebuild(&value)?.build(),
        CoreErrorCode::ProfileKindViolation,
        "fixture_scope",
    )?;
    reject_profile(value, CoreErrorCode::ProfileKindViolation, "fixture_scope")
}

#[test]
fn profile_003_004_005_008_020_release_requirements_are_independent() -> TestResult {
    let base = release()?;
    let valid = rebuild(&base)?.build()?;
    assert_eq!(serde_json::to_value(&valid)?, base);
    for missing in [
        "builder_id",
        "builder_version",
        "correction_set_digest",
        "client_build",
        "client_version",
    ] {
        let mut value = base.clone();
        value
            .as_object_mut()
            .ok_or("profile object")?
            .remove(missing);
        reject_profile(value, CoreErrorCode::InvalidProfileIdentity, missing)?;
    }
    let mut value = base.clone();
    value["source_kind"] = "synthetic_fixture".into();
    reject_profile(value, CoreErrorCode::ProfileKindViolation, "source_kind")?;
    let mut value = base;
    value["fixture_scope"] = "fixture only".into();
    reject_profile(value, CoreErrorCode::ProfileKindViolation, "fixture_scope")
}

#[test]
fn profile_006_release_rejects_reserved_floating_revisions() -> TestResult {
    for revision in [
        "main", "master", "live", "latest", "HEAD", "current", "default", "auto",
    ] {
        let mut value = release()?;
        value["source_revision"] = revision.into();
        assert_error(
            rebuild(&value)?.build(),
            CoreErrorCode::InvalidProfileIdentity,
            "source_revision",
        )?;
        reject_profile(
            value,
            CoreErrorCode::InvalidProfileIdentity,
            "source_revision",
        )?;
    }
    Ok(())
}

#[test]
fn profile_007_008_019_021_optional_build_is_positive_when_present() -> TestResult {
    for mut value in [fixture()?, release()?] {
        for key in ["interface", "client_build"] {
            let original = value.get(key).cloned();
            value[key] = 0.into();
            assert_error(
                rebuild(&value)?.build(),
                CoreErrorCode::InvalidProfileIdentity,
                key,
            )?;
            reject_profile(value.clone(), CoreErrorCode::InvalidProfileIdentity, key)?;
            match original {
                Some(original) => {
                    value[key] = original;
                }
                None => {
                    value.as_object_mut().ok_or("profile object")?.remove(key);
                }
            }
        }
        value["client_build"] = 1.into();
        value["interface"] = 1.into();
        rebuild(&value)?.build()?.validate()?;
    }
    let mut value = fixture()?;
    value
        .as_object_mut()
        .ok_or("profile object")?
        .remove("client_build");
    value
        .as_object_mut()
        .ok_or("profile object")?
        .remove("client_version");
    assert_eq!(serde_json::to_value(rebuild(&value)?.build()?)?, value);
    Ok(())
}

#[test]
fn profile_022_fixture_builder_is_absent_or_an_exact_pair() -> TestResult {
    let mut base = fixture()?;
    base["builder_id"] = "fixture.profile_builder".into();
    base["builder_version"] = "0.1.0".into();
    for missing in ["builder_id", "builder_version"] {
        let mut value = base.clone();
        value
            .as_object_mut()
            .ok_or("profile object")?
            .remove(missing);
        reject_profile(value, CoreErrorCode::InvalidProfileIdentity, missing)?;
    }
    let mut minimal = fixture()?;
    for key in ["builder_id", "builder_version", "correction_set_digest"] {
        minimal.as_object_mut().ok_or("profile object")?.remove(key);
    }
    assert_eq!(serde_json::to_value(rebuild(&minimal)?.build()?)?, minimal);
    minimal["correction_set_digest"] = format!("sha256:{}", "34".repeat(32)).into();
    reject_profile(minimal, CoreErrorCode::InvalidProfileIdentity, "builder_id")?;
    assert_eq!(serde_json::to_value(rebuild(&base)?.build()?)?, base);
    Ok(())
}

#[test]
fn profile_023_scope_is_bounded_nonempty_text_without_controls_or_trimming() -> TestResult {
    for scope in [
        "".to_owned(),
        " ".to_owned(),
        "\u{a0}".to_owned(),
        " PRIVATE_MARKER".to_owned(),
        "PRIVATE_MARKER ".to_owned(),
        "PRIVATE_MARKER\nrest".to_owned(),
        "PRIVATE_MARKER\0".to_owned(),
        "я".repeat(2049),
    ] {
        let mut value = fixture()?;
        value["fixture_scope"] = scope.into();
        assert_error(
            rebuild(&value)?.build(),
            CoreErrorCode::ProfileKindViolation,
            "fixture_scope",
        )?;
        reject_profile(value, CoreErrorCode::ProfileKindViolation, "fixture_scope")?;
    }
    for scope in [
        "я".repeat(2048),
        "source-only fixture; runtime not tested".to_owned(),
    ] {
        let mut value = fixture()?;
        value["fixture_scope"] = scope.into();
        assert_eq!(serde_json::to_value(rebuild(&value)?.build()?)?, value);
    }
    Ok(())
}

#[test]
fn profile_024_revision_has_the_same_byte_bound_for_both_kinds() -> TestResult {
    for base in [fixture()?, release()?] {
        for revision in [
            "".to_owned(),
            " PRIVATE_MARKER".to_owned(),
            "PRIVATE_MARKER\nrest".to_owned(),
            "я".repeat(513),
        ] {
            let mut value = base.clone();
            value["source_revision"] = revision.into();
            assert_error(
                rebuild(&value)?.build(),
                CoreErrorCode::InvalidProfileIdentity,
                "source_revision",
            )?;
            reject_profile(
                value,
                CoreErrorCode::InvalidProfileIdentity,
                "source_revision",
            )?;
        }
        let mut value = base;
        value["source_revision"] = "я".repeat(512).into();
        assert_eq!(serde_json::to_value(rebuild(&value)?.build()?)?, value);
    }
    Ok(())
}

#[test]
fn profile_009_010_011_schema_admission_rejects_duplicates_and_only_builder_sorts() -> TestResult {
    let mut value = fixture()?;
    value["schema_versions"] = json!([
        {"schema_id": "schema:fixture:a", "version": "0.1.0"},
        {"schema_id": "schema:fixture:b", "version": "0.1.0"},
        {"schema_id": "schema:fixture:c", "version": "0.1.0"}
    ]);
    let expected = canonical_json_bytes(&value)?;
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let mut shuffled = value.clone();
        shuffled["schema_versions"] = Value::Array(
            order
                .iter()
                .map(|&i| value["schema_versions"][i].clone())
                .collect(),
        );
        assert_eq!(
            canonical_json_bytes(&rebuild(&shuffled)?.build()?)?,
            expected
        );
        if order != [0, 1, 2] {
            reject_profile(
                shuffled,
                CoreErrorCode::InvalidProfileIdentity,
                "schema_versions",
            )?;
        }
    }
    for version in ["0.1.0", "1.0.0"] {
        let mut duplicated = value.clone();
        duplicated["schema_versions"][1] = duplicated["schema_versions"][0].clone();
        duplicated["schema_versions"][1]["version"] = version.into();
        assert_error(
            rebuild(&duplicated)?.build(),
            CoreErrorCode::DuplicateSchemaId,
            "schema_versions",
        )?;
        reject_profile(
            duplicated,
            CoreErrorCode::DuplicateSchemaId,
            "schema_versions",
        )?;
    }
    value["schema_versions"] = json!([]);
    reject_profile(
        value,
        CoreErrorCode::InvalidProfileIdentity,
        "schema_versions",
    )
}

#[test]
fn profile_015_real_snapshot_fixture_stays_fixture_and_core_does_not_lookup_builds() -> TestResult {
    let mut value = fixture()?;
    value["source_kind"] = "blizzard_snapshot".into();
    value["source_revision"] = "0123456789abcdef0123456789abcdef01234567".into();
    value["interface"] = 1.into();
    value["client_build"] = 2.into();
    let profile = rebuild(&value)?.build()?;
    assert_eq!(profile.profile_kind(), wow_core::ProfileKind::Fixture);
    assert_eq!(serde_json::to_value(&profile)?, value);
    assert!(require_profile_identity_match(&profile, &rebuild(&release()?)?.build()?).is_err());
    Ok(())
}

#[test]
fn profile_017_018_025_strict_wire_and_typed_fields_are_not_repaired() -> TestResult {
    for (key, invalid) in [
        ("builder_id", json!("bad id")),
        ("builder_version", json!("not-a-version")),
        ("client_version", json!("not-a-version")),
        ("source_logical_digest", json!("sha1:abc")),
        ("interface", json!(-1)),
        ("client_build", json!(1.5)),
        ("client_build", json!("1")),
        ("profile_kind", json!("unknown")),
        ("source_kind", json!("unknown")),
        ("unrecognized", json!(true)),
    ] {
        let mut value = fixture()?;
        value[key] = invalid;
        assert!(
            serde_json::from_value::<ProfileIdentity>(value).is_err(),
            "{key}"
        );
    }
    Ok(())
}

#[test]
fn profile_013_014_and_compare_001_008_all_material_fields_participate() -> TestResult {
    let mut base = fixture()?;
    base["edition_id"] = "test".into();
    base["client_build"] = 1.into();
    base["client_version"] = "1.2.3".into();
    base["builder_id"] = "fixture.profile_builder".into();
    base["builder_version"] = "0.1.0".into();
    base["correction_set_digest"] = format!("sha256:{}", "12".repeat(32)).into();
    let left = rebuild(&base)?.build()?;
    let mut relabelled = base.clone();
    relabelled["profile_id"] = "profile:fixture:another-label".into();
    let alias = rebuild(&relabelled)?.build()?;
    assert_eq!(
        compare_profile_identity(&left, &alias)?,
        ProfileComparison::DifferentLabelSameMaterial
    );
    assert_error(
        require_profile_identity_match(&left, &alias),
        CoreErrorCode::ProfileMismatch,
        "profile",
    )?;
    let mut schema = base["schema_versions"].clone();
    schema[0]["version"] = "2.0.0".into();
    for (key, changed) in [
        ("flavor_id", json!("other")),
        ("edition_id", json!("other")),
        ("interface", json!(2)),
        ("client_version", json!("1.2.4")),
        ("client_build", json!(2)),
        ("source_kind", json!("blizzard_snapshot")),
        ("source_revision", json!("fixture:other-revision")),
        (
            "source_logical_digest",
            json!(format!("sha256:{}", "56".repeat(32))),
        ),
        ("builder_id", json!("fixture.another_builder")),
        ("builder_version", json!("0.2.0")),
        ("schema_versions", schema),
        (
            "correction_set_digest",
            json!(format!("sha256:{}", "78".repeat(32))),
        ),
        ("fixture_scope", json!("another fixture scope")),
    ] {
        let mut value = base.clone();
        value[key] = changed;
        let right = rebuild(&value)?.build()?;
        for (left, right) in [(&left, &right), (&right, &left)] {
            assert_eq!(
                compare_profile_identity(left, right)?,
                ProfileComparison::SameLabelDifferentIdentity {
                    differing_fields: vec![key.to_owned()],
                }
            );
            assert_error(
                require_profile_identity_match(left, right),
                CoreErrorCode::ProfileMismatch,
                "profile",
            )?;
        }
        value["profile_id"] = "profile:fixture:another-label".into();
        assert_eq!(
            compare_profile_identity(&left, &rebuild(&value)?.build()?)?,
            ProfileComparison::Different
        );
    }
    Ok(())
}

#[test]
fn profile_026_resealed_context_and_raw_envelope_recheck_nested_profile() -> TestResult {
    let original: Value = serde_json::from_str(GOLDEN)?;
    let limits = E0DecodeLimits::new(1024 * 1024, 64, 100_000, 64 * 1024)?;
    for (key, value, code, path) in [
        (
            "client_build",
            json!(0),
            CoreErrorCode::InvalidProfileIdentity,
            "client_build",
        ),
        (
            "fixture_scope",
            json!("PRIVATE_MARKER\n"),
            CoreErrorCode::ProfileKindViolation,
            "fixture_scope",
        ),
    ] {
        let mut mutated = original.clone();
        mutated["context"]["profile"][key] = value;
        let mut projection = mutated["context"].clone();
        projection
            .as_object_mut()
            .ok_or("context object")?
            .remove("context_id");
        let context_id = GenerationContextId::derive(&projection)?;
        mutated["context"]["context_id"] = context_id.to_string().into();
        let context: GenerationContext = serde_json::from_value(mutated["context"].clone())?;
        assert_eq!(context.context_id(), context_id);
        assert_error(context.validate(), code, path)?;
        assert_error(require_same_generation(&context, &context), code, path)?;
        let bytes = serde_json::to_vec(&mutated)?;
        assert_error(
            E0CheckResultEnvelope::from_json_slice(&bytes, limits),
            code,
            path,
        )?;
    }
    let accepted = E0CheckResultEnvelope::from_json_slice(GOLDEN.as_bytes(), limits)?;
    assert_eq!(
        accepted.canonical_bytes()?,
        canonical_json_bytes(&original)?
    );
    Ok(())
}
