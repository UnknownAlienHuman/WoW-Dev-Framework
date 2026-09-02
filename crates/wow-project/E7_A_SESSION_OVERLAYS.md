# E7-A project-owned session overlay seam

**Status:** normative cross-crate seam; implementation has not started.

`wow-project` owns all source identity, document content, editor version, coordinate conversion, overlay generation, and overlay-backed project-analysis semantics. `wow-service` coordinates sessions and calls this seam; applications/transports never patch project state directly.

## Operations

```text
project_overlay_open
project_overlay_replace_full
project_overlay_apply_incremental
project_overlay_save
project_overlay_close
project_overlay_snapshot_get
project_overlay_validate
project_overlay_reconcile
```

`project_overlay_apply_incremental` is optional and cannot be advertised until the incremental UTF-16 profile passes all conformance tests.

## Base identity

Every overlay request binds one exact retained:

```text
ProjectStore
ProjectPublication
ProjectGeneration
project profile
owner file/source handle
```

No current/latest refresh occurs inside an overlay lifetime or request. A file URI/path is resolved only through the exact project owner's source map and path policy.

## Overlay open

Input:

```text
session/client scope
exact base generation and file handle
transport document URI as data
monotonic editor document version
full UTF-8 source bytes and digest
encoding/line-ending/coordinate profile
privacy/license/resource policy
OperationId + CanonicalRequestDigest
```

The owner validates URI-to-file identity, source kind, file membership, content limits, encoding, and base compatibility. It publishes one immutable `SessionOverlayDocument` and `ProjectOverlayGeneration`.

## Full replacement

`project_overlay_replace_full` requires the exact prior overlay document/generation and a strictly greater editor version. It validates the complete new bytes and emits a new immutable generation.

No in-place mutable document is exposed outside the owner. Internal mutable analyzer state may exist but must publish exact immutable snapshot identities for consumers.

## Incremental changes

The optional profile accepts ordered changes in LSP-style UTF-16 coordinates plus exact prior version/digest. The owner maintains the UTF-16-to-UTF-8 coordinate map.

Validation includes:

```text
valid UTF-8 prior document
valid UTF-16 code-unit boundaries
no split surrogate pair
range start <= end and in bounds
change order and overlap policy
integer/size overflow checks
exact final bytes and digest
line-ending policy
```

The operation either produces the exact declared final document or fails. It never guesses ranges, applies against a newer version, or reorders ambiguous edits.

## Save

Save records exact editor intent and optional supplied saved text/digest. It does not read disk, mutate the base publication, or assume a new project generation.

The overlay state becomes `SavedPendingBaseRefresh`. Closure to a new base requires an explicit project refresh/publication and exact owner proof that the corresponding file digest matches.

A differing refreshed digest is `Conflict`; no last-writer-wins behavior.

## Close

Close creates a new overlay generation without the document. It returns the session view to the exact bound base generation. It does not resolve current disk/project state and does not delete prior overlay records still retained by in-flight operations or artifacts.

## Overlay snapshot view

The owner may expose a narrow immutable `ProjectOverlaySnapshotView` containing:

```text
base project generation
ordered overlay file replacements
overlay document/version/digest/source handles
analyzer/project fact publication IDs produced for the overlay
capability/coverage/conflict state
retention/lease/cancellation state
```

Consumers cannot mutate it or enumerate source outside privacy/resource profiles.

## Analysis publication

Overlay analysis uses the exact base generation plus exact replacement set. It may publish session-private analyzer/project/graph candidate generations according to existing ownership rules. These are not canonical persistent project generations or current state by implication.

A result carries explicit `session_overlay` universe/provenance. Clean negative authority requires complete relevant overlay-aware coverage.

## Concurrency

Overlay updates use expected-current compare-and-swap on exact session overlay generation. Concurrent operations against one prior generation cannot both become the next generation. Reads bind immutable snapshots.

## Reconciliation

Every effecting operation returns/reconciles an exact owner receipt keyed by operation/request/base/session/document identity. Response loss is not proof of no update. Conflicting duplicate generations are quarantined.

## Hard boundaries

- no dependency on `wow-service`, `apps/wow`, LSP, MCP, or daemon protocol crates;
- no arbitrary editor/cwd/Git/WoW discovery;
- no execution of overlay source;
- no repository hook/build/script invocation;
- no source write/save/apply edit;
- no floating project/profile/current state;
- no cross-session overlay visibility;
- no guessed URI/path/version/range conflict resolution;
- no provider text or external Candidate metadata in project facts;
- no historical base generation mutation.

## Tests

Required owner tests cover full replacement, optional incremental UTF-16 conformance, multibyte/emoji/surrogate coordinates, CRLF/LF, stale/out-of-order versions, invalid ranges, oversized content, URI/path attacks, save/base-refresh equality and conflict, close/revert, concurrent CAS, response loss, cancellation, retention, 1/2/N workers, and deterministic output.