# E7-A transport-independent service operations

**Status:** normative public operation surface.

## Common request fields

Every operation uses a closed tagged request containing, as applicable:

```text
protocol/session/profile IDs
exact SessionViewSetId and optional OverlayGenerationId
exact document/root/entity/finding/result/continuation IDs
requested capability and output profile
position/source coordinate profile
privacy/authorization/consumer profile
budgets/cancellation/progress token
OperationId + CanonicalRequestDigest for effecting work
```

Unknown/cross-operation fields fail. Protocol request IDs remain adapter metadata.

## Protocol/session operations

### `protocol_status`

Returns implemented protocol profiles, owner capability availability, authorization/privacy requirements, and unavailable/deferred features. It performs no workspace acquisition.

### `protocol_profile_validate`

Read-only validation of one exact protocol profile/spec/transport/capability bundle. It never downloads or selects a newer specification.

### `session_initialize`

Validates handshake/authentication/authorization, creates one session, optionally invokes one explicit workspace bind, and returns exact server capabilities. No successful result before mandatory resources are retained and initialization closure completes.

### `session_capabilities`

Returns the exact capability decision for an active session and its current `SessionViewSet`, preserving unavailable/denied/`NotEvaluated` reasons.

### `session_rebind_exact`

Explicitly acquires and validates a new exact retained generation set under expected-old guards. It never silently reapplies overlays or migrates continuations.

### `session_shutdown`

Stops admission, cancels/drains according to profile, reconciles effects, closes owner resources, and returns one terminal receipt. No detached cleanup.

## Workspace operations

### `workspace_bind`

Resolves a permitted outer selector exactly once, maps explicit root handles, acquires project/graph/reference/source views, validates compatibility, and creates `WorkspaceBindingReceipt` plus `SessionViewSet`.

### `workspace_status`

Reports exact bound identities, capabilities, coverage/conflicts, overlays, active operations, retention, and closure state without refreshing current.

## Document operations

### `document_open`

Creates an exact first overlay record from full text and version.

### `document_change`

Validates monotonic version and edit sequence against exact prior content, creates a new full-content digest and overlay generation, and requests owner overlay analysis only when the operation profile requires it.

### `document_save`

Records the save notification and optional client text/digest as observation. It never writes disk or rebinds project current.

### `document_close`

Creates a new overlay generation without the document and closes session-local state. It never saves content.

### `document_snapshot_validate`

Read-only closure check over document identity/version/content/edit/source-map/base/overlay-analysis records.

## Analysis/navigation operations

### `analysis_diagnostics`

Invokes exact owner diagnostic/rule operations over the captured published/overlay view and returns an immutable diagnostic result set. Supports exact previous-result IDs only under the frozen profile. Empty findings do not imply complete clean status without full capability coverage.

### `analysis_hover`

Maps an exact document position to owner entity/source records and returns bounded typed hover facts, ReferenceView facts, source handles, evidence, coverage, and conflicts. No generated free-form authority.

### `analysis_definition`

Returns exact accepted definition targets and source locations. Multiple targets remain multiple; unresolved/partial remains explicit. Same name or best search candidate is not silently selected.

### `analysis_references`

Returns bounded exact reference relations/source locations under owner graph/project coverage. A partial result cannot claim all references.

### `analysis_symbols`

Returns bounded exact document/workspace symbols or exact search candidates according to the explicit request type. Document and workspace symbol semantics remain separate; fuzzy search remains Candidate.

### `analysis_code_actions`

Returns advisory typed action candidates already supported by finding/remediation owners. No `WorkspaceEdit`, arbitrary command, shell, or source mutation in E7-A.

### `analysis_resolve_action`

Resolves one exact action ID to additional evidence, validation requirements, and advisory edit-plan data. It does not execute or apply the plan.

## Context/search operations

### `context_request`

Invokes the existing exact `wow-context` service path from exact root IDs. Source excerpts remain subject to privacy/license/budget/source-boundary profiles. Protocol/client text does not become context authority.

### `search_request`

Invokes exact-generation search through the existing E4 service path. Ranked outputs remain candidates; no automatic selection or context handoff unless the request supplies an existing explicit selection receipt through the owner contract.

MCP external-candidate discovery continues through the E6-B operations, not a hidden E7 shortcut.

## Operation control

### `operation_cancel`

Signals cancellation for one exact session/request/OperationId, records receipt, and returns actual owner/terminal/effect state. It cannot claim cancellation when outcome is unknown.

### `operation_status`

Reports exact operation state, progress receipts, owner effects, result/error/retention/closure state. It never resumes or retries by itself.

## Common result rules

- One result binds one exact captured session view and overlay generation.
- Owner result identities and evidence are preserved.
- `Partial`, `CandidateOnly`, `Blocked`, `ConflictBlocked`, `Truncated`, `OutcomeUnknown`, `NotEvaluated`, `Cancelled`, and `Failed` remain explicit.
- Authorization denial is not absence or semantic invalidity.
- Protocol projection cannot upgrade confidence/negative authority.
- No success before mandatory retention and reverse closure.
- No raw owner handles, SQL, credentials, private paths, or unrestricted source.

## Deferred operations

Not present in E7-A:

```text
completion
signature help
rename
formatting
execute command
apply workspace edit
semantic tokens
inlay hints
call/type hierarchy
file operations
MCP sampling/elicitation/prompts/dynamic tools
arbitrary resource/file/URL fetch
remote daemon/session administration
release/build/install/update
```

A future operation requires a reviewed owner seam, protocol profile, security model, fixtures, and implementation freeze.
