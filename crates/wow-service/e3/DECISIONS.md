# E3-C service decisions

**Status:** normative.

## SVC3-001 — Service is the only multi-owner orchestration boundary

Applications and context do not coordinate stores/project/graph/reference directly.

## SVC3-002 — Symbolic current resolves exactly once

A current selector becomes one exact publication/store generation before context binding. Service never refreshes it during the operation.

## SVC3-003 — No distributed-current atomicity claim

Independent stores are acquired in a fixed order and validated as an immutable exact set. E3-C does not pretend they were atomically current at one global instant.

## SVC3-004 — No hidden retry or fallback

Current change, incompatibility, stale guard, missing generation, or acquisition race returns a typed result. Retrying creates a new request.

## SVC3-005 — Reference selection is exact

ReferenceGeneration/View is derived from the selected publication contract or guarded by an exact caller value. Floating reference current is forbidden.

## SVC3-006 — E3-B owns context artifacts

Service invokes/validates ProjectMap, L0/L1, ContextSemanticPack, renderer, continuation, and comparison operations; it does not construct their internal semantics.

## SVC3-007 — Exact roots only

Map may use the exact acquired project root. Inspect/build require exact root selectors. Search/natural language is E4.

## SVC3-008 — Profiles are resolved before semantic request identity

Compiled/configured aliases may resolve to exact profile IDs at the outer service boundary. Canonical context requests contain exact IDs, not aliases.

## SVC3-009 — Owner ports are narrow and typed

No raw SQL, connection, transaction, analyzer session, parser, mutable project/graph object, arbitrary path, or callback crosses service.

## SVC3-010 — Acquisition order is global

Primary project, optional Blizzard UI project, exact ReferenceView, then context binding. Owner-specific inner ordering remains owner-controlled.

## SVC3-011 — Release order is reverse acquisition

All success, failure, cancellation, and panic-boundary paths close the partially/full acquired set.

## SVC3-012 — Success finalization follows closure

A draft semantic outcome may exist before release, but no public success envelope is finalized until mandatory close reports success.

## SVC3-013 — Close failures are not warnings on complete success

A mandatory resource-close/lease-release failure yields failed operation state or a dedicated failure result, never `complete`.

## SVC3-014 — Operational lease state is nonsemantic

Lock handles, process IDs, timestamps, deadlines, and close timing do not enter context or service semantic identities.

## SVC3-015 — Continuation reopens exact retained generations

It never resolves current. Missing/expired retained generations return unavailable; no restart on a newer publication.

## SVC3-016 — Continuation retention is explicit

When a context result offers continuation across operations, service requests/records owner retention receipts according to policy. A receipt does not upgrade context completeness.

## SVC3-017 — One public operation, one owner operation plan

Service cannot opportunistically invoke extra context/search/rule lanes not declared in the request/profile.

## SVC3-018 — Service status is conservative

Readiness, installed components, or a valid current pointer do not mean context generation/evaluation passed.

## SVC3-019 — Context status and E0 status are distinct payloads

They can share generic failure/cancellation forms but keep operation-specific capability/status schemas.

## SVC3-020 — Context service status is operation-level

`complete`, `partial`, `truncated`, `not_evaluated`, `failed`, and `cancelled` do not replace lower coverage/conflict/omission records.

## SVC3-021 — Artifact validation is nonrepairing

Service/context validation reports mismatch; it does not rewrite, regenerate, or substitute artifact bytes under the same ID.

## SVC3-022 — Rendering is delegated

Service selects an exact renderer profile and calls `wow-context`; it never formats semantic context itself.

## SVC3-023 — Service envelope is transport-neutral

No terminal width/color, CLI path, stdout/stderr, file name, host, locale, or exit code enters semantic service output.

## SVC3-024 — Canonical output is separate from logs/metrics

Operational timings/acquisition traces can be supplemental but are noncanonical and cannot alter result IDs/status.

## SVC3-025 — Privacy/license decisions stay enforceable

Service cannot broaden source access beyond the exact E3-B privacy/license/consumer profile or expose source in errors/logs.

## SVC3-026 — Existing findings are references, not rerun rules

A context request may root exact published finding/evidence IDs. E3-C does not execute `wow-rules` to refresh them.

## SVC3-027 — Context operations do not use models or external candidates

Search, embeddings, LLM summaries, and Codebase Memory are absent from canonical E3-C paths.

## SVC3-028 — App reads transport artifacts; service receives bytes

For validate/render, the CLI may read one explicitly supplied bounded context artifact. Service never receives or trusts the host path, only typed bytes/media/profile input.

## SVC3-029 — No physical context cache in E3-C

Context cache keys may be returned/validated, but storage/eviction is deferred to a separately reviewed layer.

## SVC3-030 — Missing implementation evidence blocks activation

Documentation, null fixtures, or declared ports are not executable success. All prerequisite/freeze/evaluation gates must pass before Rust activation.
