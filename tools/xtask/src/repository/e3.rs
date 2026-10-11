//! E3-A reconciliation declarations, not source analysis or acceptance verification.
use crate::Result;
use serde_json::{Map, Value};
use std::collections::BTreeSet;

const ROOT: &str = "crates/wow-project/e3/";
const CONTRACT: &str = "crates/wow-project/e3/CONTRACT.json";
const CHECKSUMS: &str = "crates/wow-project/e3/examples/CHECKSUMS.json";
const CONTRACT_ID: &str = "wow-project/e3-a/blizzard-ui-source-universe-index";
const OPERATIONS: &[&str] = &[
    "validate_blizzard_ui_index_request",
    "validate_blizzard_ui_source_profile",
    "validate_blizzard_ui_source_snapshot",
    "select_blizzard_ui_packages_and_toc_variants",
    "build_blizzard_ui_load_model",
    "build_blizzard_ui_analyzer_plan",
    "validate_blizzard_ui_analyzer_snapshot",
    "build_blizzard_ui_recognizer_fact_bundles",
    "run_blizzard_ui_core_recognizers",
    "validate_blizzard_ui_graph_proposals",
    "assemble_blizzard_ui_index_candidate",
    "validate_blizzard_ui_index_candidate",
    "build_blizzard_ui_publication_bundle",
    "publish_blizzard_ui_index",
    "open_blizzard_ui_project_view",
    "open_blizzard_ui_skeleton_input_view",
    "plan_blizzard_ui_incremental_update",
];

#[derive(Default)]
pub(super) struct Scope {
    contract: Option<Value>,
    checksums: Option<Value>,
}

impl Scope {
    // Values come from the existing bounded inventory reader and strict decoder.
    pub(super) fn observe(&mut self, path: &str, value: Value) -> Result<()> {
        let slot = match path {
            CONTRACT => &mut self.contract,
            CHECKSUMS => &mut self.checksums,
            _ => return Ok(()),
        };
        if slot.replace(value).is_some() {
            return Err("E3 reconciliation document appears twice in inventory".into());
        }
        Ok(())
    }

    pub(super) fn finish(self, files: &BTreeSet<&str>) -> Result<()> {
        if self.contract.is_none()
            && self.checksums.is_none()
            && !files.iter().any(|path| path.starts_with(ROOT))
        {
            return Ok(());
        }
        let contract = self.contract.ok_or("E3 scope lacks CONTRACT.json")?;
        let checksums = self.checksums.ok_or("E3 scope lacks CHECKSUMS.json")?;
        pending_status(&contract)?;
        pending_status(&checksums)?;
        if text(&contract, "contract_id")? != CONTRACT_ID
            || text(&contract, "work_package")? != "E3-A"
            || text(&contract, "crate")? != "wow-project"
            || text(&contract, "state")? != "partial-functional-freeze-open"
            || text(&checksums, "manifest_id")? != "wow-project-e3-a-pending-acceptance-v2"
            || text(&checksums, "state")? != "pending-acceptance-freeze"
            || text(&checksums, "algorithm")? != "sha256"
        {
            return Err("E3 reconciliation identity/state mismatch".into());
        }
        operation_routes(&contract, files)?;
        // Preserve pending acceptance inputs without treating them as frozen evidence.
        for field in [
            "prerequisite_pins",
            "source_provider_pins",
            "profile_pins",
            "synthetic_fixture_pins",
            "real_mirror_fixture_pins",
            "frozen_vectors",
        ] {
            if !checksums[field]
                .as_object()
                .is_some_and(|map| !map.is_empty())
            {
                return Err("E3 pending acceptance pin group is missing or invalid".into());
            }
        }
        if text(&checksums["profile_pins"], "e3_a_contract_id")? != CONTRACT_ID
            || !checksums["member_sha256"]
                .as_object()
                .is_some_and(Map::is_empty)
            || !matches!(checksums.get("bundle_sha256"), Some(Value::Null))
        {
            return Err("E3 pending acceptance manifest cannot claim frozen checksums".into());
        }
        let members = list(&checksums, "member_paths")?;
        if members.is_empty() {
            return Err("E3 pending acceptance member list is empty".into());
        }
        let mut unique = BTreeSet::new();
        for member in members {
            let path = member
                .as_str()
                .ok_or("E3 acceptance member must be a path")?;
            crate::manifest::validate_path(path)?;
            if !files.contains(path) || !unique.insert(path) {
                return Err("E3 acceptance member is absent or repeated".into());
            }
        }
        Ok(())
    }
}

fn pending_status(value: &Value) -> Result<()> {
    if value["schema_version"].as_u64() != Some(2)
        || text(value, "implementation_status")? != "partial-functional"
        || text(value, "acceptance_status")? != "pending"
        || ["implementation_state", "freeze_gate", "freeze_rules"]
            .iter()
            .any(|field| value.get(*field).is_some())
    {
        return Err(
            "E3 requires reconciled v2 partial implementation and pending acceptance".into(),
        );
    }
    let historical = &value["preimplementation_freeze"];
    exact(historical, &["status", "evaluation"])?;
    if text(historical, "status")? != "missed" || text(historical, "evaluation")? != "NotEvaluated"
    {
        return Err("E3 preimplementation freeze must remain missed/NotEvaluated".into());
    }
    Ok(())
}

fn operation_routes(contract: &Value, files: &BTreeSet<&str>) -> Result<()> {
    let mut operations = BTreeSet::new();
    for operation in list(contract, "operations")? {
        let name = operation.as_str().ok_or("E3 operation must be a string")?;
        if !OPERATIONS.contains(&name) || !operations.insert(name) {
            return Err("E3 operation is unknown or repeated".into());
        }
    }
    let routing = contract["operation_routing"]
        .as_object()
        .ok_or("E3 operation_routing must be an object")?;
    if operations.len() != OPERATIONS.len()
        || routing.len() != operations.len()
        || routing
            .keys()
            .any(|name| !operations.contains(name.as_str()))
    {
        return Err(
            "E3 operation list and routing must cover the exact declared operation set".into(),
        );
    }
    let mut implemented = false;
    for name in operations {
        let row = routing.get(name).ok_or("E3 operation routing is missing")?;
        exact(
            row,
            &["status", "owners", "application_routes", "blocking_issues"],
        )?;
        let owners = routes(&row["owners"], files)?;
        let applications = routes(&row["application_routes"], files)?;
        let mut issues = BTreeSet::new();
        for value in list(row, "blocking_issues")? {
            let issue = value
                .as_u64()
                .filter(|id| *id > 0)
                .ok_or("invalid E3 blocker issue")?;
            if !issues.insert(issue) {
                return Err("E3 blocker issue is repeated".into());
            }
        }
        let available = match text(row, "status")? {
            "application-selected" if owners > 0 && applications > 0 => true,
            "library-only" if owners > 0 && applications == 0 => true,
            "partial" if owners > 0 => true,
            "specified-only" if owners == 0 && applications == 0 && issues.is_empty() => false,
            "blocked" if applications == 0 && !issues.is_empty() => false,
            _ => return Err("E3 operation status/routing/blocker contradiction".into()),
        };
        implemented |= available;
    }
    if !implemented {
        return Err(
            "E3 partial implementation requires an explicitly routed implemented operation".into(),
        );
    }
    Ok(())
}

fn routes(value: &Value, files: &BTreeSet<&str>) -> Result<usize> {
    let routes = value.as_array().ok_or("E3 routes must be an array")?;
    let mut unique = BTreeSet::new();
    for route in routes {
        exact(route, &["path", "symbol"])?;
        let path = text(route, "path")?;
        let symbol = text(route, "symbol")?;
        crate::manifest::validate_path(path)?;
        // Validate declared anchors, never grep Rust source for symbol spellings.
        if !path.ends_with(".rs")
            || !files.contains(path)
            || symbol.is_empty()
            || symbol.trim() != symbol
            || symbol.chars().any(char::is_control)
            || !unique.insert((path, symbol))
        {
            return Err(
                "E3 route must name a checked Rust file and a unique nonempty symbol".into(),
            );
        }
    }
    Ok(routes.len())
}

fn text<'a>(value: &'a Value, field: &str) -> Result<&'a str> {
    value[field]
        .as_str()
        .ok_or_else(|| format!("E3 {field} must be a string").into())
}

fn list<'a>(value: &'a Value, field: &str) -> Result<&'a [Value]> {
    value[field]
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| format!("E3 {field} must be an array").into())
}

fn exact(value: &Value, fields: &[&str]) -> Result<()> {
    let object = value
        .as_object()
        .ok_or("E3 status/route must be an object")?;
    if object.len() != fields.len() || object.keys().any(|key| !fields.contains(&key.as_str())) {
        return Err("E3 status/route has missing or unknown fields".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> (Value, Value, BTreeSet<&'static str>) {
        let owner = "crates/wow-project/src/platform_source/profile.rs";
        // Policy fixture only; root must review the real 17-operation classifications.
        let mut routing = Map::new();
        for name in OPERATIONS {
            routing.insert(
                (*name).into(),
                json!({"status":"specified-only","owners":[],"application_routes":[],"blocking_issues":[]}),
            );
        }
        routing.insert(
            "validate_blizzard_ui_source_profile".into(),
            json!({
                "status":"library-only",
                "owners":[{"path":owner,"symbol":"BlizzardUiSourceProfile::validate"}],
                "application_routes":[], "blocking_issues":[]
            }),
        );
        let historical = json!({"status":"missed","evaluation":"NotEvaluated"});
        let contract = json!({
            "schema_version":2, "contract_id":CONTRACT_ID, "work_package":"E3-A", "crate":"wow-project",
            "state":"partial-functional-freeze-open", "implementation_status":"partial-functional",
            "acceptance_status":"pending", "preimplementation_freeze":historical.clone(),
            "operations":OPERATIONS, "operation_routing":routing
        });
        let checksums = json!({
            "schema_version":2, "manifest_id":"wow-project-e3-a-pending-acceptance-v2", "algorithm":"sha256",
            "state":"pending-acceptance-freeze", "implementation_status":"partial-functional",
            "acceptance_status":"pending", "preimplementation_freeze":historical,
            "prerequisite_pins":{"commit":null}, "source_provider_pins":{"revision":null},
            "profile_pins":{"e3_a_contract_id":CONTRACT_ID,"profile":null},
            "synthetic_fixture_pins":{"generation":null}, "real_mirror_fixture_pins":{"generation":null},
            "frozen_vectors":{"vector":null}, "member_paths":[CONTRACT],
            "member_sha256":{}, "bundle_sha256":null
        });
        (
            contract,
            checksums,
            BTreeSet::from([CONTRACT, CHECKSUMS, owner]),
        )
    }

    fn check(contract: Value, checksums: Value, files: &BTreeSet<&str>) -> Result<()> {
        let mut scope = Scope::default();
        scope.observe(CONTRACT, contract)?;
        scope.observe(CHECKSUMS, checksums)?;
        scope.finish(files)
    }

    #[test]
    fn e3_reconciliation_is_scoped_and_never_accepts_pending_pins() -> Result<()> {
        Scope::default().finish(&BTreeSet::from(["README.md"]))?;
        assert!(
            Scope::default()
                .finish(&BTreeSet::from([CONTRACT]))
                .is_err()
        );
        let (contract, checksums, files) = fixture();
        check(contract.clone(), checksums.clone(), &files)?;
        let mut incomplete = Scope::default();
        incomplete.observe(CONTRACT, contract.clone())?;
        assert!(incomplete.finish(&files).is_err());
        let mut stale = contract.clone();
        stale["schema_version"] = json!(1);
        stale["state"] = json!("implementation-ready-documentation-no-rust-code");
        assert!(check(stale, checksums.clone(), &files).is_err());
        let mut historical = checksums.clone();
        historical["preimplementation_freeze"]["evaluation"] = json!("Pass");
        assert!(check(contract.clone(), historical, &files).is_err());
        let mut accepted = checksums.clone();
        accepted["acceptance_status"] = json!("accepted");
        assert!(check(contract.clone(), accepted, &files).is_err());
        let mut missing = contract.clone();
        missing["operation_routing"]
            .as_object_mut()
            .ok_or("routing")?
            .remove("validate_blizzard_ui_source_profile");
        assert!(check(missing, checksums.clone(), &files).is_err());
        let mut unrouted = contract.clone();
        unrouted["operation_routing"]["validate_blizzard_ui_source_profile"]["owners"] = json!([]);
        assert!(check(unrouted, checksums.clone(), &files).is_err());
        let mut selected = contract.clone();
        selected["operation_routing"]["validate_blizzard_ui_source_profile"]["status"] =
            json!("application-selected");
        assert!(check(selected, checksums.clone(), &files).is_err());
        let mut absent = contract.clone();
        absent["operation_routing"]["validate_blizzard_ui_source_profile"]["owners"][0]["path"] =
            json!("crates/wow-project/src/not-present.rs");
        assert!(check(absent, checksums.clone(), &files).is_err());
        let mut guessed = contract.clone();
        guessed["operation_routing"]["validate_blizzard_ui_source_profile"]["status"] =
            json!("specified-only");
        assert!(check(guessed, checksums.clone(), &files).is_err());
        let mut frozen = checksums;
        frozen["bundle_sha256"] = json!("not-an-acceptance-proof");
        assert!(check(contract, frozen, &files).is_err());
        Ok(())
    }
}
