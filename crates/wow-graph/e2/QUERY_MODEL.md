# Snapshot-bound bounded query model

**Status:** normative E2-A read API.

## Common request fields

```text
exact GraphGenerationId / GraphViewId
universe/profile/generation assertions
root EntityKey(s)
relation kind whitelist or one axis ID
direction
confidence/provenance policy
coverage/conflict policy
entity/relation filters using declared fields only
budgets/cancellation
continuation cursor: optional
```

No executable predicate, SQL, regex over full source, or arbitrary script.

## Operations

### `entity_exact`

Returns an entity view by exact key or a typed absence decision. A missing row is not authoritative absence without relevant partition coverage.

### `neighbors`

One-hop source/target/both traversal over exact relation kinds with deterministic ordering.

### `traverse_axis`

Breadth/depth policy is profile-defined. Returns paths/view nodes under depth/node/edge/byte/time budgets and cycle-safe visited rules.

### `bounded_paths`

Returns at most `max_paths`, `max_depth`, and `max_expansions` paths using an explicit relation whitelist. Path ordering is canonical. Candidate/possible inclusion is explicit.

### `project_subgraph`

Returns only the requested bounded neighborhood and supporting assertion/evidence handles. It is not a full graph export.

### `explain_entity` / `explain_relation`

Returns exact assertion, producer, evidence, coverage, conflict, registry, and derivation records.

## Progressive read discipline

Normal agent flow:

```text
exact root
-> direct relevant neighbors/axis
-> expand selected branch/path
-> inspect exact relation evidence
-> stop when no new evidence or budget reached
```

The query layer records `no_new_evidence` separately from authoritative absence.

## Confidence policy

Default:

```text
include Proven and Derived
include Possible only when operation/profile asks
exclude Candidate
```

Impact/reachability cannot report possible/candidate paths as proven.

## Coverage and absence

A query result distinguishes:

```text
found
not_found_with_authority
not_found_with_partial_coverage
conflict
not_evaluated
truncated
```

Graph completeness is scoped to producer partitions and capabilities.

## Determinism

Sort by frozen tuple such as:

```text
relation kind order
source EntityKey
target EntityKey
semantic qualifiers
confidence/provenance order
assertion ID
```

Never row ID or traversal completion order.

## Continuation

A cursor binds:

- snapshot/query/profile IDs;
- normalized request digest;
- ordering version;
- last stable key;
- remaining/used budget state where supported;
- integrity digest.

Cursor against another snapshot/request is rejected. Continuation cannot hide prior truncation.

## Cancellation

No background continuation. Cancel returns typed cancellation or a profile-permitted explicit partial result; it never becomes complete/clean.
