# E4-A graph-assisted retrieval

**Status:** normative bounded candidate expansion.

## Purpose

Use exact existing graph structure to retrieve related exact entities after an exact query seed or an exact caller-provided seed exists.

Graph retrieval does not scan the whole graph for semantic similarity and does not infer new relations.

## Seeds

Allowed seeds:

```text
exact caller EntityKey/EntityId
candidate from an exact identity/name/alias/member lane
candidate from an approximate lane only when the request/profile explicitly permits candidate-seeded expansion
```

Candidate-seeded expansion retains the seed's lower match class and cannot produce an exact authority band by propagation.

## Expansion profile

```text
GraphSearchExpansionProfile
    relation/axis whitelist
    direction
    per-relation confidence/provenance policy
    maximum depth
    per-node and total fanout
    maximum visited nodes/edges/paths
    path cost table
    allowed universe bridges
    seed-class permissions
    cycle policy
    result field projection
    budgets/cancellation
```

The profile is repository-owned and nonexecutable.

## Initial relation classes

Potential bounded relations include:

```text
contains / owns
loads / depends_on / optional_depends_on
declares / defines / exports
calls / possible_calls
registers_event / handles_event
triggers_callback / subscribes_callback
hooks / sets_script
reads_state / writes_state
created_by / instantiates
inherits / mixes_in / references_template
uses_api
```

The actual active set is profile-specific and graph-registry-versioned.

## Path semantics

A graph candidate signal contains:

```text
seed candidate/entity
target exact entity
ordered relation/assertion IDs
path direction and cost
per-edge confidence/provenance/coverage/conflicts
path-level weakest confidence
truncation/cycle state
```

A path is never rewritten as a direct edge.

## Cross-universe bridges

Only explicit validated graph relations may cross project, Blizzard UI, and Reference universes. Same name, same source text, same signature, or query similarity cannot create a bridge.

A `uses_api` path can explain a project function's connection to a Reference entity. It does not merge them.

## Confidence and authority

- graph signals inherit the seed match class and edge confidence limits;
- a path through a `Possible` edge remains possible;
- candidate-seeded paths remain candidate retrieval;
- multiple paths do not automatically upgrade to proven;
- graph distance/overlap is ranking evidence, not domain authority;
- graph no-hit is not entity absence without exact graph coverage and an appropriate owner negative decision.

## Expansion order

Use deterministic bounded traversal:

```text
seed order from normalized query/result
relation kind order
direction order
path cost
depth
source entity key
target entity key
assertion/path ID
```

No row, hash, worker, or completion order.

## Cycles and duplicates

- cycle-safe visited state is bound to exact graph snapshot and request;
- distinct reason paths may be retained up to a bounded per-target limit;
- same target candidate deduplicates at candidate level while preserving path signals;
- recursive/cyclic call/state relations are valid and bounded;
- invalid/conflicted hierarchy/load cycles remain explicit graph conflicts.

## Graph-wide expansion is forbidden

Public E4-A has no operation equivalent to “rank every graph node by proximity” without seeds. The lane cannot materialize the entire graph, all-pairs paths, or transitive closure.

## Budgets

Bound:

- seeds;
- depth;
- relation classes;
- per-node fanout;
- visited nodes/edges;
- paths per target;
- total candidates;
- evidence/conflict refs;
- serialized bytes;
- time/memory;
- continuation state.

## Failure and truncation

Graph unavailable, partial, conflict-blocked, cancelled, or truncated states remain in the lane result and candidate explanation. A truncated expansion cannot report full graph-retrieval coverage.

## Security

- no caller-supplied relation expression or graph query language;
- no source text as relation selector;
- no raw graph-store/SQL handle;
- no filesystem/network/process/model callbacks;
- no source snippets by default;
- all exact graph handles are validated against the bound snapshot.

## Tests

Include:

- direct exact neighbor;
- multi-hop reason path;
- same target via multiple paths;
- possible/conflicted edge;
- cross-universe explicit bridge;
- same-name entities without bridge;
- cyclic high-fanout graph;
- candidate-seeded authority cap;
- graph no-hit under partial coverage;
- deterministic truncation and continuation;
- path-to-direct-edge mutation rejection.
