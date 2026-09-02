# E7-A session, workspace, and exact-generation binding

**Status:** normative.

## Session lifecycle

```text
Created
-> Initializing
-> Active
-> Rebinding (optional explicit operation)
-> Active
-> ShuttingDown
-> Closed
```

Failures enter `Failed` or `Quarantined` with exact resource/effect state. No request is accepted before `Active` except operations explicitly allowed during initialization or shutdown by the protocol profile.

## Workspace identity

A workspace is supplied explicitly by the application/deployment profile. Service normalizes it through owner ports into stable workspace/root handles.

It does not discover:

- current working directory;
- home directory;
- Git repository;
- editor workspace state beyond explicit protocol input;
- installed addon folders;
- WoW installation/SavedVariables/logs;
- provider or network resources.

A filesystem path is not a ProjectId. Project owners materialize/publish source and return exact project identities.

## Selector resolution

Allowed outer selectors may include an explicit `CurrentPublished` request for a known owner/workspace/profile. During `workspace_bind` or `session_rebind_exact`:

```text
validate selector and authorization
-> query exact owner current record once
-> retain exact publication/view
-> validate project/graph/reference/source compatibility
-> record selector resolution receipt
-> replace selector with exact generation IDs
-> build SessionViewSet
```

Normal analysis/context/search operations accept only the exact `SessionViewSetId`.

## Session view compatibility

Validate:

- ProjectStore generation matches ProjectSnapshot/GraphSnapshot/AnalyzerSnapshot;
- project and ReferenceProfile product/flavor/build/Interface compatibility;
- annotation/library/analyzer configuration identity;
- optional Blizzard UI source profile compatibility;
- graph registries and search/context profiles;
- optional external Candidate descriptor/session/generation compatibility;
- required capability coverage and conflicts;
- source privacy/license/consumer policy;
- retention/lease availability.

Independent stores are acquired in fixed order and closed in reverse order. Compatibility validation does not assert a distributed atomic commit.

## Rebinding

`session_rebind_exact` is explicit and guarded:

```text
active old SessionViewSet
+ requested exact or permitted outer selector
+ expected old SessionViewSetId
-> acquire/validate new exact views
-> build new SessionViewSet
-> invalidate incompatible overlay/continuation/result references
-> atomically switch session-local active view reference
-> close old owner resources after in-flight old-view operations terminate or are cancelled under policy
```

No request silently migrates from old to new. In-flight operations retain the view they started with.

## Rebind outcomes

```text
Rebound
NoChangeWithExactProof
BlockedByInFlightOperation
BlockedByUnsavedOverlay
Conflict
NotEvaluated
Cancelled
Failed
OutcomeUnknown (only if an effecting owner boundary is uncertain)
```

An overlay policy may require explicit discard, rebase validation, or session fork. E7-A never heuristically reapplies unsaved edits to a new base.

## Multiple workspace folders

The initial profile may support one workspace binding only. A future multi-root profile must define:

- exact root ordering and identity;
- independent project publications;
- cross-root privacy and authorization;
- duplicate URI/source mapping conflicts;
- cross-project search/context semantics;
- resource acquisition/closure and retention;
- whether any operation can claim coherent cross-project state.

Absent that profile, multiple roots are rejected rather than merged.

## View retention

A session obtains owner retention/lease receipts sufficient for:

- active requests;
- diagnostic result IDs;
- document overlay bases;
- continuation cursors;
- context/search result artifacts exposed by the session;
- bounded post-response explain/resolve operations where declared.

The session does not promise indefinite retention. Expiry/eviction is explicit and makes stale handles unavailable, not automatically rebound.

## Session operation registry

Every accepted request records:

```text
transport request ID -> OperationId
SessionId / exact SessionViewSetId / OverlayGenerationId
operation kind and normalized request digest
cancellation/progress tokens
state/result/error/closure references
```

Transport request IDs are scoped to the session and are not reused as domain artifact IDs.

## Concurrent requests

- Requests can execute concurrently only when owner/session profiles allow it.
- Each request captures its exact view/overlay at acceptance.
- Document mutations serialize per document and session.
- Rebind/shutdown establish barriers under a frozen policy.
- Completion order does not alter semantic result ordering or IDs.
- Effecting operations retain their existing durable idempotency contracts.

## Abrupt disconnect

On transport loss:

- stop accepting requests;
- signal typed cancellation to cancellable operations;
- reconcile uncertain effects by exact operation identity;
- close session resources synchronously within bounded shutdown policy;
- preserve durable artifacts/effect receipts under their true state;
- do not save overlays or mutate publications;
- do not continue detached work;
- do not report cancellation if outcome is actually unknown.

## Session nonclaims

A bound session does not prove:

- the local filesystem still equals the publication;
- the editor buffer equals disk or repository content;
- the WoW client loaded the addon;
- runtime behavior or safety;
- cross-store atomicity;
- source-edit permission;
- model/tool authorization;
- currentness after binding.
