//! Bind reviewed semantic fixtures to genuine prepared physical store identities.
//! This helper never accepts or normalizes a consuming operation's output.
use super::*;

pub(super) fn runtime_bound_fixture(
    frozen: &[u8],
    frozen_request: &[u8],
    generation: &wow_store::project::StoreGenerationId,
    current: &CurrentPublication,
    host: &Value,
) -> TestResult<(Vec<u8>, Vec<u8>)> {
    let mut envelope: Value = serde_json::from_slice(frozen)?;
    assert_eq!(
        wow_core::canonical_json_bytes(&envelope["request"])?,
        frozen_request,
        "the independently frozen request still matches the reviewed envelope",
    );
    let operation = envelope["operation"].as_str().ok_or("fixture operation")?;
    let operation = match operation {
        "context_map" => "map",
        "context_inspect" => "inspect",
        "context_build" => "build",
        _ => return Err("unsupported frozen fixture operation".into()),
    };
    assert_eq!(
        envelope["request"]["publication"]["kind"],
        "store_generation"
    );
    let old_current = envelope["payload"]["current_at_acquisition"]
        .as_object()
        .ok_or("frozen Current observation")?;
    let prepared_current = serde_json::to_value(current)?;
    let prepared_fields = prepared_current.as_object().ok_or("prepared Current")?;
    assert_eq!(
        old_current.keys().collect::<BTreeSet<_>>(),
        prepared_fields.keys().collect::<BTreeSet<_>>()
    );
    assert!(old_current.keys().all(|key| matches!(
        key.as_str(),
        "epoch_id" | "generation_id" | "predecessor" | "record_id" | "validation_id"
    )));

    // These values come from the independent publication/read owner, not from
    // the Service result under test. Every other frozen field stays untouched.
    envelope["request"]["publication"]["id"] = json!(generation.as_str());
    envelope["payload"]["store_generation_id"] = json!(generation.as_str());
    envelope["payload"]["current_at_acquisition"] = prepared_current;
    envelope["payload"]["current_record_digest"] = json!(observed_current_digest(current)?);
    let request = envelope["request"].clone();
    let request_domain = format!("wow-service/context-{operation}-request/1");
    let request_id = format!(
        "context-{operation}-request:{}",
        ContentDigest::<CanonicalResult>::from_bytes(wow_core::domain_separated_digest(
            &request_domain,
            &request
        )?)
    );

    let publication = &envelope["payload"]["publication_set_id"];
    let material = if operation == "map" {
        let reference = &request["reference"];
        let root = match request["root"]["kind"].as_str() {
            Some("graph_entity") => request["root"]["id"].clone(),
            Some("project") => request["root"].clone(),
            _ => return Err("frozen map root kind".into()),
        };
        json!([
            request_id,
            publication,
            generation.as_str(),
            reference["profile"],
            reference["generation"],
            reference["view"],
            root,
            host["limits"]
        ])
    } else {
        // The universe identity is a frozen semantic value. It must not be
        // obtained from a newly observed response merely to bless that response.
        let mut pending = vec![&envelope["payload"]];
        let mut universes = BTreeSet::new();
        while let Some(value) = pending.pop() {
            match value {
                Value::Object(fields) => {
                    if let Some(id) = fields.get("universe_set_id") {
                        universes.insert(id.as_str().ok_or("frozen universe identity")?);
                    }
                    pending.extend(fields.values());
                }
                Value::Array(values) => pending.extend(values),
                _ => {}
            }
        }
        assert_eq!(universes.len(), 1, "one exact frozen semantic universe");
        let universe = universes.first().ok_or("missing frozen universe")?;
        json!([
            request_id,
            publication,
            generation.as_str(),
            universe,
            request["root"],
            host["limits"]
        ])
    };
    let resolved_domain = format!("wow-service/context-{operation}-resolved-request/1");
    let resolved_id = format!(
        "context-{operation}-resolved-request:{}",
        ContentDigest::<CanonicalResult>::from_bytes(wow_core::domain_separated_digest(
            &resolved_domain,
            &material
        )?)
    );
    envelope["request_id"] = json!(request_id);
    envelope["payload"]["resolved_request_id"] = json!(resolved_id);
    Ok((
        wow_core::canonical_json_bytes(&envelope)?,
        wow_core::canonical_json_bytes(&request)?,
    ))
}
