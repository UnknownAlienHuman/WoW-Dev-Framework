# E3-A continuation and stopping contract

**Status:** normative exact-snapshot continuation, frontier, stop-reason, and no-background-work contract.

## Principle

A continuation resumes one exact context request over one exact immutable input snapshot. It is not a request to refresh `Current`, widen scope, reset budgets, or rerun search.

## Continuation identity

```text
ContextContinuation
    continuation ID/version
    exact ContextInputSnapshot IDs
    normalized ContextRequest ID/digest
    context/project-map/skeleton/expansion/source/budget/security/evaluation profile IDs
    tokenizer and renderer profile IDs when applicable
    ordering and continuation profile version
    current ContextFrontier ID/digest
    included and visited set digests
    used/reserved/remaining budget state
    last stable work-item key
    stopping records that created the continuation
    integrity digest
```

Operational expiry/lease metadata can exist outside canonical identity. It cannot change the exact publication/generation bound to the continuation.

## Continuation creation

Create a cursor only when:

- request/profile permits continuation;
- input snapshot remains exact and immutable;
- work remains in a deterministic frontier;
- omission/truncation state identifies what remains;
- used/reserved budgets are known;
- mandatory evidence/blocker records have already been preserved;
- cursor can be validated without trusting caller-modified state.

Do not create a cursor for fatal invalid input, cross-generation state, unsupported root identity, or an unclassified internal failure.

## Resume validation

Before work:

1. validate cursor schema/version/integrity;
2. validate exact publication/project/graph/reference identities;
3. validate normalized request/profile/tokenizer/renderer/budget identities;
4. validate included/visited/frontier digests;
5. validate query/view availability for the same immutable snapshot;
6. validate remaining budget and system maxima;
7. reject cancellation before any query;
8. ensure no continuation work already completed under an idempotency/result record where the profile defines one.

A cursor from another `Current`, even for the same project ID, is stale.

## Budget inheritance

Continuation carries the original total-request budget state. It cannot reset:

```text
entities/skeletons/members
relations/path expansions/depth
source/evidence handles
source excerpts/bytes/lines
output bytes/structured nodes
exact tokenizer token limit
loss/omission/report limits
```

A caller wanting a larger or different budget starts a new request with a new request/profile identity. The new request may reuse exact cached semantic artifacts only through a later explicit cache contract; it is not continuation.

## Frontier ordering

Pending work items retain the frozen stable order defined by `DetailExpansionProfile`:

```text
priority class
root ordinal/semantic key
lane/direction
path length and stable relation/entity tuple
requested detail level
source position where semantic
work-item ID
```

Never query completion time, hash order, database row, worker assignment, or cursor serialization order.

## Stopping reasons

### `RequestedComplete`

All requested fields/roots/lanes/detail and mandatory evidence requirements are satisfied within the declared profile. No pending required work.

### `NoNewEvidence`

The processed branch/query added no new semantic or evidence records after exact deduplication. The branch stops. This is not authoritative absence or proof of global closure.

### `BudgetExhausted`

A hard budget prevents the next atomic work item. Exact used/reserved/remaining state and omitted frontier are recorded. Continuation may be available if policy permits.

### `DepthLimit`

The selected lane/path reached its explicit maximum depth. Deeper relations remain unexamined unless another request/profile is created.

### `CycleClosed`

A cycle or already-visited exact semantic state closes the branch under lane policy. The cycle path/conflict state is retained.

### `CoverageBoundary`

Required project/graph/reference/source capability is Partial, Unknown, Failed, NotEvaluated, or truncated. Missing detail is not treated as absence.

### `ConflictBoundary`

Unresolved competing assertions/identities/values make the requested field/path unsafe to select. Independent lanes can continue.

### `UnsupportedDetail`

The active profile/input view cannot represent or query the requested detail. A loss/route record is emitted; no guess.

### `Cancelled`

Cancellation observed before complete bundle publication. No background continuation.

### `Failed`

Fatal request/input/invariant/security/determinism failure. No cursor unless an explicit recovery protocol defines one; E3-A does not define such recovery cursors.

## Stop scope

A stopping record is scoped to:

```text
whole request
artifact stage
root
relation lane
path branch
source excerpt request
renderer/tokenizer/evaluation stage
```

One blocked branch does not imply the whole bundle failed. Whole-bundle status derives conservatively from all mandatory scopes.

## No-new-evidence detection

Compare exact semantic/evidence IDs after query validation and deduplication. Examples:

```text
all returned relations already included with same assertion/evidence closure
all source handles already mapped to existing context records
path reaches only previously visited exact nodes under the same lane/profile
query returns empty but domain coverage is incomplete
```

The last case can be both `NoNewEvidence` and `CoverageBoundary`; it is never authoritative absence unless the domain result says so.

## Completion status

```text
Complete
    all mandatory requested scopes RequestedComplete; no fatal blocker/truncation

Partial
    useful coherent records exist but one or more mandatory scopes stop at coverage/conflict/unsupported/budget boundaries permitted by policy

Truncated
    budget/depth/output limit leaves deterministic pending work

Cancelled
    cancellation before complete publication

Failed
    no valid coherent bundle under request contract

NoChange
    exact request/frontier already represented; no new artifact or budget use
```

Findings/records can coexist with Partial/Truncated; status does not erase them.

## Continuation output

A resumed step returns:

```text
new exact ContextBundle or bundle delta under frozen policy
new expansion-step records
updated metrics/loss/omission/stopping records
new continuation cursor when work remains
explicit previous continuation ID
same exact input snapshot and request identity
```

Whether the transport returns a full cumulative bundle or validated delta is a renderer/service policy. Semantic cumulative identity and closure must remain deterministic.

## Cursor security

- opaque integrity-protected serialization;
- bounded total size and collection counts;
- no raw SQL, paths, source bodies, credentials, object handles, or executable payloads;
- reject unknown fields/version when compatibility is not explicit;
- validate every referenced ID against exact snapshot/profile;
- no caller-controlled priority, confidence, visited-set deletion, or budget increase;
- no decompression/resource bomb;
- errors do not echo unsafe cursor bytes.

## Cursor privacy

Canonical cursor contains IDs/digests and bounded frontier state, not source excerpts or private paths. If transport encrypts/signs cursors, keys/nonces/expiry are operational and outside semantic identity.

## Cancellation and late results

- check cancellation before each query/source/read/render batch;
- ignore/reject late parallel result not admitted before cancellation according to deterministic merge protocol;
- no complete artifact after cancellation;
- no autonomous worker resumes from cursor;
- caller must explicitly invoke continuation.

## Input generation change

When a newer project publication exists:

- old cursor remains valid only against retained exact old publication while readable;
- it never migrates to new Current;
- if old generation is unavailable, return exact stale/unavailable state;
- create a new request for the new publication;
- do not merge old included/visited state with new graph/source identities.

## Determinism

Equivalent cursor/input/profile and available exact views yield identical next work item, queries, records, budget delta, stopping decision, output frontier, and next cursor under 1/2/N internal workers.

## Required tests

- every stop reason and scope;
- RequestedComplete versus NoNewEvidence;
- empty query with complete versus partial domain coverage;
- continuation after byte/node/edge/source/token budget cutoff;
- no total-budget reset;
- tampered visited/included/frontier/budget/profile/input fields;
- another Current/publication/reference generation;
- retained old publication versus GC-unavailable old generation;
- cycles and independent lane continuation;
- cancellation before/during/after query completion;
- late parallel result rejection;
- full cumulative versus delta projection equivalence;
- cursor size/resource/privacy attacks;
- 1/2/N and randomized completion determinism.

## Hard stops

- no continuation across generations;
- no hidden refresh/search/root widening;
- no budget reset or confidence change;
- no NoNewEvidence-as-absence claim;
- no caller-controlled visited-set removal/priority;
- no background work;
- no complete artifact after cancellation;
- no unsafe cursor payload or source data.
