# E3-A public operation contracts

**Status:** normative transport-independent operation behavior.

All operations are pure/read-only with respect to project, graph, reference, source, store, editor, and client state. They require exact views supplied by the caller and return typed artifacts, scoped partial state, or structured errors.

## Common preconditions

- exact coherent `ContextInputSnapshot`;
- exact root IDs where required;
- compatible frozen profiles and query catalogs;
- finite system/profile/request budgets;
- source/license/security policy for any excerpt;
- cancellation state;
- no floating `Current`/latest/search request.

## `validate_context_profiles`

Validates profile schemas, registries, cross-profile compatibility, mandatory reserves, ordering, allowed lanes/kinds, tokenizer/source/security constraints, and canonical digests.

It does not probe source/project/graph content. Output is a deterministic profile-validation report.

## `validate_context_request`

Normalizes and validates exact input identities, roots, artifact target, lanes, confidence/coverage policy, profiles, budget overrides, continuation, and requested renderer/tokenizer. It returns a `ValidatedContextRequest`; it does not perform search.

## `build_project_map`

Inputs exact project/graph/reference views and a validated request/profile. It:

1. obtains exact required project identity/package/TOC/load/source-unit records;
2. obtains profile-declared principal graph roots/direct lanes;
3. includes mandatory capability/conflict/gap records;
4. groups only by frozen exact keys;
5. allocates section budgets deterministically;
6. emits detail routes, loss, omissions, stopping, metrics, and evidence closure;
7. validates the resulting map.

It does not include full source or infer architecture prose.

## `build_l0_skeletons`

For each exact root, builds the profile-declared identity, role, owner/load, public surface, direct relation, evidence, blocker, and detail-route projection. Unsupported roots remain explicit; no generic guessed skeleton.

## `build_l1_skeletons`

Builds selected exact signatures/members/direct relations/reason paths and published control/effect nodes under `CONTROL_AND_EFFECT_MODEL.md`. It cannot reparse source or query analyzer internals.

## `plan_context_expansion`

Creates a deterministic `ContextPlan` and initial `ContextFrontier` from exact roots, requested lanes/detail, mandatory inclusions, costs, budgets, coverage/conflict policy, and stop rules. Estimates are labeled and cannot create semantic eligibility.

## `expand_context_frontier`

Processes one or a profile-bounded deterministic batch of frontier work items. It issues exact bounded registered queries, validates their snapshot and coverage, classifies new/duplicate/rejected/blocked results, updates artifacts/evidence/budgets/frontier, and emits `ContextExpansionStep` records.

No hidden root/lane/universe/confidence broadening.

## `build_context_source_excerpts`

Resolves explicitly requested exact source handles through the owning project/reference source-detail seam, validates generation/digest/span/origin/license/privacy/security, applies deterministic context expansion/escaping, and returns faithful excerpt records plus source-specific loss.

No path lookup, reconstruction, arbitrary object enumeration, or full source default.

## `build_context_coverage_and_loss`

Reconciles input coverage/conflicts with context field/section/lane/source/renderer/tokenizer coverage. It emits exact context coverage, projection loss, omission, blocker, and stopping records. It cannot upgrade source/domain completeness.

## `build_context_bundle`

Assembles validated Project Map, skeleton, relation/path, evidence, source, loss/omission/stopping, frontier/continuation, metric, and profile/input manifests in canonical order. It rejects dangling refs, mixed generations, hidden mandatory blockers, and semantic renderer mutation.

## `continue_context_bundle`

Validates an opaque exact-snapshot continuation, original total budget state, included/visited/frontier digests, profiles, query availability, and cancellation. It resumes deterministic frontier work against the same retained input. It never refreshes Current, resets budget, reruns search, or changes confidence.

## `measure_context_bundle`

Computes deterministic structural, source, byte, scalar, evidence, omission, and optional pinned-tokenizer measures over exact artifact/renderer bytes. Estimates and exact token counts are separate types.

## `compare_context_bundles`

Compares bundles only under an explicit comparison profile. It classifies input/profile/request differences before semantic record differences. It never treats a newer bundle as inherently more authoritative.

Possible outputs:

```text
EquivalentSemanticContent
RendererOnlyDifference
ProfileDifference
InputGenerationDifference
AddedRemovedChangedRecords
CoverageOrBlockerDifference
BudgetOrContinuationDifference
Incomparable
```

## `validate_context_bundle`

Nonrepairing validation of identity closure, evidence, coverage/conflicts, profile fields, canonical ordering/digests, budgets, source security/license, continuation, metrics, and artifact eligibility. Validation never fills missing records or rewrites the bundle.

## Common statuses

```text
Complete
Partial
Truncated
Cancelled
Failed
NoChange
Unsupported
NotEvaluated
```

A scoped unsupported/partial lane can coexist with a useful bundle. Whole-bundle status follows mandatory scopes conservatively.

## Idempotency and retries

Operations are deterministic functions of exact immutable inputs/profiles/request/budget. A higher layer may cache/retry by exact IDs/digests, but E3-A owns no persistence or durable operation log. A retry cannot switch input publication or mutate a continuation.

## Cancellation

Check before and after each bounded query/source/render/tokenizer/evaluation batch. No complete artifact after cancellation and no background continuation. Late parallel results are admitted only through the deterministic merge protocol if cancellation policy permits the already completed atomic batch.

## Required operation tests

- valid and invalid preconditions for every operation;
- exact view/profile/query catalog mismatch;
- complete/partial/truncated/unsupported/cancelled outcomes;
- cross-operation semantic reference closure;
- nonrepairing validation;
- no search/parser/analyzer/store/source mutation side effect;
- deterministic retry/1-2-N worker behavior;
- comparison classifications;
- exact token versus estimate;
- source forbidden/unavailable with structured artifact retained;
- continuation same versus changed snapshot.
