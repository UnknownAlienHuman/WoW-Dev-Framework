# Explicit source checkout updates

Implemented scope: a native, guarded fast-forward of an existing trusted
standalone checkout plus a separate explicit managed-checkout materializer. These
are internal acquisition tools in `tools/xtask`, not an analyzer, updater daemon,
package installer or durable source-store service.
It follows the [reference acquisition boundary](../crates/wow-reference/e1/SOURCE_SNAPSHOT_AND_PROFILES.md):
materialize outside the source library, then supply an exact revision/manifest.

## Commands and authorization

```sh
cargo xtask check-source /path/to/checkout live
cargo xtask update-source /path/to/checkout live --expected-head <observed-local-SHA>
cargo xtask materialize-source /path/to/request.json
cargo xtask materialize-source-api /path/to/request.json
```

`check-source` stays read-only. `update-source` is a separately authorized write:
the caller explicitly supplies the local HEAD it observed. No startup check or
background mutation is introduced. An automation may invoke this command under
its operator-approved update policy; interactive clients should propose it first.
The same command supports any explicitly selected branch of a compatible public
HTTPS source checkout, including separately maintained Blizzard and annotation
resources. No provider name, WoW build or donor revision selects hidden behavior.

For `update-source`, the root must already exist and be its repository's top-level
directory, with a regular `.git` directory. Bare repositories, linked worktrees,
sparse checkouts, detached HEAD, an unexpected branch or stale expected HEAD are
rejected. Missing managed roots may use `materialize-source`; a checkout-free
public GitHub fallback is available as `materialize-source-api`. Their strict
contracts are documented in [SOURCE_MATERIALIZATION.md](SOURCE_MATERIALIZATION.md)
and [SOURCE_API_MATERIALIZATION.md](SOURCE_API_MATERIALIZATION.md). Private/SSH
authentication, provider discovery and whole-source scheduling are not implemented.

## Managed materialization lane

`materialize-source` accepts one explicit credential-free GitHub HTTPS origin,
branch, selector, managed root and manifest output. `never` observes without
mutation; `prompt` returns a content-addressed exact plan and requires a repeat
with that plan's revisions; `auto` executes the exact observed plan. A missing
managed root is built in private same-parent staging from one selected commit,
inventory-verified, then installed without overwriting an existing path.

Only checkouts carrying the exact managed marker are automatically fast-forwarded.
An ordinary local checkout remains preferred for reads and exact manifest
publication but is not mutated by this lane. Durable operation/checkpoint/receipt
records are loaded before any new remote observation. Completed replay performs no
Git or filesystem effect; partial/foreign staging, manifest conflicts and unknown
update locks require operator review. The command still relies on trusted operator
configuration and bounded Git subprocesses; it is not a hostile-host network-byte
sandbox or background freshness service.

## Checkout-free API/blob lane

`materialize-source-api` accepts the same explicit policy model but does not create
or update Git refs. It observes one GitHub branch once, resolves the exact commit
and recursive tree, then downloads every selected regular blob by its exact Git
object ID. The adapter enforces finite request, recursive-tree, per-body, total-body
and manifest-selection budgets, independently re-hashes each Git blob, stages an
immutable snapshot and reopens every member before installation. Existing snapshot
roots are never updated in place. Durable replay adopts only an exact staged or
installed snapshot; foreign/partial state requires operator review. This path is a
trusted-public-GitHub adapter, not authorship/license proof or a lower-layer hostile
network sandbox.

## One observation, one guarded transition

1. Acquire a cooperative lock and inspect local branch, HEAD, origin, config,
   index flags and tracked/staged/untracked state.
2. Observe the explicitly configured origin's selected branch once. If identical,
   return `current` without fetching or moving any ref.
3. Fetch that exact observed commit, not the moving branch. Recheck local state
   after network IO. Tags, tracking refs, FETCH_HEAD and submodules are not updated.
4. Require the previous HEAD to be an ancestor. A divergent, unproven or incomplete
   history is a refusal, not permission to reset or invent a merge.
5. Write the selected revision to the journal, perform one `merge --ff-only`,
   recheck the resulting checkout, then close/remove the lock before success.

No reset, rebase, stash, new branch, merge commit or implicit retry is performed.
The original source objects remain accessible; an old manifest still identifies
and verifies its old revision. A new generation must resolve and inventory the
new revision independently. A remote advance after selection belongs to the next
operation, never an extra fetch or mixed-generation build in this operation.

The implementation uses Git's [fast-forward and ignored-file controls](https://git-scm.com/docs/git-merge)
and [exact-ref fetch controls](https://git-scm.com/docs/git-fetch). Old local
tracking refs intentionally remain unchanged; do not use them as proof of new
network freshness. The returned selected/after revisions are explicit evidence.

## Safety and isolation

The checkout must be exclusively owned by the caller for the operation. The
cooperative lock serializes this updater; it cannot lock out arbitrary editors,
other Git commands or a hostile operating system. State changes during network
calls are detected. This is not a cross-process transaction over every file.

Hooks, inherited Git environment/configuration, external attribute configuration,
askpass callbacks, lazy fetch and recursive submodule updates are disabled.
Local includes, URL rewrites, credentials, filters, custom branch merge options,
sparse/index concealment flags, grafts and unfinished Git operations are refused.
HTTPS redirects are refused rather than silently accepting a changed endpoint.
Ignored-file overwrites are explicitly forbidden too; private ignored files are
not discarded merely because ordinary status reports a clean checkout.

The existing Git runner bounds captured output and process deadlines. These do
not cap all downloaded object bytes or provide a hostile-host sandbox. Origins
must be operator-approved; URL syntax and TLS alone do not authorize a provider,
prove authorship, certify licenses or validate changed addon/API semantics.
Reports contain fixed status classes and revisions, not URLs, host paths,
credentials, source contents, commit messages or raw Git/network errors.

## Results and interrupted updates

| Exit | Result | Meaning |
|---|---|---|
| 0 | `current` / `updated` | Expected local state and final revision verified; lock closed. |
| 2 | Invalid or refused request | Caller/state/configuration guard failed. |
| 3 | `not_fast_forward_or_incomplete_history` | Ancestry was not established; no checkout update attempted. |
| 4 | `unverified_current` / `fetch_failed` | Remote observation/acquisition failed; do not claim current data. |
| 5 | `reconciliation_required` | Apply/read-back/closure failed; no automatic retry or reset. |

JSON schema `wow-source-update/1` records before, selected and observed-after
revisions. Missing observations are null. Fetch failure may have added objects;
it is not an assertion that the whole Git directory is unchanged. A successful
update concerns the selected remote observation only, not eternal freshness.

`.git/wow-source-update.lock` is a minimal journal, not a general durable
catalog. A direct `update-source` interruption still requires explicit inspection.
A matching managed materialization operation may close an applying lock only when
its recorded expected/selected revisions match and a clean read-back already shows
the exact selected revision. Every mismatch remains operator review; there is no
stale-lock deletion, blind retry, guessed rollback or fabricated success.

## Update cost and boundary

Compatible Gethe/Ketho changes update only source/resource data: existing Git
objects are reused, and no Rust host or WASM rebuild is needed to fetch them.
Changed algorithms still use separately built, approved WASM modules through the
[existing bridge](WASM_BRIDGES.md). Network/filesystem authority stays in the
native acquisition adapter and is never given to the guest. This step does not
implement incremental analysis, new type semantics or signed module distribution.

Rust tests use real SHA-1/SHA-256 and shallow repositories through a test-only local
transport. They cover drift, divergence, moving remotes, mutation during network
calls, hidden edits, ignored collisions, lock retention, disabled hooks, privacy,
old-manifest verification and idempotence. CI additionally executes the production
HTTPS path on explicitly cloned current donor sources; that path may correctly
return `current` when the clone already matches the remote.
