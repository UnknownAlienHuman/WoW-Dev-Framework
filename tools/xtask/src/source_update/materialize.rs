//! Managed source checkout materialization with exact plans and durable local receipts.
//!
//! This adapter is intentionally narrow: one explicit GitHub HTTPS origin, one branch,
//! one managed root and one manifest output. It never discovers providers, runs source
//! code, resets operator data or deletes an uncertain staging/update lock.

use super::remote::Remote;
use super::{lock, remote, state, update_selected};
use crate::{Result, git, manifest};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

const REQUEST_SCHEMA: &str = "wow-source-materialization-request/1";
const REPORT_SCHEMA: &str = "wow-source-materialization-result/1";
const PLAN_SCHEMA: &str = "wow-source-materialization-plan/1";
const OPERATION_SCHEMA: &str = "wow-source-materialization-operation/1";
const CHECKPOINT_SCHEMA: &str = "wow-source-materialization-checkpoint/1";
const RECEIPT_SCHEMA: &str = "wow-source-materialization-receipt/1";
const MARKER_SCHEMA: &str = "wow-managed-source-checkout/1";
const MAX_REQUEST_BYTES: u64 = 1024 * 1024;
const MAX_OPERATION_BYTES: u64 = 1024 * 1024;
const MAX_MARKER_BYTES: u64 = 16 * 1024;
const TOKEN_HEX_BYTES: usize = 12;

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
            _ => Err("source materialization policy must be auto, prompt or never".into()),
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
    Clone,
    Update,
    Publish,
}

impl Action {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "clone_managed_checkout" => Ok(Self::Clone),
            "fast_forward_managed_checkout" => Ok(Self::Update),
            "publish_exact_local_manifest" => Ok(Self::Publish),
            _ => Err("source materialization action is invalid".into()),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Clone => "clone_managed_checkout",
            Self::Update => "fast_forward_managed_checkout",
            Self::Publish => "publish_exact_local_manifest",
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
    managed_root: PathBuf,
    manifest_output: PathBuf,
    origin: String,
    branch: String,
    selector: String,
    authorization: Option<Authorization>,
}

#[derive(Debug)]
struct Context {
    request: Request,
    root: PathBuf,
    root_parent: PathBuf,
    output: PathBuf,
    origin: String,
    origin_digest: String,
    root_digest: String,
    output_digest: String,
    request_digest: String,
    operation_dir: PathBuf,
    staging: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ownership {
    Managed,
    Explicit,
}

#[derive(Debug)]
struct Checkout {
    state: state::State,
    ownership: Ownership,
}

#[derive(Debug)]
enum RootObservation {
    Missing,
    Checkout(Checkout),
}

#[derive(Debug, Clone)]
struct Plan {
    action: Action,
    before_revision: Option<String>,
    selected_revision: Option<String>,
    freshness: &'static str,
    relation: &'static str,
    managed: bool,
    plan_id: String,
}

enum ExistingOperation {
    None,
    Incomplete,
    Registered(Plan),
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
                return Err("never policy cannot resume a mutating operation".into());
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
                        "network_unavailable_for_missing_checkout",
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
        let origin = normalize_github_origin(&request.origin)?;
        let root = canonical_target(&request.managed_root, "managed root")?;
        let output = canonical_target(&request.manifest_output, "manifest output")?;
        let root_parent = root
            .parent()
            .ok_or("managed root has no parent")?
            .to_path_buf();
        if output.starts_with(&root) {
            return Err("manifest output must be outside the managed checkout".into());
        }
        git::isolated_text(
            &root_parent,
            &[
                "check-ref-format",
                &format!("refs/heads/{}", request.branch),
            ],
        )?;
        let origin_digest = digest_text(&origin);
        let root_digest = path_digest(&root)?;
        let output_digest = path_digest(&output)?;
        let request_identity = json!({
            "schema": REQUEST_SCHEMA,
            "operation_id": request.operation_id,
            "policy": request.policy.as_str(),
            "managed_root_sha256": root_digest,
            "manifest_output_sha256": output_digest,
            "origin_sha256": origin_digest,
            "branch": request.branch,
            "selector": request.selector,
        });
        let request_digest = content_id("source-materialization-request", &request_identity)?;
        let operation_token = digest_text(&request.operation_id);
        let token = &operation_token[..TOKEN_HEX_BYTES * 2];
        let operation_dir = root_parent.join(format!(".wow-source-materialization-{token}"));
        let staging = root_parent.join(format!(".wow-source-materialization-{token}.staging"));
        let paths = [&root, &output, &operation_dir, &staging];
        if paths.iter().enumerate().any(|(index, left)| {
            paths[index + 1..]
                .iter()
                .any(|right| paths_overlap(left, right))
        }) {
            return Err("source materialization paths overlap".into());
        }
        Ok(Self {
            request,
            root,
            root_parent,
            output,
            origin,
            origin_digest,
            root_digest,
            output_digest,
            request_digest,
            operation_dir,
            staging,
        })
    }
}

fn parse_request(path: &Path) -> Result<Request> {
    let bytes = read_safe_file(path, MAX_REQUEST_BYTES, "source materialization request")?;
    let value = parse_strict_json(&bytes, "source materialization request")?;
    let object = exact_object(
        &value,
        &[
            "schema",
            "operation_id",
            "policy",
            "managed_root",
            "manifest_output",
            "origin",
            "branch",
            "selector",
            "authorization",
        ],
        "source materialization request",
    )?;
    if string_field(object, "schema")? != REQUEST_SCHEMA {
        return Err("unsupported source materialization request schema".into());
    }
    let operation_id = string_field(object, "operation_id")?.to_owned();
    if !valid_token(&operation_id, 128) {
        return Err("source materialization operation ID is invalid".into());
    }
    let policy = Policy::parse(string_field(object, "policy")?)?;
    let managed_root = PathBuf::from(string_field(object, "managed_root")?);
    let manifest_output = PathBuf::from(string_field(object, "manifest_output")?);
    let origin = string_field(object, "origin")?.to_owned();
    let branch = string_field(object, "branch")?.to_owned();
    let selector = string_field(object, "selector")?.to_owned();
    if !valid_text(&branch, 256) || !valid_text(&selector, 256) {
        return Err("source branch or selector label is invalid".into());
    }
    let authorization = match object.get("authorization") {
        Some(Value::Null) | None => None,
        Some(value) => Some(parse_authorization(value)?),
    };
    Ok(Request {
        operation_id,
        policy,
        managed_root,
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
        "source materialization authorization",
    )?;
    let plan_id = string_field(object, "plan_id")?.to_owned();
    if !valid_content_id(&plan_id, "source-materialization-plan") {
        return Err("source materialization authorization plan ID is invalid".into());
    }
    let before_revision = optional_string_field(object, "before_revision")?;
    let selected_revision = optional_string_field(object, "selected_revision")?;
    for revision in [before_revision.as_deref(), selected_revision.as_deref()]
        .into_iter()
        .flatten()
    {
        if !git::oid(revision) {
            return Err("source materialization authorization revision is invalid".into());
        }
    }
    if let (Some(before), Some(selected)) = (&before_revision, &selected_revision)
        && before.len() != selected.len()
    {
        return Err("source materialization authorization revisions use different formats".into());
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
                return Err("managed source root is not a safe directory".into());
            }
            let root = state::root(&context.root)?;
            if root != context.root {
                return Err("managed source root canonical identity changed".into());
            }
            let observed = state::inspect(&root)?;
            if normalize_github_origin(&observed.origin)? != context.origin {
                return Err(
                    "source checkout origin differs from the configured GitHub origin".into(),
                );
            }
            if observed.branch != context.request.branch.as_str() {
                return Err("source checkout is on an unexpected operator-owned branch".into());
            }
            let ownership = read_marker(context, &root)?;
            Ok(RootObservation::Checkout(Checkout {
                state: observed,
                ownership,
            }))
        }
    }
}

fn read_marker(context: &Context, root: &Path) -> Result<Ownership> {
    let path = root.join(".git/wow-source-managed.json");
    let bytes = match read_optional_safe_file(&path, MAX_MARKER_BYTES, "managed checkout marker")? {
        Some(value) => value,
        None => return Ok(Ownership::Explicit),
    };
    let value = parse_strict_json(&bytes, "managed checkout marker")?;
    require_canonical_json(&bytes, &value, "managed checkout marker")?;
    let object = exact_object(
        &value,
        &["schema", "origin_sha256", "branch"],
        "managed checkout marker",
    )?;
    if string_field(object, "schema")? != MARKER_SCHEMA
        || string_field(object, "origin_sha256")? != context.origin_digest.as_str()
        || string_field(object, "branch")? != context.request.branch.as_str()
    {
        return Err("managed checkout marker does not match the requested source".into());
    }
    Ok(Ownership::Managed)
}

fn observed_plan(context: &Context, observation: &RootObservation) -> Result<Option<Plan>> {
    let remote_root = match observation {
        RootObservation::Missing => context.root_parent.as_path(),
        RootObservation::Checkout(_) => context.root.as_path(),
    };
    let selected = remote::Https
        .head(remote_root, &context.origin, &context.request.branch)
        .ok()
        .filter(|value| git::oid(value));
    match observation {
        RootObservation::Missing => {
            let Some(selected) = selected else {
                return Ok(None);
            };
            if selected.len() != 40 {
                return Err(
                    "configured GitHub source returned an unsupported object format".into(),
                );
            }
            Ok(Some(build_plan(
                context,
                Action::Clone,
                None,
                Some(selected),
                "selected_remote_observation_only",
                "missing",
                true,
            )?))
        }
        RootObservation::Checkout(checkout) => {
            let before = checkout.state.head.clone();
            let (freshness, relation) = match selected.as_deref() {
                Some(value) if value.len() == before.len() && value == before => {
                    ("selected_remote_observation_only", "current")
                }
                Some(value) if value.len() == before.len() => {
                    ("selected_remote_observation_only", "different")
                }
                _ => ("unverified_current", "unverified"),
            };
            let selected = selected.filter(|value| value.len() == before.len());
            let action = if checkout.ownership == Ownership::Managed
                && selected.as_deref().is_some_and(|value| value != before)
            {
                Action::Update
            } else {
                Action::Publish
            };
            Ok(Some(build_plan(
                context,
                action,
                Some(before),
                selected,
                freshness,
                relation,
                checkout.ownership == Ownership::Managed,
            )?))
        }
    }
}

fn authorized_plan(
    context: &Context,
    observation: &RootObservation,
    authorization: &Authorization,
) -> Result<Plan> {
    let (action, before, managed, relation) = match observation {
        RootObservation::Missing => {
            if authorization.before_revision.is_some()
                || authorization
                    .selected_revision
                    .as_deref()
                    .is_none_or(|value| value.len() != 40)
            {
                return Err(
                    "clone authorization does not match the missing managed checkout".into(),
                );
            }
            (Action::Clone, None, true, "missing")
        }
        RootObservation::Checkout(checkout) => {
            if authorization.before_revision.as_deref() != Some(checkout.state.head.as_str()) {
                return Err("source changed after the materialization plan was authorized".into());
            }
            let selected = authorization.selected_revision.as_deref();
            if selected.is_some_and(|value| value.len() != checkout.state.head.len()) {
                return Err("authorized source revision uses another object format".into());
            }
            let action = if checkout.ownership == Ownership::Managed
                && selected.is_some_and(|value| value != checkout.state.head)
            {
                Action::Update
            } else {
                Action::Publish
            };
            let relation = match selected {
                Some(value) if value == checkout.state.head => "current",
                Some(_) => "different",
                None => "unverified",
            };
            (
                action,
                Some(checkout.state.head.clone()),
                checkout.ownership == Ownership::Managed,
                relation,
            )
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
        managed,
    )?;
    if plan.plan_id != authorization.plan_id {
        return Err("source materialization authorization does not match the exact plan".into());
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
    managed: bool,
) -> Result<Plan> {
    let identity = json!({
        "schema": PLAN_SCHEMA,
        "request_digest": context.request_digest,
        "action": action.as_str(),
        "before_revision": before_revision,
        "selected_revision": selected_revision,
        "freshness": freshness,
        "relation": relation,
        "managed_checkout": managed,
    });
    let plan_id = content_id("source-materialization-plan", &identity)?;
    Ok(Plan {
        action,
        before_revision,
        selected_revision,
        freshness,
        relation,
        managed,
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
        return Err("source materialization operation path is unsafe".into());
    }
    let Some(bytes) = read_optional_safe_file(
        &context.operation_dir.join("operation.json"),
        MAX_OPERATION_BYTES,
        "source materialization operation",
    )?
    else {
        return Ok(ExistingOperation::Incomplete);
    };
    let value = parse_strict_json(&bytes, "source materialization operation")?;
    require_canonical_json(&bytes, &value, "source materialization operation")?;
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
            "managed_checkout",
            "origin_sha256",
            "managed_root_sha256",
            "manifest_output_sha256",
            "branch",
            "selector",
        ],
        "source materialization operation",
    )?;
    if string_field(object, "schema")? != OPERATION_SCHEMA
        || string_field(object, "operation_id")? != context.request.operation_id.as_str()
        || string_field(object, "request_digest")? != context.request_digest.as_str()
        || string_field(object, "policy")? != context.request.policy.as_str()
        || string_field(object, "origin_sha256")? != context.origin_digest.as_str()
        || string_field(object, "managed_root_sha256")? != context.root_digest.as_str()
        || string_field(object, "manifest_output_sha256")? != context.output_digest.as_str()
        || string_field(object, "branch")? != context.request.branch.as_str()
        || string_field(object, "selector")? != context.request.selector.as_str()
    {
        return Err("source materialization operation belongs to another request".into());
    }
    let action = Action::parse(string_field(object, "action")?)?;
    let before_revision = optional_string_field(object, "before_revision")?;
    let selected_revision = optional_string_field(object, "selected_revision")?;
    validate_revision_pair(before_revision.as_deref(), selected_revision.as_deref())?;
    let freshness = parse_freshness(string_field(object, "freshness")?)?;
    let relation = parse_relation(string_field(object, "relation")?)?;
    let managed = bool_field(object, "managed_checkout")?;
    let plan = build_plan(
        context,
        action,
        before_revision,
        selected_revision,
        freshness,
        relation,
        managed,
    )?;
    if string_field(object, "plan_id")? != plan.plan_id.as_str() {
        return Err("source materialization operation plan identity is invalid".into());
    }
    Ok(ExistingOperation::Registered(plan))
}

fn validate_supplied_authorization(context: &Context, plan: &Plan) -> Result<()> {
    let Some(authorization) = &context.request.authorization else {
        return Ok(());
    };
    if authorization.plan_id != plan.plan_id.as_str()
        || authorization.before_revision != plan.before_revision
        || authorization.selected_revision != plan.selected_revision
    {
        return Err(
            "source materialization authorization conflicts with the durable operation".into(),
        );
    }
    Ok(())
}

fn parse_freshness(value: &str) -> Result<&'static str> {
    match value {
        "selected_remote_observation_only" => Ok("selected_remote_observation_only"),
        "unverified_current" => Ok("unverified_current"),
        _ => Err("source materialization freshness is invalid".into()),
    }
}

fn parse_relation(value: &str) -> Result<&'static str> {
    match value {
        "current" => Ok("current"),
        "different" => Ok("different"),
        "missing" => Ok("missing"),
        "unverified" => Ok("unverified"),
        _ => Err("source materialization relation is invalid".into()),
    }
}

fn validate_revision_pair(before: Option<&str>, selected: Option<&str>) -> Result<()> {
    for revision in [before, selected].into_iter().flatten() {
        if !git::oid(revision) {
            return Err("source materialization operation revision is invalid".into());
        }
    }
    if let (Some(before), Some(selected)) = (before, selected)
        && before.len() != selected.len()
    {
        return Err("source materialization operation revisions use different formats".into());
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

    let checkout = match plan.action {
        Action::Clone => match execute_clone(context, plan) {
            Ok(value) => value,
            Err(_) => return reconciliation_required(context, plan, "clone_or_install_uncertain"),
        },
        Action::Update => match execute_update(context, plan) {
            Ok(UpdateResult::Ready(value)) => value,
            Ok(UpdateResult::Terminal(code, status)) => {
                println!("{}", plan_report(context, plan, status, true));
                return Ok(code);
            }
            Err(_) => return reconciliation_required(context, plan, "update_outcome_uncertain"),
        },
        Action::Publish => match execute_publish_preflight(context, plan) {
            Ok(value) => value,
            Err(_) => return reconciliation_required(context, plan, "local_snapshot_changed"),
        },
    };

    let revision = checkout.state.head.clone();
    checkpoint(context, plan, 7, "manifest_intent", Some(&revision))?;
    let manifest_value = manifest::build(&context.root, &revision, &context.request.selector)?;
    if manifest_value["source"]["revision"].as_str() != Some(revision.as_str()) {
        return reconciliation_required(context, plan, "manifest_revision_mismatch");
    }
    let manifest_sha256 = manifest_value["manifest_sha256"]
        .as_str()
        .ok_or("materialized source manifest is missing its identity")?
        .to_owned();
    let publication = match manifest::publish_exact(&context.output, &manifest_value) {
        Ok(value) => value,
        Err(_) => return reconciliation_required(context, plan, "manifest_publication_conflict"),
    };
    checkpoint(
        context,
        plan,
        8,
        "manifest_published",
        Some(&manifest_sha256),
    )?;
    let receipt = build_receipt(
        context,
        plan,
        &revision,
        &manifest_sha256,
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

fn execute_clone(context: &Context, plan: &Plan) -> Result<Checkout> {
    let selected = plan
        .selected_revision
        .as_deref()
        .ok_or("clone plan is missing selected revision")?;
    match observe_root(context)? {
        RootObservation::Checkout(checkout)
            if checkout.ownership == Ownership::Managed && checkout.state.head == selected =>
        {
            checkpoint(context, plan, 6, "installed", Some(selected))?;
            return Ok(checkout);
        }
        RootObservation::Checkout(_) => {
            return Err("managed root appeared with foreign or unexpected content".into());
        }
        RootObservation::Missing => {}
    }

    let staging_ready = match fs::symlink_metadata(&context.staging) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(error.into()),
        Ok(metadata) => {
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err("source staging path is unsafe".into());
            }
            let checkout = observe_checkout_at(context, &context.staging)?;
            if checkout.ownership != Ownership::Managed || checkout.state.head != selected {
                return Err("retained source staging does not match the exact plan".into());
            }
            true
        }
    };
    if !staging_ready {
        checkpoint(context, plan, 1, "staging_intent", Some(selected))?;
        clone_exact(context, selected)?;
    }
    let staged = observe_checkout_at(context, &context.staging)?;
    if staged.ownership != Ownership::Managed || staged.state.head != selected {
        return Err("source staging verification failed".into());
    }
    let staged_manifest = manifest::build(&context.staging, selected, &context.request.selector)?;
    let staged_manifest_id = staged_manifest["manifest_sha256"]
        .as_str()
        .ok_or("staged source manifest is missing its identity")?;
    checkpoint(
        context,
        plan,
        2,
        "staging_verified",
        Some(staged_manifest_id),
    )?;
    checkpoint(context, plan, 3, "install_intent", Some(selected))?;
    if fs::symlink_metadata(&context.root).is_ok() {
        return Err("managed root appeared before exact staging installation".into());
    }
    fs::rename(&context.staging, &context.root)?;
    sync_directory(&context.root_parent)?;
    checkpoint(context, plan, 4, "installed", Some(selected))?;
    let installed = match observe_root(context)? {
        RootObservation::Checkout(value) => value,
        RootObservation::Missing => return Err("installed managed checkout disappeared".into()),
    };
    if installed.ownership != Ownership::Managed || installed.state.head != selected {
        return Err("installed managed checkout failed exact read-back".into());
    }
    checkpoint(context, plan, 6, "checkout_verified", Some(selected))?;
    Ok(installed)
}

fn clone_exact(context: &Context, selected: &str) -> Result<()> {
    if selected.len() != 40 {
        return Err("GitHub managed clone requires a SHA-1 commit identity".into());
    }
    create_private_directory(&context.staging)?;
    git::isolated_text(
        &context.staging,
        &[
            "init",
            "-q",
            &format!("--initial-branch={}", context.request.branch),
        ],
    )?;
    git::isolated_text(
        &context.staging,
        &["remote", "add", "origin", &context.origin],
    )?;
    git::isolated_text(
        &context.staging,
        &[
            "-c",
            "credential.helper=",
            "-c",
            "core.askPass=",
            "-c",
            "http.followRedirects=false",
            "-c",
            "protocol.allow=never",
            "-c",
            "protocol.https.allow=always",
            "fetch",
            "--depth=1",
            "--no-tags",
            "--no-write-fetch-head",
            "--no-recurse-submodules",
            "--no-auto-maintenance",
            "--refmap=",
            &context.origin,
            selected,
        ],
    )?;
    if state::resolve(&context.staging, selected)? != selected {
        return Err("managed clone fetched a different commit".into());
    }
    git::isolated_text(
        &context.staging,
        &[
            "checkout",
            "-q",
            "--no-track",
            "--no-recurse-submodules",
            "-b",
            &context.request.branch,
            selected,
        ],
    )?;
    write_marker(context, &context.staging)?;
    let checkout = observe_checkout_at(context, &context.staging)?;
    if checkout.state.head != selected || checkout.ownership != Ownership::Managed {
        return Err("managed clone failed exact state verification".into());
    }
    Ok(())
}

enum UpdateResult {
    Ready(Checkout),
    Terminal(u8, &'static str),
}

fn execute_update(context: &Context, plan: &Plan) -> Result<UpdateResult> {
    let before = plan
        .before_revision
        .as_deref()
        .ok_or("update plan is missing its previous revision")?;
    let selected = plan
        .selected_revision
        .as_deref()
        .ok_or("update plan is missing its selected revision")?;
    let checkout = match observe_root(context)? {
        RootObservation::Checkout(value) => value,
        RootObservation::Missing => return Err("managed checkout disappeared before update".into()),
    };
    if checkout.ownership != Ownership::Managed {
        return Err("automatic update refuses an operator-owned checkout".into());
    }
    if checkout.state.head == selected {
        if let Some(record) = lock::read(&context.root)? {
            if record.expected_head != before || record.selected_revision != selected {
                return Err("retained source update lock belongs to another operation".into());
            }
            lock::release_reconciled(&context.root, before, selected)?;
            sync_directory(&context.root.join(".git"))?;
        }
        checkpoint(context, plan, 6, "checkout_verified", Some(selected))?;
        return Ok(UpdateResult::Ready(checkout));
    }
    if checkout.state.head != before {
        return Err("managed checkout changed after the exact update plan".into());
    }
    if lock::read(&context.root)?.is_some() {
        return Err("retained applying lock requires operator reconciliation".into());
    }
    checkpoint(context, plan, 3, "update_intent", Some(selected))?;
    let (code, report) = update_selected(
        &context.root,
        &context.request.branch,
        before,
        selected,
        &remote::Https,
    )?;
    match code {
        0 => {
            let updated = match observe_root(context)? {
                RootObservation::Checkout(value) => value,
                RootObservation::Missing => return Err("updated checkout disappeared".into()),
            };
            if updated.ownership != Ownership::Managed || updated.state.head != selected {
                return Err("updated checkout failed exact read-back".into());
            }
            checkpoint(context, plan, 6, "checkout_verified", Some(selected))?;
            Ok(UpdateResult::Ready(updated))
        }
        3 => Ok(UpdateResult::Terminal(
            3,
            "not_fast_forward_or_incomplete_history",
        )),
        4 => Ok(UpdateResult::Terminal(4, "fetch_failed")),
        5 => Err("source update requires reconciliation".into()),
        _ => {
            let _ = report;
            Err("source update returned an unsupported result".into())
        }
    }
}

fn execute_publish_preflight(context: &Context, plan: &Plan) -> Result<Checkout> {
    let checkout = match observe_root(context)? {
        RootObservation::Checkout(value) => value,
        RootObservation::Missing => return Err("local source checkout is missing".into()),
    };
    if plan.before_revision.as_deref() != Some(checkout.state.head.as_str()) {
        return Err("local source changed before manifest publication".into());
    }
    if lock::read(&context.root)?.is_some() {
        return Err("source checkout has an unresolved update lock".into());
    }
    checkpoint(
        context,
        plan,
        6,
        "checkout_verified",
        Some(&checkout.state.head),
    )?;
    Ok(checkout)
}

fn observe_checkout_at(context: &Context, path: &Path) -> Result<Checkout> {
    let root = state::root(path)?;
    if root != path {
        return Err("source checkout root identity changed".into());
    }
    let observed = state::inspect(path)?;
    if normalize_github_origin(&observed.origin)? != context.origin
        || observed.branch != context.request.branch.as_str()
    {
        return Err("source checkout does not match the configured origin and branch".into());
    }
    let ownership = read_marker(context, path)?;
    Ok(Checkout {
        state: observed,
        ownership,
    })
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
                return Err("source materialization operation path is unsafe".into());
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
        "managed_checkout": plan.managed,
        "origin_sha256": context.origin_digest,
        "managed_root_sha256": context.root_digest,
        "manifest_output_sha256": context.output_digest,
        "branch": context.request.branch,
        "selector": context.request.selector,
    });
    publish_json_exact(&context.operation_dir.join("operation.json"), &operation)?;
    Ok(())
}

fn checkpoint(
    context: &Context,
    plan: &Plan,
    ordinal: u8,
    phase: &str,
    evidence: Option<&str>,
) -> Result<()> {
    let value = json!({
        "schema": CHECKPOINT_SCHEMA,
        "request_digest": context.request_digest,
        "plan_id": plan.plan_id,
        "ordinal": ordinal,
        "phase": phase,
        "evidence": evidence,
    });
    publish_json_exact(
        &context
            .operation_dir
            .join(format!("checkpoint-{ordinal:02}-{phase}.json")),
        &value,
    )?;
    sync_directory(&context.operation_dir)
}

fn reconciliation_required(context: &Context, plan: &Plan, reason: &str) -> Result<u8> {
    let value = json!({
        "schema": CHECKPOINT_SCHEMA,
        "request_digest": context.request_digest,
        "plan_id": plan.plan_id,
        "phase": "outcome_unknown",
        "reason": reason,
        "next_action": "operator_review_before_any_retry",
        "blind_retry": "prohibited",
    });
    let _ = publish_json_exact(&context.operation_dir.join("outcome-unknown.json"), &value);
    println!(
        "{}",
        plan_report(context, plan, "reconciliation_required", true)
    );
    Ok(5)
}

fn build_receipt(
    context: &Context,
    plan: &Plan,
    revision: &str,
    manifest_sha256: &str,
    manifest_publication: &str,
) -> Result<Value> {
    let mut value = json!({
        "schema": RECEIPT_SCHEMA,
        "operation_id": context.request.operation_id,
        "request_digest": context.request_digest,
        "plan_id": plan.plan_id,
        "policy": context.request.policy.as_str(),
        "action": plan.action.as_str(),
        "before_revision": plan.before_revision,
        "selected_revision": plan.selected_revision,
        "after_revision": revision,
        "manifest_sha256": manifest_sha256,
        "manifest_publication": manifest_publication,
        "freshness": plan.freshness,
        "relation": plan.relation,
        "managed_checkout": plan.managed,
    });
    let receipt_id = content_id("source-materialization-receipt", &value)?;
    value["receipt_id"] = json!(receipt_id);
    Ok(value)
}

fn publish_receipt(context: &Context, receipt: &Value) -> Result<()> {
    publish_json_exact(&context.operation_dir.join("receipt.json"), receipt)?;
    sync_directory(&context.operation_dir)
}

fn read_receipt(context: &Context, plan: &Plan) -> Result<Option<Value>> {
    let path = context.operation_dir.join("receipt.json");
    let Some(bytes) = read_optional_safe_file(&path, MAX_OPERATION_BYTES, "source receipt")? else {
        return Ok(None);
    };
    let value = parse_strict_json(&bytes, "source materialization receipt")?;
    require_canonical_json(&bytes, &value, "source materialization receipt")?;
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
            "after_revision",
            "manifest_sha256",
            "manifest_publication",
            "freshness",
            "relation",
            "managed_checkout",
            "receipt_id",
        ],
        "source materialization receipt",
    )?;
    if string_field(object, "schema")? != RECEIPT_SCHEMA
        || string_field(object, "operation_id")? != context.request.operation_id.as_str()
        || string_field(object, "request_digest")? != context.request_digest.as_str()
        || string_field(object, "plan_id")? != plan.plan_id.as_str()
        || string_field(object, "policy")? != context.request.policy.as_str()
        || string_field(object, "action")? != plan.action.as_str()
        || optional_string_field(object, "before_revision")? != plan.before_revision
        || optional_string_field(object, "selected_revision")? != plan.selected_revision
        || string_field(object, "freshness")? != plan.freshness
        || string_field(object, "relation")? != plan.relation
        || bool_field(object, "managed_checkout")? != plan.managed
    {
        return Err("source materialization receipt belongs to another operation".into());
    }
    let after_revision = string_field(object, "after_revision")?;
    if !git::oid(after_revision) {
        return Err("source materialization receipt has an invalid final revision".into());
    }
    let manifest_sha256 = string_field(object, "manifest_sha256")?;
    if !is_sha256(manifest_sha256) {
        return Err("source materialization receipt has an invalid manifest digest".into());
    }
    if !matches!(
        string_field(object, "manifest_publication")?,
        "created" | "existing_exact"
    ) {
        return Err("source materialization receipt has an invalid publication state".into());
    }
    let receipt_id = string_field(object, "receipt_id")?;
    let mut identity = value.clone();
    identity
        .as_object_mut()
        .ok_or("source materialization receipt is invalid")?
        .remove("receipt_id");
    if content_id("source-materialization-receipt", &identity)? != receipt_id {
        return Err("source materialization receipt identity is invalid".into());
    }
    Ok(Some(value))
}

fn write_marker(context: &Context, root: &Path) -> Result<()> {
    let marker = json!({
        "schema": MARKER_SCHEMA,
        "origin_sha256": context.origin_digest,
        "branch": context.request.branch,
    });
    publish_json_exact(&root.join(".git/wow-source-managed.json"), &marker)?;
    sync_directory(&root.join(".git"))
}

fn publish_json_exact(path: &Path, value: &Value) -> Result<()> {
    let bytes = pretty_json_bytes(value)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.is_file()
                || metadata.file_type().is_symlink()
                || metadata.len() > MAX_OPERATION_BYTES
            {
                return Err("durable source record is not a safe bounded regular file".into());
            }
            let existing = read_safe_file(path, MAX_OPERATION_BYTES, "durable source record")?;
            if existing != bytes {
                return Err("durable source record conflicts with existing content".into());
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
            file.write_all(&bytes)?;
            file.sync_all()?;
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
        "managed_checkout": plan.managed,
        "journaled": journaled,
        "mutation_performed": false,
        "next_action": match status {
            "authorization_required" => "repeat_with_exact_authorization",
            "reconciliation_required" => "operator_review_before_any_retry",
            "fetch_failed" => "retry_exact_selected_revision_or_review_network",
            "not_fast_forward_or_incomplete_history" => "preserve_checkout_and_review_divergence",
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
        "manifest_sha256": receipt["manifest_sha256"],
        "manifest_publication": receipt["manifest_publication"],
        "freshness": receipt["freshness"],
        "relation": receipt["relation"],
        "managed_checkout": receipt["managed_checkout"],
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

fn normalize_github_origin(origin: &str) -> Result<String> {
    super::remote::validate_origin(origin)?;
    let tail = origin
        .strip_prefix("https://")
        .ok_or("managed source origin must use HTTPS")?;
    let mut parts = tail.split('/');
    let host = parts
        .next()
        .ok_or("managed source origin is missing a host")?;
    let owner = parts
        .next()
        .ok_or("managed source origin is missing an owner")?;
    let repository = parts
        .next()
        .ok_or("managed source origin is missing a repository")?;
    if parts.next().is_some()
        || !host.eq_ignore_ascii_case("github.com")
        || !valid_github_component(owner)
    {
        return Err("managed fallback must be an explicit github.com repository".into());
    }
    let repository = repository.strip_suffix(".git").unwrap_or(repository);
    if !valid_github_component(repository) {
        return Err("managed fallback repository name is invalid".into());
    }
    Ok(format!("https://github.com/{owner}/{repository}.git"))
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
    crate::repository::validate_json(text)?;
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

fn bool_field(object: &Map<String, Value>, name: &str) -> Result<bool> {
    object
        .get(name)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("{name} must be a boolean").into())
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
        .is_some_and(|tail| {
            tail.len() == 64
                && tail
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
}

fn path_digest(path: &Path) -> Result<String> {
    let value = path
        .to_str()
        .ok_or("source materialization paths must be UTF-8")?;
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
