//! Pure request decoding compatibility and admission coverage.
use super::*;
use crate::context::{PRIMARY_L1_PROFILE_ID, PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID};
use serde_json::{Value, json};
use std::sync::atomic::AtomicBool;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

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

    // Explicit selection survives actual decode/serialize/decode for both operations.
    for (profile, value) in [
        (
            PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
            inspect("l1", Some(PRIMARY_STATE_DETAIL_L1_PROFILE_ID))?,
        ),
        (
            PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
            build("l1", Some(PRIMARY_STATE_DETAIL_L1_PROFILE_ID))?,
        ),
        (
            PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID,
            inspect("l1", Some(PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID))?,
        ),
        (
            PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID,
            build("l1", Some(PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID))?,
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
        ("both", PRIMARY_STATE_DETAIL_L1_PROFILE_ID),
        ("l0", PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID),
        ("both", PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID),
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
        inspect("l1", Some(PRIMARY_STATE_DETAIL_L1_PROFILE_ID))?,
        build("l1", Some(PRIMARY_STATE_DETAIL_L1_PROFILE_ID))?,
        inspect("l1", Some(PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID))?,
        build("l1", Some(PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID))?,
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
