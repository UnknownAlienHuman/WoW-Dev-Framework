# E6-B exact-root context handoff

**Status:** normative composition over the existing E3-C context use case.

## Sequence

```text
validated immutable E6-A result set
-> exact Candidate
-> validated ExactMapped owner receipt
-> validated explicit selection receipt
-> exact project/reference owner root
-> exact ContextUniverseSet acquisition through E3-C
-> wow-context semantic pack/rendering
-> E6-B outer envelope with separately labelled external evidence
```

## Context input

The context request receives only:

- exact mapped owner root and owner universe/generation;
- exact context universe selectors/IDs;
- exact intent/map/L0/L1/expansion/budget/tokenizer/privacy/license/renderer profiles;
- exact operation/idempotency/cancellation state.

It does **not** receive provider rank, score, prose summary, inferred relation, candidate recommendation, or unverified locator as a framework fact.

## External evidence attachment

The E6-B outer envelope may include a separate `ExternalCandidateEvidenceAttachment`:

```text
exact provider/result/candidate/artifact IDs
exact external-state class and receipt
E6-A authority ceiling
mapping receipt and selection receipt IDs
allowed bounded provider fields/snippet under privacy policy
coverage/conflicts/truncation/continuation
mandatory nonclaims
```

The attachment is not part of the canonical `ContextSemanticPack` fact graph unless a future context contract explicitly accepts external Candidate items while preserving their class.

## Exact context owner

`ContextUseCasePort` exposes the existing E3-C operation contract. E6-B does not directly invoke `wow-context` internals, project/graph/reference views, source readers, tokenizer, renderer, or cache.

```text
build_context_from_exact_root(request)
validate_context_result(result)
reconcile_context_effect(operation identity)
close_context_resources()
```

## Current resolution

Any permitted symbolic project/reference current selector is resolved once by the existing service acquisition layer before context construction. The mapping receipt and selected root must either target that exact generation or the request is rejected. The service never remaps silently to current.

## Cross-owner roots

Project and Reference roots remain distinct:

- Project mapping can root a project/source/graph entity context.
- Reference mapping can root an exact Reference entity context under a compatible universe set.

A candidate mapping to both owners requires two explicit mapping receipts and separate explicit selection/context requests in E6-B v1.

## Provider context nonclaims

Context output does not claim:

- provider summary correctness;
- provider relationship correctness;
- provider rank as relevance authority;
- same-name identity beyond the exact mapping receipt;
- lineage/replacement/migration/impact;
- runtime behavior or API contract from provider output;
- source-edit/tool authorization;
- absence from a zero-result or unmapped candidate.

## Privacy/license intersection

Output is limited by the intersection of:

```text
provider result disclosure policy
mapped owner publication/source policy
context consumer/privacy/license profile
E6-B outer-envelope profile
renderer/output profile
```

The outer service cannot widen context or owner restrictions. A provider snippet may be omitted while exact mapped owner context remains available.

## Failure behavior

- candidate/mapping/selection unavailable: context not invoked;
- context owner unavailable: explicit `NotEvaluated`/failure, no provider-summary fallback;
- exact generation mismatch: reject;
- provider evidence attachment denied: context may succeed with explicit omission when policy allows;
- context partial/conflict/truncated: preserve exact state;
- response loss after context effect: reconcile by exact operation/request identity;
- close/retention failure: no public success.

## Idempotency

The context operation request digest includes the exact selection receipt, mapped root, universe set, all context profiles/budgets, and external attachment policy. Retry cannot choose another candidate/root or refresh provider/current state.

## Audit

Record:

- exact external result/candidate/mapping/selection;
- exact mapped owner root and context universe;
- context request/result IDs;
- fields explicitly excluded from framework facts;
- external attachment disclosure decision;
- status/coverage/conflicts/omissions;
- response-loss/retention/closure state.

## Tests

- valid project-root context;
- valid reference-root context;
- provider prose/rank injected into framework facts;
- mapping/current generation mismatch;
- multiple/partial mapping;
- selection receipt for another root;
- denied provider snippet with valid local context;
- context unavailable with forbidden fallback;
- response loss and exact reconciliation;
- same exact request under shuffled provider presentation order.
