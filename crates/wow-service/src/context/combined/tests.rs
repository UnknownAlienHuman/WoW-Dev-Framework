//! Pure request decoding compatibility and admission coverage.
use super::*;
use crate::context::{PRIMARY_L1_PROFILE_ID, PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID};
use serde_json::{Value, json};
use std::sync::atomic::AtomicBool;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

mod reference_root;

// Synthetic, project-owned values with the complete typed platform request shape.
const FIXTURE: &str = r#"{
  "schema":"wow-service/context-combined-request/1",
  "operation":"build",
  "profile_id":"wow-context/combined-native-one-hop/1",
  "primary":{
    "project_store_id":"project-store:fixture-combined-primary",
    "store_generation_id":"project-store-generation:sha256:0000000000000000000000000000000000000000000000000000000000000000",
    "publication_set_id":"project-publication-set:sha256:0000000000000000000000000000000000000000000000000000000000000000",
    "map_profile_id":"wow-context/primary-direct-navigation/1",
    "root":{"kind":"graph_entity","id":"graph-node:sha256:0000000000000000000000000000000000000000000000000000000000000000"}
  },
  "platform":{
    "schema":"wow-service/context-platform-request/1",
    "operation":"map",
    "project_store_id":"project-store:fixture-combined-platform",
    "store_generation_id":"project-store-generation:sha256:0000000000000000000000000000000000000000000000000000000000000000",
    "publication_set_id":"project-publication-set:sha256:0000000000000000000000000000000000000000000000000000000000000000",
    "candidate_id":"fixture-platform-candidate:sha256:0000000000000000000000000000000000000000000000000000000000000000",
    "candidate_digest":"sha256:0000000000000000000000000000000000000000000000000000000000000000",
    "profile_id":"wow-context/platform-native-one-hop/1",
    "input":{
      "binding":{
        "analyzer_snapshot_id":"project-analyzer-snapshot:sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "configuration_digest":"sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "graph_generation":"graph-generation:sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "graph_snapshot_id":"graph-snapshot:sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "native_prefix_generation":"graph-generation:sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "native_prefix_snapshot_id":"graph-snapshot:sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "native_profile":"wow-project/platform-capabilities/1",
        "package_binding_digest":"sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "project_generation":"generation:project:sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "project_snapshot_id":"project-snapshot:sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "registry_digest":"graph-registry:sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "source_assertion_scope":{
          "generation":"source-graph-input:sha256:0000000000000000000000000000000000000000000000000000000000000000",
          "source_context_id":"context:sha256:0000000000000000000000000000000000000000000000000000000000000000",
          "universe":"fixture_platform_source:sha256:0000000000000000000000000000000000000000000000000000000000000000:fixture-platform-source-snapshot:0000000000000000000000000000000000000000000000000000000000000000"
        },
        "source_context_id":"context:sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "source_profile_digest":"sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "source_profile_id":"profile:fixture:combined-platform-source",
        "source_snapshot_id":"fixture-platform-source-snapshot:0000000000000000000000000000000000000000000000000000000000000000"
      },
      "limits":{
        "max_entities":64,"max_output_bytes":1048576,"max_support_records":4096,
        "root_evidence":{"max_derivation_depth":32,"max_evidence_records":1024,"max_source_handles":4096,"max_work":100000},
        "root_explanation":{"max_derivation_depth":32,"max_output_bytes":1048576,"max_scanned_assertions":500000,"max_supports":128}
      },
      "neighborhood":{
        "confidence":"include_candidate",
        "limits":{"max_output_bytes":1048576,"max_scanned_edges":500000},
        "query":{"direction":"both","max_edges":8,"node_id":"graph-node:sha256:0000000000000000000000000000000000000000000000000000000000000000","relations":["owns"]},
        "snapshot_id":"graph-snapshot:sha256:0000000000000000000000000000000000000000000000000000000000000000"
      }
    }
  },
  "build":{
    "context_profile_id":"wow-context/combined-native-structural-pack/1",
    "level":"both",
    "renderer_profile_ids":["wow-context/combined-pack-json/1","wow-context/combined-pack-markdown/1"]
  }
}"#;

fn fixture() -> TestResult<Value> {
    Ok(serde_json::from_str(FIXTURE)?)
}

fn inspect(level: &str, profile: Option<&str>) -> TestResult<Value> {
    let mut value = fixture()?;
    value["operation"] = json!("inspect");
    value
        .as_object_mut()
        .ok_or("request object")?
        .remove("build");
    let mut selection = json!({"level": level});
    if let Some(profile) = profile {
        selection["l1_profile_id"] = json!(profile);
    }
    value["inspection"] = selection;
    Ok(value)
}

fn build(level: &str, profile: Option<&str>) -> TestResult<Value> {
    let mut value = fixture()?;
    value["build"]["level"] = json!(level);
    if let Some(profile) = profile {
        value["build"]["l1_profile_id"] = json!(profile);
    }
    Ok(value)
}

fn inspect_l0(profile: &str) -> TestResult<Value> {
    let mut value = inspect("l0", None)?;
    value["inspection"]["l0_profile_id"] = json!(profile);
    Ok(value)
}

fn build_l0(profile: &str) -> TestResult<Value> {
    let mut value = build("l0", None)?;
    value["build"]["l0_profile_id"] = json!(profile);
    Ok(value)
}

fn refused(value: &Value, expected: ServiceErrorCode) -> TestResult {
    let bytes = serde_json::to_vec(value)?;
    let actual = ContextCombinedRequest::from_json_slice(&bytes, &AtomicBool::new(false))
        .err()
        .map(|error| error.code());
    assert_eq!(actual, Some(expected));
    Ok(())
}

#[test]
fn supported_l1_decode_preserves_compatibility_and_refuses_invalid_admission() -> TestResult {
    // Compare to the independently specified omitted-field request, rather than
    // only checking that the new decoder roundtrips its own output.
    for omitted in [inspect("l1", None)?, build("l1", None)?] {
        let omitted_request = ContextCombinedRequest::from_json_slice(
            &serde_json::to_vec(&omitted)?,
            &AtomicBool::new(false),
        )?;
        assert_eq!(
            wow_core::canonical_json_bytes(&omitted_request)?,
            wow_core::canonical_json_bytes(&omitted)?,
        );
    }

    // Both state detail and ReferenceFacts L1/Both roundtrip for both operations.
    for (profile, level, value) in [
        (
            PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
            "l1",
            inspect("l1", Some(PRIMARY_STATE_DETAIL_L1_PROFILE_ID))?,
        ),
        (
            PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
            "l1",
            build("l1", Some(PRIMARY_STATE_DETAIL_L1_PROFILE_ID))?,
        ),
        (
            PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID,
            "l1",
            inspect("l1", Some(PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID))?,
        ),
        (
            PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID,
            "l1",
            build("l1", Some(PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID))?,
        ),
        (
            PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID,
            "both",
            inspect("both", Some(PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID))?,
        ),
        (
            PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID,
            "both",
            build("both", Some(PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID))?,
        ),
        (
            PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
            "both",
            inspect("both", Some(PRIMARY_STATE_DETAIL_L1_PROFILE_ID))?,
        ),
        (
            PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
            "both",
            build("both", Some(PRIMARY_STATE_DETAIL_L1_PROFILE_ID))?,
        ),
    ] {
        let bytes = serde_json::to_vec(&value)?;
        let request = ContextCombinedRequest::from_json_slice(&bytes, &AtomicBool::new(false))?;
        let encoded = serde_json::to_vec(&request)?;
        let encoded_value: Value = serde_json::from_slice(&encoded)?;
        let selection = if request.operation() == ContextCombinedOperation::Inspect {
            &encoded_value["inspection"]
        } else {
            &encoded_value["build"]
        };
        assert_eq!(selection["l1_profile_id"], profile);
        assert_eq!(selection["level"], level);
        assert_eq!(
            request.l1_recipe,
            Some(
                if profile == PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID && level == "both" {
                    CombinedL1Recipe::ReferenceFactsBoth
                } else if profile == PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID {
                    CombinedL1Recipe::ReferenceFacts
                } else {
                    CombinedL1Recipe::StateDetail
                }
            ),
        );
        let roundtrip = ContextCombinedRequest::from_json_slice(&encoded, &AtomicBool::new(false))?;
        assert_eq!(
            wow_core::canonical_json_bytes(&request)?,
            wow_core::canonical_json_bytes(&value)?,
        );
        assert_eq!(
            wow_core::canonical_json_bytes(&roundtrip)?,
            wow_core::canonical_json_bytes(&value)?
        );
    }

    for mut value in [inspect("l1", None)?, build("l1", None)?] {
        let key = if value["operation"] == "inspect" {
            "inspection"
        } else {
            "build"
        };
        for invalid in [
            Value::Null,
            json!(false),
            json!("wow-context/unknown-l1/1"),
            json!(PRIMARY_L1_PROFILE_ID),
        ] {
            value[key]["l1_profile_id"] = invalid;
            refused(&value, ServiceErrorCode::InvalidRequest)?;
        }
    }

    for (level, profile) in [
        ("l0", PRIMARY_STATE_DETAIL_L1_PROFILE_ID),
        ("l0", PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID),
    ] {
        refused(
            &inspect(level, Some(profile))?,
            ServiceErrorCode::InvalidRequest,
        )?;
        refused(
            &build(level, Some(profile))?,
            ServiceErrorCode::InvalidRequest,
        )?;
    }
    for mut value in [
        inspect("both", Some(PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID))?,
        build("both", Some(PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID))?,
    ] {
        let key = if value["operation"] == "inspect" {
            "inspection"
        } else {
            "build"
        };
        value[key]["l0_profile_id"] = json!(PRIMARY_STATE_ROOT_L0_PROFILE_ID);
        refused(&value, ServiceErrorCode::InvalidRequest)?;
    }
    for mut value in [
        inspect("l1", Some(PRIMARY_STATE_DETAIL_L1_PROFILE_ID))?,
        inspect("both", Some(PRIMARY_STATE_DETAIL_L1_PROFILE_ID))?,
        build("l1", Some(PRIMARY_STATE_DETAIL_L1_PROFILE_ID))?,
        build("both", Some(PRIMARY_STATE_DETAIL_L1_PROFILE_ID))?,
        inspect("l1", Some(PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID))?,
        inspect("both", Some(PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID))?,
        build("l1", Some(PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID))?,
        build("both", Some(PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID))?,
    ] {
        value["primary"]["root"] = json!({"kind":"project","id":"fixture-primary"});
        refused(&value, ServiceErrorCode::InvalidRequest)?;
    }

    let mut inspect_with_build = inspect("l1", Some(PRIMARY_STATE_DETAIL_L1_PROFILE_ID))?;
    inspect_with_build["build"] = fixture()?["build"].clone();
    refused(&inspect_with_build, ServiceErrorCode::InvalidRequest)?;
    let mut build_with_inspect = build("l1", Some(PRIMARY_STATE_DETAIL_L1_PROFILE_ID))?;
    build_with_inspect["inspection"] = json!({"level":"l1"});
    refused(&build_with_inspect, ServiceErrorCode::InvalidRequest)?;

    assert_eq!(
        ContextCombinedRequest::from_json_slice(FIXTURE.as_bytes(), &AtomicBool::new(true))
            .err()
            .map(|error| error.code()),
        Some(ServiceErrorCode::Cancelled),
        "cancellation is observed before request admission",
    );
    Ok(())
}

#[test]
fn explicit_state_root_l0_decode_is_narrow_and_preserves_omission() -> TestResult {
    for omitted in [inspect("l0", None)?, build("l0", None)?] {
        let request = ContextCombinedRequest::from_json_slice(
            &serde_json::to_vec(&omitted)?,
            &AtomicBool::new(false),
        )?;
        assert_eq!(
            wow_core::canonical_json_bytes(&request)?,
            wow_core::canonical_json_bytes(&omitted)?,
        );
    }

    for value in [
        inspect_l0(PRIMARY_STATE_ROOT_L0_PROFILE_ID)?,
        build_l0(PRIMARY_STATE_ROOT_L0_PROFILE_ID)?,
    ] {
        let request = ContextCombinedRequest::from_json_slice(
            &serde_json::to_vec(&value)?,
            &AtomicBool::new(false),
        )?;
        let encoded: Value = serde_json::from_slice(&serde_json::to_vec(&request)?)?;
        let selection = if request.operation() == ContextCombinedOperation::Inspect {
            &encoded["inspection"]
        } else {
            &encoded["build"]
        };
        assert_eq!(selection["level"], "l0");
        assert_eq!(selection["l0_profile_id"], PRIMARY_STATE_ROOT_L0_PROFILE_ID);
        assert_eq!(
            wow_core::canonical_json_bytes(&request)?,
            wow_core::canonical_json_bytes(&value)?,
        );
    }

    for mut value in [
        inspect_l0(PRIMARY_STATE_ROOT_L0_PROFILE_ID)?,
        build_l0(PRIMARY_STATE_ROOT_L0_PROFILE_ID)?,
    ] {
        let key = if value["operation"] == "inspect" {
            "inspection"
        } else {
            "build"
        };
        for invalid in [
            Value::Null,
            json!(false),
            json!("wow-context/primary-l0/1"),
            json!("wow-context/unknown-l0/1"),
        ] {
            value[key]["l0_profile_id"] = invalid;
            refused(&value, ServiceErrorCode::InvalidRequest)?;
        }
    }

    for (level, l1_profile) in [
        ("l1", None),
        ("both", None),
        ("l0", Some(PRIMARY_STATE_DETAIL_L1_PROFILE_ID)),
        ("l0", Some(PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID)),
    ] {
        for mut value in [
            inspect_l0(PRIMARY_STATE_ROOT_L0_PROFILE_ID)?,
            build_l0(PRIMARY_STATE_ROOT_L0_PROFILE_ID)?,
        ] {
            let key = if value["operation"] == "inspect" {
                "inspection"
            } else {
                "build"
            };
            value[key]["level"] = json!(level);
            if let Some(profile) = l1_profile {
                value[key]["l1_profile_id"] = json!(profile);
            }
            refused(&value, ServiceErrorCode::InvalidRequest)?;
        }
    }

    for mut value in [
        inspect_l0(PRIMARY_STATE_ROOT_L0_PROFILE_ID)?,
        build_l0(PRIMARY_STATE_ROOT_L0_PROFILE_ID)?,
    ] {
        value["primary"]["root"] = json!({"kind":"project","id":"fixture-primary"});
        refused(&value, ServiceErrorCode::InvalidRequest)?;
    }
    let mut map = fixture()?;
    map["operation"] = json!("map");
    map.as_object_mut().ok_or("request object")?.remove("build");
    map.as_object_mut().ok_or("request object")?.insert(
        "l0_profile_id".into(),
        json!(PRIMARY_STATE_ROOT_L0_PROFILE_ID),
    );
    refused(&map, ServiceErrorCode::InvalidRequest)?;
    Ok(())
}

#[test]
fn platform_root_facets_compose_with_valid_primary_recipes_in_inspect_and_build() -> TestResult {
    let recipes = [
        (
            "l0",
            "l0_profile_id",
            PRIMARY_STATE_ROOT_L0_PROFILE_ID,
            Some(CombinedL0Recipe::StateRoot),
            None,
        ),
        (
            "l1",
            "l1_profile_id",
            PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
            None,
            Some(CombinedL1Recipe::StateDetail),
        ),
        (
            "both",
            "l1_profile_id",
            PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
            None,
            Some(CombinedL1Recipe::StateDetail),
        ),
        (
            "l1",
            "l1_profile_id",
            PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID,
            None,
            Some(CombinedL1Recipe::ReferenceFacts),
        ),
        (
            "both",
            "l1_profile_id",
            PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID,
            None,
            Some(CombinedL1Recipe::ReferenceFactsBoth),
        ),
    ];
    for facet in [
        "callable_metadata",
        "xml_handler_metadata",
        "xml_declaration_metadata",
    ] {
        for operation in ["inspect", "build"] {
            for (level, field, profile, expected_l0, expected_l1) in recipes {
                let mut value = if operation == "inspect" {
                    inspect(level, None)?
                } else {
                    build(level, None)?
                };
                let key = if operation == "inspect" {
                    "inspection"
                } else {
                    "build"
                };
                value[key]["platform_root_facet"] = json!(facet);
                value[key][field] = json!(profile);
                let request = ContextCombinedRequest::from_json_slice(
                    &serde_json::to_vec(&value)?,
                    &AtomicBool::new(false),
                )?;
                assert_eq!(
                    request.platform_root_facet,
                    Some(match facet {
                        "callable_metadata" => wow_context::PlatformRootFacet::CallableMetadata,
                        "xml_handler_metadata" => {
                            wow_context::PlatformRootFacet::XmlHandlerMetadata
                        }
                        _ => wow_context::PlatformRootFacet::XmlDeclarationMetadata,
                    })
                );
                assert!(request.l0_recipe == expected_l0);
                assert!(request.l1_recipe == expected_l1);
                let expected = wow_core::canonical_json_bytes(&value)?;
                assert_eq!(wow_core::canonical_json_bytes(&request)?, expected);
                let roundtrip =
                    ContextCombinedRequest::from_json_slice(&expected, &AtomicBool::new(false))?;
                assert_eq!(wow_core::canonical_json_bytes(&roundtrip)?, expected);
            }
        }
    }

    for facet in [
        "callable_metadata",
        "xml_handler_metadata",
        "xml_declaration_metadata",
    ] {
        for operation in ["inspect", "build"] {
            let mut mixed = if operation == "inspect" {
                inspect("l0", None)?
            } else {
                build("l0", None)?
            };
            let key = if operation == "inspect" {
                "inspection"
            } else {
                "build"
            };
            mixed[key]["platform_root_facet"] = json!(facet);
            mixed[key]["l0_profile_id"] = json!(PRIMARY_STATE_ROOT_L0_PROFILE_ID);
            mixed[key]["l1_profile_id"] = json!(PRIMARY_STATE_DETAIL_L1_PROFILE_ID);
            refused(&mixed, ServiceErrorCode::InvalidRequest)?;

            for (level, field, profile) in [
                ("l1", "l0_profile_id", PRIMARY_STATE_ROOT_L0_PROFILE_ID),
                ("l0", "l1_profile_id", PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID),
                ("l0", "l1_profile_id", PRIMARY_STATE_DETAIL_L1_PROFILE_ID),
            ] {
                let mut invalid = if operation == "inspect" {
                    inspect(level, None)?
                } else {
                    build(level, None)?
                };
                invalid[key]["platform_root_facet"] = json!(facet);
                invalid[key][field] = json!(profile);
                refused(&invalid, ServiceErrorCode::InvalidRequest)?;
            }
        }
    }
    Ok(())
}

#[test]
fn platform_callable_selection_preserves_ordinary_admission_and_request_bytes() -> TestResult {
    for level in ["l0", "l1", "both"] {
        for omitted in [inspect(level, None)?, build(level, None)?] {
            let request = ContextCombinedRequest::from_json_slice(
                &serde_json::to_vec(&omitted)?,
                &AtomicBool::new(false),
            )?;
            assert!(request.platform_root_facet.is_none());
            assert_eq!(
                wow_core::canonical_json_bytes(&request)?,
                wow_core::canonical_json_bytes(&omitted)?,
            );
        }
        let mut values = vec![inspect(level, None)?];
        for renderers in [
            json!([]),
            json!([COMBINED_CONTEXT_JSON_RENDERER_PROFILE_ID]),
            json!([COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID]),
            json!([
                COMBINED_CONTEXT_JSON_RENDERER_PROFILE_ID,
                COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID
            ]),
        ] {
            let mut value = build(level, None)?;
            value["build"]["renderer_profile_ids"] = renderers;
            values.push(value);
        }
        for mut value in values {
            let key = if value["operation"] == "inspect" {
                "inspection"
            } else {
                "build"
            };
            value[key]["platform_root_facet"] = json!("callable_metadata");
            let request = ContextCombinedRequest::from_json_slice(
                &serde_json::to_vec(&value)?,
                &AtomicBool::new(false),
            )?;
            assert_eq!(
                request.platform_root_facet,
                Some(wow_context::PlatformRootFacet::CallableMetadata),
            );
            assert!(request.l0_recipe.is_none() && request.l1_recipe.is_none());
            // The independent input catches leaks of any derived field,
            // including selected_level/render/recipe and selected facet.
            let expected = wow_core::canonical_json_bytes(&value)?;
            let encoded = wow_core::canonical_json_bytes(&request)?;
            assert_eq!(encoded, expected);
            let roundtrip =
                ContextCombinedRequest::from_json_slice(&encoded, &AtomicBool::new(false))?;
            assert_eq!(wow_core::canonical_json_bytes(&roundtrip)?, expected);
        }
    }
    for mut value in [inspect("l0", None)?, build("l0", None)?] {
        value["primary"]["root"] = json!({"kind":"project","id":"fixture-primary"});
        value["primary"]["map_profile_id"] = json!(PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID);
        let key = if value["operation"] == "inspect" {
            "inspection"
        } else {
            "build"
        };
        for selected in [false, true] {
            if selected {
                value[key]["platform_root_facet"] = json!("callable_metadata");
            }
            let request = ContextCombinedRequest::from_json_slice(
                &serde_json::to_vec(&value)?,
                &AtomicBool::new(false),
            )?;
            assert_eq!(
                wow_core::canonical_json_bytes(&request)?,
                wow_core::canonical_json_bytes(&value)?,
            );
        }
        for level in ["l1", "both"] {
            value[key]["level"] = json!(level);
            refused(&value, ServiceErrorCode::InvalidRequest)?;
        }
    }
    Ok(())
}

#[test]
fn platform_callable_selection_refuses_invalid_and_mixed_intent_before_acquisition() -> TestResult {
    for level in ["l0", "l1", "both"] {
        for mut value in [inspect(level, None)?, build(level, None)?] {
            let key = if value["operation"] == "inspect" {
                "inspection"
            } else {
                "build"
            };
            for invalid in [
                Value::Null,
                json!(false),
                json!(1),
                json!([]),
                json!({"callable_metadata": null}),
                json!("default"),
                json!("unknown"),
            ] {
                value[key]["platform_root_facet"] = invalid;
                refused(&value, ServiceErrorCode::InvalidRequest)?;
            }
            value[key]["platform_root_facet"] = json!("callable_metadata");

            value[key]["level"] = json!("l2");
            refused(&value, ServiceErrorCode::InvalidRequest)?;
        }
    }
    let mut map = fixture()?;
    map["operation"] = json!("map");
    map.as_object_mut().ok_or("request object")?.remove("build");
    // Map has no inspection/build selection or top-level facet selector.
    for field in ["inspection", "build", "platform_root_facet"] {
        let mut value = map.clone();
        value[field] = match field {
            "inspection" => json!({"level":"l0","platform_root_facet":"callable_metadata"}),
            "build" => {
                let mut selection = fixture()?["build"].clone();
                selection["platform_root_facet"] = json!("callable_metadata");
                selection
            }
            _ => json!("callable_metadata"),
        };
        refused(&value, ServiceErrorCode::InvalidRequest)?;
    }
    Ok(())
}

#[test]
fn platform_xml_metadata_selectors_are_closed_and_additive() -> TestResult {
    for (selector, expected_facet) in [
        (
            "xml_handler_metadata",
            wow_context::PlatformRootFacet::XmlHandlerMetadata,
        ),
        (
            "xml_declaration_metadata",
            wow_context::PlatformRootFacet::XmlDeclarationMetadata,
        ),
    ] {
        for level in ["l0", "l1", "both"] {
            for omitted in [inspect(level, None)?, build(level, None)?] {
                let request = ContextCombinedRequest::from_json_slice(
                    &serde_json::to_vec(&omitted)?,
                    &AtomicBool::new(false),
                )?;
                assert!(request.platform_root_facet.is_none());
                assert_eq!(
                    wow_core::canonical_json_bytes(&request)?,
                    wow_core::canonical_json_bytes(&omitted)?,
                );

                let mut selected = omitted;
                let key = if selected["operation"] == "inspect" {
                    "inspection"
                } else {
                    "build"
                };
                selected[key]["platform_root_facet"] = json!(selector);
                let request = ContextCombinedRequest::from_json_slice(
                    &serde_json::to_vec(&selected)?,
                    &AtomicBool::new(false),
                )?;
                assert_eq!(request.platform_root_facet, Some(expected_facet));
                assert!(request.l0_recipe.is_none() && request.l1_recipe.is_none());
                let expected = wow_core::canonical_json_bytes(&selected)?;
                assert_eq!(wow_core::canonical_json_bytes(&request)?, expected);
                let roundtrip =
                    ContextCombinedRequest::from_json_slice(&expected, &AtomicBool::new(false))?;
                assert_eq!(wow_core::canonical_json_bytes(&roundtrip)?, expected);
            }

            for mut value in [inspect(level, None)?, build(level, None)?] {
                let key = if value["operation"] == "inspect" {
                    "inspection"
                } else {
                    "build"
                };
                for invalid in [
                    Value::Null,
                    json!(false),
                    json!(1),
                    json!([]),
                    json!({(selector): null}),
                    json!("default"),
                    json!("unknown"),
                ] {
                    value[key]["platform_root_facet"] = invalid;
                    refused(&value, ServiceErrorCode::InvalidRequest)?;
                }
                value[key]["platform_root_facet"] = json!(selector);
            }
        }
    }
    Ok(())
}
