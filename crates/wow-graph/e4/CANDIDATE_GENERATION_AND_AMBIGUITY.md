# E4-B candidate generation and ambiguity components

**Status:** normative bounded discovery algorithm. Candidate generation never establishes lineage authority.

## Inputs

Candidate generation consumes exact immutable records already produced by owners:

```text
before GenerationEntityRefs and typed owner/fingerprint facts
after GenerationEntityRefs and typed owner/fingerprint facts
optional exact Reference transition records
optional E4-A search lineage-candidate signals
exact relation/proof/blocking/component profiles
coverage/conflict/budget/cancellation state
```

It does not parse source, execute source, query mutable current state, call a model, or scan arbitrary stores.

## Staged blocking

Unrestricted all-pairs comparison is forbidden. The profile defines ordered bounded blocking stages such as:

```text
0 exact owner-stable identity key
1 explicit owner/reference transition key
2 exact entity kind + canonical qualified name
3 exact owner/container/package + short name
4 exact receiver/member or namespace/member shape
5 exact explainable fingerprint bucket
6 bounded signature/type/restriction shape bucket
7 bounded E4-A candidate signals
8 bounded graph-neighborhood candidate bucket
```

Each stage specifies:

- eligible entity/universe/kind classes;
- exact key/feature schema;
- maximum bucket size and pair count;
- confidence ceiling;
- dedup and ordering rules;
- required coverage;
- overflow/truncation behavior;
- whether later stages run when a stronger exact stage succeeds.

Stage order is a work/budget policy, not proof precedence. The proposal records the actual evidence class and ceiling.

## Exact versus candidate blocks

An exact owner-stable key can emit a higher-ceiling proposal because the key itself is owner evidence. Exact string equality in a search/name block remains Candidate lineage evidence unless the owner contract separately declares stable identity.

The same pair can have multiple proposal records from independent producers/stages. They are not converted into votes.

## Pair generation

For every bounded block:

```text
validate all entity refs and feature origins
-> sort before and after members by canonical key
-> generate only profile-permitted pair shapes
-> enforce per-block/per-entity/global pair budgets
-> produce immutable proposal candidates with evidence refs
-> deduplicate exact proposal identity while retaining all origins
-> emit skipped/overflow/partial/NotEvaluated records
```

No random sampling, nearest-neighbor hidden fallback, first match, or source-order dependence.

## Candidate graph

Construct a deterministic bipartite candidate graph:

```text
left nodes  = exact before entities
right nodes = exact after entities
edges       = lineage proposals
```

The graph is partitioned into connected `LineageCandidateComponent` records under explicit component-size limits.

## Component shapes

```text
OneToOne
OneToMany
ManyToOne
ManyToMany
BeforeOnly
AfterOnly
```

These are structural shapes, not semantic conclusions.

Examples:

- `OneToMany` may be split, copy/extraction, generated duplicates, one true successor plus unrelated candidates, or unresolved ambiguity.
- `ManyToOne` may be merge/consolidation, duplicate collapse, shared helper extraction, or unresolved ambiguity.
- `BeforeOnly` may be removed, out-of-scope, unindexed, truncated, renamed beyond candidate coverage, or NotEvaluated.
- `AfterOnly` may be introduced or the symmetric unknown cases.

## Ambiguity

A component is ambiguous whenever more than one relation assignment remains compatible with the active proposals/evidence/profile, including competing one-to-one proposals or split/merge possibilities.

Ambiguity records include:

- all before/after entity IDs;
- every proposal and producer;
- proof ceilings and blockers;
- incompatible/exclusive assignment sets;
- coverage/truncation state;
- review requirements;
- allowed next operations;
- stable canonical order.

No component is silently simplified.

## Assignment policy

E4-B does not solve components with a generic maximum-weight matching algorithm as truth. A profile can compute a Candidate-only suggested assignment for review, but:

- it remains a separate proposal;
- all alternatives remain visible;
- search/fingerprint scores remain Candidate evidence;
- a unique numerical optimum is not a proof;
- tie or near-tie thresholds do not become hidden acceptance rules;
- split/merge relations are not forced into bijection.

## Deterministic rule-based acceptance

A component can be accepted without manual review only when a reviewed deterministic relation rule states exact sufficient evidence, for example:

```text
same exact owner-stable entity identity
+ compatible exact generations
+ no competing entity using that identity
+ complete identity coverage
+ no conflict
-> Proven lineage_successor_of
```

or:

```text
accepted lineage_successor_of
+ exact before/after owner canonical names differ
+ complete name-field coverage
-> Derived renamed_from
```

Candidate cardinality alone is never sufficient.

## Copy, split and merge hard cases

### Copy versus move

If old and new similar entities both exist after the transition, `moved_from` is not established. The profile may emit `copied_or_extracted_candidate` and preserve both continuities/unknowns.

### Split

`split_from` requires qualifying evidence that multiple after entities descend semantically from one before entity. Multiple similar after candidates alone are insufficient.

### Merge

`merged_from` requires qualifying evidence that one after entity consolidates multiple before entities. Similarity to each alone is insufficient.

### Generated/vendor duplicates

Generated, vendored, embedded, mirrored or copied source can create identical fingerprints. Source class, ownership and generation facts must remain explicit; identical content is Candidate only.

## Removal and introduction candidate frontier

BeforeOnly/AfterOnly entities do not immediately become absence decisions. Candidate generation reports:

```text
unmatched under executed blocks
blocks skipped/unavailable
pair/component budgets reached
producer coverage
possible unresolved relations
```

`GenerationAbsenceDecision` is evaluated later under `COVERAGE_NEGATIVE_AUTHORITY.md`.

## Pagination and continuation

Large candidate sets/components use snapshot-bound continuation. Cursor binds:

- exact lineage universe and profiles;
- before/after entity manifests;
- blocking stage/frontier;
- generated proposal/component manifests;
- stable ordering keys;
- cumulative pair/component budgets;
- prior overflow/omission records;
- integrity digest.

Continuation never refreshes generations, reruns against current, resets budgets, or hides earlier truncation.

## Resource bounds

Bound at minimum:

```text
entities per generation/scope
blocking keys and stages
members per bucket
pairs per bucket/entity/stage/request
proposals per pair
component nodes/edges
fingerprint/shape features
search signal/evidence refs
serialized bytes
wall/CPU/memory
continuation pages/cursor bytes
```

If a required exact identity block exceeds its configured limit, fail/NotEvaluate that capability; do not silently fall back to approximate matching and call it complete.

## Determinism

Equivalent exact inputs/profile must produce identical:

- blocking keys/buckets;
- generated pairs/proposals;
- proposal IDs/order;
- candidate graph/components;
- ambiguity and overflow records;
- continuations;
- canonical bytes.

Worker count, hash/SQL/source enumeration order, cache history, host, clock, repository popularity and search completion order cannot change them.

## Required mutations

- same name, unrelated entities;
- same body/fingerprint, copied entities;
- moved and copied simultaneously;
- one true successor plus many noisy search hits;
- split/merge ambiguity;
- exact stable identity conflict;
- partial before/after coverage;
- pair/component budget overflow;
- high rank but false lineage;
- shuffled producer/block/entity order;
- 1/2/N workers;
- cancellation at every stage.
