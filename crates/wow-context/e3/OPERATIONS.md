# E3-B public operation contracts

**Status:** normative transport-independent behavior.

All operations are read-only with respect to source, project, graph, reference, store, analyzer, editor, client, cache storage, and external services. They require exact immutable public views and return typed artifacts, explicit partial state, or structured errors.

## Common preconditions

- exact `ContextUniverseSet`;
- exact resolved root IDs where required;
- compatible frozen profiles and owner read catalogs;
- finite system/profile/request budgets;
- source/privacy/license policy for excerpts;
- cancellation state;
- no floating current/latest/search request;
- no historical alias type interpreted as a second object.

## `bind_context_universe_set`

Inputs exact primary user project and graph views, optional exact E3-A Blizzard UI project/graph/SkeletonInputView, and exact ReferenceView. It resolves no floating pointer and performs no source acquisition.

Output:

```text
ContextUniverseSet
ContextUniverseCompatibilityReport
```

## `validate_context_universe_set`

Validates project/graph/source/reference generation closure, profile/build compatibility, source-coordinate identity, graph registries, required capabilities, coverage, and conflicts. It is nonrepairing.

## `validate_context_profiles`

Validates closed profile schemas, registries, cross-profile compatibility, mandatory reserves, ordering, allowed intent/facet/axis classes, tokenizer/source/privacy/security constraints, renderer compatibility, canonical digests, and milestone alias rules.

It does not inspect source text for relevance and does not probe a model.

## `validate_context_request`

Normalizes exact universe ID, roots, intent, requested facets/axes, confidence/coverage policy, source/reference policy, budgets, tokenizer, privacy/consumer trust, renderers, and continuation. It performs no search.

## `build_project_map`

1. obtains exact project/package/TOC/load/source-unit roots;
2. obtains profile-declared graph roots/direct relations;
3. creates only declared nodes, edges, groups, and facets;
4. includes mandatory capability/conflict/coverage records;
5. groups only by exact reviewed keys;
6. allocates map budgets deterministically;
7. emits exact detail routes, omissions, continuation, and evidence closure;
8. validates the map.

No full source or free-form architecture prose.

## `open_project_map_view`

Validates and opens one immutable map for bounded exact lookup, group membership, page/continuation, and route operations. It does not mutate or lazily add semantic records.

## `build_l0_skeleton`

Builds the declared identity, source/package/load, role, top-level declaration, direct relation, evidence, blocker, and detail-route projection for one exact container scope.

Unsupported or partial scopes remain explicit. No body and no name-based guessed role.

## `build_l1_skeleton`

Builds selected exact signatures, members, direct relations, reason paths, ReferenceView facts, source-span candidates, and closed control/effect nodes for exact roots.

It cannot parse source, query analyzer internals, or turn paths into direct relations.

## `plan_context_expansion`

Creates a deterministic `ContextExpansionPlan` and initial `ContextFrontier` from exact roots, intent, required/optional facets, allowed owner operations, dependencies, estimated costs, budgets, coverage/conflict policy, and stopping rules.

Cost estimates cannot create semantic eligibility.

## `expand_context_frontier`

Processes one profile-bounded deterministic frontier batch. It:

- issues exact bounded owner queries;
- validates snapshot/generation/capability/coverage;
- classifies new, duplicate, rejected, blocked, and omitted records;
- updates candidates, evidence, budgets, and frontier;
- emits one `ContextExpansionStep`.

No hidden root, relation, universe, confidence, or source-scope broadening.

## `build_context_source_excerpts`

Resolves only selected exact source handles through the owning source-detail seam. It validates generation, digest, span, source-map, privacy, license, consumer trust, boundary, and budget profiles.

It returns exact, transformed, denied, unsupported, or continued excerpt records. No path fallback, reconstruction, or full-source default.

## `build_context_coverage_and_omissions`

Reconciles upstream coverage/conflicts with map, skeleton, expansion, selection, source, renderer, tokenizer, and evaluation coverage. It emits explicit context coverage, projection loss, omission, blocker, and stopping records.

It cannot upgrade source/domain completeness.

## `build_context_semantic_pack`

Assembles validated universe, map, skeleton, control/effect, relation/path, evidence, reference, source, loss, omission, stopping, frontier/continuation, and budget records in canonical order.

It rejects dangling references, mixed generations, hidden mandatory blockers, alias duplication, and semantic mutation by a renderer.

Historical `build_context_bundle` is a documentation alias to this operation only.

## `continue_context_semantic_pack`

Validates an exact-snapshot continuation, original total-budget state, selected/visited/frontier digests, profiles, owner query availability, and cancellation. It resumes against the same retained inputs.

It never refreshes current, resets budget, reruns search, changes privacy, or broadens confidence.

Historical `continue_context_bundle` is an alias only.

## `validate_context_semantic_pack`

Nonrepairing validation of identity closure, origin/evidence, coverage/conflicts/loss/omissions, profile fields, canonical ordering/digests, budgets, source/privacy/license/boundaries, continuation, and semantic artifact eligibility.

Validation never fills missing records or rewrites the pack.

## `render_context_pack_json`

Produces canonical lossless JSON for one exact semantic pack and renderer profile. It validates exact bytes, mapping, source-data escaping, and optional exact token accounting.

## `render_context_pack_markdown`

Produces deterministic template-based Markdown for one exact semantic pack. It cannot add free-form claims. Source text uses the structural data boundary defined by the source-boundary contract.

## `validate_rendered_context_artifact`

Checks semantic item/output-range mapping, required field representation, rendering loss declarations, source boundaries, byte/token limits, encoding/line endings, and exact digests. It is nonrepairing.

## `measure_context_pack`

Computes deterministic structural, source, byte, scalar, evidence, omission, and optional tokenizer measures over exact semantic and rendered bytes. Estimates, upper bounds, and exact token counts are distinct types.

## `compare_context_packs`

Compares packs only under an explicit profile. It classifies input/profile/request differences before semantic differences.

```text
EquivalentSemanticContent
RendererOnlyDifference
ProfileDifference
InputGenerationDifference
AddedRemovedChangedItems
CoverageConflictOrOmissionDifference
BudgetOrContinuationDifference
Incomparable
```

A newer pack is not inherently more authoritative.

## `build_context_cache_key`

Builds and validates an exact storage-independent cache key. It performs no cache I/O.

## Common statuses

```text
CompleteForRequest
Partial
Truncated
Cancelled
Failed
NoChange
Unsupported
NotEvaluated
NoNewEvidence
ContinuationAvailable
```

A scoped unsupported/partial facet can coexist with a useful pack. Overall status follows mandatory scopes conservatively.

## Idempotency and retries

Operations are deterministic functions of exact immutable inputs, profiles, request, and total budget. A higher layer may cache/retry by exact IDs, but `wow-context` owns no durable operation log or physical cache.

A retry cannot switch input publication, privacy policy, tokenizer, renderer, or continuation.

## Cancellation

Check before and during every bounded query, expansion loop, source read, tokenization, canonicalization, render, and evaluation batch. No background continuation. A cancelled artifact never becomes complete.

## Architecture tests

Every operation must prove:

- exact and invalid preconditions;
- same-snapshot behavior;
- typed partial/truncated/unsupported/cancelled outcomes;
- cross-operation semantic reference closure;
- nonrepairing validation;
- no search/parser/analyzer/store/source mutation;
- no model or external side effect;
- deterministic retry and 1/2/N workers;
- source denial with structural context retained;
- continuation same versus changed snapshot;
- historical alias names do not create duplicate operations.
