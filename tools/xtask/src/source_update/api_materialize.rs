//! Exact GitHub API/blob source snapshots with bounded network admission.
//!
//! This lane is an explicit fallback when a managed Git checkout is undesirable.
//! It resolves one public branch once, fetches one immutable commit/tree and each
//! selected blob by exact object ID, then installs an immutable data-only snapshot.
//! It never executes source code, updates refs, runs hooks or rewrites an existing
//! snapshot root.

use crate::{Result, git, manifest, repository};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

const REQUEST_SCHEMA: &str = "wow-source-api-materialization-request/1";
const REPORT_SCHEMA: &str = "wow-source-api-materialization-result/1";
const PLAN_SCHEMA: &str = "wow-source-api-materialization-plan/1";
const OPERATION_SCHEMA: &str = "wow-source-api-materialization-operation/1";
const CHECKPOINT_SCHEMA: &str = "wow-source-api-materialization-checkpoint/1";
const RECEIPT_SCHEMA: &str = "wow-source-api-materialization-receipt/1";
const SNAPSHOT_SCHEMA: &str = "wow-source-api-snapshot/1";
const INTERNAL_MANIFEST: &str = ".wow-source-manifest.json";
const SNAPSHOT_MARKER: &str = ".wow-source-api-snapshot.json";
const MAX_REQUEST_BYTES: u64 = 1024 * 1024;
const MAX_OPERATION_BYTES: u64 = 1024 * 1024;
const MAX_MARKER_BYTES: u64 = 64 * 1024;
const MAX_API_METADATA_BYTES: usize = 1024 * 1024;
const MAX_API_TREE_BYTES: usize = 64 * 1024 * 1024;
const MAX_API_REQUESTS: usize = 4096;
const MAX_API_BODY_BYTES: usize = 384 * 1024 * 1024;
const MAX_TREE_ENTRIES: usize = 400_000;
const TOKEN_HEX_BYTES: usize = 12;

#[derive(Debug)]
struct NetworkFailure;

impl fmt::Display for NetworkFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("GitHub API acquisition failed or exceeded its bound")
    }
}

impl Error for NetworkFailure {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Policy {
    Auto,
    Prompt,
    Never,
}

impl Policy {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "auto" => Ok(Self::Auto),
            "prompt" => Ok(Self::Prompt),
            "never" => Ok(Self::Never),
            _ => Err("source API materialization policy must be auto, prompt or never".into()),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Prompt => "prompt",
            Self::Never => "never",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Materialize,
    Publish,
}

impl Action {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "materialize_github_api_snapshot" => Ok(Self::Materialize),
            "publish_exact_api_snapshot_manifest" => Ok(Self::Publish),
            _ => Err("source API materialization action is invalid".into()),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Materialize => "materialize_github_api_snapshot",
            Self::Publish => "publish_exact_api_snapshot_manifest",
        }
    }
}

#[derive(Debug, Clone)]
struct Authorization {
    plan_id: String,
    before_revision: Option<String>,
    selected_revision: Option<String>,
}

#[derive(Debug, Clone)]
struct Request {
    operation_id: String,
    policy: Policy,
    snapshot_root: PathBuf,
    manifest_output: PathBuf,
    origin: String,
    branch: String,
    selector: String,
    authorization: Option<Authorization>,
}

#[derive(Debug)]
struct RepositoryIdentity {
    owner: String,
    repository: String,
}

#[derive(Debug)]
struct Context {
    request: Request,
    root: PathBuf,
    root_parent: PathBuf,
    output: PathBuf,
    repository: RepositoryIdentity,
    origin_digest: String,
    root_digest: String,
    output_digest: String,
    request_digest: String,
    operation_dir: PathBuf,
    staging: PathBuf,
    hash_repository: PathBuf,
}

#[derive(Debug, Clone)]
struct Snapshot {
    revision: String,
    tree_sha: String,
    manifest: Value,
    manifest_sha256: String,
    network_requests: usize,
    network_body_bytes: usize,
}

#[derive(Debug)]
enum RootObservation {
    Missing,
    Snapshot(Snapshot),
}

#[derive(Debug, Clone)]
struct Plan {
    action: Action,
    before_revision: Option<String>,
    selected_revision: Option<String>,
    freshness: &'static str,
    relation: &'static str,
    plan_id: String,
}

enum ExistingOperation {
    None,
    Incomplete,
    Registered(Plan),
}

#[derive(Debug)]
struct NetworkBudget {
    requests: usize,
    bytes: usize,
}

impl NetworkBudget {
    const fn new() -> Self {
        Self {
            requests: 0,
            bytes: 0,
        }
    }

    fn get(&mut self, url: &str, accept: &str, limit: usize) -> Result<Vec<u8>> {
        if self.requests >= MAX_API_REQUESTS
            || limit > MAX_API_BODY_BYTES.saturating_sub(self.bytes)
        {
            return Err("GitHub API request/body budget exceeded".into());
        }
        let bytes = curl_get(url, accept, limit)?;
        self.requests += 1;
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .ok_or("GitHub API byte budget overflow")?;
        if self.bytes > MAX_API_BODY_BYTES {
            return Err("GitHub API body budget exceeded".into());
        }
        Ok(bytes)
    }
}

pub(super) fn run(request_path: &Path) -> Result<u8> {
    let request = parse_request(request_path)?;
    let context = Context::new(request)?;

    match read_existing_operation(&context)? {
        ExistingOperation::Incomplete => {
            println!("{}", incomplete_operation_report(&context));
            return Ok(5);
        }
        ExistingOperation::Registered(plan) => {
            validate_supplied_authorization(&context, &plan)?;
            if let Some(receipt) = read_receipt(&context, &plan)? {
                println!("{}", receipt_report(&context, &plan, &receipt, true));
                return Ok(execution_exit(&plan));
            }
            if context.request.policy == Policy::Never {
                return Err("never policy cannot resume a mutating API operation".into());
            }
            return execute(&context, &plan);
        }
        ExistingOperation::None => {}
    }

    let observation = observe_root(&context)?;
    let plan = match &context.request.authorization {
        Some(authorization) => {
            if context.request.policy == Policy::Never {
                return Err("never policy cannot carry mutation authorization".into());
            }
            authorized_plan(&context, &observation, authorization)?
        }
        None => match observed_plan(&context, &observation)? {
            Some(value) => value,
            None => {
                println!(
                    "{}",
                    observation_report(
                        &context,
                        "unverified_missing",
                        None,
                        None,
                        "unverified_current",
                        "network_unavailable_for_missing_snapshot",
                    )
                );
                return Ok(4);
            }
        },
    };

    if context.request.policy == Policy::Never {
        let status = match plan.relation {
            "current" => "read_only_current",
            "different" => "read_only_different",
            "missing" => "read_only_missing",
            _ => "read_only_unverified_current",
        };
        println!("{}", plan_report(&context, &plan, status, false));
        return Ok(observation_exit(&plan));
    }
    if context.request.policy == Policy::Prompt && context.request.authorization.is_none() {
        println!(
            "{}",
            plan_report(&context, &plan, "authorization_required", false)
        );
        return Ok(3);
    }

    execute(&context, &plan)
}

impl Context {
    fn new(request: Request) -> Result<Self> {
        let (origin, repository) = normalize_github_origin(&request.origin)?;
        let root = canonical_target(&request.snapshot_root, "source API snapshot root")?;
        let output = canonical_target(&request.manifest_output, "manifest output")?;
        let root_parent = root
            .parent()
            .ok_or("source API snapshot root has no parent")?
            .to_path_buf();
        let origin_digest = digest_text(&origin);
        let root_digest = path_digest(&root)?;
        let output_digest = path_digest(&output)?;
        let request_identity = json!({
            "schema": REQUEST_SCHEMA,
            "operation_id": request.operation_id,
            "policy": request.policy.as_str(),
            "snapshot_root_sha256": root_digest,
            "manifest_output_sha256": output_digest,
            "origin_sha256": origin_digest,
            "branch": request.branch,
            "selector": request.selector,
        });
        let request_digest = content_id("source-api-materialization-request", &request_identity)?;
        let operation_token = digest_text(&request.operation_id);
        let token = &operation_token[..TOKEN_HEX_BYTES * 2];
        let operation_dir = root_parent.join(format!(".wow-source-api-materialization-{token}"));
        let staging = root_parent.join(format!(".wow-source-api-materialization-{token}.staging"));
        let hash_repository = operation_dir.join("sha1-hash-repository");
        let paths = [&root, &output, &operation_dir, &staging];
        if paths.iter().enumerate().any(|(index, left)| {
            paths[index + 1..]
                .iter()
                .any(|right| paths_overlap(left, right))
        }) {
            return Err("source API materialization paths overlap".into());
        }
        Ok(Self {
            request,
            root,
            root_parent,
            output,
            repository,
            origin_digest,
            root_digest,
            output_digest,
            request_digest,
            operation_dir,
            staging,
            hash_repository,
        })
    }
}

fn parse_request(path: &Path) -> Result<Request> {
    let bytes = read_safe_file(
        path,
        MAX_REQUEST_BYTES,
        "source API materialization request",
    )?;
    let value = parse_strict_json(&bytes, "source API materialization request")?;
    let object = exact_object(
        &value,
        &[
            "schema",
            "operation_id",
            "policy",
            "snapshot_root",
            "manifest_output",
            "origin",
            "branch",
            "selector",
            "authorization",
        ],
        "source API materialization request",
    )?;
    if string_field(object, "schema")? != REQUEST_SCHEMA {
        return Err("unsupported source API materialization request schema".into());
    }
    let operation_id = string_field(object, "operation_id")?.to_owned();
    if !valid_token(&operation_id, 128) {
        return Err("source API materialization operation ID is invalid".into());
    }
    let policy = Policy::parse(string_field(object, "policy")?)?;
    let snapshot_root = PathBuf::from(string_field(object, "snapshot_root")?);
    let manifest_output = PathBuf::from(string_field(object, "manifest_output")?);
    let origin = string_field(object, "origin")?.to_owned();
    let branch = string_field(object, "branch")?.to_owned();
    let selector = string_field(object, "selector")?.to_owned();
    if !valid_text(&branch, 256) || !valid_text(&selector, 256) {
        return Err("source API branch or selector label is invalid".into());
    }
    let authorization = match object.get("authorization") {
        Some(Value::Null) | None => None,
        Some(value) => Some(parse_authorization(value)?),
    };
    Ok(Request {
        operation_id,
        policy,
        snapshot_root,
        manifest_output,
        origin,
        branch,
        selector,
        authorization,
    })
}

fn parse_authorization(value: &Value) -> Result<Authorization> {
    let object = exact_object(
        value,
        &["plan_id", "before_revision", "selected_revision"],
        "source API materialization authorization",
    )?;
    let plan_id = string_field(object, "plan_id")?.to_owned();
    if !valid_content_id(&plan_id, "source-api-materialization-plan") {
        return Err("source API authorization plan ID is invalid".into());
    }
    let before_revision = optional_string_field(object, "before_revision")?;
    let selected_revision = optional_string_field(object, "selected_revision")?;
    for revision in [before_revision.as_deref(), selected_revision.as_deref()]
        .into_iter()
        .flatten()
    {
        if revision.len() != 40 || !git::oid(revision) {
            return Err("source API authorization revision is invalid".into());
        }
    }
    Ok(Authorization {
        plan_id,
        before_revision,
        selected_revision,
    })
}

fn observe_root(context: &Context) -> Result<RootObservation> {
    match fs::symlink_metadata(&context.root) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(RootObservation::Missing),
        Err(error) => Err(error.into()),
        Ok(metadata) => {
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err("source API snapshot root is not a safe directory".into());
            }
            Ok(RootObservation::Snapshot(observe_snapshot(
                context,
                &context.root,
            )?))
        }
    }
}

fn observed_plan(context: &Context, observation: &RootObservation) -> Result<Option<Plan>> {
    let selected = api_head(context).ok();
    match observation {
        RootObservation::Missing => {
            let Some(selected) = selected else {
                return Ok(None);
            };
            Ok(Some(build_plan(
                context,
                Action::Materialize,
                None,
                Some(selected),
                "selected_remote_observation_only",
                "missing",
            )?))
        }
        RootObservation::Snapshot(snapshot) => {
            let (freshness, relation) = match selected.as_deref() {
                Some(value) if value == snapshot.revision => {
                    ("selected_remote_observation_only", "current")
                }
                Some(_) => ("selected_remote_observation_only", "different"),
                None => ("unverified_current", "unverified"),
            };
            Ok(Some(build_plan(
                context,
                Action::Publish,
                Some(snapshot.revision.clone()),
                selected,
                freshness,
                relation,
            )?))
        }
    }
}

fn authorized_plan(
    context: &Context,
    observation: &RootObservation,
    authorization: &Authorization,
) -> Result<Plan> {
    let (action, before, relation) = match observation {
        RootObservation::Missing => {
            if authorization.before_revision.is_some()
                || authorization
                    .selected_revision
                    .as_deref()
                    .is_none_or(|revision| revision.len() != 40 || !git::oid(revision))
            {
                return Err("source API authorization does not match the missing snapshot".into());
            }
            (Action::Materialize, None, "missing")
        }
        RootObservation::Snapshot(snapshot) => {
            if authorization.before_revision.as_deref() != Some(snapshot.revision.as_str()) {
                return Err("source API snapshot changed after plan authorization".into());
            }
            let relation = match authorization.selected_revision.as_deref() {
                Some(value) if value == snapshot.revision => "current",
                Some(_) => "different",
                None => "unverified",
            };
            (Action::Publish, Some(snapshot.revision.clone()), relation)
        }
    };
    let freshness = if authorization.selected_revision.is_some() {
        "selected_remote_observation_only"
    } else {
        "unverified_current"
    };
    let plan = build_plan(
        context,
        action,
        before,
        authorization.selected_revision.clone(),
        freshness,
        relation,
    )?;
    if plan.plan_id != authorization.plan_id {
        return Err("source API authorization does not match the exact plan".into());
    }
    Ok(plan)
}

fn build_plan(
    context: &Context,
    action: Action,
    before_revision: Option<String>,
    selected_revision: Option<String>,
    freshness: &'static str,
    relation: &'static str,
) -> Result<Plan> {
    let identity = json!({
        "schema": PLAN_SCHEMA,
        "request_digest": context.request_digest,
        "action": action.as_str(),
        "before_revision": before_revision,
        "selected_revision": selected_revision,
        "freshness": freshness,
        "relation": relation,
    });
    let plan_id = content_id("source-api-materialization-plan", &identity)?;
    Ok(Plan {
        action,
        before_revision,
        selected_revision,
        freshness,
        relation,
        plan_id,
    })
}

fn read_existing_operation(context: &Context) -> Result<ExistingOperation> {
    let metadata = match fs::symlink_metadata(&context.operation_dir) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ExistingOperation::None);
        }
        Err(error) => return Err(error.into()),
        Ok(value) => value,
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("source API operation path is unsafe".into());
    }
    let Some(bytes) = read_optional_safe_file(
        &context.operation_dir.join("operation.json"),
        MAX_OPERATION_BYTES,
        "source API operation",
    )?
    else {
        return Ok(ExistingOperation::Incomplete);
    };
    let value = parse_strict_json(&bytes, "source API operation")?;
    require_canonical_json(&bytes, &value, "source API operation")?;
    let object = exact_object(
        &value,
        &[
            "schema",
            "operation_id",
            "request_digest",
            "plan_id",
            "policy",
            "action",
            "before_revision",
            "selected_revision",
            "freshness",
            "relation",
            "origin_sha256",
            "snapshot_root_sha256",
            "manifest_output_sha256",
            "branch",
            "selector",
        ],
        "source API operation",
    )?;
    if string_field(object, "schema")? != OPERATION_SCHEMA
        || string_field(object, "operation_id")? != context.request.operation_id.as_str()
        || string_field(object, "request_digest")? != context.request_digest.as_str()
        || string_field(object, "policy")? != context.request.policy.as_str()
        || string_field(object, "origin_sha256")? != context.origin_digest.as_str()
        || string_field(object, "snapshot_root_sha256")? != context.root_digest.as_str()
        || string_field(object, "manifest_output_sha256")? != context.output_digest.as_str()
        || string_field(object, "branch")? != context.request.branch.as_str()
        || string_field(object, "selector")? != context.request.selector.as_str()
    {
        return Err("source API operation belongs to another request".into());
    }
    let action = Action::parse(string_field(object, "action")?)?;
    let before_revision = optional_string_field(object, "before_revision")?;
    let selected_revision = optional_string_field(object, "selected_revision")?;
    validate_revision_pair(before_revision.as_deref(), selected_revision.as_deref())?;
    let freshness = parse_freshness(string_field(object, "freshness")?)?;
    let relation = parse_relation(string_field(object, "relation")?)?;
    let plan = build_plan(
        context,
        action,
        before_revision,
        selected_revision,
        freshness,
        relation,
    )?;
    if string_field(object, "plan_id")? != plan.plan_id.as_str() {
        return Err("source API operation plan identity is invalid".into());
    }
    Ok(ExistingOperation::Registered(plan))
}

fn validate_supplied_authorization(context: &Context, plan: &Plan) -> Result<()> {
    let Some(authorization) = &context.request.authorization else {
        return Ok(());
    };
    if authorization.plan_id != plan.plan_id
        || authorization.before_revision != plan.before_revision
        || authorization.selected_revision != plan.selected_revision
    {
        return Err("source API authorization conflicts with the durable operation".into());
    }
    Ok(())
}

fn parse_freshness(value: &str) -> Result<&'static str> {
    match value {
        "selected_remote_observation_only" => Ok("selected_remote_observation_only"),
        "unverified_current" => Ok("unverified_current"),
        _ => Err("source API freshness is invalid".into()),
    }
}

fn parse_relation(value: &str) -> Result<&'static str> {
    match value {
        "current" => Ok("current"),
        "different" => Ok("different"),
        "missing" => Ok("missing"),
        "unverified" => Ok("unverified"),
        _ => Err("source API relation is invalid".into()),
    }
}

fn validate_revision_pair(before: Option<&str>, selected: Option<&str>) -> Result<()> {
    for revision in [before, selected].into_iter().flatten() {
        if revision.len() != 40 || !git::oid(revision) {
            return Err("source API operation revision is invalid".into());
        }
    }
    Ok(())
}

fn incomplete_operation_report(context: &Context) -> Value {
    json!({
        "schema": REPORT_SCHEMA,
        "status": "reconciliation_required",
        "operation_id": context.request.operation_id,
        "policy": context.request.policy.as_str(),
        "journaled": true,
        "mutation_performed": false,
        "next_action": "operator_review_before_any_retry",
        "reason": "operation_registration_incomplete",
    })
}

fn execute(context: &Context, plan: &Plan) -> Result<u8> {
    prepare_operation(context, plan)?;
    if let Some(receipt) = read_receipt(context, plan)? {
        println!("{}", receipt_report(context, plan, &receipt, true));
        return Ok(execution_exit(plan));
    }
    checkpoint(context, plan, 0, "registered", None)?;

    let snapshot = match plan.action {
        Action::Materialize => match execute_materialize(context, plan)? {
            MaterializeOutcome::Ready(value) => value,
            MaterializeOutcome::NetworkUnavailable => {
                println!(
                    "{}",
                    plan_report(context, plan, "exact_api_acquisition_unavailable", true,)
                );
                return Ok(4);
            }
            MaterializeOutcome::ReconciliationRequired(reason) => {
                return reconciliation_required(context, plan, reason);
            }
        },
        Action::Publish => match observe_root(context)? {
            RootObservation::Snapshot(value)
                if plan.before_revision.as_deref() == Some(value.revision.as_str()) =>
            {
                checkpoint(context, plan, 6, "snapshot_verified", Some(&value.revision))?;
                value
            }
            _ => return reconciliation_required(context, plan, "local_snapshot_changed"),
        },
    };

    checkpoint(
        context,
        plan,
        7,
        "manifest_intent",
        Some(&snapshot.revision),
    )?;
    let publication = match manifest::publish_exact(&context.output, &snapshot.manifest) {
        Ok(value) => value,
        Err(_) => return reconciliation_required(context, plan, "manifest_publication_conflict"),
    };
    checkpoint(
        context,
        plan,
        8,
        "manifest_published",
        Some(&snapshot.manifest_sha256),
    )?;
    let receipt = build_receipt(
        context,
        plan,
        &snapshot,
        match publication {
            manifest::PublishState::Created => "created",
            manifest::PublishState::Existing => "existing_exact",
        },
    )?;
    publish_receipt(context, &receipt)?;
    checkpoint(
        context,
        plan,
        9,
        "completed",
        receipt["receipt_id"].as_str(),
    )?;
    println!("{}", receipt_report(context, plan, &receipt, false));
    Ok(execution_exit(plan))
}

enum MaterializeOutcome {
    Ready(Snapshot),
    NetworkUnavailable,
    ReconciliationRequired(&'static str),
}

fn execute_materialize(context: &Context, plan: &Plan) -> Result<MaterializeOutcome> {
    let selected = plan
        .selected_revision
        .as_deref()
        .ok_or("source API materialization plan is missing selected revision")?;
    match observe_root(context)? {
        RootObservation::Snapshot(snapshot) if snapshot.revision == selected => {
            checkpoint(context, plan, 5, "installed", Some(selected))?;
            checkpoint(context, plan, 6, "snapshot_verified", Some(selected))?;
            return Ok(MaterializeOutcome::Ready(snapshot));
        }
        RootObservation::Snapshot(_) => {
            return Ok(MaterializeOutcome::ReconciliationRequired(
                "snapshot_root_contains_another_revision",
            ));
        }
        RootObservation::Missing => {}
    }

    let staging_ready = match fs::symlink_metadata(&context.staging) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(error.into()),
        Ok(metadata) => {
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Ok(MaterializeOutcome::ReconciliationRequired(
                    "retained_staging_path_is_unsafe",
                ));
            }
            let snapshot = match observe_snapshot(context, &context.staging) {
                Ok(value) => value,
                Err(_) => {
                    return Ok(MaterializeOutcome::ReconciliationRequired(
                        "retained_staging_is_partial_or_foreign",
                    ));
                }
            };
            if snapshot.revision != selected {
                return Ok(MaterializeOutcome::ReconciliationRequired(
                    "retained_staging_revision_mismatch",
                ));
            }
            true
        }
    };

    if !staging_ready {
        checkpoint(context, plan, 1, "staging_intent", Some(selected))?;
        match build_snapshot(context, selected) {
            Ok(_) => {}
            Err(error) if error.downcast_ref::<NetworkFailure>().is_some() => {
                if fs::symlink_metadata(&context.staging).is_ok()
                    && fs::remove_dir_all(&context.staging).is_err()
                {
                    return Ok(MaterializeOutcome::ReconciliationRequired(
                        "network_failure_with_retained_partial_staging",
                    ));
                }
                return Ok(MaterializeOutcome::NetworkUnavailable);
            }
            Err(error) => {
                if fs::symlink_metadata(&context.staging).is_ok() {
                    let _ = fs::remove_dir_all(&context.staging);
                }
                return Err(error);
            }
        }
    }

    let staged = observe_snapshot(context, &context.staging)?;
    if staged.revision != selected {
        return Err("source API staging verification failed".into());
    }
    checkpoint(
        context,
        plan,
        2,
        "staging_verified",
        Some(&staged.manifest_sha256),
    )?;
    checkpoint(context, plan, 3, "install_intent", Some(selected))?;
    if fs::symlink_metadata(&context.root).is_ok() {
        return Ok(MaterializeOutcome::ReconciliationRequired(
            "snapshot_root_appeared_before_installation",
        ));
    }
    fs::rename(&context.staging, &context.root)?;
    sync_directory(&context.root_parent)?;
    checkpoint(context, plan, 5, "installed", Some(selected))?;
    let installed = match observe_root(context) {
        Ok(RootObservation::Snapshot(value)) => value,
        Ok(RootObservation::Missing) | Err(_) => {
            return Ok(MaterializeOutcome::ReconciliationRequired(
                "installed_snapshot_failed_exact_read_back",
            ));
        }
    };
    if installed.revision != selected {
        return Ok(MaterializeOutcome::ReconciliationRequired(
            "installed_snapshot_revision_mismatch",
        ));
    }
    checkpoint(context, plan, 6, "snapshot_verified", Some(selected))?;
    Ok(MaterializeOutcome::Ready(installed))
}

fn build_snapshot(context: &Context, revision: &str) -> Result<Snapshot> {
    if revision.len() != 40 || !git::oid(revision) {
        return Err("GitHub API snapshot requires an exact SHA-1 commit".into());
    }
    create_private_directory(&context.staging)?;
    ensure_hash_repository(context)?;
    let mut budget = NetworkBudget::new();
    let tree_sha = api_commit_tree(context, revision, &mut budget)?;
    let tree = api_tree(context, &tree_sha, &mut budget)?;
    let entries = select_tree_entries(&tree)?;
    if entries.len() + 3 > MAX_API_REQUESTS {
        return Err("selected source files exceed the GitHub API request budget".into());
    }

    let mut version = None;
    let mut total = 0usize;
    let mut files = Vec::with_capacity(entries.len());
    for entry in entries {
        let bytes = api_blob(context, &entry.blob_id, entry.bytes, &mut budget)?;
        if bytes.len() != entry.bytes {
            return Err("GitHub blob length does not match the exact tree".into());
        }
        let actual_blob = hash_blob(context, &bytes)?;
        if actual_blob != entry.blob_id {
            return Err("GitHub blob bytes do not match the exact tree object ID".into());
        }
        write_snapshot_file(&context.staging, &entry.path, &bytes)?;
        total = total
            .checked_add(bytes.len())
            .ok_or("source API selected-byte overflow")?;
        if entry.path == "version.txt" {
            let text = std::str::from_utf8(&bytes)?.trim();
            if text.is_empty() || text.chars().any(char::is_control) {
                return Err("invalid source version".into());
            }
            version = Some(text.to_owned());
        }
        files.push(json!({
            "path": entry.path,
            "kind": manifest::class(&entry.path),
            "bytes": bytes.len(),
            "git_blob_algorithm": "sha1",
            "git_blob_id": entry.blob_id,
            "content_sha256": manifest::digest(&bytes),
        }));
    }
    let tracked = tree
        .get("tree")
        .and_then(Value::as_array)
        .ok_or("GitHub tree response is missing entries")?
        .iter()
        .filter(|entry| entry["type"].as_str() == Some("blob"))
        .count();
    let manifest_value = manifest::assemble(
        &context.request.selector,
        revision,
        "sha1",
        version.ok_or("version.txt is absent from the selected revision")?,
        "github_api_exact_blob_snapshot",
        tracked,
        total,
        files,
    )?;
    let manifest_sha256 = manifest_value["manifest_sha256"]
        .as_str()
        .ok_or("source API manifest is missing its identity")?
        .to_owned();
    manifest::write_new_bytes(
        &context.staging.join(INTERNAL_MANIFEST),
        &manifest::serialized(&manifest_value)?,
    )?;
    let marker = json!({
        "schema": SNAPSHOT_SCHEMA,
        "origin_sha256": context.origin_digest,
        "branch": context.request.branch,
        "selector": context.request.selector,
        "revision": revision,
        "tree_sha": tree_sha,
        "manifest_sha256": manifest_sha256,
        "included_files": manifest_value["coverage"]["included_files"],
        "included_bytes": manifest_value["coverage"]["included_bytes"],
    });
    manifest::write_new_bytes(
        &context.staging.join(SNAPSHOT_MARKER),
        &pretty_json_bytes(&marker)?,
    )?;
    sync_directory(&context.staging)?;
    Ok(Snapshot {
        revision: revision.to_owned(),
        tree_sha,
        manifest: manifest_value,
        manifest_sha256,
        network_requests: budget.requests,
        network_body_bytes: budget.bytes,
    })
}

#[derive(Debug)]
struct TreeEntry {
    path: String,
    blob_id: String,
    bytes: usize,
}

fn select_tree_entries(tree: &Value) -> Result<Vec<TreeEntry>> {
    if tree["truncated"].as_bool() != Some(false) {
        return Err("GitHub recursive tree is truncated".into());
    }
    let entries = tree["tree"]
        .as_array()
        .ok_or("GitHub tree response is missing entries")?;
    if entries.len() > MAX_TREE_ENTRIES {
        return Err("GitHub tree entry-count limit exceeded".into());
    }
    let mut tracked = 0usize;
    let mut total = 0usize;
    let mut casefold = BTreeSet::new();
    let mut selected = Vec::new();
    for entry in entries {
        let path = entry["path"]
            .as_str()
            .ok_or("GitHub tree entry path is missing")?;
        manifest::validate_path(path)?;
        let kind = entry["type"]
            .as_str()
            .ok_or("GitHub tree entry type is missing")?;
        let mode = entry["mode"]
            .as_str()
            .ok_or("GitHub tree entry mode is missing")?;
        match kind {
            "tree" if mode == "040000" => continue,
            "blob" => {
                tracked += 1;
                if tracked > manifest::MAX_FILES {
                    return Err("source file-count limit exceeded".into());
                }
                if !matches!(mode, "100644" | "100755") {
                    return Err("nonregular GitHub source entry rejected".into());
                }
            }
            _ => return Err("nonregular GitHub source entry rejected".into()),
        }
        if !manifest::selected_path(path) {
            continue;
        }
        let blob_id = entry["sha"]
            .as_str()
            .ok_or("GitHub tree blob ID is missing")?;
        if blob_id.len() != 40 || !git::oid(blob_id) {
            return Err("GitHub tree blob ID is invalid".into());
        }
        let bytes = entry["size"]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or("GitHub tree blob size is invalid")?;
        total = total.checked_add(bytes).ok_or("source byte limit")?;
        if bytes > manifest::MAX_FILE || total > manifest::MAX_TOTAL {
            return Err("source byte limit exceeded".into());
        }
        if !casefold.insert(path.to_ascii_lowercase()) {
            return Err("case-insensitive source path collision rejected".into());
        }
        selected.push(TreeEntry {
            path: path.to_owned(),
            blob_id: blob_id.to_owned(),
            bytes,
        });
    }
    selected.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
    Ok(selected)
}

fn observe_snapshot(context: &Context, root: &Path) -> Result<Snapshot> {
    let marker_bytes = read_safe_file(
        &root.join(SNAPSHOT_MARKER),
        MAX_MARKER_BYTES,
        "source API snapshot marker",
    )?;
    let marker = parse_strict_json(&marker_bytes, "source API snapshot marker")?;
    require_canonical_json(&marker_bytes, &marker, "source API snapshot marker")?;
    let marker_object = exact_object(
        &marker,
        &[
            "schema",
            "origin_sha256",
            "branch",
            "selector",
            "revision",
            "tree_sha",
            "manifest_sha256",
            "included_files",
            "included_bytes",
        ],
        "source API snapshot marker",
    )?;
    let revision = string_field(marker_object, "revision")?.to_owned();
    let tree_sha = string_field(marker_object, "tree_sha")?.to_owned();
    let manifest_sha256 = string_field(marker_object, "manifest_sha256")?.to_owned();
    if string_field(marker_object, "schema")? != SNAPSHOT_SCHEMA
        || string_field(marker_object, "origin_sha256")? != context.origin_digest.as_str()
        || string_field(marker_object, "branch")? != context.request.branch.as_str()
        || string_field(marker_object, "selector")? != context.request.selector.as_str()
        || revision.len() != 40
        || !git::oid(&revision)
        || tree_sha.len() != 40
        || !git::oid(&tree_sha)
        || !is_sha256(&manifest_sha256)
    {
        return Err("source API snapshot marker does not match the requested source".into());
    }
    let manifest_bytes = read_safe_file(
        &root.join(INTERNAL_MANIFEST),
        manifest::MAX_MANIFEST as u64,
        "source API internal manifest",
    )?;
    let manifest_value = parse_strict_json(&manifest_bytes, "source API internal manifest")?;
    require_canonical_json(
        &manifest_bytes,
        &manifest_value,
        "source API internal manifest",
    )?;
    manifest::validate_value(&manifest_value)?;
    if manifest_value["manifest_sha256"].as_str() != Some(manifest_sha256.as_str())
        || manifest_value["source"]["revision"].as_str() != Some(revision.as_str())
        || manifest_value["source"]["selector"].as_str() != Some(context.request.selector.as_str())
        || manifest_value["source"]["git_object_format"].as_str() != Some("sha1")
        || manifest_value["source"]["acquisition"].as_str()
            != Some("github_api_exact_blob_snapshot")
    {
        return Err("source API snapshot manifest binding is invalid".into());
    }
    let files = manifest_value["files"]
        .as_array()
        .ok_or("source API snapshot manifest files are missing")?;
    if marker_object["included_files"].as_u64() != Some(files.len() as u64)
        || marker_object["included_bytes"].as_u64()
            != manifest_value["coverage"]["included_bytes"].as_u64()
    {
        return Err("source API snapshot marker coverage does not match manifest".into());
    }
    let mut expected = BTreeSet::new();
    expected.insert(INTERNAL_MANIFEST.to_owned());
    expected.insert(SNAPSHOT_MARKER.to_owned());
    for file in files {
        let path = file["path"]
            .as_str()
            .ok_or("source API snapshot file path is missing")?;
        manifest::validate_path(path)?;
        if !expected.insert(path.to_owned()) {
            return Err("source API snapshot file inventory contains duplicates".into());
        }
        let bytes = read_safe_file(
            &root.join(path),
            manifest::MAX_FILE as u64,
            "source API snapshot member",
        )?;
        if file["bytes"].as_u64() != Some(bytes.len() as u64)
            || file["content_sha256"].as_str() != Some(manifest::digest(&bytes).as_str())
        {
            return Err("source API snapshot member differs from its manifest".into());
        }
    }
    let actual = collect_snapshot_files(root)?;
    if actual != expected {
        return Err("source API snapshot contains missing or extra files".into());
    }
    Ok(Snapshot {
        revision,
        tree_sha,
        manifest: manifest_value,
        manifest_sha256,
        network_requests: 0,
        network_body_bytes: 0,
    })
}

fn collect_snapshot_files(root: &Path) -> Result<BTreeSet<String>> {
    let mut files = BTreeSet::new();
    let mut directories = vec![(root.to_path_buf(), String::new())];
    let mut entries = 0usize;
    while let Some((directory, prefix)) = directories.pop() {
        for entry in fs::read_dir(&directory)? {
            entries += 1;
            if entries > manifest::MAX_FILES + MAX_TREE_ENTRIES {
                return Err("source API snapshot filesystem entry limit exceeded".into());
            }
            let entry = entry?;
            let metadata = fs::symlink_metadata(entry.path())?;
            if metadata.file_type().is_symlink() {
                return Err("source API snapshot symlink rejected".into());
            }
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| "source API snapshot path is not UTF-8")?;
            let relative = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            manifest::validate_path(&relative)?;
            if metadata.is_dir() {
                directories.push((entry.path(), relative));
            } else if metadata.is_file() {
                files.insert(relative);
            } else {
                return Err("source API snapshot special file rejected".into());
            }
        }
    }
    Ok(files)
}

fn api_head(context: &Context) -> Result<String> {
    let mut budget = NetworkBudget::new();
    let branch = percent_encode(&context.request.branch);
    let url = format!(
        "https://api.github.com/repos/{}/{}/git/ref/heads/{branch}",
        context.repository.owner, context.repository.repository
    );
    let value = api_json(&mut budget, &url, MAX_API_METADATA_BYTES)?;
    let revision = value["object"]["sha"]
        .as_str()
        .ok_or("GitHub branch ref is missing commit identity")?;
    if value["object"]["type"].as_str() != Some("commit")
        || revision.len() != 40
        || !git::oid(revision)
    {
        return Err("GitHub branch ref does not identify a SHA-1 commit".into());
    }
    Ok(revision.to_owned())
}

fn api_commit_tree(
    context: &Context,
    revision: &str,
    budget: &mut NetworkBudget,
) -> Result<String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/git/commits/{revision}",
        context.repository.owner, context.repository.repository
    );
    let value = api_json(budget, &url, MAX_API_METADATA_BYTES)?;
    if value["sha"].as_str() != Some(revision) {
        return Err("GitHub commit response does not match selected revision".into());
    }
    let tree_sha = value["tree"]["sha"]
        .as_str()
        .ok_or("GitHub commit response is missing tree identity")?;
    if tree_sha.len() != 40 || !git::oid(tree_sha) {
        return Err("GitHub commit tree identity is invalid".into());
    }
    Ok(tree_sha.to_owned())
}

fn api_tree(context: &Context, tree_sha: &str, budget: &mut NetworkBudget) -> Result<Value> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/git/trees/{tree_sha}?recursive=1",
        context.repository.owner, context.repository.repository
    );
    let value = api_json(budget, &url, MAX_API_TREE_BYTES)?;
    if value["sha"].as_str() != Some(tree_sha) {
        return Err("GitHub tree response does not match selected tree".into());
    }
    Ok(value)
}

fn api_blob(
    context: &Context,
    blob_id: &str,
    expected_bytes: usize,
    budget: &mut NetworkBudget,
) -> Result<Vec<u8>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/git/blobs/{blob_id}",
        context.repository.owner, context.repository.repository
    );
    budget.get(&url, "application/vnd.github.raw+json", expected_bytes)
}

fn api_json(budget: &mut NetworkBudget, url: &str, limit: usize) -> Result<Value> {
    let bytes = budget.get(url, "application/vnd.github+json", limit)?;
    serde_json::from_slice(&bytes).map_err(Into::into)
}

fn curl_get(url: &str, accept: &str, limit: usize) -> Result<Vec<u8>> {
    if !url.starts_with("https://api.github.com/")
        || url.chars().any(|character| character.is_control())
        || !accept.starts_with("application/vnd.github.")
    {
        return Err("invalid GitHub API request".into());
    }
    let mut config = format!("url = \"{}\"\n", curl_quote(url)?);
    if let Some(token) = std::env::var_os("WOW_SOURCE_GITHUB_TOKEN") {
        let token = token
            .into_string()
            .map_err(|_| "WOW_SOURCE_GITHUB_TOKEN is not UTF-8")?;
        if token.is_empty()
            || token.len() > 4096
            || token
                .chars()
                .any(|character| character.is_control() || matches!(character, '\\' | '"'))
        {
            return Err("WOW_SOURCE_GITHUB_TOKEN is invalid".into());
        }
        config.push_str(&format!("header = \"Authorization: Bearer {}\"\n", token));
    }
    let transport_limit = limit.max(1).to_string();
    let accept_header = format!("Accept: {accept}");
    let mut command = Command::new("curl");
    command
        .args([
            "--disable",
            "--fail",
            "--silent",
            "--show-error",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--max-redirs",
            "0",
            "--connect-timeout",
            "15",
            "--max-time",
            "120",
            "--max-filesize",
            &transport_limit,
            "--user-agent",
            "WoW-Dev-Framework-source-snapshot/1",
            "--header",
            &accept_header,
            "--header",
            "X-GitHub-Api-Version: 2022-11-28",
            "--config",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = command.spawn().map_err(|_| NetworkFailure)?;
    let mut writer = child.stdin.take().ok_or(NetworkFailure)?;
    let reader = child.stdout.take().ok_or(NetworkFailure)?;
    let config_bytes = config.into_bytes();
    let write_thread = std::thread::spawn(move || writer.write_all(&config_bytes));
    let (sender, receiver) = mpsc::channel();
    let read_thread = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = reader
            .take(limit as u64 + 1)
            .read_to_end(&mut bytes)
            .map(|_| bytes);
        let _ = sender.send(result);
    });
    let result = receiver.recv_timeout(Duration::from_secs(125));
    if !matches!(&result, Ok(Ok(bytes)) if bytes.len() <= limit) {
        let _ = child.kill();
        let _ = child.wait();
        let _ = read_thread.join();
        let _ = write_thread.join();
        return Err(NetworkFailure.into());
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|_| NetworkFailure)? {
            break status;
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            let _ = read_thread.join();
            let _ = write_thread.join();
            return Err(NetworkFailure.into());
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    read_thread.join().map_err(|_| NetworkFailure)?;
    write_thread
        .join()
        .map_err(|_| NetworkFailure)?
        .map_err(|_| NetworkFailure)?;
    if !status.success() {
        return Err(NetworkFailure.into());
    }
    Ok(result.map_err(|_| NetworkFailure)??)
}

fn curl_quote(value: &str) -> Result<String> {
    if value
        .chars()
        .any(|character| character.is_control() || matches!(character, '\\' | '"'))
    {
        return Err("GitHub API URL contains an unsafe character".into());
    }
    Ok(value.to_owned())
}

fn ensure_hash_repository(context: &Context) -> Result<()> {
    match fs::symlink_metadata(&context.hash_repository) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            create_private_directory(&context.hash_repository)?;
            git::isolated_text(
                &context.hash_repository,
                &["init", "--bare", "--object-format=sha1", "-q"],
            )?;
        }
        Err(error) => return Err(error.into()),
        Ok(metadata) => {
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err("source API hash repository is unsafe".into());
            }
        }
    }
    if git::isolated_text(
        &context.hash_repository,
        &["rev-parse", "--show-object-format"],
    )? != "sha1"
    {
        return Err("source API hash repository uses another object format".into());
    }
    Ok(())
}

fn hash_blob(context: &Context, bytes: &[u8]) -> Result<String> {
    let output = git::isolated_run(
        &context.hash_repository,
        &["hash-object", "--no-filters", "--stdin"],
        Some(bytes.to_vec()),
        128,
    )?;
    let value = std::str::from_utf8(&output)?.trim();
    if value.len() != 40 || !git::oid(value) {
        return Err("source API Git blob hash is invalid".into());
    }
    Ok(value.to_owned())
}

fn write_snapshot_file(root: &Path, relative: &str, bytes: &[u8]) -> Result<()> {
    manifest::validate_path(relative)?;
    let path = root.join(relative);
    let parent = path.parent().ok_or("source API member has no parent")?;
    create_snapshot_directories(root, parent)?;
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&path)?;
    let result = file.write_all(bytes).and_then(|_| file.sync_all());
    drop(file);
    if result.is_err() {
        let _ = fs::remove_file(&path);
    }
    result?;
    Ok(())
}

fn create_snapshot_directories(root: &Path, parent: &Path) -> Result<()> {
    let relative = parent
        .strip_prefix(root)
        .map_err(|_| "source API member escaped staging root")?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err("source API member parent is noncanonical".into());
        };
        current.push(name);
        match fs::symlink_metadata(&current) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                create_private_directory(&current)?;
            }
            Err(error) => return Err(error.into()),
            Ok(metadata) => {
                if !metadata.is_dir() || metadata.file_type().is_symlink() {
                    return Err("source API member parent is unsafe".into());
                }
            }
        }
    }
    Ok(())
}

fn prepare_operation(context: &Context, plan: &Plan) -> Result<()> {
    match fs::symlink_metadata(&context.operation_dir) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            create_private_directory(&context.operation_dir)?;
            sync_directory(&context.root_parent)?;
        }
        Err(error) => return Err(error.into()),
        Ok(metadata) => {
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err("source API operation path is unsafe".into());
            }
        }
    }
    let operation = json!({
        "schema": OPERATION_SCHEMA,
        "operation_id": context.request.operation_id,
        "request_digest": context.request_digest,
        "plan_id": plan.plan_id,
        "policy": context.request.policy.as_str(),
        "action": plan.action.as_str(),
        "before_revision": plan.before_revision,
        "selected_revision": plan.selected_revision,
        "freshness": plan.freshness,
        "relation": plan.relation,
        "origin_sha256": context.origin_digest,
        "snapshot_root_sha256": context.root_digest,
        "manifest_output_sha256": context.output_digest,
        "branch": context.request.branch,
        "selector": context.request.selector,
    });
    publish_json_exact(&context.operation_dir.join("operation.json"), &operation)
}

fn checkpoint(
    context: &Context,
    plan: &Plan,
    sequence: u8,
    stage: &str,
    evidence: Option<&str>,
) -> Result<()> {
    let value = json!({
        "schema": CHECKPOINT_SCHEMA,
        "operation_id": context.request.operation_id,
        "request_digest": context.request_digest,
        "plan_id": plan.plan_id,
        "sequence": sequence,
        "stage": stage,
        "evidence": evidence,
    });
    publish_json_exact(
        &context
            .operation_dir
            .join(format!("checkpoint-{sequence:02}-{stage}.json")),
        &value,
    )
}

fn reconciliation_required(context: &Context, plan: &Plan, reason: &str) -> Result<u8> {
    let value = json!({
        "schema": REPORT_SCHEMA,
        "status": "reconciliation_required",
        "operation_id": context.request.operation_id,
        "policy": context.request.policy.as_str(),
        "action": plan.action.as_str(),
        "plan_id": plan.plan_id,
        "before_revision": plan.before_revision,
        "selected_remote_revision": plan.selected_revision,
        "freshness": plan.freshness,
        "relation": plan.relation,
        "journaled": true,
        "mutation_performed": false,
        "next_action": "operator_review_before_any_retry",
        "reason": reason,
    });
    let _ = publish_json_exact(&context.operation_dir.join("outcome-unknown.json"), &value);
    println!("{value}");
    Ok(5)
}

fn build_receipt(
    context: &Context,
    plan: &Plan,
    snapshot: &Snapshot,
    manifest_publication: &str,
) -> Result<Value> {
    let mut value = json!({
        "schema": RECEIPT_SCHEMA,
        "operation_id": context.request.operation_id,
        "request_digest": context.request_digest,
        "plan_id": plan.plan_id,
        "action": plan.action.as_str(),
        "before_revision": plan.before_revision,
        "selected_revision": plan.selected_revision,
        "after_revision": snapshot.revision,
        "tree_sha": snapshot.tree_sha,
        "manifest_sha256": snapshot.manifest_sha256,
        "manifest_publication": manifest_publication,
        "freshness": plan.freshness,
        "relation": plan.relation,
        "acquisition": "github_api_exact_blob_snapshot",
        "network_request_count": snapshot.network_requests,
        "network_body_bytes": snapshot.network_body_bytes,
    });
    value["receipt_id"] = json!(content_id("source-api-materialization-receipt", &value)?);
    Ok(value)
}

fn publish_receipt(context: &Context, receipt: &Value) -> Result<()> {
    publish_json_exact(&context.operation_dir.join("receipt.json"), receipt)
}

fn read_receipt(context: &Context, plan: &Plan) -> Result<Option<Value>> {
    let Some(bytes) = read_optional_safe_file(
        &context.operation_dir.join("receipt.json"),
        MAX_OPERATION_BYTES,
        "source API receipt",
    )?
    else {
        return Ok(None);
    };
    let value = parse_strict_json(&bytes, "source API receipt")?;
    require_canonical_json(&bytes, &value, "source API receipt")?;
    let object = exact_object(
        &value,
        &[
            "schema",
            "operation_id",
            "request_digest",
            "plan_id",
            "action",
            "before_revision",
            "selected_revision",
            "after_revision",
            "tree_sha",
            "manifest_sha256",
            "manifest_publication",
            "freshness",
            "relation",
            "acquisition",
            "network_request_count",
            "network_body_bytes",
            "receipt_id",
        ],
        "source API receipt",
    )?;
    if string_field(object, "schema")? != RECEIPT_SCHEMA
        || string_field(object, "operation_id")? != context.request.operation_id.as_str()
        || string_field(object, "request_digest")? != context.request_digest.as_str()
        || string_field(object, "plan_id")? != plan.plan_id.as_str()
        || string_field(object, "action")? != plan.action.as_str()
        || string_field(object, "freshness")? != plan.freshness
        || string_field(object, "relation")? != plan.relation
        || string_field(object, "acquisition")? != "github_api_exact_blob_snapshot"
        || optional_string_field(object, "before_revision")? != plan.before_revision
        || optional_string_field(object, "selected_revision")? != plan.selected_revision
    {
        return Err("source API receipt does not match its operation".into());
    }
    for name in ["after_revision", "tree_sha"] {
        let revision = string_field(object, name)?;
        if revision.len() != 40 || !git::oid(revision) {
            return Err("source API receipt has an invalid object identity".into());
        }
    }
    if !is_sha256(string_field(object, "manifest_sha256")?)
        || !matches!(
            string_field(object, "manifest_publication")?,
            "created" | "existing_exact"
        )
        || object["network_request_count"].as_u64().is_none()
        || object["network_body_bytes"].as_u64().is_none()
    {
        return Err("source API receipt contains invalid result fields".into());
    }
    let receipt_id = string_field(object, "receipt_id")?;
    let mut projection = value.clone();
    projection
        .as_object_mut()
        .ok_or("source API receipt projection is invalid")?
        .remove("receipt_id");
    if receipt_id != content_id("source-api-materialization-receipt", &projection)? {
        return Err("source API receipt identity is invalid".into());
    }
    Ok(Some(value))
}

fn publish_json_exact(path: &Path, value: &Value) -> Result<()> {
    let bytes = pretty_json_bytes(value)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.is_file()
                || metadata.file_type().is_symlink()
                || metadata.len() > MAX_OPERATION_BYTES
            {
                return Err("durable source API record is unsafe".into());
            }
            let existing = read_safe_file(path, MAX_OPERATION_BYTES, "durable source API record")?;
            if existing != bytes {
                return Err("durable source API record conflicts with existing content".into());
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut options = OpenOptions::new();
            options.create_new(true).write(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(path)?;
            let result = file.write_all(&bytes).and_then(|_| file.sync_all());
            drop(file);
            if result.is_err() {
                let _ = fs::remove_file(path);
            }
            result?;
            sync_directory(
                path.parent()
                    .ok_or("durable source API record has no parent")?,
            )?;
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

fn plan_report(context: &Context, plan: &Plan, status: &str, journaled: bool) -> Value {
    json!({
        "schema": REPORT_SCHEMA,
        "status": status,
        "operation_id": context.request.operation_id,
        "policy": context.request.policy.as_str(),
        "action": plan.action.as_str(),
        "plan_id": plan.plan_id,
        "before_revision": plan.before_revision,
        "selected_remote_revision": plan.selected_revision,
        "freshness": plan.freshness,
        "relation": plan.relation,
        "acquisition": "github_api_exact_blob_snapshot",
        "journaled": journaled,
        "mutation_performed": false,
        "next_action": match status {
            "authorization_required" => "repeat_with_exact_authorization",
            "reconciliation_required" => "operator_review_before_any_retry",
            "exact_api_acquisition_unavailable" => "retry_exact_revision_or_review_network",
            _ => "none",
        },
    })
}

fn observation_report(
    context: &Context,
    status: &str,
    before: Option<&str>,
    selected: Option<&str>,
    freshness: &str,
    next_action: &str,
) -> Value {
    json!({
        "schema": REPORT_SCHEMA,
        "status": status,
        "operation_id": context.request.operation_id,
        "policy": context.request.policy.as_str(),
        "before_revision": before,
        "selected_remote_revision": selected,
        "freshness": freshness,
        "acquisition": "github_api_exact_blob_snapshot",
        "mutation_performed": false,
        "next_action": next_action,
    })
}

fn receipt_report(context: &Context, plan: &Plan, receipt: &Value, replayed: bool) -> Value {
    json!({
        "schema": REPORT_SCHEMA,
        "status": if replayed { "completed_replay" } else { "materialized" },
        "operation_id": context.request.operation_id,
        "policy": context.request.policy.as_str(),
        "action": plan.action.as_str(),
        "plan_id": plan.plan_id,
        "receipt_id": receipt["receipt_id"],
        "before_revision": receipt["before_revision"],
        "selected_remote_revision": receipt["selected_revision"],
        "after_revision": receipt["after_revision"],
        "tree_sha": receipt["tree_sha"],
        "manifest_sha256": receipt["manifest_sha256"],
        "manifest_publication": receipt["manifest_publication"],
        "freshness": receipt["freshness"],
        "relation": receipt["relation"],
        "acquisition": receipt["acquisition"],
        "network_request_count": receipt["network_request_count"],
        "network_body_bytes": receipt["network_body_bytes"],
        "replayed": replayed,
        "mutation_performed": !replayed,
    })
}

fn observation_exit(plan: &Plan) -> u8 {
    match plan.relation {
        "current" => 0,
        "unverified" => 4,
        _ => 3,
    }
}

fn execution_exit(plan: &Plan) -> u8 {
    if plan.freshness == "unverified_current" {
        4
    } else if plan.relation == "different" && plan.action == Action::Publish {
        3
    } else {
        0
    }
}

fn normalize_github_origin(origin: &str) -> Result<(String, RepositoryIdentity)> {
    super::remote::validate_origin(origin)?;
    let tail = origin
        .strip_prefix("https://")
        .ok_or("source API origin must use HTTPS")?;
    let mut parts = tail.split('/');
    let host = parts.next().ok_or("source API origin is missing a host")?;
    let owner = parts
        .next()
        .ok_or("source API origin is missing an owner")?;
    let repository = parts
        .next()
        .ok_or("source API origin is missing a repository")?;
    if parts.next().is_some()
        || !host.eq_ignore_ascii_case("github.com")
        || !valid_github_component(owner)
    {
        return Err("source API fallback requires an explicit github.com repository".into());
    }
    let repository = repository.strip_suffix(".git").unwrap_or(repository);
    if !valid_github_component(repository) {
        return Err("source API repository name is invalid".into());
    }
    Ok((
        format!("https://github.com/{owner}/{repository}.git"),
        RepositoryIdentity {
            owner: owner.to_owned(),
            repository: repository.to_owned(),
        },
    ))
}

fn valid_github_component(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 100
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        && value != "."
        && value != ".."
}

fn percent_encode(value: &str) -> String {
    let mut output = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            output.push(char::from(byte));
        } else {
            output.push('%');
            output.push(char::from(b"0123456789ABCDEF"[usize::from(byte >> 4)]));
            output.push(char::from(b"0123456789ABCDEF"[usize::from(byte & 0x0f)]));
        }
    }
    output
}

fn paths_overlap(left: &Path, right: &Path) -> bool {
    left == right || left.starts_with(right) || right.starts_with(left)
}

fn canonical_target(path: &Path, label: &str) -> Result<PathBuf> {
    if !path.is_absolute() {
        return Err(format!("{label} must be absolute").into());
    }
    if path
        .components()
        .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err(format!("{label} contains a noncanonical component").into());
    }
    let parent = path
        .parent()
        .ok_or_else(|| format!("{label} has no parent"))?;
    let parent = parent.canonicalize()?;
    let name = path
        .file_name()
        .ok_or_else(|| format!("{label} has no final component"))?;
    let target = parent.join(name);
    if let Ok(metadata) = fs::symlink_metadata(&target)
        && metadata.file_type().is_symlink()
    {
        return Err(format!("{label} is a symlink").into());
    }
    Ok(target)
}

fn create_private_directory(path: &Path) -> Result<()> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)?;
    Ok(())
}

fn sync_directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        File::open(path)?.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

fn pretty_json_bytes(value: &Value) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn parse_strict_json(bytes: &[u8], label: &str) -> Result<Value> {
    let text = std::str::from_utf8(bytes).map_err(|_| format!("{label} is not UTF-8 JSON"))?;
    repository::validate_json(text)?;
    Ok(serde_json::from_str(text)?)
}

fn require_canonical_json(bytes: &[u8], value: &Value, label: &str) -> Result<()> {
    if pretty_json_bytes(value)? != bytes {
        return Err(format!("{label} is not canonical JSON").into());
    }
    Ok(())
}

fn read_safe_file(path: &Path, max_bytes: u64, label: &str) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > max_bytes {
        return Err(format!("{label} is not a safe bounded regular file").into());
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take(max_bytes + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 != metadata.len() || bytes.len() as u64 > max_bytes {
        return Err(format!("{label} changed while being read").into());
    }
    Ok(bytes)
}

fn read_optional_safe_file(path: &Path, max_bytes: u64, label: &str) -> Result<Option<Vec<u8>>> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
        Ok(_) => Ok(Some(read_safe_file(path, max_bytes, label)?)),
    }
}

fn exact_object<'a>(
    value: &'a Value,
    fields: &[&str],
    label: &str,
) -> Result<&'a Map<String, Value>> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("{label} must be a JSON object"))?;
    if object.len() != fields.len() || object.keys().any(|key| !fields.contains(&key.as_str())) {
        return Err(format!("{label} has missing or unknown fields").into());
    }
    Ok(object)
}

fn string_field<'a>(object: &'a Map<String, Value>, name: &str) -> Result<&'a str> {
    object
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{name} must be a string").into())
}

fn optional_string_field(object: &Map<String, Value>, name: &str) -> Result<Option<String>> {
    match object.get(name) {
        Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        _ => Err(format!("{name} must be a string or null").into()),
    }
}

fn valid_token(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_text(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && !value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_content_id(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(&format!("{prefix}:sha256:"))
        .is_some_and(is_sha256)
}

fn path_digest(path: &Path) -> Result<String> {
    let value = path.to_str().ok_or("source API paths must be UTF-8")?;
    Ok(digest_text(value))
}

fn digest_text(value: &str) -> String {
    hex(&Sha256::digest(value.as_bytes()))
}

fn content_id(prefix: &str, value: &Value) -> Result<String> {
    Ok(format!(
        "{prefix}:sha256:{}",
        hex(&Sha256::digest(serde_json::to_vec(value)?))
    ))
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
