# E3-C current resolution and exact acquisition

**Status:** normative multi-owner acquisition protocol.

## Request boundary

A caller may request `CurrentPublished` only at the service boundary. The CLI/application passes a typed symbolic selector; it does not read current records itself.

After resolution, all canonical downstream requests contain exact IDs.

## Fixed resolution/acquisition order

```text
0 validate service configuration, request, profiles, budgets, cancellation
1 resolve and acquire primary user-project publication/view lease
2 resolve and acquire optional Blizzard UI publication/view lease
3 derive and acquire the exact ReferenceView required by the selected publications
4 validate project/platform/reference compatibility and owner capabilities
5 call wow-context bind_context_universe_set
6 validate the resulting ContextUniverseSet
```

Owner-specific internal locks/snapshots remain hidden behind ports.

## Resolve once

For each `CurrentPublished` selector:

```text
read one exact CurrentPublicationRecord
-> acquire/pin its exact StoreGeneration and publication set
-> open exact immutable ProjectView/GraphView
-> record all resolved IDs
-> never reread current for this request
```

A later activation does not affect the acquired operation.

## Exact selector

`ExactStoreGeneration` and `ExactPublicationSet` require:

- existence in the named/configured store/project;
- retained and openable immutable state;
- exact owner, epoch, project, publication, project/graph/analyzer/profile bindings;
- validation state permitted by the operation policy;
- no current/last-known-good/neighbor substitution.

Ambiguous publication-to-store-generation resolution is rejected unless the owner port contract guarantees uniqueness and returns the exact binding.

## Expected-current guard

A caller can pass an exact current record ID/digest observed earlier. On mismatch:

- fail before context execution;
- report the expected guard and safe actual record ID where policy permits;
- close acquired resources;
- do not resolve again or proceed with a newer current;
- caller retry is a new request.

## Cross-store consistency

E3-C explicitly does not provide a distributed transaction across independent stores. It provides:

- one immutable primary publication;
- zero/one immutable platform publication;
- one immutable exact ReferenceView;
- exact post-acquisition compatibility validation.

Compatibility checks include:

```text
product/client flavor/build/Interface/ProfileIdentity
ReferenceGeneration/View expected by each project publication
annotation/analyzer binding recorded in the publication
GraphRegistry/relation/source-coordinate/context-input profile compatibility
source universe and SkeletonInputView generation
required owner read catalogs/capabilities
coverage/conflict/validation state
```

If the independently resolved currents are incompatible, return `service_context_selection_incompatible`; do not choose an older platform/reference automatically.

## Reference selection

Primary source of reference identity is the exact selected primary publication. The platform publication must be compatible with it under the selected compatibility profile.

An optional caller guard may require exact Profile/ReferenceGeneration/ReferenceView IDs. Mismatch fails.

Forbidden:

- latest/current Reference Pack lookup;
- selecting by date/name similarity;
- using platform source implementation as ReferenceView contract;
- mixing fields from multiple reference generations.

## Capability gates

Before E3-B binding, require operation-specific capabilities. Examples:

```text
context_status
    bounded publication/reference metadata reads

context_map
    exact project/graph map-read capabilities

context_inspect/build
    exact roots, project/graph/reference and source-slice capabilities required by profiles

context_continue
    exact retained generation and continuation capabilities

context_validate/render
    wow-context schema/profile/renderer capabilities; project views only when artifact validation requires origin closure
```

Do not flatten to `ready=true`. Retain exact missing/partial/conflicted capability records.

## Partial acquisition

A `ServiceContextLeaseSet` is published internally only after every mandatory owner binding validates. There is no public half-lease.

An operation profile may permit an omitted optional Blizzard UI universe. This is an explicit selector/profile state, not acquisition failure.

If an optional capability within an acquired view is partial, the lease can form with exact blockers when E3-B permits a partial result.

## Failure cases

- primary current missing/invalid: fail;
- exact primary generation unavailable: fail;
- optional platform selector `Omitted`: continue under profile;
- requested platform current unavailable: fail/NotEvaluated only as operation contract states;
- platform/reference compatibility mismatch: fail;
- owner record switches/mismatches after open: fail;
- capability partial: preserve blockers and derive operation status conservatively;
- cancellation: close all acquired resources and return cancelled;
- acquisition budget exhausted: fail or not_evaluated as explicitly defined; never fallback.

## Determinism

Canonical resolution records depend on exact selector/request and owner records, not:

- acquisition timing;
- thread completion;
- lock IDs;
- current activation that occurs after resolution;
- host or process;
- retry count;
- CLI alias spelling.

The fixed order prevents scheduler-dependent selection.

## Tests

- each selector family success/failure;
- current changes immediately before/after resolve;
- expected-current guard mismatch;
- primary/platform current compatibility mismatch;
- exact retained generation while a new current activates;
- no second current read mutation;
- no last-known-good substitution;
- exact reference derivation and guard;
- missing/partial/conflicted capabilities;
- cancellation after each acquisition stage;
- owner returns wrong generation/universe;
- randomized scheduling yields same resolved IDs or same typed failure.
