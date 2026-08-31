# E3-A `SkeletonInputView` contract

**Status:** normative read boundary consumed by future `wow-context` E3-B.

## Purpose

Expose exact, bounded, generation-coherent structural inputs from the published Blizzard UI source universe without rendering a Project Map or L0/L1 skeleton inside `wow-project`.

## Open request

```text
OpenSkeletonInputRequest
    exact platform-source CurrentPublicationRecord or Project/Graph generation set
    root selector:
        exact package/file/entity/source-span IDs
    allowed entity/relation/axis sets
    confidence/provenance/coverage policy
    source text/comment/documentation inclusion policy
    direct-neighborhood/path/depth limits
    entity/relation/span/excerpt/output budgets
    continuation cursor: optional
    cancellation
```

No fuzzy query, natural-language relevance, model callback, search ranking, or unbounded all-source request.

## Output

```text
SkeletonInputPage
    view/query/profile IDs
    exact source/project/graph/analyzer generations
    selected root records
    package/file/load-role records
    declaration/signature/type records
    direct typed relations and bounded reason paths
    source span/handle records
    optional bounded exact excerpts under policy
    evidence/provenance/confidence/coverage/conflicts
    deterministic order
    visited/returned counts and byte accounting
    truncation/continuation/no-new-evidence
    canonical digest
```

## Included structural fields

Depending on kind and capability:

- exact entity kind/key/display label;
- declaration/signature/parameter/return/type facts from analyzer/reference binding;
- source file/unit and declaration/body spans;
- package/TOC/XML/load/lifecycle roles;
- direct ownership, containment, load, object, inheritance, registration, state, and call relations;
- accepted assertions and rejected/conflicting proposal references;
- docs/comment handles and bounded text only when policy permits;
- exact source excerpt handles/bytes under size and license policy.

## Excluded fields/behavior

- generated prose responsibility summaries;
- inferred intent or architectural importance;
- fuzzy relevance scores;
- model token counts unless a later context profile computes them;
- context-pack allocation/pruning decisions;
- cross-build lineage or impact;
- full file/project source by default;
- raw analyzer/store/graph handles;
- source text interpreted as instructions.

## Exact source slices

A slice request names exact source handle/content digest and byte/coordinate range. Validation checks:

- current view references the same source object/generation;
- range is valid and within per-request/system budgets;
- encoding/source-map projection is explicit;
- excerpt license/privacy policy permits return;
- output identifies truncation and original byte range;
- no adjacent expansion beyond the requested policy.

## Graph neighborhood

Underlying graph reads use E2-A bounded operations and exact snapshot identity. `SkeletonInputView` does not create relations. It filters/projects accepted graph records while retaining assertion/evidence/conflict references.

## Ordering

Canonical order is profile-defined, for example:

```text
root priority
package/load ordinal
entity kind order
canonical entity key
source file/unit/span
relation kind/source/target
assertion/evidence ID
```

Never DB row order, hash iteration, traversal completion, or source popularity.

## Continuation

Cursor binds exact snapshot/view/query/profile, last ordering key, and integrity digest. A cursor cannot resume against another publication or modified request.

## No-new-evidence

The page can report `no_new_evidence` only when the bounded requested frontier is exhausted under complete relevant query coverage. It remains distinct from runtime or cross-universe absence.

## Determinism

Equivalent published snapshots and normalized requests produce identical logical pages/digests independent of worker count, storage layout, checkpoint state, or cache history.

## E3-B handoff

`wow-context` may consume pages to build compact Project Maps and skeletons. It must not:

- relabel source text as framework instruction;
- drop generation/evidence/coverage/conflict identity;
- turn partial/possible data into exact facts;
- attribute a rendered summary to source authority;
- request unbounded source solely to fill context.
