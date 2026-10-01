# Managed source materialization

`cargo xtask materialize-source REQUEST.json` is the explicit managed-checkout
adapter for source inputs. It extends the existing read-only `check-source`, exact
manifest producer and guarded `update-source`; it does not replace those owners or
introduce a daemon, provider discovery, scheduler, reset/clean path or source code
execution.

## Request contract

The request is strict JSON. Unknown or missing fields fail.

```json
{
  "schema": "wow-source-materialization-request/1",
  "operation_id": "gethe-live-2026-10-01-a",
  "policy": "auto",
  "managed_root": "/absolute/private/source-root",
  "manifest_output": "/absolute/new-or-exact/source-manifest.json",
  "origin": "https://github.com/Gethe/wow-ui-source.git",
  "branch": "live",
  "selector": "live",
  "authorization": null
}
```

Only an explicit credential-free `https://github.com/<owner>/<repository>[.git]`
origin is admitted. Roots and outputs are absolute, canonical sibling-safe host
inputs; neither origin nor paths enter the result, manifest, receipt identity or
logs. The selected branch is resolved at most once for an unapproved operation.

Policy behavior is distinct:

- `auto` executes one exact observed clone/update/publication plan.
- `prompt` reports a content-addressed plan without mutation. To authorize it,
  repeat the same request with `authorization` containing the returned `plan_id`,
  `before_revision` and `selected_revision`. The exact selected revision is
  fetched directly; the moving branch is not observed again.
- `never` observes and reports only. It creates no checkout, manifest, journal or
  receipt.

An authorization object has this shape:

```json
{
  "plan_id": "source-materialization-plan:sha256:<64 lowercase hex>",
  "before_revision": null,
  "selected_revision": "<exact Git commit>"
}
```

For an existing checkout, `before_revision` is the observed local HEAD. For a
missing checkout it is null. A local change invalidates the plan rather than
silently authorizing a new one.

## Managed ownership and local preference

A checkout created by this command contains a private
`.git/wow-source-managed.json` marker bound to the configured origin digest and
branch. Only a matching managed checkout is eligible for `auto`/authorized-prompt
fast-forward. An ordinary explicit local checkout remains usable for exact manifest
publication but is never retargeted, switched or automatically updated; use the
separate guarded `update-source` command under explicit operator authorization.

A missing managed root is cloned into a deterministic private same-parent staging
directory from one selected commit. The adapter disables credentials, redirects,
hooks, recursive submodules, maintenance and moving ref updates. It verifies the
branch/HEAD/origin/clean state, builds the ordinary bounded source manifest from
raw Git objects, and only then renames staging into the absent destination. It
never overwrites an existing root.

## Durable effects and recovery

Mutation creates a private same-parent operation directory identified by the
operation/request digest. Records are immutable create-new JSON files:

```text
operation.json
checkpoint-00-registered.json
...
receipt.json
outcome-unknown.json
```

Checkpoints bracket staging, install, managed fast-forward, exact checkout
read-back and manifest publication. Repeating a completed operation returns the
stored receipt without Git/network/filesystem effects.

Before continuing an interrupted operation the adapter inspects the real managed
root, deterministic staging directory and existing source-update lock. It may
adopt an exact installed/staged checkout or an exact completed update. It removes
an applying update lock only when the lock's expected/selected revisions match the
operation and the clean checkout is already at the selected revision. A partial
or foreign staging directory, conflicting manifest, changed checkout, mismatched
lock or ambiguous effect returns exit 5, retains evidence and reports
`operator_review_before_any_retry`. No uncertain lock or staging tree is deleted,
and no unknown apply is blindly retried.

## Checkout-free API fallback

When a Git checkout or pack fetch is not desired, use the separate
`cargo xtask materialize-source-api REQUEST.json` lane. It preserves the exact
`auto`/`prompt`/`never` authorization model while materializing a read-only source
snapshot from one GitHub commit/tree and exact per-blob reads under fixed request
and response-body budgets. It never converts the snapshot into a managed checkout
or silently switches transport. See [API/blob source snapshots](SOURCE_API_MATERIALIZATION.md).

## Identity, bounds and nonclaims

The resulting manifest remains `schema_version: 1` and uses the existing fixed
selection/budgets: 200,000 tracked entries, 32 MiB per selected file, 256 MiB
selected bytes and 64 MiB manifest. Git subprocess output and execution deadlines
remain bounded. The configured repository is trusted operator input; this slice
does not provide a hostile-host network-byte sandbox, authenticated authorship,
license review, semantic compatibility, runtime truth, automatic currentness or
background scheduling.

Exit classes:

| Exit | Meaning |
|---|---|
| 0 | Exact current/clone/update/publication completed or replayed. |
| 2 | Invalid request, unsafe path/state or contract failure. |
| 3 | Prompt authorization required, remote drift, missing read-only root or non-fast-forward refusal. |
| 4 | Network observation/fetch unavailable; any local result is explicitly unverified-current. |
| 5 | Interrupted or ambiguous effect requires operator reconciliation. |
