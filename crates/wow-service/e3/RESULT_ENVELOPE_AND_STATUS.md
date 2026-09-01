# E3-C context service envelopes and status

**Status:** normative public result contract.

## Result family

```text
ContextServiceOperationResult
    Status(ContextStatusResult)
    Map(ContextServiceResultEnvelope<ContextMapPayload>)
    Inspect(ContextServiceResultEnvelope<ContextInspectPayload>)
    Build(ContextServiceResultEnvelope<ContextBuildPayload>)
    Continue(ContextServiceResultEnvelope<ContextContinuePayload>)
    Validate(ContextServiceResultEnvelope<ContextValidationPayload>)
    Render(ContextServiceResultEnvelope<ContextRenderPayload>)
    Failure(ContextServiceFailureResult)
    Cancelled(ContextServiceCancelledResult)
```

The outer serialization contract is versioned separately from E0-F check/status while reusing common service identity/error primitives where compatible.

## Common envelope

```text
ContextServiceResultEnvelope<T>
    envelope schema/version
    result_id
    operation
    request_id
    service_configuration_id
    original selector metadata
    ResolvedContextSelectionSet
    ContextUniverseSetId when applicable
    exact root/profile/budget/privacy/renderer IDs
    ContextServiceStatus
    operation payload T
    context artifact/validation IDs
    capability/coverage/conflict/omission/loss summaries
    budget/truncation/continuation state
    safe warnings
    ResourceClosureReportId and Complete state
    owner/service/context schema and producer versions
    canonical digest
```

No raw lease, connection, transaction, path, source body outside validated artifact, credential, timing, terminal, or process data.

## Status precedence

```text
failed
cancelled
not_evaluated (no useful semantic artifact)
truncated (useful artifact stopped by explicit hard bound)
partial (useful artifact with other incomplete requested scopes)
complete
```

Lower records remain orthogonal. A `truncated` result may also contain partial input coverage; the payload exposes both.

## `complete`

The requested service operation finished, all mandatory owner/context/validation/renderer/closure stages completed, and no requested semantic scope is partial/truncated/NotEvaluated.

For `context_validate`, `complete` means validation executed completely. The payload may say `Invalid`; validity is not confused with service execution success.

## `partial`

A useful validated operation payload exists, but one or more requested nonbudget scopes are incomplete under an explicitly degradable policy. Exact blockers/omissions/coverage remain present.

## `truncated`

A useful validated payload stopped at a declared byte/token/depth/fanout/item/time work bound and records every known omission plus continuation state. Truncation can never be rendered as complete.

## `not_evaluated`

No useful semantic artifact could be produced because a required capability/profile/evidence lane was explicitly unsupported, unavailable, or NotEvaluated and the operation contract permits a structured nonfailure response.

Missing exact generations, invalid requests, incompatibility, security errors, malformed owner outputs, and resource-close failures are failures, not `not_evaluated`.

## `failed`

Represented as `ContextServiceFailureResult`; no malformed/partially finalized success envelope. Safe existing context artifact IDs may be referenced for diagnostics, but incomplete closure can prohibit returning source-containing bytes.

## `cancelled`

Represented as `ContextServiceCancelledResult`; no success envelope or late result.

## Operation payloads

### `ContextStatusPayload`

```text
resolved exact publication/reference identities
owner-port/contract/profile registry state
available/partial/deferred capabilities
coverage/conflicts/warnings
last-known-good and failed targets distinctly labeled
no context-generation pass claim
```

### `ContextMapPayload`

Exact unchanged `ProjectMap`, validation record, optional continuation, no service-generated semantic rendering.

### `ContextInspectPayload`

Ordered exact L0/L1 artifacts, root mapping, validation, optional frontier/continuation.

### `ContextBuildPayload`

Exact `ContextSemanticPack`, zero/more `RenderedContextArtifact`, validation records, continuation retention receipts, budget/omission summaries.

### `ContextContinuePayload`

Original/next continuation identities, next semantic pack/page and render artifacts, accumulated total-budget state, replaced/released retention receipts.

### `ContextValidationPayload`

```text
validation_state = Valid | Invalid | NotEvaluated
artifact kind/schema/digest
structural and optional owner-closure checks
errors/warnings/coverage
nonrepair assertion
```

### `ContextRenderPayload`

Validated input semantic pack identity, exact renderer/tokenizer profiles, one unchanged semantic pack reference, rendered artifact and validation.

## Selector metadata

Record symbolic selector kind and exact resolved record IDs. The symbolic token does not replace exact identities in result semantics.

For continuation/validate exact-owner/render exact-owner operations, selectors are exact artifact-bound identities; current is absent.

## Resource closure

Every success envelope requires:

```text
resource_closure_state = Complete
```

The closure report ID may be included in canonical service identity because it is a stable safe result over the declared resource set, but timings/handles/process state are excluded.

## Canonical ordering

- selections in fixed primary/platform/reference order;
- roots in normalized request order or profile-defined set order;
- artifacts by operation order/root/profile ID;
- warnings/errors by stage/owner/code/subject ID;
- capabilities/coverage/conflicts/omissions by canonical owner keys;
- renderers by normalized profile order;
- no map/hash/query-completion order.

## Identity exclusions

Do not include:

```text
clock/duration
host/process/thread
terminal/locale/color
CLI command spelling/input file path
lock/connection/transaction ID
physical SQLite/WAL/page state
log text/progress
current pointer after resolution
cache hit/miss/location
```

## Failure and cancellation outputs

Failure/cancelled results include bounded safe acquisition/closure summaries so leaked resources can be diagnosed without exposing raw handles or source.

Primary failure remains primary; close failures are linked secondary errors unless they reveal integrity/security compromise.

## Validation

- all result refs resolve;
- exact selection/universe/artifact/profile coherence;
- status matches lower coverage/truncation/outcome;
- invalid-artifact payload is not misclassified as service failure;
- no success before closure;
- source/privacy fields allowed;
- canonical bytes deterministic;
- app exit mapping does not feed back into status.
