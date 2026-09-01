# E3-B input universes and immutable view binding

**Status:** normative cross-crate input contract.

## Active universe set

E3-B binds one explicit set:

```text
primary user project
    exact PublishedProjectView / ProjectSnapshot
    exact GraphView / GraphSnapshot
    exact source/context-input view

optional Blizzard UI platform source
    exact BlizzardUiProjectView / ProjectSnapshot
    exact GraphView / GraphSnapshot
    exact SkeletonInputView

reference contract
    exact ReferenceProfile / ReferenceGeneration / ReferenceView
```

No source is discovered or selected inside `wow-context`.

## Required public capabilities

### From `wow-project`

- exact project/package/file/load/source-handle lookup;
- bounded project source/context-input pages;
- exact source slice by source handle/content digest/range under privacy/license policy;
- exact publication/generation/profile/capability/coverage/conflict records;
- Blizzard UI `SkeletonInputView` when bound.

### From `wow-graph`

- exact entity/relation lookup;
- bounded neighbors, axes, paths, subgraphs, and explain operations;
- exact assertion/evidence/coverage/conflict refs;
- deterministic continuation.

### From `wow-reference`

- exact API/type/event/restriction entity lookup by already-resolved key;
- exact source/evidence/coverage/conflict and negative-authority records;
- bounded related-entity reads declared by ReferenceView.

No dependency supplies raw source parser, analyzer actor, database connection, SQL, mutable transaction, recognizer engine, or service callback.

## Binding request

```text
BindContextUniverseRequest
    primary exact project publication identity
    optional exact Blizzard UI publication identity
    exact ReferenceView identity
    expected client/flavor/build/profile compatibility assertions
    required capabilities
    allowed partial-universe policy
    budgets/cancellation
```

Using a symbolic `current` is allowed only at an application/service boundary before this request is canonicalized. The resulting request stores exact resolved identities. `wow-context` never resolves current again.

## Compatibility validation

Validate at least:

- primary ProjectSnapshot matches its GraphSnapshot and source/project generation;
- Blizzard UI ProjectSnapshot matches its GraphSnapshot/source profile;
- ReferenceProfile product/flavor/build/Interface is explicitly compatible with each bound project view;
- annotation/analyzer-derived facts cited by the project were produced against the expected ReferenceProfile;
- graph registries and relation profiles used by context profiles are compatible;
- source-coordinate and content-digest identities resolve;
- required input capabilities and coverage exist;
- no identity belongs to another universe/generation under the same display name.

Compatibility is an explicit report, not a date/name heuristic.

## Immutable operation lifetime

After binding:

- every read names exact view/snapshot IDs;
- no source/project/graph/reference view may switch generations;
- current activation by another writer does not affect the operation;
- cache lookup must use the same exact universe set;
- continuation cursors bind the same universe set;
- cancellation closes input views/read leases according to owner contracts;
- a retry after rebinding is a new request/pack identity.

## Optional platform source

A request can omit Blizzard UI source only when its intent/profile does not require it or explicitly allows partial context. The pack records:

```text
platform universe absent
reason
requested capabilities affected
omission/completeness impact
```

It cannot silently substitute a different build or prior platform snapshot.

## Multiple user/dependency projects

E3-B v1 binds one primary user project. Dependency source records already represented inside its exact published project/graph universe may be included under their existing identities. Binding multiple independent mutable user projects is deferred until a separate multi-project coherence contract exists.

## Combined views

A combined context view contains references to distinct project maps and exact cross-universe relations. It does not:

- rewrite keys into one namespace;
- select one generic parent;
- merge same-name entities;
- copy platform-source rows into the user project;
- treat reference entities as source declarations;
- infer lineage across generations.

## Input partial/failure behavior

- generation mismatch: fail binding;
- required capability missing/failed: fail or explicit partial according to request policy;
- optional capability missing: omission + coverage impact;
- conflict: preserve conflict and block dependent exact claims;
- truncated upstream view: explicit partial, never complete;
- stale continuation: reject;
- source slice denied: retain source handle and omission/privacy decision when allowed.

## Input-view security

- normalize all IDs and limits before owner calls;
- no arbitrary path or SQL string;
- no unbounded page size/depth/output;
- no application-provided executable filters;
- no source text in control fields;
- validate every returned record's universe/generation before use;
- reject unexpected extra records or raw handles.
