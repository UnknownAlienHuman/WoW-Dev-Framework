# E4-C lineage build, review, publication, and query operations

**Status:** normative orchestration over project/reference/search producers and E4-B `wow-graph`.

## `lineage_status`

Reports exact comparison/profile catalog state, available producer partitions, lineage snapshots, validation/retention state, unresolved ambiguity/review requirements, and missing implementation gates.

It never calls the newest artifact current unless an explicit catalog record under an exact comparison says so; normal E4-B snapshots are exact immutable artifacts.

## `lineage_build`

```text
validate request/idempotency/selectors/profiles
-> acquire exact before/after owner views
-> invoke required project and/or Reference producer operations
-> optionally acquire exact E4-A shards and build Candidate-only search partitions
-> validate every producer partition
-> invoke E4-B candidate-component/proposal validation
-> validate optional review envelopes through ReviewAuthorizationPort
-> invoke graph review application under proof ceilings
-> plan and publish one inactive immutable lineage graph snapshot
-> reopen exact snapshot and invoke E4-B validation/golden queries
-> admit exact comparison/profile catalog record
-> retain result when requested
-> close resources
-> return lineage build outcome
```

Search-derived input is optional and never raises proof above Candidate.

## `lineage_validate`

Delegates exact read-only snapshot validation. It never repairs proposals, accepts a candidate, fills coverage, changes a review or republishes under the same ID.

## `lineage_review_validate`

Validates two independent dimensions:

```text
authorization
    principal/key/role/scope/attestation/expiry/revocation/profile

graph semantics
    exact comparison/component/proposal/relation/requested confidence/proof ceiling/conflicts/coverage
```

Both must pass. Authorization cannot raise the graph proof ceiling; graph validity cannot bypass authorization.

Review notes are untrusted bounded data and are not proof.

## `lineage_review_apply`

Requires exact base LineageGraphSnapshot, exact validated review envelope set, expected catalog/base guard and operation ID.

```text
open exact base snapshot
-> verify decision still targets the same proposal/component/profile
-> revalidate authorization and graph semantics
-> apply immutable review-decision producer partition
-> recompute affected components/assertions/conflicts/change/migration/impact indexes
-> publish and validate a new lineage snapshot
-> catalog/retain exact new result
```

No in-place mutation, force acceptance, hidden conflict deletion or current rebase.

## `lineage_compare`

Delegates E4-B exact entity/generation comparison and change classification. The service may enrich the response with exact owner display/detail handles but cannot create a relation or change record.

## `lineage_trace`

Delegates explicit accepted relation traversal over exact generation sequence. Candidate/Possible inclusion is explicit and labeled. Multi-hop paths remain paths.

## `lineage_explain`

Returns complete proposal, producer, evidence, source/reference/search, proof-ceiling, component, review, conflict, coverage and nonclaim closure for one exact assertion or proposal.

## Review decision states

```text
Accepted
Rejected
Deferred
ConflictMarked
Superseded
AuthorizationInvalid
SemanticInvalid
NotEvaluated
```

A rejected review is retained and does not delete the proposal. A superseding decision names the exact prior decision and profile rules.

## Proof preservation

Service validates:

```text
accepted confidence <= producer maximum
accepted confidence <= relation profile maximum
accepted confidence <= review authorization scope maximum if configured
accepted confidence <= remaining coverage/conflict ceiling
```

The minimum applicable ceiling controls.

## Ambiguity

Service never converts a unique ranked pair or suggested assignment into acceptance. Unresolved one-to-many, many-to-one and many-to-many components remain explicit. `Removed`/`Introduced` conclusions stay blocked until E4-B negative-authority gates pass.

## Idempotency and response loss

Lineage build/review operations use exact operation ID plus canonical request digest. Durable operation/catalog receipts distinguish planned, producing, published-inactive, validating, validated, cataloged and returned states. A retry after lost response returns the recorded exact snapshot receipt; it does not publish another snapshot.

## Security

- no raw review credential/token/signature in logs or public envelope;
- no review authorization inferred from GitHub/OS/CLI identity;
- no source/model/embedding/CBM execution;
- no raw SearchStore/GraphStore/SQLite access;
- all producers/components/reviews/output/time/memory bounded and cancellable;
- cancellation/failure leaves prior snapshots/catalog records unchanged.
