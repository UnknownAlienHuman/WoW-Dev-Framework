//! Bounded validation for the two reviewed machine-state owners.
use crate::Result;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const ANNOTATION_CONTRACT: &str = "crates/wow-annotations/e1/CONTRACT.json";
const ANNOTATION_MANIFEST: &str = "crates/wow-annotations/e1/examples/CHECKSUMS.json";
const PROJECT_CONTRACT: &str = "crates/wow-project/CONTRACT.json";
const PROJECT_MANIFEST: &str = "crates/wow-project/examples/CHECKSUMS.json";

struct CapabilitySpec {
    name: &'static str,
    owners: &'static [(&'static str, &'static str)],
    application_routes: &'static [(&'static str, &'static str)],
}

struct OwnerSpec {
    id: &'static str,
    contract_path: &'static str,
    manifest_path: &'static str,
    manifest_id: &'static str,
    capabilities: &'static [CapabilitySpec],
    manifest_history_is_object: bool,
}

const PROJECT_CAPABILITIES: &[CapabilitySpec] = &[
    CapabilitySpec {
        name: "native-project-publication",
        owners: &[
            (
                "crates/wow-project/src/publication.rs",
                "publish_initial_cancellable",
            ),
            (
                "crates/wow-project/src/publication.rs",
                "apply_update_cancellable",
            ),
            ("crates/wow-project/src/snapshot.rs", "open_view"),
        ],
        application_routes: &[
            ("crates/wow-service/src/local/backend.rs", "acquire_project"),
            ("apps/wow/src/project.rs", "run"),
        ],
    },
    CapabilitySpec {
        name: "native-library-byte-view",
        owners: &[
            ("crates/wow-project/src/library_view.rs", "qualify"),
            ("crates/wow-project/src/library_view.rs", "file_view"),
            ("crates/wow-project/src/library_view.rs", "slice"),
        ],
        application_routes: &[
            (
                "crates/wow-service/src/graph/build/native_api.rs",
                "reference_origins_prepared",
            ),
            ("apps/wow/src/context.rs", "run"),
        ],
    },
];

const ANNOTATION_CAPABILITIES: &[CapabilitySpec] = &[
    CapabilitySpec {
        name: "native-library-projection",
        owners: &[(
            "crates/wow-annotations/src/native.rs",
            "project_with_alias_catalogs",
        )],
        application_routes: &[
            (
                "crates/wow-service/src/local/native_library_input.rs",
                "rebuild_for_project",
            ),
            ("apps/wow/src/native.rs", "run"),
        ],
    },
    CapabilitySpec {
        name: "source-bound-callable-lowering",
        owners: &[(
            "crates/wow-annotations/src/reference_links.rs",
            "callable_lowering",
        )],
        application_routes: &[
            (
                "crates/wow-service/src/graph/build/native_api.rs",
                "reference_origins_prepared",
            ),
            ("apps/wow/src/context.rs", "run"),
        ],
    },
    CapabilitySpec {
        name: "native-artifact-and-projection-sidecars",
        owners: &[
            (
                "crates/wow-annotations/src/artifact.rs",
                "from_native_library",
            ),
            (
                "crates/wow-annotations/src/sidecars.rs",
                "from_library_cancellable",
            ),
        ],
        application_routes: &[
            (
                "crates/wow-service/src/local/native_input.rs",
                "from_native_manifest",
            ),
            ("apps/wow/src/native.rs", "run"),
        ],
    },
];

const OWNERS: &[OwnerSpec] = &[
    OwnerSpec {
        id: "wow-annotations/e1-projection-and-parity",
        contract_path: ANNOTATION_CONTRACT,
        manifest_path: ANNOTATION_MANIFEST,
        manifest_id: "wow-annotations-e1-pending-acceptance-v2",
        capabilities: ANNOTATION_CAPABILITIES,
        manifest_history_is_object: false,
    },
    OwnerSpec {
        id: "wow-project/e0-d/1",
        contract_path: PROJECT_CONTRACT,
        manifest_path: PROJECT_MANIFEST,
        manifest_id: "wow-project-e0-d-pending-acceptance-v2",
        capabilities: PROJECT_CAPABILITIES,
        manifest_history_is_object: true,
    },
];

#[derive(Default)]
pub(super) struct Scope {
    documents: BTreeMap<&'static str, Value>,
}

impl Scope {
    pub(super) fn observe(&mut self, path: &str, value: &Value) -> Result<()> {
        if let Some(owner) = OWNERS
            .iter()
            .find(|owner| path == owner.contract_path || path == owner.manifest_path)
        {
            let key = if path == owner.contract_path {
                owner.contract_path
            } else {
                owner.manifest_path
            };
            if self.documents.insert(key, value.clone()).is_some() {
                return Err("machine-state document appears twice in inventory".into());
            }
        }
        Ok(())
    }

    pub(super) fn finish(self, files: &BTreeSet<&str>) -> Result<()> {
        if !OWNERS.iter().any(|owner| {
            files.contains(owner.contract_path)
                || files.contains(owner.manifest_path)
                || owner.capabilities.iter().any(|capability| {
                    capability
                        .owners
                        .iter()
                        .any(|(path, _)| files.contains(path))
                })
        }) {
            return Ok(());
        }
        for owner in OWNERS {
            let contract = self
                .documents
                .get(owner.contract_path)
                .ok_or("machine-state contract is absent")?;
            let manifest = self
                .documents
                .get(owner.manifest_path)
                .ok_or("machine-state checksum manifest is absent")?;
            validate_contract(owner, contract, files)?;
            validate_manifest(owner, manifest)?;
        }
        Ok(())
    }
}

fn validate_contract(owner: &OwnerSpec, value: &Value, files: &BTreeSet<&str>) -> Result<()> {
    common_state(value, "partial-functional-freeze-open")?;
    if value["schema_version"].as_u64() != Some(2) || text(value, "contract_id")? != owner.id {
        return Err("machine-state contract identity/schema mismatch".into());
    }
    if !value
        .get("historical_preimplementation_requirements")
        .is_some_and(Value::is_array)
        || value.get("implementation_gate").is_some()
        || value.get("freeze_gate").is_some()
        || value.get("freeze_rules").is_some()
    {
        return Err(
            "contract must retain historical requirements without an active old gate".into(),
        );
    }
    let checksum = value
        .get("checksum_contract")
        .ok_or("checksum contract is absent")?;
    if text(checksum, "state")? != "pending-acceptance-freeze"
        || text(checksum, "manifest")? != owner.manifest_path
        || text(checksum, "rule")?.is_empty()
    {
        return Err("contract checksum rule must remain structurally pending".into());
    }
    validate_checkpoint(
        value
            .get("functional_checkpoint")
            .ok_or("functional checkpoint is absent")?,
    )?;
    validate_capabilities(owner, value, files)
}

fn validate_manifest(owner: &OwnerSpec, value: &Value) -> Result<()> {
    common_state(value, "pending-acceptance-freeze")?;
    if value["schema_version"].as_u64() != Some(2)
        || text(value, "manifest_id")? != owner.manifest_id
        || text(value, "contract_id")? != owner.id
    {
        return Err("machine-state manifest identity/schema mismatch".into());
    }
    let historical = value
        .get("historical_preimplementation_requirements")
        .ok_or("historical manifest requirements are absent")?;
    if if owner.manifest_history_is_object {
        !historical.is_object()
    } else {
        !historical.is_array()
    } || value.get("freeze_gate").is_some()
        || value.get("freeze_rules").is_some()
    {
        return Err("manifest historical requirements have an invalid shape".into());
    }
    let rules = value
        .get("acceptance_rules")
        .and_then(Value::as_array)
        .ok_or("acceptance_rules must be an array")?;
    if rules.is_empty() || rules.iter().any(|rule| !rule.is_string()) {
        return Err("acceptance_rules must contain strings".into());
    }
    // A pending manifest cannot claim a frozen output. Other pending pins remain data.
    let digest = if owner.id == "wow-annotations/e1-projection-and-parity" {
        value
            .get("bundle")
            .and_then(|bundle| bundle.get("bundle_sha256"))
    } else {
        value.get("bundle_sha256")
    };
    if !digest.is_some_and(Value::is_null) {
        return Err("pending manifest cannot claim a frozen bundle digest".into());
    }
    Ok(())
}

fn common_state(value: &Value, expected_state: &str) -> Result<()> {
    if text(value, "state")? != expected_state
        || text(value, "implementation_status")? != "partial-functional"
        || text(value, "acceptance_status")? != "pending"
        || value["canonical_issue"].as_u64() != Some(120)
        || value.get("implementation_state").is_some()
    {
        return Err(
            "machine state must remain partial-functional and pending on Issue #120".into(),
        );
    }
    let freeze = value
        .get("preimplementation_freeze")
        .ok_or("preimplementation freeze record is absent")?;
    exact(freeze, &["status", "evaluation"])?;
    if text(freeze, "status")? != "missed" || text(freeze, "evaluation")? != "NotEvaluated" {
        return Err("preimplementation freeze must remain missed/NotEvaluated".into());
    }
    Ok(())
}

fn validate_checkpoint(value: &Value) -> Result<()> {
    exact(value, &["commit", "quick_run"])?;
    let commit = text(value, "commit")?;
    if commit.len() != 40
        || !commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || value["quick_run"].as_u64().is_none_or(|run| run == 0)
    {
        return Err(
            "functional checkpoint must contain lowercase 40-hex commit and positive run".into(),
        );
    }
    Ok(())
}

fn validate_capabilities(
    owner: &OwnerSpec,
    contract: &Value,
    files: &BTreeSet<&str>,
) -> Result<()> {
    let rows = contract
        .get("capability_routing")
        .and_then(Value::as_array)
        .ok_or("capability_routing must be an array")?;
    let mut names = BTreeSet::new();
    for row in rows {
        exact(
            row,
            &[
                "capability",
                "status",
                "owners",
                "application_routes",
                "acceptance_status",
                "blocking_issues",
            ],
        )?;
        let name = text(row, "capability")?;
        if !owner
            .capabilities
            .iter()
            .any(|expected| expected.name == name)
        {
            return Err("capability is outside the explicit owner mapping".into());
        }
        if !names.insert(name) {
            return Err("capability routing contains a duplicate".into());
        }
        text(row, "status")?;
        text(row, "acceptance_status")?;
        if !row["blocking_issues"].is_array() {
            return Err("blocking_issues must be an array".into());
        }
        routes(row.get("owners").ok_or("owners are absent")?, files)?;
        routes(
            row.get("application_routes")
                .ok_or("application_routes are absent")?,
            files,
        )?;
    }
    for expected in owner.capabilities {
        let row = rows
            .iter()
            .find(|row| row["capability"].as_str() == Some(expected.name))
            .ok_or("required selected capability is absent")?;
        if text(row, "status")? != "partial-functional"
            || text(row, "acceptance_status")? != "pending"
        {
            return Err("selected capability must remain partial-functional/pending".into());
        }
        require_routes(&row["owners"], expected.owners)?;
        require_routes(&row["application_routes"], expected.application_routes)?;
        let blockers = row["blocking_issues"]
            .as_array()
            .ok_or("blocking_issues must be an array")?;
        if !blockers.iter().any(|issue| issue.as_u64() == Some(120)) {
            return Err("selected capability must retain Issue #120".into());
        }
    }
    Ok(())
}

fn routes(value: &Value, files: &BTreeSet<&str>) -> Result<()> {
    let rows = value.as_array().ok_or("routes must be an array")?;
    let mut seen = BTreeSet::new();
    for row in rows {
        exact(row, &["path", "symbol"])?;
        let path = text(row, "path")?;
        let symbol = text(row, "symbol")?;
        crate::manifest::validate_path(path)?;
        if !path.ends_with(".rs")
            || !files.contains(path)
            || symbol.trim().is_empty()
            || symbol.trim() != symbol
            || symbol.chars().any(char::is_control)
            || !seen.insert((path, symbol))
        {
            return Err("route must name a checked Rust path and unique explicit symbol".into());
        }
    }
    Ok(())
}

fn require_routes(value: &Value, expected: &[(&str, &str)]) -> Result<()> {
    let rows = value.as_array().ok_or("routes must be an array")?;
    if rows.len() != expected.len() {
        return Err("route set differs from the explicit owner mapping".into());
    }
    for (path, symbol) in expected {
        if !rows
            .iter()
            .any(|row| row["path"].as_str() == Some(path) && row["symbol"].as_str() == Some(symbol))
        {
            return Err("required explicit owner/application route is missing".into());
        }
    }
    Ok(())
}

fn text<'a>(value: &'a Value, field: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{field} must be a string").into())
}

fn exact(value: &Value, fields: &[&str]) -> Result<()> {
    let object = value
        .as_object()
        .ok_or("machine-state record must be an object")?;
    if object.len() != fields.len() || object.keys().any(|key| !fields.contains(&key.as_str())) {
        return Err("machine-state record has missing or unknown fields".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid(spec: &OwnerSpec) -> (Value, Value, BTreeSet<&'static str>) {
        let mut files = BTreeSet::from([spec.contract_path, spec.manifest_path]);
        let mut routing = Vec::new();
        for capability in spec.capabilities {
            let owners = capability
                .owners
                .iter()
                .map(|(path, symbol)| {
                    files.insert(*path);
                    json!({"path":path,"symbol":symbol})
                })
                .collect::<Vec<_>>();
            let routes = capability
                .application_routes
                .iter()
                .map(|(path, symbol)| {
                    files.insert(*path);
                    json!({"path":path,"symbol":symbol})
                })
                .collect::<Vec<_>>();
            routing.push(json!({"capability":capability.name,"status":"partial-functional",
                "owners":owners,"application_routes":routes,"acceptance_status":"pending","blocking_issues":[120,122]}));
        }
        let contract = json!({"schema_version":2,"contract_id":spec.id,"state":"partial-functional-freeze-open",
            "implementation_status":"partial-functional","acceptance_status":"pending","canonical_issue":120,
            "preimplementation_freeze":{"status":"missed","evaluation":"NotEvaluated"},
            "functional_checkpoint":{"commit":"abcdefabcdefabcdefabcdefabcdefabcdefabcd","quick_run":7},
            "historical_preimplementation_requirements":[],
            "checksum_contract":{"state":"pending-acceptance-freeze","manifest":spec.manifest_path,"rule":"pending"},
            "capability_routing":routing});
        let history = if spec.manifest_history_is_object {
            json!({})
        } else {
            json!([])
        };
        let mut manifest = json!({"schema_version":2,"manifest_id":spec.manifest_id,"contract_id":spec.id,
            "state":"pending-acceptance-freeze","implementation_status":"partial-functional",
            "acceptance_status":"pending","canonical_issue":120,
            "preimplementation_freeze":{"status":"missed","evaluation":"NotEvaluated"},
            "historical_preimplementation_requirements":history,
            "acceptance_rules":["pending"]});
        if spec.id == "wow-annotations/e1-projection-and-parity" {
            manifest["bundle"] = json!({"bundle_sha256":null});
        } else {
            manifest["bundle_sha256"] = Value::Null;
        }
        (contract, manifest, files)
    }

    fn check(
        spec: &OwnerSpec,
        contract: Value,
        manifest: Value,
        files: &BTreeSet<&str>,
    ) -> Result<()> {
        validate_contract(spec, &contract, files)?;
        validate_manifest(spec, &manifest)
    }

    #[test]
    fn both_explicit_owners_accept_partial_pending_state() -> Result<()> {
        for spec in OWNERS {
            let (contract, manifest, files) = valid(spec);
            check(spec, contract, manifest, &files)?;
        }
        Ok(())
    }

    #[test]
    fn representative_stale_and_frozen_mutations_are_rejected() -> Result<()> {
        let spec = &OWNERS[0];
        let (contract, manifest, files) = valid(spec);
        let mut stale = contract.clone();
        stale["state"] = json!("implementation-ready-documentation-no-rust-code");
        assert!(check(spec, stale, manifest.clone(), &files).is_err());
        let mut legacy_state = contract.clone();
        legacy_state["implementation_state"] = json!("not-started");
        assert!(check(spec, legacy_state, manifest.clone(), &files).is_err());
        let mut unknown_capability = contract.clone();
        unknown_capability["capability_routing"][0]["capability"] = json!("future-owner");
        assert!(check(spec, unknown_capability, manifest.clone(), &files).is_err());
        let mut missing_owner = contract.clone();
        missing_owner["capability_routing"][0]["owners"] = json!([]);
        assert!(check(spec, missing_owner, manifest.clone(), &files).is_err());
        let mut missing_route = contract.clone();
        missing_route["capability_routing"][0]["application_routes"] = json!([]);
        assert!(check(spec, missing_route, manifest.clone(), &files).is_err());
        let mut extra_route = contract.clone();
        let mut owner_routes = extra_route["capability_routing"][0]["owners"]
            .as_array()
            .ok_or("fixture owners must be an array")?
            .clone();
        owner_routes
            .push(json!({"path": spec.capabilities[0].owners[0].0, "symbol": "fabricated_symbol"}));
        extra_route["capability_routing"][0]["owners"] = Value::Array(owner_routes);
        assert!(check(spec, extra_route, manifest.clone(), &files).is_err());
        let mut invalid_checkpoint = contract.clone();
        invalid_checkpoint["functional_checkpoint"]["commit"] = json!("UPPERCASE");
        assert!(check(spec, invalid_checkpoint, manifest.clone(), &files).is_err());
        let mut missing_file = files.clone();
        missing_file.remove("crates/wow-annotations/src/native.rs");
        assert!(check(spec, contract.clone(), manifest.clone(), &missing_file).is_err());
        let mut accepted = manifest.clone();
        accepted["acceptance_status"] = json!("accepted");
        assert!(check(spec, contract.clone(), accepted, &files).is_err());
        let mut frozen = manifest;
        frozen["bundle"]["bundle_sha256"] = json!("digest");
        assert!(check(spec, contract, frozen, &files).is_err());
        Ok(())
    }

    #[test]
    fn implemented_owner_without_machine_records_is_rejected() {
        let files = BTreeSet::from(["crates/wow-project/src/publication.rs"]);
        assert!(Scope::default().finish(&files).is_err());
    }

    #[test]
    fn unrelated_future_only_inventory_is_out_of_scope() -> Result<()> {
        Scope::default().finish(&BTreeSet::from(["docs/future-contract.json"]))
    }
}
