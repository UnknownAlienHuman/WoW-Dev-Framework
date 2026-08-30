# Service context acquisition

**Status:** normative E0-F snapshot-coherence protocol.

A service operation runs against one immutable set of lower-layer identities. Acquisition is not a best-effort collection of whichever components answer first.

## 1. Inputs

```text
ServiceContextRequest
    operation
    project_id
    ProjectGenerationSelector
    expected profile/reference identity: optional exact guard
    requested scope
    requested rules/policy
    budget/cancellation
```

Service configuration supplies the exact configured reference/project/analyzer/rule providers and fixture identities.

## 2. Selection flow

```text
validate service configuration
-> validate request/project/selector
-> resolve selector:
      Exact(id) -> require exact published snapshot
      CurrentPublished(project) -> atomically read one current snapshot identity
-> acquire immutable ProjectSnapshot / ProjectView
-> read its exact ProfileIdentity / ReferenceGenerationId / AnalyzerSnapshot binding
-> acquire exact ReferenceView for that profile/reference generation
-> validate exact AnalyzerSnapshot/pin/probe/config identity through ProjectSnapshot binding
-> acquire exact E0 RuleRegistry/fixture policy identity
-> acquire capability/coverage/conflict registries and budgets
-> validate all cross-component identities
-> construct immutable ServiceContextLease
```

No operation work begins before the lease validates.

## 3. Atomic `CurrentPublished` resolution

`CurrentPublished(ProjectId)` is resolved once:

```text
project registry current pointer read
-> immutable ProjectSnapshot acquired
-> exact ProjectGenerationId recorded
```

After acquisition:

- service does not reread current pointer;
- a newer publication does not change this request;
- result includes exact selected generation;
- selector may be supplemental metadata but not canonical generation identity;
- failure to acquire a coherent immutable snapshot rejects the request.

## 4. Exact selector

`Exact(ProjectGenerationId)` requires:

- snapshot exists and is published;
- project ID matches;
- snapshot is valid/available;
- no substitution by current/last-known-good/nearby generation;
- result uses the exact requested/acquired identity.

If unavailable, return typed `service_project_generation_unavailable` or context mismatch.

## 5. Cross-component equality checks

Require equality/compatibility of:

```text
ProfileIdentity
ReferenceGenerationId
ProjectGenerationId
ProjectSnapshot identity/digest
AnalyzerSnapshotId
accepted analyzer pin ID
compatibility probe/configuration identity
Main file manifest and source registry identity
E0 RuleRegistry ID/version set
fixture policy profile/reference/rule versions
schema/tool contract versions
```

Every source/evidence/fact/finding/coverage/conflict record later consumed must resolve to this lease context.

## 6. Capability acquisition

Acquire exact:

- reference capability/coverage/conflict records;
- project capability/coverage/deferred records;
- analyzer capability/coverage and generic finding availability;
- rule registry/descriptors/capability requirements;
- service operation budgets/deferred registry.

Do not flatten these into one `ready` boolean. The operation-specific gate evaluates narrow requirements later.

## 7. Project scope validation

Service resolves request scope through ProjectView:

```text
AllProjectFiles
FileIds[]
Explicit analyzer fact/function/use IDs
```

Reject:

- unknown/removed/stale file ID;
- path glob/fuzzy/host filesystem scope;
- source/fact ID from another generation;
- Library source as first-party project scope;
- scope over budget.

Scope canonicalization occurs before generic/rule collection.

## 8. Last-known-good

Status may report:

```text
current published generation
last-known-good generation
failed target/candidate generation
```

Acquisition rules:

- exact request for failed target cannot use last-known-good;
- `CurrentPublished` returns the actual current published pointer, which may be an older generation after a failed target, and result records that exact ID plus status warnings;
- no target candidate data is merged into retained snapshot;
- if caller requires target ID, acquisition fails;
- service never relabels retained snapshot.

## 9. Component health during acquisition

### Ready

Component can provide exact required immutable identity/view.

### Degraded

Component/view coherent, but some nonmandatory capabilities Partial/Failed/NotEvaluated. Lease may still form with exact blockers.

### Failed

Mandatory structural identity/view invalid; lease cannot form.

### Unavailable

Configured component/snapshot absent; lease cannot form for dependent operation.

### Deferred

Component/operation intentionally inactive; status reports it; E0 check must not require it.

## 10. Acquisition failure isolation

If reference acquisition fails after project snapshot acquisition:

- no rules/check execution;
- no partial context lease;
- failure records project identity and missing/mismatched reference safely;
- no fallback profile/reference.

If rule registry acquisition fails:

- generic-only E0 check is not silently returned as complete because requested WoW rule scopes are unavailable;
- operation fails if registry structurally invalid;
- may return partial only if a future explicit operation policy allows it; E0 default requires valid registry.

If nonmandatory analyzer fact capability is degraded:

- lease forms with exact coverage blockers;
- check may become partial through rule `NotEvaluated`.

## 11. Context lease identity

Canonical lease ID derives from:

```text
service configuration ID
operation
exact GenerationContext
ReferenceView identity
ProjectSnapshot/View identity
AnalyzerSnapshot/pin/config identity
RuleRegistry/fixture policy identity
canonical scope
capability/coverage/conflict registry identities
budget policy/operation budget
context lease schema version
```

Exclude acquisition timing, lock/lease object ID, temp path, thread/process, current selector token after exact resolution, and rendered messages.

## 12. Lifetime

E0 can use in-process immutable references without async lease management, provided:

- published snapshots/views are immutable;
- component registries keep them alive for request duration;
- no component mutation is performed through lease;
- cancellation does not invalidate already held immutable records unsafely.

Do not create a daemon/distributed lease framework for E0.

## 13. Required operations

```text
validate_service_configuration
validate_service_context_request
resolve_project_generation_selector
acquire_project_snapshot
acquire_reference_view_for_project
validate_analyzer_binding_for_context
acquire_rule_registry
acquire_capability_registries
validate_service_context_coherence
canonicalize_service_scope
build_service_context_lease
validate_service_context_lease
```

## 14. Required tests

- exact selector success;
- current-published selector resolves once/exact ID output;
- current pointer changes after acquisition but request stays old coherent snapshot;
- unavailable exact generation fails;
- last-known-good not substituted for required failed target;
- profile/reference mismatch fails;
- analyzer pin/config/snapshot mismatch fails;
- rule registry/profile policy mismatch fails;
- nonmandatory per-file capability degraded still forms lease with blocker;
- stale source/fact ID scope rejected;
- scope order canonical;
- temp root/time/scheduling do not change lease ID;
- no partial lease exposed on mandatory failure;
- no hidden retry/fallback.

## 15. Hard stops

- no unscoped latest;
- no snapshot switch mid-request;
- no context assembled from mixed generations;
- no last-known-good relabel/substitution;
- no lower-layer mutable handles in public result;
- no profile/reference fallback;
- no filesystem scope resolution;
- no component-ready boolean replacing exact capabilities;
- no operation execution before lease validation.
