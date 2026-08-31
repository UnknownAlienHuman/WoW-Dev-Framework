# E3-B cache identity, reproducibility, and determinism

**Status:** normative. `wow-context` defines cache identities and validation only; physical cache storage is outside the crate.

## Cacheable artifacts

```text
ProjectMap
L0Skeleton
L1Skeleton
ContextExpansionPlan
ContextSemanticPack
RenderedContextArtifact
```

Each remains immutable under its exact identity.

## Cache key closure

A cache key includes every semantic input that can change output:

```text
context schema and implementation-contract profile
exact ContextUniverseSetId and underlying project/graph/reference/source generations
exact root selectors and normalized request
map/L0/L1/intent/expansion/selection profiles
confidence/provenance/coverage policy
budget profile and hard request limits
tokenizer/estimator implementation/version/config/vocabulary digest
source excerpt/privacy/license/consumer trust/boundary profiles
renderer/template/canonicalization/encoding/line-ending profile
continuation chain/page identity when applicable
```

Clock, host, process, thread, local cache path, hit count, SQLite row/page order, WAL/checkpoint state, and network state are excluded.

## Cache validation

A cache hit is accepted only when:

- exact cache key matches;
- artifact schema/profile supported;
- artifact digest/size verifies;
- every bound generation/view/profile still matches the request;
- origin/evidence/source/reference records resolve under exact retained views when validation level requires;
- privacy/license/consumer trust decision is still exactly the same profile/version;
- tokenizer/renderer implementation digests match;
- no artifact is marked partial/cancelled/failed as complete;
- continuation chain is exact.

A missing validation capability causes miss/NotEvaluated, not trust by filename.

## No floating cache entries

Forbidden cache keys:

```text
current
latest
project path alone
repository/branch name alone
symbol/display name alone
model name alone
budget number without profile
renderer format without version
```

Service/application code may map a user request for current to exact identities before E3-B; that mapping is not cached as semantic context truth inside this crate.

## Cross-generation reuse

Default E3-B cache keys are generation-bound. A prior map/skeleton/pack cannot be returned as the target generation.

A future implementation may reuse an immutable content-addressed subrecord only if:

- its semantic payload and all origin/profile dependencies are identical;
- it retains the original content identity;
- a new generation-bound wrapper validates the new context;
- no source/evidence/coverage/conflict/omission field differs;
- the reuse is explicit in the validation/selection trace.

No same-name or same-text shortcut.

## Partial/cancelled artifacts

Partial semantic packs may be cacheable only under an explicit profile and retain exact partial/stop/continuation state. They cannot satisfy a complete request or a request with different optional-universe policy.

Cancelled/failed intermediate plans are not reusable as successful artifacts.

## Deterministic input normalization

- canonical ID/string/enum/number encoding;
- exact root list order or profile-declared set ordering;
- immutable profile IDs/digests;
- graph/project/reference records sorted by canonical stable keys;
- source text represented by exact digest/range/encoding;
- duplicate requests normalize identically;
- unknown fields rejected.

## Deterministic algorithms

- no dependence on map/hash/DB iteration;
- no randomized traversal/sample;
- no wall-clock recency/timeout ordering in semantic choices;
- no thread completion ordering;
- stable expansion stages/frontiers;
- stable candidate dependency graph;
- stable tier/tie/pruning order;
- exact deterministic tokenizer/estimator where used;
- stable source-boundary escaping;
- stable renderer templates and line endings.

Wall/CPU budgets may cause an explicit timeout/partial result, but a completed semantic result cannot vary based on which worker finished first.

## Canonical identities

Conceptual order:

```text
UniverseSetId
-> ProjectMapId / L0SkeletonId / L1SkeletonId
-> ContextRequestId / ExpansionPlanId
-> ContextSemanticPackId
-> RenderedContextArtifactId
-> ContextCacheKey(s)
```

IDs are domain-separated. A rendered artifact ID includes exact bytes/renderer profile; it is not reused as semantic pack ID.

## Rebuild comparison

A deterministic rebuild test runs with:

- 1, 2, and N workers;
- shuffled owner result batches;
- cold and warm higher-layer cache;
- different temp roots/host paths;
- different SQLite physical layout/checkpoint state;
- reordered equivalent request JSON;
- repeated source text/evidence duplicates.

Compare:

```text
normalized requests and profile IDs
map/skeleton semantic IDs and canonical bytes
candidate/selected/omitted sets
selection trace and budget reports
context semantic pack ID/bytes
rendered artifact ID/bytes/token records
continuation cursors/page chain
validation reports excluding explicitly noncanonical operational metrics
```

## Operational metrics

Durations, CPU/memory peaks, cache hit/miss, read counts, and worker utilization may be reported for benchmarks but are noncanonical. They do not enter artifact IDs or selection.

## Physical cache owner seam

A higher layer can provide:

```text
get(CacheKey) -> bytes/object handle
put_if_absent(CacheKey, verified artifact)
retain/release policy
```

E3-B validates returned artifacts. It does not accept arbitrary callbacks, file paths, SQL, or mutable cache objects in semantic operations.

## Corruption and mismatch

- digest/schema/profile mismatch: reject/miss;
- partial file/object: reject;
- wrong generation/privacy/tokenizer/renderer: reject;
- cache entry points to unavailable source evidence: explicit validation failure or miss by profile;
- never auto-repair by changing artifact content under the same ID;
- recompute from exact inputs when permitted.

## Acceptance

No implementation is complete until all cache/determinism mutation tests prove exact outputs across worker/order/cache/storage variations and reject stale, floating, corrupted, cross-generation, cross-privacy, and cross-tokenizer entries.
