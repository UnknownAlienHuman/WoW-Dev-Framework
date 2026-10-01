# Exact GitHub API/blob source snapshots

`cargo xtask materialize-source-api REQUEST.json` is the explicit checkout-free
fallback for public GitHub source inputs. It resolves one branch once, then reads
one immutable commit, its recursive tree and every selected blob by exact Git
object ID. It does not clone, fetch a pack, create or update refs, execute source
files, run hooks/submodules/package managers, or mutate an existing snapshot.

This lane is separate from `materialize-source`, which owns managed Git checkouts.
Both use the same manifest selection and limits: at most 200,000 regular tracked
files, 32 MiB per selected file, 256 MiB selected bytes and a 64 MiB manifest.
The API lane additionally limits the recursive-tree response to 64 MiB, total
response bodies to 384 MiB and the operation to 4,096 HTTP requests.

## Request contract

The request is strict JSON. Duplicate, unknown or missing fields fail.

```json
{
  "schema": "wow-source-api-materialization-request/1",
  "operation_id": "gethe-live-api-2026-10-01-a",
  "policy": "auto",
  "snapshot_root": "/absolute/private/source-snapshot",
  "manifest_output": "/absolute/new-or-exact/source-manifest.json",
  "origin": "https://github.com/Gethe/wow-ui-source.git",
  "branch": "live",
  "selector": "live",
  "authorization": null
}
```

Only a credential-free `https://github.com/<owner>/<repository>[.git]` origin is
admitted. Roots and outputs are absolute and must be pairwise disjoint from the
private operation/staging paths. The origin and host paths are represented only
by SHA-256 bindings in durable records and never enter the manifest or result.

Policy behavior matches the managed-checkout lane:

- `auto` executes one exact observed plan;
- `prompt` returns a content-addressed plan without mutation and requires an exact
  `plan_id`, `before_revision` and `selected_revision` authorization on replay;
- `never` observes and reports only and creates no snapshot, manifest or journal.

The selected branch is not re-resolved after authorization or durable replay.
An already completed operation returns its stored receipt without HTTP or
filesystem effects.

## API and blob admission

The adapter uses GitHub's REST commit/tree/blob endpoints over HTTPS. Curl's
per-user config is disabled, redirects are refused, protocol is fixed to HTTPS,
response bodies and deadlines are bounded, and stderr is discarded. Public reads
work without credentials. An optional `WOW_SOURCE_GITHUB_TOKEN` environment value
may be supplied for GitHub rate limits; it is sent through curl stdin, never via
request JSON, command arguments, manifests, receipts or logs.

The recursive tree must be complete (`truncated: false`). Tree, submodule, symlink
and special entries are handled explicitly: directory records are traversed,
regular `100644`/`100755` blobs are admitted, and every other leaf kind rejects.
Selected paths are canonical and case-insensitively unique. Each blob is fetched
through its exact tree object ID, length-checked, independently re-hashed through
an isolated SHA-1 Git object-format repository and SHA-256 hashed for the manifest.

The resulting manifest remains schema version 1 and identifies acquisition as:

```text
github_api_exact_blob_snapshot
```

## Immutable snapshot and recovery

Files are written create-new into a private same-parent staging directory. The
snapshot contains only selected source members plus:

```text
.wow-source-manifest.json
.wow-source-api-snapshot.json
```

The marker binds the origin digest, branch, selector, exact commit/tree, manifest
identity and coverage. Before installation the entire staging tree is reopened;
every file length and SHA-256 plus the exact file set are verified. Installation
uses a same-parent rename into an absent root. Existing snapshots are immutable:
a later branch revision is reported as different and requires a new destination.

Durable operation/checkpoint/receipt records bracket staging and installation.
On restart the adapter may adopt an exact installed or staged snapshot. Partial,
foreign or contradictory staging/root state produces exit 5 and
`operator_review_before_any_retry`; it is not deleted or retried blindly. A normal
network failure removes only staging created by that attempt when safe and returns
exit 4 while retaining the exact plan for retry.

## Nonclaims

The GitHub origin is operator-approved public input. TLS and exact object IDs do
not authenticate authorship, establish license/redistribution rights, prove branch
freshness after the one observation, certify semantic compatibility or provide a
hostile-network sandbox. The stdout/body bounds cap admitted response content;
they do not prove a hard bound on all lower-layer network traffic. There is no
background scheduler, provider discovery or automatic fallback from one transport
to another.
